#!/usr/bin/env bash
# Runs ON the game server as the deploy user; the GitHub workflow pipes it over SSH:
#   ssh game "bash -s -- <release>" < deploy/server/remote-deploy.sh
# The release folder holds the static `foxing-driver` binary and deploy/. Switch the
# `current` symlink, restart the service, wait for /health, roll back if it does not come up.
set -euo pipefail

# The host migrated to /srv/foxing and foxing.service on 11-01 (migrate-to-foxing.sh).
APP_DIR=/srv/foxing
UNIT=foxing
KEEP_RELEASES=5
REL_NAME="${1:?release name required}"
REL="$APP_DIR/releases/$REL_NAME"

log() { printf '\n==> %s\n' "$*"; }

[ -x "$REL/foxing-driver" ] || { echo "No executable foxing-driver in release, aborting"; exit 1; }
"$REL/foxing-driver" --version >/dev/null 2>&1 || echo "(binary does not answer --version; continuing)"

PREV=$(readlink -f "$APP_DIR/current" 2>/dev/null || true)
point_current() {
  ln -sfn "$1" "$APP_DIR/current.tmp"
  mv -Tf "$APP_DIR/current.tmp" "$APP_DIR/current"
}

healthy() {
  for _ in $(seq 1 20); do
    if out=$(curl -fsS --max-time 2 http://127.0.0.1:7777/health 2>/dev/null) && printf '%s' "$out" | grep -q '"ok":true'; then
      echo "health: $out"
      return 0
    fi
    sleep 0.5
  done
  return 1
}

log "Switching current -> $REL_NAME"
point_current "$REL"
sudo systemctl restart "$UNIT"

if healthy; then
  echo "driver is up"
else
  echo "driver did not answer /health; last log lines:"
  journalctl -u "$UNIT" -n 30 --no-pager || true
  if [ -n "$PREV" ] && [ -d "$PREV" ]; then
    echo "rolling back to $(basename "$PREV")"
    point_current "$PREV"
    sudo systemctl restart "$UNIT"
    healthy && echo "rollback is up" || echo "rollback is ALSO down; look at journalctl -u $UNIT"
  fi
  exit 1
fi

# The Docent runs the release's tools/docent.py; a new release restarts it when the
# unit is there and sudo allows (setup-bindery.sh grants it). Not fatal either way.
if [ -f /etc/systemd/system/docent.service ]; then
  if sudo -n systemctl restart docent 2>/dev/null; then echo "docent restarted on the new release"; else echo "(docent not restarted: no sudo right yet; rerun deploy/setup-bindery.sh once)"; fi
fi

log "Pruning old releases (keeping $KEEP_RELEASES)"
cd "$APP_DIR/releases"
ls -1dt -- */ 2>/dev/null | tail -n +$((KEEP_RELEASES + 1)) | while read -r old; do
  old=${old%/}
  [ "$APP_DIR/releases/$old" = "$(readlink -f "$APP_DIR/current")" ] && continue
  rm -rf -- "$old" && echo "removed $old"
done

log "Deployed $REL_NAME"
