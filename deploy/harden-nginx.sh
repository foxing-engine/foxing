#!/usr/bin/env bash
# The fixes from the HTTP error sweep of 11-28 that need root. Run once on EACH server
# (the game server and the site server), as root:
#
#     bash harden-nginx.sh            # shows what it would change, changes nothing
#     bash harden-nginx.sh --apply    # makes the changes, tests the config, reloads
#
# What it does, and why:
#  1. A default server that closes the connection (444) for any request not addressed
#     to one of our names. On the game server there was none: 18,000 requests in a
#     fortnight for the bare address and for other people's hostnames (open-proxy
#     probes, secret scanners, one sending 2,166 a minute) were handed to the game
#     itself. TLS for an unknown name is refused before a certificate is shown.
#  2. The probe snippet (dotfiles, .env, wp-login and the like closed with 444) in
#     every one of our server blocks that lacks it. agentscanplay.com had it;
#     foxing.dev, disremembered.com and play.agentscanplay.com answered each with a
#     404 page.
#  3. Strict-Transport-Security on our HTTPS blocks, and nginx's version off the
#     Server header.
# Every file it touches is copied to /var/backups/nginx-harden-<date>/ first; if `nginx -t` fails
# after the changes, every copy is put back and nginx is left as it was.
set -euo pipefail
APPLY=0; [ "${1:-}" = "--apply" ] && APPLY=1
[ "$(id -u)" = 0 ] || { echo "run as root"; exit 1; }
OURS='play\.agentscanplay\.com|agentscanplay\.com|foxing\.dev|foxingengine\.com|disremembered\.com'
STAMP=$(date -u +%Y%m%d-%H%M%S)
ETC=${NGINX_ETC:-/etc/nginx}   # (a rehearsal points this at a copy)
SN=$ETC/snippets
say() { echo "  $*"; }
# Copies go OUTSIDE nginx's tree: a copy left in sites-enabled would be loaded as a site.
BAK=/var/backups/nginx-harden-$STAMP
keep() { mkdir -p "$BAK$(dirname "$1")"; cp -p "$1" "$BAK$1"; changed+=("$1"); }
restore() { for f in "${changed[@]}"; do [ -f "$BAK$f" ] && cp -p "$BAK$f" "$f"; done; [ ${#made[@]} -gt 0 ] && rm -f "${made[@]}"; return 0; }
made=()
changed=()
write() { # write <path>  (content on stdin), only if different
  local tmp; tmp=$(mktemp); cat > "$tmp"
  if [ -f "$1" ] && cmp -s "$tmp" "$1"; then rm -f "$tmp"; say "ok already: $1"; return; fi
  say "$([ $APPLY = 1 ] && echo write || echo would write): $1"
  if [ $APPLY = 1 ]; then [ -f "$1" ] && keep "$1" || made+=("$1"); install -m 644 "$tmp" "$1"; fi
  rm -f "$tmp"
}

echo "## nginx $(nginx -v 2>&1 | sed 's/.*\///'), host $(hostname)"

echo "## 1. the default server"
if grep -rqsE 'listen[^;]*default_server' $ETC/sites-enabled $ETC/conf.d 2>/dev/null; then
  say "ok already: a default_server is declared in $(grep -rlsE 'listen[^;]*default_server' $ETC/sites-enabled $ETC/conf.d | tr '\n' ' ')"
  say "(check that it answers 444 and not a site: grep -A6 default_server on those files)"
else
  write $ETC/conf.d/00-default-server.conf <<'CONF'
# Anything not addressed to one of this box's names: closed, not answered.
server {
    listen 80 default_server;
    listen [::]:80 default_server;
    server_name _;
    access_log off;
    return 444;
}
server {
    listen 443 ssl default_server;
    listen [::]:443 ssl default_server;
    server_name _;
    access_log off;
    ssl_reject_handshake on;
}
CONF
fi

echo "## 2 and 3. the snippets"
write $SN/security-headers.conf <<'CONF'
# Browsers that have seen this site over HTTPS do not try plain HTTP for six months.
add_header Strict-Transport-Security "max-age=15552000" always;
CONF
write $ETC/conf.d/00-server-tokens.conf <<'CONF'
server_tokens off;
CONF
if [ ! -f $SN/harden-probes.conf ]; then
  src=$(ls /srv/foxing/current/deploy/nginx-snippets/harden-probes.conf /srv/agentscanplay/current/deploy/nginx-snippets/harden-probes.conf 2>/dev/null | head -1 || true)
  if [ -n "$src" ]; then write $SN/harden-probes.conf < "$src"; else say "MISSING: harden-probes.conf is not on this box (no release has it); step 2 is skipped"; fi
fi

echo "## our server blocks"
for f in $ETC/sites-enabled/*; do
  real=$(readlink -f "$f"); [ -f "$real" ] || continue
  grep -qE "server_name[^;]*($OURS)" "$real" || continue
  # In each server block that names one of ours: the includes it lacks, placed on the
  # line after its server_name. HSTS only where the block listens on 443.
  plan=$(awk -v ours="$OURS" -v have_probes="$([ -f $SN/harden-probes.conf ] || [ $APPLY = 0 ] && echo 1 || echo 0)" '
    function flush(   i, tls, mine, hp, hs, out) {
      if (n == 0) return
      for (i = 1; i <= n; i++) { if (b[i] ~ /listen[^;]*443/) tls = 1; if (b[i] ~ /server_name/ && b[i] ~ ours) mine = 1
        if (b[i] ~ /harden-probes\.conf/) hp = 1; if (b[i] ~ /security-headers\.conf|Strict-Transport-Security/) hs = 1; if (b[i] ~ /return 30[12]/ && b[i] !~ /location/) redirect = 1 }
      for (i = 1; i <= n; i++) { print b[i]
        if (mine && b[i] ~ /^[ \t]*server_name/ && !done) { done = 1
          if (!hp && have_probes) { print "    include snippets/harden-probes.conf;"; adds++ }
          if (!hs && tls) { print "    include snippets/security-headers.conf;"; adds++ } } }
      n = 0; done = 0
    }
    /^[ \t]*server[ \t]*\{/ { flush() }
    { b[++n] = $0 }
    END { flush(); print "#ADDS " adds + 0 > "/dev/stderr" }' "$real" 2> /tmp/harden-adds.$$)
  adds=$(sed 's/#ADDS //' /tmp/harden-adds.$$); rm -f /tmp/harden-adds.$$
  if [ "$adds" = 0 ]; then say "ok already: $real"; continue; fi
  say "$([ $APPLY = 1 ] && echo edit || echo would edit): $real ($adds include lines)"
  diff <(cat "$real") <(printf '%s\n' "$plan") | grep '^>' | sed 's/^/      /' || true
  if [ $APPLY = 1 ]; then keep "$real"; printf '%s\n' "$plan" > "$real"; fi
done

if [ $APPLY = 0 ]; then echo; echo "Nothing was changed. Run again with --apply to make these changes."; exit 0; fi
echo "## test and reload"
if nginx -t 2>&1 | sed 's/^/  /'; [ "${PIPESTATUS[0]}" = 0 ]; then
  systemctl reload nginx && echo "  reloaded. Copies of what was changed are under $BAK (to undo: copy them back and reload)."
else
  echo "  nginx -t FAILED: putting everything back."; restore; nginx -t 2>&1 | sed 's/^/  /'; exit 1
fi
