#!/usr/bin/env bash
# S16 (c), the live move (David, 11-15: "let's go for it"): the live world's events
# older than its checkpoint, less a tail of 60,000, from `events` to `events_archive`.
#
# Run on the game box as the deploy user: the `archive-live` server check does.
#   1. a dump of the live `events` table to a file beside the data (kept; the way
#      back, should one ever be wanted);
#   2. the dry run: what would move, and the checkpoint verified against the log;
#   3. the move, 20,000 events at a time, verified before it starts;
#   4. the checkpoint verified against both tables after it, and the counts.
# It stops at the first thing that fails. A move that stops part-way is safe where
# it stands (an event is in one table or the other) and is taken up by running this
# again.
set -euo pipefail
APP=${APP:-/srv/foxing}
BIN=$APP/current/foxing-driver
ENVF=/etc/foxing/driver.env
val() { sed -n -E "s/^(FOXING|MOONDIAL)_$1=//p" "$ENVF" | tail -1; }
URL=$(val LOG_URL); WORLD=$(val WORLD); WORLD=${WORLD:-disremembered}; DATA=$(val DATA); DATA=${DATA:-$APP/shared/data}
[ -n "$URL" ] || { echo "no live log URL"; exit 1; }
run() { FOXING_LOG_URL="$URL" FOXING_WORLD="$WORLD" FOXING_DATA="$DATA" "$@"; }
say() { echo; echo "######## $*"; }
q() { psql "$URL" -Atc "$1"; }

say "Before: $(date -u)"
q "select 'events: '||count(*)||' ('||min(seq)||' to '||max(seq)||')' from events where world='$WORLD' union all select 'archive: '||count(*) from events_archive where world='$WORLD'"
ls -la "$DATA"/checkpoint-* | cut -c1-140

say "A dump of the live events table, kept"
dump="$DATA/events-before-archive-$(date -u +%Y%m%dT%H%M%SZ).dump"
t=$(date +%s)
nice -n 10 pg_dump "$URL" -Fc -Z 1 -t events -f "$dump"
echo "$(ls -la "$dump" | cut -c1-140)"
echo "dumped in $(( $(date +%s) - t )) s; to restore it: pg_restore --clean -d <url> $dump"

say "The dry run"
run "$BIN" archive 2>&1 | grep -v "holds the world" | cut -c1-600

say "The move"
run "$BIN" archive --yes 2>&1 | grep -v "holds the world" | cut -c1-600
[ "${PIPESTATUS[0]}" = 0 ] || { echo "THE MOVE DID NOT FINISH CLEANLY; see above"; exit 1; }

say "After: $(date -u)"
q "select 'events: '||count(*)||' ('||min(seq)||' to '||max(seq)||')' from events where world='$WORLD' union all select 'archive: '||count(*)||' ('||min(seq)||' to '||max(seq)||')' from events_archive where world='$WORLD'"
q "select 'sizes: events '||pg_size_pretty(pg_total_relation_size('events'))||', archive '||pg_size_pretty(pg_total_relation_size('events_archive'))"
curl -fsS --max-time 5 http://127.0.0.1:7777/health 2>/dev/null | cut -c1-140 || true
echo
