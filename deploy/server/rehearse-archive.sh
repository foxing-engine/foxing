#!/usr/bin/env bash
# S16 (c), the rehearsal: the archive move, on a copy of the live log.
#
# Run on the game box as the deploy user (no root): the `rehearse-archive` server
# check does. It reads the live log once (one COPY) and writes nothing to it.
#
#   1. a fresh load database (sudo foxing-load start), and in it a schema of its
#      own, `rehearsal`, beside the load world's;
#   2. every event of the live world copied into it, and the live checkpoint beside;
#   3. `foxing-driver archive`: a dry run, then the move, the checkpoint verified
#      against the whole log before and after;
#   4. a driver booted on the archived copy from the checkpoint (ports 7797/7798,
#      no webhook, no Docent), its verbs that read back asked, then a boot from the
#      whole log (both tables) and the two boards held together;
#   5. everything dropped (sudo foxing-load stop).
set -euo pipefail
APP=${APP:-/srv/foxing}
BIN=$APP/current/foxing-driver
LIVE_ENV=/etc/foxing/driver.env
LOAD_ENV=/etc/foxing/load.env
WORK=$(mktemp -d /tmp/foxing-rehearsal.XXXXXX)
PORT=7797
say() { echo; echo "######## $*"; }
val() { sed -n -E "s/^(FOXING|MOONDIAL)_$2=//p" "$1" | tail -1; }
LIVE_URL=$(val $LIVE_ENV LOG_URL)
WORLD=$(val $LIVE_ENV WORLD); WORLD=${WORLD:-disremembered}
[ -n "$LIVE_URL" ] || { echo "no live log URL"; exit 1; }
[ -f "$LOAD_ENV" ] || { echo "no load world on this box (setup-load-world.sh)"; exit 1; }
LOAD_URL=$(val $LOAD_ENV LOG_URL)
case "$LOAD_URL" in *\?*) REH_URL="$LOAD_URL&options=-c%20search_path%3Drehearsal";; *) REH_URL="$LOAD_URL?options=-c%20search_path%3Drehearsal";; esac
driver_pid=""
cleanup() {
  [ -n "$driver_pid" ] && kill "$driver_pid" 2>/dev/null || true
  sudo /usr/local/sbin/foxing-load stop >/dev/null 2>&1 || true
  rm -rf "$WORK"
}
trap cleanup EXIT

say "Room for it"
size=$(psql "$LIVE_URL" -Atc "select pg_total_relation_size('events')")
free=$(df --output=avail -B1 /var/lib/postgresql 2>/dev/null | tail -1 || df --output=avail -B1 / | tail -1)
echo "the live events table is $((size / 1048576)) MB; $((free / 1048576)) MB free"
[ "$free" -gt $((size * 3)) ] || { echo "not three times the table free: not rehearsing"; exit 1; }
psql "$LIVE_URL" -Atc "select 'live: '||count(*)||' events, '||min(seq)||' to '||max(seq) from events where world='$WORLD'"

say "A fresh load database, and a schema of its own in it"
sudo /usr/local/sbin/foxing-load start | tail -1
psql "$LOAD_URL" -qc "CREATE SCHEMA rehearsal"
mkdir -p "$WORK/data"
# The driver makes its own tables: it opens the log and stops for want of a checkpoint.
FOXING_LOG_URL="$REH_URL" FOXING_WORLD="$WORLD" FOXING_DATA="$WORK/data" "$BIN" archive >/dev/null 2>&1 || true
psql "$REH_URL" -Atc "select 'tables in rehearsal: '||string_agg(tablename, ', ') from pg_tables where schemaname='rehearsal'"

say "The live log copied (one COPY out of the live database, read only)"
t=$(date +%s)
cp "$APP/shared/data/checkpoint-$WORLD.json" "$WORK/data/"
psql "$LIVE_URL" -qc "\copy (SELECT world, seq, at, kind, actor, cause, payload FROM events WHERE world='$WORLD' ORDER BY seq) TO STDOUT" \
  | psql "$REH_URL" -qc "\copy events FROM STDIN"
psql "$REH_URL" -Atc "select 'copy: '||count(*)||' events, '||min(seq)||' to '||max(seq) from events where world='$WORLD'"
echo "copied in $(( $(date +%s) - t )) s"

run() { FOXING_LOG_URL="$REH_URL" FOXING_WORLD="$WORLD" FOXING_DATA="$WORK/data" "$@"; }
say "The dry run"
run "$BIN" archive 2>&1 | grep -v "holds the world" | cut -c1-500
say "The move"
run "$BIN" archive --yes 2>&1 | grep -v "holds the world" | cut -c1-500
psql "$REH_URL" -Atc "select 'archive: '||count(*)||' events, '||min(seq)||' to '||max(seq) from events_archive where world='$WORLD' union all select 'events: '||count(*)||' events, '||min(seq)||' to '||max(seq) from events where world='$WORLD'"

boot() { # $1: FOXING_CHECKPOINT
  FOXING_CHECKPOINT=$1 FOXING_LOG_URL="$REH_URL" FOXING_WORLD="$WORLD" FOXING_DATA="$WORK/data" FOXING_OPS_LOG=none \
    FOXING_ADMIN_TOKEN=rehearsal FOXING_HTTP=127.0.0.1:$PORT FOXING_TCP=127.0.0.1:$((PORT + 1)) \
    nohup "$BIN" > "$WORK/driver-$1.log" 2>&1 &
  driver_pid=$!
  t=$(date +%s)
  for _ in $(seq 1 600); do
    curl -fsS --max-time 2 http://127.0.0.1:$PORT/health 2>/dev/null | grep -q '"ok":true' && break
    kill -0 "$driver_pid" 2>/dev/null || { echo "the driver died:"; tail -20 "$WORK/driver-$1.log"; exit 1; }
    sleep 0.5
  done
  echo "answering after $(( $(date +%s) - t )) s: $(grep -m1 'events in' "$WORK/driver-$1.log" | cut -c1-200)"
}
stop() { kill "$driver_pid" 2>/dev/null || true; wait "$driver_pid" 2>/dev/null || true; driver_pid=""; sleep 1; }
ask() { # verb, args-json
  curl -s -m 60 -X POST http://127.0.0.1:$PORT/v0/command -H 'content-type: application/json' \
    -d "{\"actor\":\"Gnarchivist\",\"passphrase\":\"pw-gnarchivist-7c1k\",\"passing\":true,\"command\":\"$1\",\"args\":$2}" \
    | python3 -c "import json,sys; j=json.load(sys.stdin); print('  $1:', j.get('ok'), '|', (j.get('summary') or str(j.get('error')) or '')[:150].replace(chr(10),' '))"
}
board() { curl -s -m 120 http://127.0.0.1:$PORT/v0/board | python3 -c "
import json,sys
def strip(o):
    if isinstance(o,dict): return {k:strip(v) for k,v in sorted(o.items()) if k not in ('now','as_of','at','here','holding','held_24h','shelf','quiet_ms','last_seen')}
    if isinstance(o,list): return [strip(x) for x in o]
    return o
print(json.dumps(strip(json.load(sys.stdin)), sort_keys=True))" > "$1"; }

say "A driver on the archived copy, booted from the checkpoint"
boot 1
board "$WORK/board-checkpoint.json"
for v in look messages session log news board wants mine chunks; do ask $v '[]'; done
ask where '["Reading Rooms"]'
stop

say "The checkpoint against the whole log (both tables), after that play"
run "$BIN" verify-checkpoint 2>&1 | tail -1 | cut -c1-500

say "A driver booted from the whole log, archive and all (no checkpoint)"
boot 0
board "$WORK/board-whole.json"
stop
if cmp -s "$WORK/board-checkpoint.json" "$WORK/board-whole.json"; then
  echo "the board is the same from the checkpoint and from the whole log ($(wc -c < "$WORK/board-whole.json") bytes)"
else
  echo "THE BOARDS DIFFER"; exit 1
fi
grep -i "panic" "$WORK"/driver-*.log | head -3 || true
say "Done: the rehearsal passed; the copy is being dropped"
