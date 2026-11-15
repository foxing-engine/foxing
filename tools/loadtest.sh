#!/usr/bin/env bash
# The load test from any machine with SSH to the game box: yours, or a runner.
#
#   tools/loadtest.sh <names> <minutes> [tag]        e.g. tools/loadtest.sh 300 6 a
#
# It opens an SSH tunnel to the driver's own port on the box and runs tools/load.py
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
#   GAME_PORT  the driver's port there    (default 7777)
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
  ssh_to=${GAME_SSH:-root@5.78.210.32}; port=${GAME_PORT:-7777}; local_port=${LOCAL_PORT:-17777}
  ssh -f -N -o ExitOnForwardFailure=yes -o ServerAliveInterval=15 \
      -L "127.0.0.1:$local_port:127.0.0.1:$port" "$ssh_to"
  trap 'pkill -f "127.0.0.1:$local_port:127.0.0.1:$port" 2>/dev/null || true' EXIT
  export LOAD_HOST="http://127.0.0.1:$local_port"
  sleep 1
fi
out="$here/../loadtest-$(date -u +%Y%m%dT%H%M%SZ)-$names.txt"
echo "load: $names names for $minutes minutes against $LOAD_HOST; the result goes to $out"
cd "$(mktemp -d)"
"$venv/bin/python" "$here/load.py" "$names" "$minutes" "$tag" > "$out"
cut -c1-300 "$out" | head -12
