#!/usr/bin/env bash
# Move the driver's event log to PostgreSQL on this server. Run as root, once:
#   sudo bash /srv/foxing/current/deploy/setup-postgres.sh
# Installs PostgreSQL if missing, creates a role and database with a generated
# password, copies the current file log in (refusing if the database already has the
# world), writes FOXING_LOG_URL into /etc/foxing/driver.env, and restarts the
# driver. The file log is left where it was, as the backup it now is.
set -euo pipefail

APP_DIR=${APP_DIR:-/srv/foxing}
ENV_FILE=/etc/foxing/driver.env
UNIT=foxing
DB=${DB:-foxing}
ROLE=${ROLE:-foxing}

ok() { printf '  OK  %s\n' "$*"; }
die() { printf '  !!  %s\n' "$*" >&2; exit 1; }
[ "$(id -u)" = 0 ] || die "run as root"
[ -f "$ENV_FILE" ] || die "$ENV_FILE is missing; run setup-server.sh first"

echo "######## PostgreSQL"
if ! command -v psql >/dev/null; then
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -q && apt-get install -y -q postgresql
fi
systemctl enable --now postgresql
ok "postgresql $(psql --version | awk '{print $3}') running"

echo; echo "######## Role and database"
PASS=$(head -c 32 /dev/urandom | base64 | tr -d '/+=' | head -c 32)
if su postgres -c "psql -tAc \"SELECT 1 FROM pg_roles WHERE rolname='$ROLE'\"" | grep -q 1; then
  su postgres -c "psql -qc \"ALTER ROLE $ROLE WITH PASSWORD '$PASS'\""
  ok "role $ROLE exists; password rotated"
else
  su postgres -c "psql -qc \"CREATE ROLE $ROLE LOGIN PASSWORD '$PASS'\""
  ok "role $ROLE created"
fi
if su postgres -c "psql -tAc \"SELECT 1 FROM pg_database WHERE datname='$DB'\"" | grep -q 1; then
  ok "database $DB exists"
else
  su postgres -c "psql -qc \"CREATE DATABASE $DB OWNER $ROLE\""
  ok "database $DB created, owned by $ROLE"
fi
URL="postgres://$ROLE:$PASS@127.0.0.1:5432/$DB"

echo; echo "######## Import the file log"
# shellcheck disable=SC1090
set -a; . "$ENV_FILE"; set +a
# The env file's keys are FOXING_* (rename-env.sh), MOONDIAL_* before it.
WORLD=${FOXING_WORLD:-${MOONDIAL_WORLD:-disremembered}}
DATA=${FOXING_DATA:-${MOONDIAL_DATA:-$APP_DIR/shared/data}}
BIN="$APP_DIR/current/foxing-driver"
[ -x "$BIN" ] || die "$BIN is not there; deploy the driver first"
systemctl stop "$UNIT"
ok "driver stopped"
if [ -f "$DATA/$WORLD.events.jsonl" ]; then
  if FOXING_LOG_URL="$URL" FOXING_WORLD="$WORLD" "$BIN" import "$DATA"; then
    ok "file log imported; $DATA/$WORLD.events.jsonl stays as the backup"
  else
    echo "  !!  import refused or failed (see above); the file log is untouched"
    echo "      if the database already holds this world, that is the live log and nothing was lost"
  fi
else
  ok "no file log for world $WORLD at $DATA; starting empty"
fi

echo; echo "######## Driver environment"
if grep -qE '^(FOXING|MOONDIAL)_LOG_URL=' "$ENV_FILE"; then
  sed -i -E "s|^(FOXING\|MOONDIAL)_LOG_URL=.*|FOXING_LOG_URL=$URL|" "$ENV_FILE"
else
  printf 'FOXING_LOG_URL=%s\n' "$URL" >> "$ENV_FILE"
fi
ok "FOXING_LOG_URL written to $ENV_FILE (the password is in there; it is root:foxing 640)"

systemctl start "$UNIT"
sleep 2
if curl -fsS --max-time 3 http://127.0.0.1:7777/health | grep -q '"ok":true'; then
  ok "driver is up on PostgreSQL: $(curl -fsS http://127.0.0.1:7777/health)"
else
  die "driver did not come up; journalctl -u $UNIT -n 50"
fi

echo
echo "Backups: su postgres -c 'pg_dump $DB' > foxing-\$(date +%F).sql"
