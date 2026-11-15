#!/usr/bin/env bash
# The load test from any machine with SSH to the game box: yours, or a runner.
#
#   tools/loadtest.sh <names> <minutes> [tag]        e.g. tools/loadtest.sh 300 6 a
#
# It runs against THE LOAD WORLD, not the live one (David, 11-15): a second driver on
# the box with its own log, started fresh for the run and dropped after it
# (deploy/server/setup-load-world.sh, once, as root). `WORLD=live` aims at the live
# world instead; that writes every event of the run to the live log for ever, and is
# for a deliberate last rehearsal only.
#
# It opens an SSH tunnel to that driver's own port on the box and runs tools/load.py
# down it. That goes around nginx on purpose: nginx limits an address to 15 logins a
# minute and 40 connections, which is right for the world and stops a crowd coming
# from one machine. The names still play the live world, at a player's pace, and are
# passing names (Gnload…) the world forgets.
#
# What it measures that the cloud workspace cannot: reply times over one ordinary
# network path, with nothing between but the tunnel.
#
# Settings (environment):
#   GAME_SSH   who to reach the box as    (default root@5.78.210.32)
#   WORLD      load (default) or live
#   GAME_PORT  the driver's port there    (default 7787, the load world; 7777 live)
#   LOCAL_PORT the tunnel's end here      (default 17777)
#   LOAD_HOST  set it to skip the tunnel and aim at a URL (a local world, say)
set -euo pipefail
names=${1:?names (e.g. 300)}; minutes=${2:?minutes (e.g. 6)}; tag=${3:-h}
here=$(cd "$(dirname "$0")" && pwd)
venv="$here/.loadtest-venv"
if [ ! -x "$venv/bin/python" ]; then
  python3 -m venv "$venv"
  "$venv/bin/pip" -q install websockets
fi
# A crowd holds a socket a name; the usual limit of 256 or 1024 open files is too few.
ulimit -n 8192 2>/dev/null || echo "note: could not raise the open-file limit; over ~200 names may fail to connect" >&2
if [ -z "${LOAD_HOST:-}" ]; then
  ssh_to=${GAME_SSH:-root@5.78.210.32}; local_port=${LOCAL_PORT:-17777}
  world=${WORLD:-load}
  case "$world" in
    load)
      port=${GAME_PORT:-7787}
      # A fresh, empty load world for this run; stopped and its log dropped after.
      ssh "$ssh_to" 'sudo /usr/local/sbin/foxing-load start' </dev/null \
        || { echo "the load world would not start: has deploy/server/setup-load-world.sh been run on the box?" >&2; exit 1; }
      stop_world() { ssh "$ssh_to" 'sudo /usr/local/sbin/foxing-load stop' </dev/null || true; } ;;
    live)
      port=${GAME_PORT:-7777}
      echo "WORLD=live: this run's events go into the live world's log for ever." >&2
      stop_world() { :; } ;;
    *) echo "WORLD is load or live" >&2; exit 2 ;;
  esac
  ssh -f -N -o ExitOnForwardFailure=yes -o ServerAliveInterval=15 \
      -L "127.0.0.1:$local_port:127.0.0.1:$port" "$ssh_to"
  trap 'kill "${probe_pid:-}" 2>/dev/null || true; pkill -f "127.0.0.1:$local_port:127.0.0.1:$port" 2>/dev/null || true; stop_world' EXIT
  export LOAD_HOST="http://127.0.0.1:$local_port"
  sleep 1
fi
out="$here/../loadtest-$(date -u +%Y%m%dT%H%M%SZ)-$names.txt"
echo "load: $names names for $minutes minutes against $LOAD_HOST; the result goes to $out"
cd "$(mktemp -d)"
# A second clock, outside the generator: above two hundred names the generator's own
# round trips are mostly its own lag (11-15), so /health is timed by a separate
# process, every two seconds, for as long as the run lasts.
probe="$out.probe"
( while :; do
    curl -s -o /dev/null -w '%{time_total}\n' --max-time 30 "$LOAD_HOST/health" >> "$probe" 2>/dev/null || echo 30 >> "$probe"
    sleep 2
  done ) &
probe_pid=$!
"$venv/bin/python" "$here/load.py" "$names" "$minutes" "$tag" > "$out"
kill "$probe_pid" 2>/dev/null || true
cut -c1-300 "$out" | head -12
python3 - "$probe" <<'PY' | tee -a "$out"
import sys
v = sorted(float(x) for x in open(sys.argv[1]) if x.strip())
if v:
    q = lambda p: v[min(len(v) - 1, int(len(v) * p))]
    ms = lambda x: f"{x * 1000:.0f}"
    print(f"  /health from a separate process, every 2 s: n={len(v)}  p50 {ms(q(.5))}  p95 {ms(q(.95))}  p99 {ms(q(.99))}  max {ms(v[-1])} ms; over 1 s: {sum(x > 1 for x in v)}")
PY
# What the driver says of its own lock and pushes, before the load world is dropped.
if [ -n "${ssh_to:-}" ]; then
  ssh "$ssh_to" "f=/etc/foxing/\$([ ${world:-load} = load ] && echo load || echo driver).env; t=\$(sed -n 's/^FOXING_ADMIN_TOKEN=//p' \$f 2>/dev/null | tail -1); curl -s -m 10 -H \"x-foxing-admin: \$t\" http://127.0.0.1:${port:-7787}/v0/ops | head -c 5000; echo" </dev/null | tee -a "$out" | cut -c1-1500 || true
fi
