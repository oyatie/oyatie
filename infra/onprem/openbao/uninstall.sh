#!/usr/bin/env bash
# uninstall.sh — reverse of openbao/install.sh.
# Default: preserves /srv/oyatie/openbao/data (user data). Pass PURGE=1 to wipe.
set -uo pipefail
PURGE=${PURGE:-0}
systemctl disable --now openbao.service 2>/dev/null || true
rm -f /etc/systemd/system/openbao.service
systemctl daemon-reload
dpkg -r openbao 2>/dev/null || true
rm -rf /etc/openbao
if [ "$PURGE" = "1" ]; then
  zfs destroy -r oyatie-bulk/srv/openbao 2>/dev/null || rm -rf /srv/oyatie/openbao
  rm -f /srv/oyatie/audit-chain/openbao-audit.jsonl
  echo "  data wiped (PURGE=1)"
else
  echo "  data preserved at /srv/oyatie/openbao (PURGE=1 to wipe)"
fi
userdel openbao 2>/dev/null || true
echo "openbao uninstalled."
