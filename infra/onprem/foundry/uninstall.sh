#!/usr/bin/env bash
# uninstall.sh — reverse of foundry/install.sh (oyatie.service).
set -uo pipefail
PURGE=${PURGE:-0}
systemctl disable --now oyatie.service oyatie-restart.timer oyatie-restart.service 2>/dev/null || true
rm -f /etc/systemd/system/oyatie.service /etc/systemd/system/oyatie-restart.service /etc/systemd/system/oyatie-restart.timer
systemctl daemon-reload
rm -f /usr/local/bin/oya-ops-workspace-shell
rm -rf /etc/oyatie
if [ "$PURGE" = "1" ]; then
  rm -rf /var/lib/oyatie
  echo "  /var/lib/oyatie wiped (PURGE=1)"
else
  echo "  /var/lib/oyatie preserved (PURGE=1 to wipe)"
fi
userdel oya 2>/dev/null || true
echo "foundry workspace-shell uninstalled."
