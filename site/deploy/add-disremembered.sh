#!/usr/bin/env bash
# One-shot, run as root ON the VPS after DNS for disremembered.com and
# www.disremembered.com points here:
#   sudo CERT_EMAIL=you@example.com bash /srv/agentscanplay/current/deploy/add-disremembered.sh
# Installs the port-80 block for disremembered.com (the play page at its root) and gets
# one certificate for the names that resolve here with certbot, which adds the 443 blocks
# and the HTTPS redirect. Safe to rerun.
set -euo pipefail
APP_DIR=${APP_DIR:-/srv/agentscanplay}
CERT_EMAIL=${CERT_EMAIL:?set CERT_EMAIL}
DEPLOY="$APP_DIR/current/deploy"
ok() { printf '  ok  %s\n' "$*"; }

install -m 644 "$DEPLOY/nginx-disremembered.conf" /etc/nginx/sites-available/disremembered
ln -sfn /etc/nginx/sites-available/disremembered /etc/nginx/sites-enabled/disremembered
nginx -t && systemctl reload nginx && ok "disremembered.com answering on port 80"

ME=$(curl -fsS --max-time 5 https://api.ipify.org || curl -fsS --max-time 5 https://ifconfig.me)
ARGS=()
for d in disremembered.com www.disremembered.com; do
  got=$(getent ahostsv4 "$d" | awk '{print $1}' | sort -u | tr '\n' ' ')
  case " $got" in *" $ME "*) ARGS+=(-d "$d"); ok "$d -> here";; *) echo "  --  $d -> ${got:-nothing}; not this host, left off the certificate";; esac
done
[ ${#ARGS[@]} -gt 0 ] || { echo "no name resolves to this host ($ME); point DNS here first"; exit 1; }
if certbot --nginx --non-interactive --agree-tos --redirect -m "$CERT_EMAIL" "${ARGS[@]}"; then
  ok "certificate issued for:${ARGS[*]//-d/}; https redirect in place"
else
  echo "  --  certbot returned non-zero; check its output above"
fi

# The play page must be let in by the world: a signed-out page that loads and cannot
# reach the world is the sign that the driver does not yet allow this origin.
code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 8 -H 'Origin: https://disremembered.com' https://play.agentscanplay.com/health || true)
allow=$(curl -s -D - -o /dev/null --max-time 8 -H 'Origin: https://disremembered.com' https://play.agentscanplay.com/health | tr -d '\r' | awk -F': ' 'tolower($1)=="access-control-allow-origin"{print $2}')
[ "$allow" = "https://disremembered.com" ] && ok "the world allows this origin" || echo "  --  the world answered $code and does not allow https://disremembered.com yet (FOXING_CORS_ORIGINS on the game box, or deploy the driver)"
echo "done: https://disremembered.com"
