#!/usr/bin/env bash
# One-shot, run as root ON the game server, in a quiet hour, after the first foxing-driver
# release has deployed (so $APP_DIR/current/foxing-driver exists). Moves the server's
# layout from the engine's old name to its new one:
#   /srv/moondial -> /srv/foxing, /etc/moondial -> /etc/foxing, moondial.service ->
#   foxing.service (ExecStart foxing-driver), the backup units, sudoers, the analytics
#   units' paths. The env file keeps its MOONDIAL_* names: the driver reads FOXING_*
#   first and falls back to them. Re-running is safe; the deploy scripts detect either
#   layout. Downtime: the restart, a few seconds.
#
#   sudo bash /srv/moondial/current/deploy/migrate-to-foxing.sh
set -euo pipefail
ok() { printf '  ok  %s\n' "$*"; }
RUN_USER=${RUN_USER:-$(stat -c %U /srv/moondial/current 2>/dev/null || echo starsailor)}

if [ -d /srv/foxing ] && [ ! -d /srv/moondial ]; then echo "already migrated"; exit 0; fi
[ -x /srv/moondial/current/foxing-driver ] || { echo "no foxing-driver in the current release; deploy the renamed driver first"; exit 1; }

systemctl stop moondial && ok "driver stopped"
systemctl disable moondial moondial-backup.timer >/dev/null 2>&1 || true
systemctl stop moondial-backup.timer >/dev/null 2>&1 || true

mv /srv/moondial /srv/foxing && ok "/srv/moondial -> /srv/foxing"
mv /etc/moondial /etc/foxing && ok "/etc/moondial -> /etc/foxing"
# The release symlinks are relative or absolute? Fix an absolute current -> releases link.
cur=$(readlink /srv/foxing/current || true)
case "$cur" in /srv/moondial/*) ln -sfn "${cur/\/srv\/moondial/\/srv\/foxing}" /srv/foxing/current && ok "current relinked";; esac
# The env file's data path, if it names the old directory.
sed -i 's|/srv/moondial|/srv/foxing|g; s|/etc/moondial|/etc/foxing|g' /etc/foxing/driver.env && ok "env file paths updated (variable names left as they were)"

sed -e 's|/etc/moondial/|/etc/foxing/|g; s|/srv/moondial|/srv/foxing|g; s|moondial-driver|foxing-driver|; s|Description=Moondial|Description=Foxing|' \
    /etc/systemd/system/moondial.service > /etc/systemd/system/foxing.service
sed -e 's|/srv/moondial|/srv/foxing|g; s|Moondial|Foxing|g' /etc/systemd/system/moondial-backup.service > /etc/systemd/system/foxing-backup.service
sed -e 's|Moondial|Foxing|g' /etc/systemd/system/moondial-backup.timer > /etc/systemd/system/foxing-backup.timer
rm -f /etc/systemd/system/moondial.service /etc/systemd/system/moondial-backup.service /etc/systemd/system/moondial-backup.timer
sed 's/moondial/foxing/g' /etc/sudoers.d/moondial > /etc/sudoers.d/foxing && chmod 440 /etc/sudoers.d/foxing && visudo -cf /etc/sudoers.d/foxing >/dev/null && rm -f /etc/sudoers.d/moondial && ok "sudoers: $RUN_USER may restart foxing"
for u in analytics-collect.service analytics-render.service; do
  f=/etc/systemd/system/$u; [ -f "$f" ] && sed -i 's|/srv/moondial|/srv/foxing|g' "$f" && ok "$u paths updated"
done
systemctl daemon-reload
systemctl enable foxing foxing-backup.timer >/dev/null 2>&1
systemctl start foxing-backup.timer
systemctl start foxing && sleep 2 && systemctl is-active foxing >/dev/null && ok "foxing.service is up" || { journalctl -u foxing -n 30 --no-pager; exit 1; }
echo "done. journalctl -u foxing -f to watch; the next deploy lands in /srv/foxing."
