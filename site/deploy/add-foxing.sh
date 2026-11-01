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

certbot --nginx --non-interactive --agree-tos --redirect -m "$CERT_EMAIL" \
  -d foxing.dev -d www.foxing.dev -d foxingengine.com -d www.foxingengine.com \
  && ok "certificate issued; https redirect in place"

# moondial.ai: the redirect, on the certificate it already has.
install -m 644 "$DEPLOY/nginx-moondial.conf" /etc/nginx/sites-available/moondial
ln -sfn /etc/nginx/sites-available/moondial /etc/nginx/sites-enabled/moondial
nginx -t && systemctl reload nginx && ok "moondial.ai redirects to foxing.dev"

# The analytics log for the new site, if the collector is set up here.
[ -f /etc/nginx/.htpasswd-moondial ] && ok "analytics: add foxing to SITES in setup-analytics.sh's crontab if you want its log counted"
echo "done: https://foxing.dev"
