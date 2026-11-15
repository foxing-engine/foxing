#!/usr/bin/env bash
# The load world: a second driver on the game box, for load tests (David, 11-15).
#
# Run once, as root, on the game box:   sudo bash setup-load-world.sh
#
# Why: six load runs on the live world in one evening added 600,000 events to its
# log for ever, and the world went down for 21 minutes when boot could no longer
# hold the log in memory (PFR, T20). A load test belongs on a world that is thrown
# away.
#
# What it makes:
#   - a PostgreSQL database `foxing_load`, owned by the live world's role: the same
#     kind of log the live world writes, so the figures mean the same, and dropped
#     and made again at every start;
#   - /etc/foxing/load.env: the live driver's settings with its own ports
#     (HTTP 127.0.0.1:7787, TCP 127.0.0.1:7788), its own data directory, that
#     database, and no Discord webhook, no Docent and no keys;
#   - foxing-load.service: the same binary (`current`), not started at boot, held to
#     2.5 GB of memory and a lesser share of the processors than the live world, so
#     it cannot starve it;
#   - /usr/local/sbin/foxing-load {start|stop|status}: `start` is always a fresh,
#     empty world; and a sudoers line so the deploy user (and so the `Load test`
#     workflow) may run it.
#
# It listens on localhost only: nothing reaches it but an SSH tunnel
# (tools/loadtest.sh). To remove it: systemctl disable --now foxing-load; rm the
# unit, /etc/foxing/load.env, /usr/local/sbin/foxing-load, /etc/sudoers.d/foxing-load;
# su postgres -c "dropdb foxing_load".
set -euo pipefail
[ "$(id -u)" = 0 ] || { echo "run as root"; exit 1; }
APP_DIR=${APP_DIR:-/srv/foxing}
ENV_FILE=${ENV_FILE:-/etc/foxing/driver.env}
LOAD_ENV=/etc/foxing/load.env
ok() { echo "  ok   $*"; }

[ -f "$ENV_FILE" ] || { echo "no $ENV_FILE: is this the game box?"; exit 1; }
RUN_USER=$(stat -c %U "$APP_DIR/shared/data")
URL=$(sed -n -E 's/^(FOXING|MOONDIAL)_LOG_URL=//p' "$ENV_FILE" | tail -1)
[ -n "$URL" ] || { echo "the live driver has no FOXING_LOG_URL; the load world wants PostgreSQL as the live one has"; exit 1; }
ROLE=$(printf '%s' "$URL" | sed -E 's|^postgres(ql)?://([^:@/]+).*|\2|')
LOAD_URL=$(printf '%s' "$URL" | sed -E 's|/[^/?]+(\?.*)?$|/foxing_load\1|')
[ "$LOAD_URL" != "$URL" ] || { echo "could not derive the load database's URL"; exit 1; }

echo "######## Settings"
install -d -o "$RUN_USER" -g "$RUN_USER" "$APP_DIR/shared/load-data"
# The live settings, less everything that speaks to the outside or names the live
# world's own places.
grep -v -E '^(FOXING|MOONDIAL)_(LOG_URL|HTTP|TCP|DATA|OPS_LOG|DISCORD[A-Z_]*|WORKERS)=|DOCENT|BINDERY|WEBHOOK|API_KEY|ANTHROPIC' "$ENV_FILE" > "$LOAD_ENV.new"
cat >> "$LOAD_ENV.new" <<ENV
FOXING_LOG_URL=$LOAD_URL
FOXING_HTTP=127.0.0.1:7787
FOXING_TCP=127.0.0.1:7788
FOXING_DATA=$APP_DIR/shared/load-data
ENV
install -m 640 -o root -g "$RUN_USER" "$LOAD_ENV.new" "$LOAD_ENV" && rm -f "$LOAD_ENV.new"
ok "$LOAD_ENV (ports 7787 and 7788, database foxing_load, no webhook, no Docent)"

echo "######## Service"
cat > /etc/systemd/system/foxing-load.service <<UNIT
[Unit]
Description=Foxing driver, the load world (thrown away at every start)
After=network-online.target postgresql.service

[Service]
User=$RUN_USER
Group=$RUN_USER
EnvironmentFile=$LOAD_ENV
WorkingDirectory=$APP_DIR/current
ExecStart=$APP_DIR/current/foxing-driver
Restart=no
# It may not starve the live world: a ceiling on memory (the live driver was killed
# for memory on 11-15), and a lesser share of the processors when both want them.
MemoryMax=2500M
CPUWeight=40
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=$APP_DIR/shared/load-data
PrivateTmp=yes
ProtectKernelTunables=yes
ProtectControlGroups=yes
RestrictSUIDSGID=yes
LimitNOFILE=65536
UNIT
systemctl daemon-reload
ok "foxing-load.service (not enabled: it runs only when started)"

echo "######## The switch"
cat > /usr/local/sbin/foxing-load <<SWITCH
#!/usr/bin/env bash
# foxing-load start|stop|status: the load world. \`start\` is a fresh, empty world.
set -euo pipefail
case "\${1:-status}" in
  start)
    systemctl stop foxing-load 2>/dev/null || true
    su postgres -c "dropdb --if-exists foxing_load" >/dev/null
    su postgres -c "createdb -O $ROLE foxing_load"
    rm -rf "$APP_DIR/shared/load-data"/* 2>/dev/null || true
    systemctl start foxing-load
    for _ in \$(seq 1 60); do
      if curl -fsS --max-time 2 http://127.0.0.1:7787/health 2>/dev/null | grep -q '"ok":true'; then
        echo "the load world is up: \$(curl -fsS http://127.0.0.1:7787/health)"; exit 0
      fi
      sleep 0.5
    done
    echo "the load world did not answer; journalctl -u foxing-load"; journalctl -u foxing-load -n 20 --no-pager; exit 1 ;;
  stop)
    systemctl stop foxing-load 2>/dev/null || true
    su postgres -c "dropdb --if-exists foxing_load" >/dev/null
    echo "the load world is stopped and its log dropped" ;;
  status)
    systemctl is-active foxing-load || true
    curl -fsS --max-time 2 http://127.0.0.1:7787/health 2>/dev/null || echo "(not answering)" ;;
  *) echo "usage: foxing-load start|stop|status"; exit 2 ;;
esac
SWITCH
chmod 755 /usr/local/sbin/foxing-load
cat > /etc/sudoers.d/foxing-load <<SUDO
$RUN_USER ALL=(root) NOPASSWD: /usr/local/sbin/foxing-load start, /usr/local/sbin/foxing-load stop, /usr/local/sbin/foxing-load status
SUDO
chmod 440 /etc/sudoers.d/foxing-load && visudo -cf /etc/sudoers.d/foxing-load >/dev/null
ok "/usr/local/sbin/foxing-load; $RUN_USER may run it with sudo"

echo "######## A first start, to see it wake, and a stop"
/usr/local/sbin/foxing-load start
/usr/local/sbin/foxing-load stop
echo; echo "Done. Load tests now run against it: Actions -> Load test, or tools/loadtest.sh from your machine."
