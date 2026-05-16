#!/usr/bin/env bash
# uninstall.sh — reverse of security/install.sh.
set -uo pipefail
systemctl disable --now oyatie-security-scan.timer 2>/dev/null || true
rm -f /etc/systemd/system/oyatie-security-scan.service /etc/systemd/system/oyatie-security-scan.timer
rm -f /usr/local/bin/oyatie-security-scan
systemctl daemon-reload

systemctl disable --now unattended-upgrades 2>/dev/null || true
apt-get -y remove --purge unattended-upgrades apt-listchanges debsecan trivy 2>/dev/null || true
rm -f /etc/apt/apt.conf.d/50unattended-upgrades-oyatie /etc/apt/apt.conf.d/20auto-upgrades
rm -f /etc/apt/sources.list.d/trivy.list /etc/apt/keyrings/trivy.gpg
rm -f /usr/local/bin/gitleaks

if [ "${PURGE:-0}" = "1" ]; then
  rm -rf /var/log/oyatie-security
fi

REAL_USER=${SUDO_USER:-oyatie}
sudo -u "$REAL_USER" -H bash -lc 'source ~/.cargo/env 2>/dev/null; cargo uninstall cargo-audit 2>/dev/null' || true

echo "security stack uninstalled."
