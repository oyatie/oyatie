#!/usr/bin/env bash
# up.sh — Sidero Metal/Talos bare-metal lane driver.
# Subcommands:
#   check                 read-only preflight for kubectl/clusterctl/talosctl and manifests.
#   bootstrap-mgmt        install CAPI + Talos + Sidero/Metal providers on the management cluster.
#   enroll                apply Sidero values and DHCP/TFTP bootstrap stubs.
#   up --role cp|worker   apply Cluster/Talos/Machine resources for the selected role.
#   down                  delete lane resources from the management cluster.
#   status                show Sidero/CAPI/Talos resource status.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
SIDERO_DIR="${SIDERO_DIR:-$ROOT/infra/sidero-metal}"
CAPI_DIR="${CAPI_DIR:-$SIDERO_DIR/capi}"
NAMESPACE="${NAMESPACE:-sidero-system}"
CLUSTER_NAME="${CLUSTER_NAME:-sidero-metal}"
HELM_RELEASE="${HELM_RELEASE:-sidero-metal}"
HELM_CHART="${HELM_CHART:-siderolabs/sidero}" # override when the repo/name changes upstream.
PROVIDER_SET="${PROVIDER_SET:-cluster-api talos-bootstrap talos-control-plane sidero-metal}"
DRY_RUN="${DRY_RUN:-0}"

log()  { printf '\n\033[1;34m==>\033[0m %s\n' "$*"; }
ok()   { printf '  \033[32m✓\033[0m %s\n' "$*"; }
warn() { printf '  \033[33m!\033[0m %s\n' "$*" >&2; }
die()  { printf '\n\033[31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || die "$1 is required"; }
run() {
  if [ "$DRY_RUN" = "1" ]; then printf 'DRY-RUN:'; printf ' %q' "$@"; printf '\n'; else "$@"; fi
}

manifest_files() {
  printf '%s\n' \
    "$SIDERO_DIR/values.yaml" \
    "$SIDERO_DIR/dhcp-tftp.yaml" \
    "$CAPI_DIR/cluster.yaml" \
    "$CAPI_DIR/talos-cp.yaml" \
    "$CAPI_DIR/metal-machine.yaml"
}

validate_yaml() {
  python3 - "$@" <<'PY'
import sys, yaml
for path in sys.argv[1:]:
    with open(path, "r", encoding="utf-8") as handle:
        list(yaml.safe_load_all(handle))
    print(f"ok yaml: {path}")
PY
}

cmd_check() {
  log "Preflight — Sidero Metal/Talos lane"
  need kubectl
  need python3
  command -v clusterctl >/dev/null 2>&1 && ok "clusterctl: $(command -v clusterctl)" || warn "clusterctl missing; bootstrap-mgmt will fail until installed"
  command -v talosctl >/dev/null 2>&1 && ok "talosctl: $(command -v talosctl)" || warn "talosctl missing; status will be Kubernetes-only"
  command -v helm >/dev/null 2>&1 && ok "helm: $(command -v helm)" || warn "helm missing; enroll will use kubectl-only stubs unless installed"
  while IFS= read -r file; do [ -f "$file" ] || die "missing manifest: $file"; done < <(manifest_files)
  validate_yaml $(manifest_files)
  ok "manifest stubs present and YAML-loadable"
}

cmd_bootstrap_mgmt() {
  log "Bootstrap management cluster providers"
  need clusterctl
  # Reuse the repo's canonical CAPI provider pins/config where possible.
  if [ -f "$ROOT/infra/capi/clusterctl.yaml" ]; then
    run clusterctl init --config "$ROOT/infra/capi/clusterctl.yaml" --infrastructure sidero --bootstrap talos --control-plane talos
  else
    run clusterctl init --infrastructure sidero --bootstrap talos --control-plane talos
  fi
  ok "CAPI/Talos/Sidero-compatible provider bootstrap requested"
}

cmd_enroll() {
  log "Install Sidero Metal enrollment stubs"
  need kubectl
  if [ "$DRY_RUN" = "1" ]; then
    echo "DRY-RUN: kubectl create+apply namespace/$NAMESPACE"
  else
    kubectl create ns "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
  fi
  if command -v helm >/dev/null 2>&1; then
    run helm upgrade --install "$HELM_RELEASE" "$HELM_CHART" --namespace "$NAMESPACE" --values "$SIDERO_DIR/values.yaml"
  else
    warn "helm unavailable; applying DHCP/TFTP ConfigMap only"
  fi
  run kubectl apply -f "$SIDERO_DIR/dhcp-tftp.yaml"
  ok "enrollment manifests applied/requested"
}

cmd_up() {
  local role=""
  while [ $# -gt 0 ]; do
    case "$1" in
      --role) role="${2:-}"; shift 2 ;;
      *) die "unknown up arg: $1" ;;
    esac
  done
  case "$role" in cp|worker) ;; *) die "usage: $0 up --role cp|worker" ;; esac
  need kubectl
  log "Apply $CLUSTER_NAME resources for role=$role"
  run kubectl apply -f "$CAPI_DIR/cluster.yaml"
  if [ "$role" = "cp" ]; then
    run kubectl apply -f "$CAPI_DIR/talos-cp.yaml"
    run kubectl apply -f "$CAPI_DIR/metal-machine.yaml"
  else
    run kubectl apply -f "$CAPI_DIR/metal-machine.yaml"
  fi
  ok "CAPI resources applied/requested for role=$role"
}

cmd_down() {
  need kubectl
  log "Delete $CLUSTER_NAME lane resources"
  run kubectl delete -f "$CAPI_DIR/metal-machine.yaml" --ignore-not-found=true
  run kubectl delete -f "$CAPI_DIR/talos-cp.yaml" --ignore-not-found=true
  run kubectl delete -f "$CAPI_DIR/cluster.yaml" --ignore-not-found=true
  run kubectl delete -f "$SIDERO_DIR/dhcp-tftp.yaml" --ignore-not-found=true
  ok "delete requested"
}

cmd_status() {
  need kubectl
  log "CAPI status"
  run kubectl get clusters,machines,machinedeployments -A -l "cluster.x-k8s.io/cluster-name=$CLUSTER_NAME" || true
  log "Sidero status"
  run kubectl get servers,serverclasses -A 2>/dev/null || warn "Sidero CRDs not installed or unavailable"
  if command -v talosctl >/dev/null 2>&1; then ok "talosctl present for node-level follow-up"; fi
}

case "${1:-}" in
  check) shift; cmd_check "$@" ;;
  bootstrap-mgmt) shift; cmd_bootstrap_mgmt "$@" ;;
  enroll) shift; cmd_enroll "$@" ;;
  up) shift; cmd_up "$@" ;;
  down) shift; cmd_down "$@" ;;
  status) shift; cmd_status "$@" ;;
  *) die "usage: $0 <check|bootstrap-mgmt|enroll|up|down|status> [args]" ;;
esac
