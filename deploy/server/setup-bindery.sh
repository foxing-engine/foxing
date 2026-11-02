#!/usr/bin/env bash
# One-shot, run as root ON the game server: installs the Bindery's nightly run.
#
#   sudo bash /srv/foxing/current/deploy/setup-bindery.sh
#
# Writes /etc/foxing/bindery.env (root:starsailor, 640) with the Bindery's own kept
# name and a passphrase minted here, the admin token copied from driver.env, the
# model, and BINDERY_MODE=dry-run; then a systemd service + timer that runs
# tools/bind.py from the current release at 04:40 UTC, after the 04:17 backup.
#
# After it runs, put the Anthropic key in /etc/foxing/bindery.env yourself:
#   ANTHROPIC_API_KEY=sk-ant-...
# The first nights are dry runs (proposals land in /var/lib/foxing/bindery/, and a
# summary goes to Discord if the driver has a webhook). When the proposals read right,
# set BINDERY_MODE=apply in the env file; nothing else changes.
#
# Re-running is safe: an existing env file keeps its key and passphrase.
set -euo pipefail
ok() { printf '  ok  %s\n' "$*"; }

ENV_DIR=/etc/foxing
DRIVER_ENV=$ENV_DIR/driver.env
ENV_FILE=$ENV_DIR/bindery.env
STATE=/var/lib/foxing/bindery
RELEASE=/srv/foxing/current
USER_NAME=starsailor
[ -f "$DRIVER_ENV" ] || { echo "no $DRIVER_ENV: run migrate-to-foxing.sh first"; exit 1; }
id "$USER_NAME" >/dev/null 2>&1 || USER_NAME=$(stat -c %U "$RELEASE/foxing-driver" 2>/dev/null || echo root)

ADMIN=$(grep -oP '^(FOXING|MOONDIAL)_ADMIN_TOKEN=\K.*' "$DRIVER_ENV" | tail -n 1 || true)
HTTP=$(grep -oP '^(FOXING|MOONDIAL)_HTTP=\K.*' "$DRIVER_ENV" | tail -n 1 || true)
WEBHOOK=$(grep -oP '^(FOXING|MOONDIAL)_DISCORD_WEBHOOK=\K.*' "$DRIVER_ENV" | tail -n 1 || true)
[ -n "$ADMIN" ] || { echo "no admin token in $DRIVER_ENV"; exit 1; }

install -d -m 750 -o root -g "$USER_NAME" "$STATE"
if [ ! -f "$ENV_FILE" ]; then
  PASS=$(tr -dc 'a-z0-9' </dev/urandom | head -c 24)
  umask 027
  cat > "$ENV_FILE" <<ENV
# The Bindery's nightly run (tools/bind.py). Written by deploy/server/setup-bindery.sh.
# Put the Anthropic key here; nowhere else. BINDERY_MODE is dry-run until you switch it.
ANTHROPIC_API_KEY=
BINDERY_MODEL=claude-haiku-5-5
BINDERY_MODE=dry-run
BINDERY_MAX=40
BINDERY_ACTOR=Bindery
BINDERY_PASSPHRASE=$PASS
BINDERY_OUT=$STATE/proposals.jsonl
FOXING_BASE=http://${HTTP:-127.0.0.1:7777}
FOXING_ADMIN_TOKEN=$ADMIN
FOXING_DISCORD_WEBHOOK=$WEBHOOK
ENV
  chown root:"$USER_NAME" "$ENV_FILE"
  chmod 640 "$ENV_FILE"
  ok "$ENV_FILE written (the key is blank: add ANTHROPIC_API_KEY=... to it)"
else
  ok "$ENV_FILE kept"
fi

cat > /etc/systemd/system/bindery.service <<UNIT
[Unit]
Description=The Bindery: binds much-appended rooms and books into prose
After=network-online.target foxing.service
Wants=network-online.target

[Service]
Type=oneshot
User=$USER_NAME
EnvironmentFile=$ENV_FILE
WorkingDirectory=$STATE
# BINDERY_MODE=apply in the env file makes the run apply; anything else is a dry run.
ExecStart=/bin/sh -c 'if [ "\$BINDERY_MODE" = apply ]; then exec /usr/bin/python3 $RELEASE/tools/bind.py --apply; else exec /usr/bin/python3 $RELEASE/tools/bind.py; fi'
Nice=10
PrivateTmp=true
NoNewPrivileges=true
UNIT

cat > /etc/systemd/system/bindery.timer <<TIMER
[Unit]
Description=The Bindery, nightly after the backup

[Timer]
OnCalendar=*-*-* 04:40:00 UTC
RandomizedDelaySec=300
Persistent=true

[Install]
WantedBy=timers.target
TIMER

systemctl daemon-reload
systemctl enable --now bindery.timer >/dev/null 2>&1
ok "bindery.timer enabled: $(systemctl list-timers bindery.timer --no-pager | sed -n 2p | awk '{print $1, $2, $3, $4}')"
[ -f "$RELEASE/tools/bind.py" ] && ok "tools/bind.py is in the current release" || echo "  !!  $RELEASE/tools/bind.py is not there yet; the next release stages it"
echo "done. Try it by hand (dry run, needs the key in $ENV_FILE):  sudo systemctl start bindery.service; journalctl -u bindery -n 40 --no-pager"
