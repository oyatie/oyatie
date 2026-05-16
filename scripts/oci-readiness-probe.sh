#!/usr/bin/env bash
# oci-readiness-probe.sh — read-only probes to prove OCI readiness for the
# non-Foundry services (auth/IAM, KMS, API gateway, OKE/compute, networking).
# Runs against the active OCI CLI profile. Captures one evidence JSON per probe
# under evidence/oci-readiness/.
#
# Prerequisite: `oci setup config` has been run (or instance principal /
# resource principal auth is configured), and `oci iam region list` returns 200.
set -euo pipefail

OCI=${OCI_BIN:-/home/oyatie/.local/bin/oci}
OUT_DIR=/home/oyatie/projects/oyatie/evidence/oci-readiness
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
DAY=$(date -u +%Y-%m-%d)

mkdir -p "$OUT_DIR"

probe() {
  local label=$1 ; shift
  local out="$OUT_DIR/${DAY}-${label}.json"
  echo "==> probe: $label"
  if "$OCI" "$@" --output json > "$out".tmp 2>"$out".err; then
    echo "    PASS → $out"
    mv "$out".tmp "$out"
    rm -f "$out".err
  else
    code=$?
    echo "    FAIL (exit $code) → $out.err"
    mv "$out".tmp "${out}.failed.json" 2>/dev/null || true
    return $code
  fi
}

# 1) Auth + tenancy resolution
probe regions iam region list

# 2) Compartment tree (audit identity scope)
probe compartments iam compartment list --compartment-id-in-subtree true --all

# 3) KMS — vault enumeration in tenancy compartment
# Caller may override with: COMPARTMENT_ID=... ./oci-readiness-probe.sh
COMPARTMENT_ID=${COMPARTMENT_ID:-$($OCI iam compartment list --query 'data[0].id' --raw-output 2>/dev/null || true)}
if [ -n "${COMPARTMENT_ID:-}" ]; then
  probe kms-vaults kms management vault list --compartment-id "$COMPARTMENT_ID"
else
  echo "    SKIP kms-vaults (no compartment id resolvable; set COMPARTMENT_ID env)"
fi

# 4) API Gateway — gateways in the chosen compartment
if [ -n "${COMPARTMENT_ID:-}" ]; then
  probe api-gateways api-gateway gateway list --compartment-id "$COMPARTMENT_ID" || true
fi

# 5) OKE clusters (Container Engine for Kubernetes — the GitOps target)
if [ -n "${COMPARTMENT_ID:-}" ]; then
  probe oke-clusters ce cluster list --compartment-id "$COMPARTMENT_ID" || true
fi

# 6) Compute (A1 instances per masterplan §7 OCI A1 → OKE stage)
if [ -n "${COMPARTMENT_ID:-}" ]; then
  probe compute-instances compute instance list --compartment-id "$COMPARTMENT_ID" || true
fi

# 7) Vault secrets (where any non-Foundry-service secret would live)
if [ -n "${COMPARTMENT_ID:-}" ]; then
  probe vault-secrets vault secret list --compartment-id "$COMPARTMENT_ID" || true
fi

echo
echo "==> readiness summary"
ls -la "$OUT_DIR" | tail -20
