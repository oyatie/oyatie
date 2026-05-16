#!/usr/bin/env bash
# enable-transit.sh — one-shot post-init: enable the OpenBao transit secrets
# engine that oya-cloud-kms-adapter-openbao (M03-P01-IP-001) uses for
# `cloud.kms.{encrypt,decrypt}`.
#
# Requires: OpenBao initialized + unsealed + admin token in BAO_TOKEN env.
# Idempotent — re-running is safe (returns 400 if already enabled, ignored).
set -uo pipefail
export BAO_ADDR=${BAO_ADDR:-http://127.0.0.1:8200}
[ -n "${BAO_TOKEN:-}" ] || { echo "ERROR: BAO_TOKEN env var required (admin token)"; exit 2; }

bao secrets enable -path=transit transit 2>/dev/null || \
  echo "  (transit already enabled — ok)"

# Seed key types we use today (per ADR-0043 KmsPurpose enum).
for key in oyatie-default oyatie-cloud-object-storage oyatie-workspace-drive-object oyatie-cross-region-replication; do
  bao write -f "transit/keys/$key" 2>&1 | head -1 || true
done

echo
echo "==> transit engine ready; available keys:"
bao list transit/keys 2>&1 | head -10
