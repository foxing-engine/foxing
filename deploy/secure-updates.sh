#!/usr/bin/env bash
# Security upkeep that looks after itself. Run as root, once, on each box; safe to rerun:
#   sudo bash /srv/agentscanplay/current/deploy/secure-updates.sh      (the site's server)
#   sudo bash /srv/foxing/current/deploy/secure-updates.sh             (the game box)
# - installs what is waiting now
# - automatic security updates every day
# - when an update needs a restart, the box restarts itself at REBOOT_TIME (UTC;
#   default 10:30, which is half past two in the morning Pacific). The world comes back
#   by itself in about two minutes. REBOOT_TIME=never leaves restarts to a person.
# - fail2ban on sshd
set -euo pipefail
REBOOT_TIME=${REBOOT_TIME:-10:30}
ok() { printf '  ok  %s\n' "$*"; }
[ "$(id -u)" = 0 ] || { echo "run as root"; exit 1; }
export DEBIAN_FRONTEND=noninteractive

echo "######## What is waiting"
apt-get update -q >/dev/null
apt-get -y -q -o Dpkg::Options::=--force-confdef -o Dpkg::Options::=--force-confold upgrade | tail -3
ok "updates installed"

echo "######## Automatic security updates"
apt-get install -y -q unattended-upgrades >/dev/null
cat > /etc/apt/apt.conf.d/20auto-upgrades <<'CONF'
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
APT::Periodic::AutocleanInterval "7";
CONF
if [ "$REBOOT_TIME" = never ]; then
  rm -f /etc/apt/apt.conf.d/52foxing-reboot
  ok "on; restarts are left to a person"
else
  cat > /etc/apt/apt.conf.d/52foxing-reboot <<CONF
Unattended-Upgrade::Automatic-Reboot "true";
Unattended-Upgrade::Automatic-Reboot-WithUsers "true";
Unattended-Upgrade::Automatic-Reboot-Time "$REBOOT_TIME";
Unattended-Upgrade::Remove-Unused-Kernel-Packages "true";
CONF
  ok "on; a needed restart happens at $REBOOT_TIME UTC"
fi
systemctl enable --now unattended-upgrades >/dev/null

echo "######## fail2ban on sshd"
apt-get install -y -q fail2ban >/dev/null
[ -f /etc/fail2ban/jail.d/sshd.local ] || printf '[sshd]\nenabled = true\nmaxretry = 5\nbantime = 1h\n' > /etc/fail2ban/jail.d/sshd.local
systemctl enable --now fail2ban >/dev/null && ok "running"

echo
if [ -f /var/run/reboot-required ]; then
  echo "A restart is needed to finish (a new kernel or library). It will happen by itself at $REBOOT_TIME UTC, or now with: reboot"
else
  echo "Nothing needs a restart."
fi
