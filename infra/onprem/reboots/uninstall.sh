#!/usr/bin/env bash
# uninstall.sh — reverse of reboots/install.sh.
# Disables the weekly restart timer; foundry/uninstall.sh handles oyatie-restart.timer.
set -uo pipefail
systemctl disable --now oyatie-restart.timer 2>/dev/null || true
rm -f /etc/systemd/system/oyatie-restart.service /etc/systemd/system/oyatie-restart.timer
systemctl daemon-reload
echo "weekly restart timer uninstalled."
