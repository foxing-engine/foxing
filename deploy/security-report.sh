#!/usr/bin/env bash
# What this box owes in security upkeep. Read-only, needs no root; run over the
# deploy's SSH at the end of every deploy (both workflows), so the answer is in each
# run's log and its warnings are on the run's summary page. Never fails a deploy.
#   ssh host 'bash -s' < deploy/security-report.sh
warn() { printf '::warning title=Security (%s)::%s\n' "$(hostname)" "$*"; }
note() { printf '  %s\n' "$*"; }
echo "######## Security posture: $(hostname), $(date -u +%F\ %H:%M) UTC"
. /etc/os-release 2>/dev/null; note "os: ${PRETTY_NAME:-?}, kernel $(uname -r), up $(awk '{printf "%d days", $1/86400}' /proc/uptime)"

if [ -f /var/run/reboot-required ]; then
  warn "a restart is required to finish updates ($(tr '\n' ' ' < /var/run/reboot-required.pkgs 2>/dev/null | cut -c1-160))"
else note "restart: none pending"; fi

up=$(apt list --upgradable 2>/dev/null | grep -c '/' || true)
sec=$(apt list --upgradable 2>/dev/null | grep -ci -- '-security' || true)
if [ "${sec:-0}" -gt 0 ]; then warn "$sec security updates are waiting ($up updates in all)"
else note "updates: no security updates waiting ($up others)"; fi

if grep -qs 'Unattended-Upgrade "1"' /etc/apt/apt.conf.d/20auto-upgrades 2>/dev/null \
   && systemctl is-enabled --quiet unattended-upgrades 2>/dev/null; then
  if grep -qsE '^\s*Unattended-Upgrade::Automatic-Reboot\s+"true"' /etc/apt/apt.conf.d/*; then
    note "automatic security updates: on, with a restart at $(grep -hsE '^\s*Unattended-Upgrade::Automatic-Reboot-Time' /etc/apt/apt.conf.d/* | grep -oE '[0-9]{2}:[0-9]{2}' | tail -1) UTC when one is needed"
  else warn "automatic security updates are on but never restart the box, so kernel fixes wait for a hand (deploy/secure-updates.sh)"; fi
else warn "automatic security updates are off (deploy/secure-updates.sh turns them on)"; fi

if systemctl is-active --quiet fail2ban 2>/dev/null; then note "fail2ban: running"
else warn "fail2ban is not running: nothing slows a password guesser at sshd (deploy/secure-updates.sh)"; fi

# What sshd itself resolves, when this runs as root; else the files, read the way
# sshd reads them: the drop-ins first, in name order, and the FIRST value wins (this
# took the last, and on 11-20 could not see that moondial-1 accepted passwords: the
# setting was in no file, and sshd's own default is yes).
eff() {
  local v; v=$(sshd -T 2>/dev/null | awk -v k="$1" '$1==k{print $2; exit}')
  [ -n "$v" ] || v=$(cat /etc/ssh/sshd_config.d/*.conf /etc/ssh/sshd_config 2>/dev/null | awk -v k="$1" 'tolower($1)==k{print tolower($2); exit}')
  printf '%s' "$v"
}
pw=$(eff passwordauthentication)
case "$pw" in no) note "ssh: keys only";; yes) warn "ssh accepts passwords (PasswordAuthentication yes)";; *) warn "ssh: PasswordAuthentication is set in no readable file, and sshd's default is yes (set it to no in /etc/ssh/sshd_config.d/00-hardening.conf)";; esac
rl=$(eff permitrootlogin)
note "ssh: PermitRootLogin ${rl:-unset (the default is keys only)}"

# Certificates this box serves, by days left (certbot renews at 30).
for d in "$@"; do
  end=$(echo | timeout 8 openssl s_client -servername "$d" -connect 127.0.0.1:443 2>/dev/null | openssl x509 -noout -enddate 2>/dev/null | cut -d= -f2)
  [ -n "$end" ] || { warn "no certificate answered for $d"; continue; }
  days=$(( ( $(date -d "$end" +%s) - $(date +%s) ) / 86400 ))
  if [ "$days" -lt 21 ]; then warn "the certificate for $d has $days days left: renewal is not happening"
  else note "certificate $d: $days days left"; fi
done
exit 0
