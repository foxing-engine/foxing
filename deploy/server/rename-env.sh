#!/usr/bin/env bash
# The last of the engine's rename, ON the game server, as root, once (R20 / Y1):
#   sudo bash /srv/foxing/current/deploy/rename-env.sh
#
# migrate-to-foxing.sh (11-01) moved the directories and the units and left three
# things under the old name, on purpose, because the driver reads both. This script
# finishes them, after showing what it would change and asking:
#   1. /etc/foxing/driver.env: every MOONDIAL_* key becomes FOXING_* (values untouched);
#      a key already present as FOXING_* is not duplicated (the FOXING_ line wins).
#   2. foxing.service: Description "Foxing driver (Disremembered)".
#   3. /etc/nginx/.htpasswd-moondial becomes .htpasswd-foxing, and the analytics
#      snippet (and any other nginx file naming it) points at the new file.
# Then the driver restarts (a few seconds) and nginx reloads. Idempotent: run again and
# it finds nothing to do. Nothing is deleted: the env file is backed up beside itself
# first (driver.env.before-rename), and the htpasswd file is moved, not copied.
# Use DRY_RUN=1 to see the diff and stop; YES=1 to skip the question.
set -euo pipefail

ENV_FILE=${ENV_FILE:-/etc/foxing/driver.env}
UNIT_FILE=${UNIT_FILE:-/etc/systemd/system/foxing.service}
OLD_HT=${OLD_HT:-/etc/nginx/.htpasswd-moondial}
NEW_HT=${NEW_HT:-/etc/nginx/.htpasswd-foxing}
NGINX_DIR=${NGINX_DIR:-/etc/nginx}

ok()   { printf '  ok   %s\n' "$*"; }
note() { printf '  --   %s\n' "$*"; }
die()  { printf '  !!   %s\n' "$*" >&2; exit 1; }
[ "$(id -u)" = 0 ] || die "run as root"
[ -f "$ENV_FILE" ] || die "$ENV_FILE is missing; this script is for a host migrate-to-foxing.sh has run on"
[ -f "$UNIT_FILE" ] || die "$UNIT_FILE is missing; this script is for a host migrate-to-foxing.sh has run on"

changes=0

echo "######## 1. $ENV_FILE"
# The new file: each MOONDIAL_KEY= line becomes FOXING_KEY=, unless a FOXING_KEY= line
# is there already (then the old line is dropped, and the FOXING_ value stands).
NEW_ENV=$(mktemp)
awk '
  /^FOXING_[A-Z0-9_]+=/ { split($0, a, "="); have[a[1]] = 1 }
  { lines[NR] = $0 }
  END {
    for (i = 1; i <= NR; i++) {
      l = lines[i]
      if (l ~ /^MOONDIAL_[A-Z0-9_]+=/) {
        split(l, a, "="); key = a[1]; sub(/^MOONDIAL_/, "FOXING_", key)
        if (have[key]) { continue }
        sub(/^MOONDIAL_/, "FOXING_", l)
      }
      gsub(/moondial\.service/, "foxing.service", l)
      gsub(/systemctl restart moondial/, "systemctl restart foxing", l)
      print l
    }
  }' "$ENV_FILE" > "$NEW_ENV"
if diff -u "$ENV_FILE" "$NEW_ENV" > /tmp/rename-env.diff; then
  ok "no MOONDIAL_* keys left in $ENV_FILE"
else
  cat /tmp/rename-env.diff
  changes=$((changes + 1))
fi

echo; echo "######## 2. $UNIT_FILE"
if grep -q '^Description=Foxing driver (Disremembered)' "$UNIT_FILE"; then
  ok "the unit's Description is already right"
else
  note "Description=$(grep '^Description=' "$UNIT_FILE" | cut -d= -f2-)  ->  Description=Foxing driver (Disremembered)"
  changes=$((changes + 1))
fi

echo; echo "######## 3. nginx"
HT_MOVE=0
if [ -f "$OLD_HT" ] && [ ! -f "$NEW_HT" ]; then
  note "$OLD_HT  ->  $NEW_HT"
  HT_MOVE=1; changes=$((changes + 1))
elif [ -f "$OLD_HT" ] && [ -f "$NEW_HT" ]; then
  note "both $OLD_HT and $NEW_HT exist; the new one stands, the old one is left for you to remove"
else
  ok "no $OLD_HT to rename"
fi
NGX_FILES=$(grep -rl 'htpasswd-moondial' "$NGINX_DIR" 2>/dev/null || true)
if [ -n "$NGX_FILES" ]; then
  for f in $NGX_FILES; do note "$f: htpasswd-moondial -> htpasswd-foxing"; done
  changes=$((changes + 1))
else
  ok "no nginx file names htpasswd-moondial"
fi

echo
if [ "$changes" = 0 ]; then
  echo "Nothing to do: the rename is complete on this host."
  rm -f "$NEW_ENV"
  exit 0
fi
if [ "${DRY_RUN:-0}" = 1 ]; then
  echo "DRY_RUN: $changes change(s) shown, nothing written."
  rm -f "$NEW_ENV"
  exit 0
fi
if [ "${YES:-0}" != 1 ]; then
  printf 'Apply the %s change(s) above, restart the driver and reload nginx? [y/N] ' "$changes"
  read -r answer
  case "$answer" in y|Y|yes|YES) ;; *) echo "nothing written"; rm -f "$NEW_ENV"; exit 0;; esac
fi

echo; echo "######## Writing"
if ! diff -q "$ENV_FILE" "$NEW_ENV" > /dev/null; then
  cp -p "$ENV_FILE" "$ENV_FILE.before-rename"
  # Same owner and mode as before: the file holds the admin token.
  install -m 640 -o root -g "$(stat -c %G "$ENV_FILE")" "$NEW_ENV" "$ENV_FILE"
  ok "$ENV_FILE rewritten (the old one is $ENV_FILE.before-rename)"
fi
rm -f "$NEW_ENV"
if ! grep -q '^Description=Foxing driver (Disremembered)' "$UNIT_FILE"; then
  sed -i 's|^Description=.*|Description=Foxing driver (Disremembered)|' "$UNIT_FILE"
  ok "unit Description set"
fi
if [ "$HT_MOVE" = 1 ]; then
  mv "$OLD_HT" "$NEW_HT" && ok "$NEW_HT in place"
fi
if [ -n "$NGX_FILES" ]; then
  for f in $NGX_FILES; do sed -i 's|htpasswd-moondial|htpasswd-foxing|g' "$f"; done
  if nginx -t > /dev/null 2>&1; then
    systemctl reload nginx && ok "nginx reloaded"
  else
    nginx -t || true
    die "nginx -t failed after the edit; the htpasswd file is at $NEW_HT; fix the snippet, then: systemctl reload nginx"
  fi
fi
systemctl daemon-reload
systemctl restart foxing
sleep 2
if curl -fsS --max-time 3 http://127.0.0.1:7777/health | grep -q '"ok":true'; then
  ok "foxing.service is up on the renamed env: $(curl -fsS http://127.0.0.1:7777/health)"
else
  echo "  !!   the driver did not answer /health; the old env is at $ENV_FILE.before-rename:"
  echo "       cp -p $ENV_FILE.before-rename $ENV_FILE && systemctl restart foxing"
  journalctl -u foxing -n 30 --no-pager || true
  exit 1
fi
echo "done. The MOONDIAL_* fallbacks in the driver and the tools can go in a later release."
