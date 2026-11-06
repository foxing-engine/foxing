#!/usr/bin/env bash
# One-shot, run as root ON the game server, after a driver built with socket activation
# has deployed (2026-11-01 or later: `foxing-driver` takes the HTTP listener from
# systemd when one is handed in). Makes systemd hold the HTTP listening socket, so a
# deploy's restart queues connections in the kernel instead of refusing them: no 502
# from nginx, no empty gate reply, the request waits the few seconds the new driver
# takes to replay its log and is then answered.
#
#   sudo bash /srv/foxing/current/deploy/enable-socket-activation.sh
#
# Re-running is safe. To undo: systemctl disable --now foxing.socket; rm the drop-in;
# systemctl daemon-reload; systemctl restart foxing (the driver binds the port itself
# when no socket is handed in).
set -euo pipefail
ok() { printf '  ok  %s\n' "$*"; }

UNIT=foxing
ENV_FILE=/etc/foxing/driver.env
# The HTTP address the driver would bind on its own: the socket takes the same one.
ADDR=$(grep -oP '^(FOXING|MOONDIAL)_HTTP=\K.*' "$ENV_FILE" | tail -n 1)
ADDR=${ADDR:-127.0.0.1:7777}

cat > /etc/systemd/system/$UNIT.socket <<SOCK
[Unit]
Description=Foxing driver HTTP socket, held across restarts

[Socket]
ListenStream=$ADDR
Backlog=1024
NoDelay=true

[Install]
WantedBy=sockets.target
SOCK
ok "$UNIT.socket listens on $ADDR"

install -d /etc/systemd/system/$UNIT.service.d
cat > /etc/systemd/system/$UNIT.service.d/socket.conf <<CONF
# Written by deploy/server/enable-socket-activation.sh: the HTTP listener comes from
# $UNIT.socket, so the port stays open while the driver restarts.
[Unit]
Requires=$UNIT.socket
After=$UNIT.socket
CONF
ok "$UNIT.service requires the socket"

systemctl daemon-reload
# The port is the driver's until it stops: stop, bind the socket, start. One gap of a
# few seconds, the last one a deploy causes.
systemctl stop "$UNIT"
systemctl enable --now "$UNIT.socket" >/dev/null 2>&1 && ok "socket bound by systemd"
systemctl start "$UNIT"
for i in 1 2 3 4 5 6 7 8 9 10; do
  if curl -fsS --max-time 5 "http://$ADDR/health" >/dev/null 2>&1; then break; fi
  sleep 1
done
curl -fsS --max-time 5 "http://$ADDR/health" && echo
journalctl -u "$UNIT" -n 5 --no-pager | grep -q "socket systemd handed in" && ok "the driver took the socket from systemd" || echo "the driver did not report the inherited socket; it may be binding the port itself (an older build?): journalctl -u $UNIT -n 20"
echo "done. A deploy's restart now queues requests instead of refusing them; systemctl status $UNIT.socket shows the socket."
