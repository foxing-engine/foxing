#!/usr/bin/env bash
# One-shot, run as root ON the game server: installs the Bindery's nightly run.
#
#   sudo bash /srv/foxing/current/deploy/setup-bindery.sh
#
# The Bindery reaches the Claude API by Workload Identity Federation, with this server
# as its own OIDC issuer: an RSA signing key is minted here and never leaves; its
# public half is what you register in the Claude Console (an inline JWKS). No API key
# exists anywhere. This script
#
#   - mints /etc/foxing/bindery-signing.pem (root:starsailor 640) if there is none,
#   - prints the JWK, issuer URL and subject to paste into the Console's
#     "Connect workload" wizard (Custom OIDC, inline JWKS), and the three env lines
#     to fill in with what the wizard hands back (the rule, org and service account
#     IDs: none of them secret),
#   - writes /etc/foxing/bindery.env (root:starsailor 640) with the Bindery's own
#     kept name and a passphrase minted here, the admin token from driver.env, the
#     model, and BINDERY_MODE=dry-run,
#   - installs bindery.service + bindery.timer: tools/bind.py from the current release
#   - installs docent.service: tools/docent.py, a daemon answering the Docent's questions
#     at 04:40 UTC, after the 04:17 backup.
#
# The first nights are dry runs (proposals in /var/lib/foxing/bindery/, a summary to
# Discord if the driver has a webhook). When they read right, set BINDERY_MODE=apply
# in the env file; nothing else changes. Re-running is safe: an existing key, env
# file and passphrase are kept; the JWK is printed again.
set -euo pipefail
ok() { printf '  ok  %s\n' "$*"; }
# Nothing here fails quietly: a failing line says which it was.
trap 'echo "  !!  setup-bindery.sh failed at line $LINENO: $BASH_COMMAND" >&2' ERR
echo "Setting up the Bindery's nightly run."

ENV_DIR=/etc/foxing
DRIVER_ENV=$ENV_DIR/driver.env
ENV_FILE=$ENV_DIR/bindery.env
KEY=$ENV_DIR/bindery-signing.pem
STATE=/var/lib/foxing/bindery
RELEASE=/srv/foxing/current
USER_NAME=starsailor
ISSUER=${BINDERY_ISSUER:-https://play.agentscanplay.com/bindery}   # compared as a string, never fetched
SUBJECT=${BINDERY_SUBJECT:-bindery@foxing}
[ -f "$DRIVER_ENV" ] || { echo "no $DRIVER_ENV: run migrate-to-foxing.sh first"; exit 1; }
id "$USER_NAME" >/dev/null 2>&1 || USER_NAME=$(stat -c %U "$RELEASE/foxing-driver" 2>/dev/null || echo root)

ADMIN=$(grep -oP '^(FOXING|MOONDIAL)_ADMIN_TOKEN=\K.*' "$DRIVER_ENV" | tail -n 1 || true)
HTTP=$(grep -oP '^(FOXING|MOONDIAL)_HTTP=\K.*' "$DRIVER_ENV" | tail -n 1 || true)
WEBHOOK=$(grep -oP '^(FOXING|MOONDIAL)_DISCORD_WEBHOOK=\K.*' "$DRIVER_ENV" | tail -n 1 || true)
[ -n "$ADMIN" ] || { echo "no admin token in $DRIVER_ENV"; exit 1; }

install -d -m 750 -o "$USER_NAME" -g "$USER_NAME" "$STATE"   # the Bindery writes its proposals here

# The signing key: born here, read by the Bindery's user, never printed.
if [ ! -f "$KEY" ]; then
  (umask 027; openssl genpkey -quiet -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "$KEY")
  chown root:"$USER_NAME" "$KEY"; chmod 640 "$KEY"
  ok "$KEY minted"
else
  ok "$KEY kept"
fi
# The public half as a JWK: the modulus from openssl, the exponent 65537 (AQAB), and a
# key id from the modulus's fingerprint, which is what the JWT header carries.
MODULUS=$(openssl rsa -in "$KEY" -noout -modulus)
MODULUS=${MODULUS#Modulus=}
KID=$(printf '%s' "$MODULUS" | sha256sum | cut -c1-16)
N=$(printf '%s' "$MODULUS" | python3 -c 'import sys,base64; print(base64.urlsafe_b64encode(bytes.fromhex(sys.stdin.read().strip())).rstrip(b"=").decode())')
# The Console takes the JWKS `keys` array (one key here), not the bare key.
JWK="[{\"kty\":\"RSA\",\"kid\":\"$KID\",\"n\":\"$N\",\"e\":\"AQAB\",\"alg\":\"RS256\",\"use\":\"sig\"}]"

if [ ! -f "$ENV_FILE" ]; then
  # head closing the pipe would make tr exit 141 under pipefail: read a fixed slice instead.
  PASS=$(head -c 64 /dev/urandom | base64 | tr -dc 'a-z0-9' | cut -c1-24)
  (umask 027; cat > "$ENV_FILE" <<ENV
# The Bindery's nightly run (tools/bind.py). Written by deploy/server/setup-bindery.sh.
# No API key lives here: the Bindery signs a JWT with $KEY and trades it for a
# short-lived token (Workload Identity Federation). Fill the three ANTHROPIC_* IDs
# from the Console's Connect-workload wizard; none of them is secret.
# BINDERY_MODE is dry-run until you switch it to apply.
BINDERY_MODE=dry-run
BINDERY_MODEL=claude-haiku-5-5
BINDERY_MAX=40
BINDERY_ACTOR=Bindery
BINDERY_PASSPHRASE=$PASS
BINDERY_OUT=$STATE/proposals.jsonl
NAMERY_OUT=$STATE/names.jsonl
EPITAPHS_OUT=$STATE/epitaphs.jsonl
DOCENT_OUT=$STATE/docent.jsonl
DOCENT_MODE=on
BINDERY_SIGNING_KEY=$KEY
BINDERY_ISSUER=$ISSUER
BINDERY_SUBJECT=$SUBJECT
BINDERY_KID=$KID
ANTHROPIC_FEDERATION_RULE_ID=
ANTHROPIC_ORGANIZATION_ID=
ANTHROPIC_SERVICE_ACCOUNT_ID=
ANTHROPIC_WORKSPACE_ID=
FOXING_BASE=http://${HTTP:-127.0.0.1:7777}
FOXING_ADMIN_TOKEN=$ADMIN
FOXING_DISCORD_WEBHOOK=$WEBHOOK
ENV
  )
  chown root:"$USER_NAME" "$ENV_FILE"; chmod 640 "$ENV_FILE"
  ok "$ENV_FILE written"
else
  ok "$ENV_FILE kept"
  grep -q '^NAMERY_OUT=' "$ENV_FILE" || { printf 'NAMERY_OUT=%s\n' "$STATE/names.jsonl" >> "$ENV_FILE"; ok "NAMERY_OUT added to $ENV_FILE"; }
  grep -q '^EPITAPHS_OUT=' "$ENV_FILE" || { printf 'EPITAPHS_OUT=%s\n' "$STATE/epitaphs.jsonl" >> "$ENV_FILE"; ok "EPITAPHS_OUT added to $ENV_FILE"; }
  grep -q '^DOCENT_OUT=' "$ENV_FILE" || { printf 'DOCENT_OUT=%s\nDOCENT_MODE=on\n' "$STATE/docent.jsonl" >> "$ENV_FILE"; ok "DOCENT_OUT and DOCENT_MODE added to $ENV_FILE"; }
  grep -q '^BINDERY_KID=' "$ENV_FILE" || { printf 'BINDERY_SIGNING_KEY=%s\nBINDERY_ISSUER=%s\nBINDERY_SUBJECT=%s\nBINDERY_KID=%s\nANTHROPIC_FEDERATION_RULE_ID=\nANTHROPIC_ORGANIZATION_ID=\nANTHROPIC_SERVICE_ACCOUNT_ID=\nANTHROPIC_WORKSPACE_ID=\n' "$KEY" "$ISSUER" "$SUBJECT" "$KID" >> "$ENV_FILE"; ok "federation lines added to $ENV_FILE"; }
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
# Then the Namery: stocked when its pool is low, whatever BINDERY_MODE says (the checks are exact).
ExecStart=/usr/bin/python3 $RELEASE/tools/names.py
# And the book of epitaphs, stocked when it runs low.
ExecStart=/usr/bin/python3 $RELEASE/tools/epitaphs.py
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

# The Docent: a daemon on the same way in, answering what is asked of the figure at
# the Threshold within the world's minute and a half. DOCENT_MODE=off in the env file
# keeps it from starting (the world then answers "not in the catalog tonight").
cat > /etc/systemd/system/docent.service <<UNIT
[Unit]
Description=The Docent: answers questions asked of the figure at the Threshold
After=network-online.target foxing.service
Wants=network-online.target

[Service]
Type=simple
User=$USER_NAME
EnvironmentFile=$ENV_FILE
WorkingDirectory=$STATE
ExecStart=/bin/sh -c 'if [ "\$DOCENT_MODE" = off ]; then echo "DOCENT_MODE=off; the Docent keeps silent"; exec sleep infinity; fi; exec /usr/bin/python3 $RELEASE/tools/docent.py'
Restart=on-failure
RestartSec=20
Nice=10
PrivateTmp=true
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
UNIT

# The deploy user may restart the Docent, so a release carries a new docent.py into it.
DEPLOY_USER=$(grep -h 'systemctl restart foxing' /etc/sudoers.d/* 2>/dev/null | head -1 | awk '{print $1}')
if [ -n "$DEPLOY_USER" ] && ! grep -qs 'systemctl restart docent' /etc/sudoers.d/docent; then
  printf '%s ALL=(root) NOPASSWD: /bin/systemctl restart docent, /bin/systemctl is-active docent\n' "$DEPLOY_USER" > /etc/sudoers.d/docent.tmp
  chmod 440 /etc/sudoers.d/docent.tmp
  if visudo -cf /etc/sudoers.d/docent.tmp >/dev/null; then mv /etc/sudoers.d/docent.tmp /etc/sudoers.d/docent; ok "sudoers: $DEPLOY_USER may restart docent"; else rm -f /etc/sudoers.d/docent.tmp; echo "  !!  sudoers line for docent did not check out; skipped"; fi
fi

systemctl daemon-reload
systemctl enable --now bindery.timer >/dev/null 2>&1
ok "bindery.timer enabled: $(systemctl list-timers bindery.timer --no-pager | sed -n 2p | awk '{print $1, $2, $3, $4}')"
if [ -f "$RELEASE/tools/docent.py" ]; then
  systemctl enable --now docent.service >/dev/null 2>&1 && ok "docent.service running (journalctl -u docent -f to watch it answer)" || echo "  !!  docent.service did not start; journalctl -u docent -n 20"
else
  echo "  !!  $RELEASE/tools/docent.py is not there yet; enable docent.service after the next release: sudo systemctl enable --now docent.service"
fi
[ -f "$RELEASE/tools/bind.py" ] && ok "tools/bind.py is in the current release" || echo "  !!  $RELEASE/tools/bind.py is not there yet; the next release stages it"

cat <<EOF

In the Claude Console, Settings > Workload identity > Connect workload > Custom OIDC:
  Issuer URL      $ISSUER
  JWKS            inline; paste this array (one key):
$JWK
  Rule match      subject_prefix: $SUBJECT      audience: https://api.anthropic.com
  Scope / lifetime  workspace:developer, 600 s (the wizard's defaults)
Then put what it hands back into $ENV_FILE:
  ANTHROPIC_FEDERATION_RULE_ID=fdrl_...   ANTHROPIC_ORGANIZATION_ID=...   ANTHROPIC_SERVICE_ACCOUNT_ID=svac_...
  (ANTHROPIC_WORKSPACE_ID only if the rule spans more than one workspace)
and, while the wizard is still listening for a first exchange, run a dry run:
  sudo systemctl start bindery.service; journalctl -u bindery -n 40 --no-pager
EOF
