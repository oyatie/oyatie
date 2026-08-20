#!/usr/bin/env bash
# init.sh — install Cluster API onto the Oyatie management (control plane) cluster.
#
# Run this AFTER the control plane is up (installation-media-installed Talos, kubeconfig in hand). No throwaway kind
# cluster needed — the installation-media-formed control plane IS the bootstrap target; clusterctl inits directly onto it.
#
#   export KUBECONFIG=<control-plane kubeconfig>           # from `talosctl kubeconfig` on the control plane
#   # OCI creds for CAPOCI (env or ~/.oci): OCI_TENANCY_ID, OCI_USER_ID, OCI_REGION, OCI_CREDENTIALS_FINGERPRINT, OCI_CREDENTIALS_KEY_B64
#   bash infra/capi/init.sh
#
# Installs: CAPI core + Talos bootstrap/control-plane + OCI + Metal3 infra providers.
# Then the control plane provisions spokes from git-committed Cluster CRs (infra/capi/clusters/).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
export CLUSTERCTL_CONFIG="${CLUSTERCTL_CONFIG:-$HERE/clusterctl.yaml}"

command -v clusterctl >/dev/null || { echo "clusterctl not on PATH — get it from github.com/kubernetes-sigs/cluster-api/releases" >&2; exit 1; }
: "${KUBECONFIG:?export KUBECONFIG to the control plane cluster}"
kubectl get nodes >/dev/null 2>&1 || { echo "control-plane cluster not reachable via KUBECONFIG" >&2; exit 1; }

# infra providers across the fleet substrates: OCI + AWS (cloud) + Metal3 (on-prem/colo bare metal).
# Cloud providers need creds (OCI: env/~/.oci; AWS: AWSCLIINI/AWS_B64ENCODED_CREDENTIALS).
INFRA="${INFRA:-oci:v0.24.0,aws:v2.11.1,metal3:v1.13.0}"

clusterctl init \
  --bootstrap     talos:v0.6.12 \
  --control-plane talos:v0.5.13 \
  --infrastructure "$INFRA"

echo
echo "CAPI installed on the control plane. Verify: clusterctl describe cluster <name>  /  kubectl get providers -A"
echo "Next: apply spoke Cluster CRs from infra/capi/clusters/ (git) — CAPI reconciles them."
