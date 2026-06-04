#!/usr/bin/env bash
# bring-up.sh — one command from a bare vfkit Talos cluster to the local Oyatie
# substrate: Cilium (CNI) -> local-path (storage) -> Forgejo (historical local SCM).
# GitHub/GitHub Actions is the temporary SCM/CI lane unlocker per ADR-0516; this
# helper no longer installs retired external CI controllers.
#
# Prereq: `infra/talos/local/talos-local.sh up --role single` has produced a
# kubeconfig at ${OYA_TALOS_WORKDIR:-$HOME/.oya/talos-local}/kubeconfig.
#
# Idempotent: every step uses create-or-apply / helm upgrade --install, so
# re-running converges rather than erroring. Validated live on Apple M5 Max /
# macOS 26.4 / Talos v1.13.3 / Kubernetes v1.36.1 / Cilium 1.19.4.
#
# This wrapper closes the Tier-1 gap from the bring-up inventory: previously the
# sequence was 6 scripts + UI steps. The 3 fixes baked in below were each found
# by an actual live boot, not by unit tests:
#   1. Talos /opt is read-only  -> local-path uses /var (see TALOS_LPP_PATH).
#   2. PodSecurity=baseline forbids the provisioner's hostPath helper pod
#      -> the provisioner namespace is labelled pod-security=privileged.
#   3. A PVC created before its StorageClass exists is stuck "immediate"
#      -> storage is installed BEFORE any workload PVC.
set -euo pipefail

# ── config ────────────────────────────────────────────────────────────────────
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
WORKDIR="${OYA_TALOS_WORKDIR:-$HOME/.oya/talos-local}"
export KUBECONFIG="${KUBECONFIG:-$WORKDIR/kubeconfig}"
CILIUM_VERSION="${CILIUM_VERSION:-1.19.4}"
LPP_VERSION="${LPP_VERSION:-v0.0.30}"          # rancher local-path-provisioner
TALOS_LPP_PATH="/var/local-path-provisioner"   # Talos /opt is read-only; /var persists

c_blue=$'\033[1;34m'; c_grn=$'\033[32m'; c_yel=$'\033[33m'; c_red=$'\033[31m'; c_rst=$'\033[0m'
log() { printf '%s==>%s %s\n' "$c_blue" "$c_rst" "$*"; }
ok()  { printf '  %s✓%s %s\n' "$c_grn" "$c_rst" "$*"; }
warn(){ printf '  %s!%s %s\n' "$c_yel" "$c_rst" "$*"; }
die() { printf '%sERROR:%s %s\n' "$c_red" "$c_rst" "$*" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || die "missing dependency: $1"; }
need kubectl; need helm; need curl

[ -r "$KUBECONFIG" ] || die "kubeconfig not found at $KUBECONFIG — run 'talos-local.sh up --role single' first"
kubectl get --raw='/readyz' >/dev/null 2>&1 || die "kube-apiserver not reachable via $KUBECONFIG"

# ── 1. Cilium CNI ───────────────────────────────────────────────────────────
cni() {
  log "Cilium $CILIUM_VERSION (CNI; cluster is cni:none like the fleet)"
  helm repo add cilium https://helm.cilium.io >/dev/null 2>&1 || true
  helm repo update cilium >/dev/null 2>&1 || true
  helm upgrade --install cilium cilium/cilium --version "$CILIUM_VERSION" \
    -n kube-system -f "$REPO/infra/talos/cilium-values.yaml" >/dev/null
  ok "cilium applied"
}

# ── 2. local-path storage (Talos-adapted, PSA-privileged) ─────────────────────
storage() {
  log "local-path storage ($LPP_VERSION, path=$TALOS_LPP_PATH)"
  local f; f="$(mktemp -t lpp.XXXXXX.yaml)"
  curl -fsSL "https://raw.githubusercontent.com/rancher/local-path-provisioner/$LPP_VERSION/deploy/local-path-storage.yaml" -o "$f" \
    || curl -fsSL "https://raw.githubusercontent.com/rancher/local-path-provisioner/master/deploy/local-path-storage.yaml" -o "$f" \
    || die "could not fetch local-path-provisioner manifest"
  # Fix 1: Talos /opt is read-only; provision under /var.
  sed -i.bak "s#/opt/local-path-provisioner#$TALOS_LPP_PATH#g" "$f" && rm -f "$f.bak"
  kubectl apply -f "$f" >/dev/null
  rm -f "$f"
  # Fix 2: the provisioner's helper pod uses hostPath, which PodSecurity=baseline
  # forbids; the provisioner is infrastructure, so its namespace is privileged.
  kubectl label namespace local-path-storage \
    pod-security.kubernetes.io/enforce=privileged \
    pod-security.kubernetes.io/warn=privileged \
    pod-security.kubernetes.io/audit=privileged --overwrite >/dev/null
  kubectl annotate storageclass local-path \
    storageclass.kubernetes.io/is-default-class=true --overwrite >/dev/null
  kubectl rollout status deployment/local-path-provisioner -n local-path-storage --timeout=120s
  ok "local-path is the default StorageClass"
}

# ── 3. Forgejo (source-of-truth SCM) ──────────────────────────────────────────
forgejo() {
  log "Forgejo (SCM)"
  kubectl apply -f "$REPO/infra/forge/forgejo.yaml" >/dev/null
  kubectl rollout status deployment/forgejo -n oya-forge --timeout=180s
  ok "Forgejo: svc forgejo.oya-forge:3000"
}

# ── access summary + remaining (human-auth) wiring ────────────────────────────
summary() {
  log "Local Oyatie substrate is up. Reach the remaining local UI from the host:"
  printf '  kubectl --kubeconfig %s -n oya-forge port-forward svc/forgejo 3000:3000\n' "$KUBECONFIG"
  warn "CI/CD is not bootstrapped here: use the temporary GitHub lane unlocker until native oya-ci/release-conveyor manifests are ready."
}

main() {
  case "${1:-all}" in
    cni) cni;; storage) storage;; forgejo) forgejo;; summary) summary;;
    all) cni; storage; forgejo; summary;;
    *) die "usage: $0 [all|cni|storage|forgejo|summary]";;
  esac
}
main "$@"
