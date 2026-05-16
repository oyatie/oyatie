#!/usr/bin/env bash
# uninstall-all.sh — reverse of setup.sh. Removes every installed component
# in reverse dependency order. Authority: ADR-0119.
#
# Default: preserves user-data (audit-chain, vault data, ZFS snapshots).
# Pass --purge to also wipe data dirs.
#
# Usage: sudo bash /home/oyatie/projects/oyatie/infra/onprem/uninstall-all.sh [--purge]
set -uo pipefail

PURGE=${PURGE:-0}
[ "${1:-}" = "--purge" ] && PURGE=1

HERE=$(cd "$(dirname "$0")" && pwd)
banner () { printf "\n══════════════════════════════════════════════════════════════════\n  %s\n══════════════════════════════════════════════════════════════════\n\n" "$*"; }

# Reverse dependency order — the inverse of setup.sh.
COMPONENTS=(
  cloudflared
  istio
  kubeadm
  containerd
  podman
  security
  openbao
  foundry
  reboots
  sanoid
  cleanup
  hardening
)

for c in "${COMPONENTS[@]}"; do
  if [ -x "$HERE/$c/uninstall.sh" ]; then
    banner "uninstall $c"
    PURGE=$PURGE bash "$HERE/$c/uninstall.sh" || echo "  (warning: $c uninstall returned non-zero — continuing)"
  else
    echo "  → $c: no uninstall.sh (skipping)"
  fi
done

banner "uninstall-all complete"
echo "If --purge was set, user-data dirs are also wiped."
echo "Re-install: sudo bash /home/oyatie/projects/oyatie/infra/onprem/setup.sh"
