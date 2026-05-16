#!/usr/bin/env bash
# setup-reboots.sh — install unattended-upgrades and configure auto-reboot at 04:00
# only when an update actually requires it. Idempotent.
set -euo pipefail

sudo apt-get install -y unattended-upgrades

# 50unattended-upgrades: which origins to apply (security + updates).
# Debian 13 ships a sensible default; we just enable the auto-reboot lines.
sudo tee /etc/apt/apt.conf.d/52oyatie-auto-reboot > /dev/null <<'EOF'
// Managed by ~/setup-reboots.sh — overrides the Debian defaults below.
Unattended-Upgrade::Automatic-Reboot "true";
Unattended-Upgrade::Automatic-Reboot-WithUsers "true";
Unattended-Upgrade::Automatic-Reboot-Time "04:00";
EOF

# 20auto-upgrades: turn on the periodic update check + unattended run.
sudo tee /etc/apt/apt.conf.d/20auto-upgrades > /dev/null <<'EOF'
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
APT::Periodic::AutocleanInterval "7";
EOF

echo "=== unattended-upgrades config ==="
sudo grep -E '^(Unattended-Upgrade::Automatic-Reboot|APT::Periodic::)' \
  /etc/apt/apt.conf.d/52oyatie-auto-reboot \
  /etc/apt/apt.conf.d/20auto-upgrades

echo
echo "=== timer status ==="
systemctl list-timers apt-daily-upgrade.timer apt-daily.timer --no-pager

echo
echo "=== dry-run (does not install anything; shows what would happen) ==="
sudo unattended-upgrade --dry-run --debug 2>&1 | tail -25
