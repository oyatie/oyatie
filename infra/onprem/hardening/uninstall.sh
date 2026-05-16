#!/usr/bin/env bash
# uninstall.sh — reverse of hardening/install.sh.
# Disables timers / services this script enabled and removes the per-script sysctl drop-in.
# Does NOT change `iptables` alternative (kubeadm depends on it) — that's reset by kubeadm/uninstall.sh.
# Does NOT remove fail2ban's ban records or smartmontools' historical SMART logs.
set -uo pipefail
systemctl disable --now zfs-scrub-monthly@oyatie-bulk.timer 2>/dev/null || true
systemctl disable --now smartmontools.service 2>/dev/null || true
systemctl disable --now zfs-zed.service 2>/dev/null || true
systemctl disable --now fail2ban 2>/dev/null || true
rm -f /etc/sysctl.d/99-oyatie-hardening.conf
sysctl --system >/dev/null
rm -f /etc/smartd.conf
echo "hardening sysctls/timers reverted (packages left installed — re-enable individually as needed)."
