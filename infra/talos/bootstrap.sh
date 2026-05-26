#!/usr/bin/env bash
# Talos multi-node bring-up on Apple Silicon (UTM-vz / Parallels).
#
# Run AFTER the VMs are created + booted off the Image Factory ISO (they idle in maintenance mode,
# awaiting config). Drives the full sequence: gen-config -> apply-config (per node) -> bootstrap
# (once) -> kubeconfig -> Cilium CNI -> Kata RuntimeClass -> smoke checks. Idempotent where it can be;
# the etcd bootstrap step runs exactly once (guarded). See README.md.
#
# Working files (configs + secrets) land in $WORKDIR (default ~/talos-mac) — NOT in git.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
CLUSTER_NAME="${CLUSTER_NAME:-oyatie-local}"
TALOS_VERSION="${TALOS_VERSION:-v1.13.2}"
SCHEMATIC_ID="$(cat "$HERE/.schematic-id")"
INSTALLER_REF="factory.talos.dev/installer/${SCHEMATIC_ID}:${TALOS_VERSION}"
WORKDIR="${WORKDIR:-$HOME/talos-mac}"
CILIUM_VERSION="${CILIUM_VERSION:-1.18.0}"

# Node IPs — normally supplied by create-cluster.sh (DHCP discovery on the Parallels Shared net,
# 10.211.55.0/24). VIP is a high static IP outside the sequential lease range.
VIP="${VIP:-10.211.55.240}"
read -ra CP_IPS    <<< "${CP_IPS:-10.211.55.11 10.211.55.12 10.211.55.13}"
read -ra WORKER_IPS<<< "${WORKER_IPS:-10.211.55.21 10.211.55.22}"

export TALOSCONFIG="$WORKDIR/talosconfig"
mkdir -p "$WORKDIR"

step() { printf '\n=== %s ===\n' "$*"; }

step "1/7 gen config (endpoint = VIP $VIP; installer = Kata/CLH-baked)"
if [ ! -f "$WORKDIR/controlplane.yaml" ]; then
  talosctl gen config "$CLUSTER_NAME" "https://${VIP}:6443" \
    --install-image "$INSTALLER_REF" --output-dir "$WORKDIR"
else
  echo "configs already generated in $WORKDIR (reusing; delete to regenerate)"
fi

step "2/7 apply-config to control-plane nodes (per-node IP, --insecure in maintenance mode)"
for ip in "${CP_IPS[@]}"; do
  echo "  -> cp $ip"
  talosctl apply-config --insecure -n "$ip" \
    --file "$WORKDIR/controlplane.yaml" --config-patch @"$HERE/controlplane.patch.yaml"
done

step "3/7 apply-config to worker nodes"
for ip in "${WORKER_IPS[@]}"; do
  echo "  -> worker $ip"
  talosctl apply-config --insecure -n "$ip" \
    --file "$WORKDIR/worker.yaml" --config-patch @"$HERE/worker.patch.yaml"
done

step "4/7 point talosctl at CP node IPs (NOT the VIP — VIP is for kubectl only)"
talosctl config endpoint "${CP_IPS[@]}"
talosctl config node "${CP_IPS[0]}"

step "4b/7 wait for nodes to install + reboot into configured Talos"
# apply-config triggers an install-to-disk + reboot a few seconds LATER. Require SUSTAINED
# reachability (3 consecutive secure-API hits) so we don't pass on the transient pre-reboot API.
for ip in "${CP_IPS[@]}" "${WORKER_IPS[@]}"; do
  streak=0
  for _ in $(seq 1 60); do
    if talosctl -n "$ip" version >/dev/null 2>&1; then streak=$((streak+1)); else streak=0; fi
    [ "$streak" -ge 3 ] && break
    sleep 5
  done
  [ "$streak" -ge 3 ] && echo "  $ip stable" || echo "  WARNING: $ip not stably reachable"
done

step "5/7 bootstrap etcd ONCE on ${CP_IPS[0]} (retry until etcd is up)"
if talosctl -n "${CP_IPS[0]}" etcd status >/dev/null 2>&1; then
  echo "etcd already bootstrapped — skipping"
else
  for i in $(seq 1 10); do
    talosctl bootstrap -n "${CP_IPS[0]}" 2>&1 | tail -1
    sleep 8
    talosctl -n "${CP_IPS[0]}" etcd status >/dev/null 2>&1 && { echo "  etcd up"; break; }
    echo "  retry bootstrap ($i)"
  done
fi

step "6/7 wait for k8s + pull kubeconfig (apiserver via VIP $VIP)"
talosctl health -n "${CP_IPS[0]}" --wait-timeout 10m || echo "(health not fully green yet — CNI still pending is expected before step 7)"
talosctl kubeconfig "$WORKDIR/kubeconfig" --force
export KUBECONFIG="$WORKDIR/kubeconfig"

step "7/7 Cilium CNI (sync-wave 0) + Kata RuntimeClass"
helm repo add cilium https://helm.cilium.io/ >/dev/null 2>&1 || true
helm repo update >/dev/null
helm upgrade --install cilium cilium/cilium --version "$CILIUM_VERSION" \
  --namespace kube-system -f "$HERE/cilium-values.yaml" --wait
kubectl apply -f "$HERE/kata-runtimeclass.yaml"

step "DONE — cluster summary"
kubectl get nodes -o wide
echo
echo "Next: validate Kata fidelity →  bash $HERE/smoke-kata.sh"
echo "KUBECONFIG=$WORKDIR/kubeconfig  TALOSCONFIG=$WORKDIR/talosconfig"
