#!/usr/bin/env bash
# uninstall.sh — reverse of cleanup/install.sh (just removes the timer).
set -uo pipefail
systemctl disable --now oyatie-cleanup.timer 2>/dev/null || true
rm -f /etc/systemd/system/oyatie-cleanup.service /etc/systemd/system/oyatie-cleanup.timer
systemctl daemon-reload
echo "cleanup timer uninstalled."
