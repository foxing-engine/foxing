#!/usr/bin/env bash
# One-shot, run as root ON the VPS after DNS for foxing.dev, www.foxing.dev,
# foxingengine.com and www.foxingengine.com points here:
#   sudo CERT_EMAIL=you@example.com bash /srv/agentscanplay/current/deploy/add-foxing.sh
# Installs the port-80 block for foxing.dev (the site itself) and the redirects, gets one
# certificate for the four names with certbot (which adds the 443 blocks and the HTTPS
# redirect), then turns moondial.ai into a redirect to foxing.dev on its existing
# certificate. Safe to rerun.
set -euo pipefail
APP_DIR=${APP_DIR:-/srv/agentscanplay}
CERT_EMAIL=${CERT_EMAIL:?set CERT_EMAIL}
DEPLOY="$APP_DIR/current/deploy"
ok() { printf '  ok  %s\n' "$*"; }

install -m 644 "$DEPLOY/nginx-foxing.conf" /etc/nginx/sites-available/foxing
ln -sfn /etc/nginx/sites-available/foxing /etc/nginx/sites-enabled/foxing
nginx -t && systemctl reload nginx && ok "foxing.dev answering on port 80"

# One certificate for every name that actually points here. foxingengine.com may be on
# the registrar's own forwarding (it 301s to foxing.dev from there), in which case it
# does not resolve to this host and cannot be on the certificate; it is left out.
ME=$(curl -fsS --max-time 5 https://api.ipify.org || curl -fsS --max-time 5 https://ifconfig.me)
ARGS=()
for d in foxing.dev www.foxing.dev foxingengine.com www.foxingengine.com; do
  got=$(getent ahostsv4 "$d" | awk '{print $1}' | sort -u | tr '\n' ' ')
  case " $got" in *" $ME "*) ARGS+=(-d "$d"); ok "$d -> here";; *) echo "  --  $d -> ${got:-nothing}; not this host, left off the certificate";; esac
done
[ ${#ARGS[@]} -gt 0 ] || { echo "no name resolves to this host ($ME); check DNS"; exit 1; }
if certbot --nginx --non-interactive --agree-tos --redirect -m "$CERT_EMAIL" "${ARGS[@]}"; then
  ok "certificate issued for:${ARGS[*]//-d/}; https redirect in place"
else
  echo "  --  certbot returned non-zero; if foxing.dev answers over https anyway, carry on (the redirect below still goes in)"
fi

# moondial.ai: the redirect, on the certificate it already has.
install -m 644 "$DEPLOY/nginx-moondial.conf" /etc/nginx/sites-available/moondial
ln -sfn /etc/nginx/sites-available/moondial /etc/nginx/sites-enabled/moondial
nginx -t && systemctl reload nginx && ok "moondial.ai redirects to foxing.dev"

# The analytics log for the new site, if the collector is set up here.
[ -f /etc/nginx/.htpasswd-foxing ] || [ -f /etc/nginx/.htpasswd-moondial ] && ok "analytics: add foxing to SITES in setup-analytics.sh's crontab if you want its log counted"
echo "done: https://foxing.dev"
