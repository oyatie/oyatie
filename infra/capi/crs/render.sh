#!/usr/bin/env bash
# render.sh — produce the ClusterResourceSet bootstrap ConfigMaps (Cilium + Argo CD + root app)
# and apply them + the CRS to the MANAGEMENT cluster. CAPI then copies them into every cluster
# labelled oya.io/bootstrap=true at provision time.
#
# Needs: helm, kubectl, KUBECONFIG=<hub>. Run once on the hub (or in CI) after `clusterctl init`.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../../.." && pwd)"
OUT="$HERE/_out"; mkdir -p "$OUT"
NS="${NS:-default}"
CILIUM_VERSION="${CILIUM_VERSION:-1.19.4}"
ARGOCD_VERSION="${ARGOCD_VERSION:-9.5.15}"   # argo/argo-cd chart (current)

command -v helm    >/dev/null || { echo "helm required" >&2; exit 1; }
command -v kubectl >/dev/null || { echo "kubectl required" >&2; exit 1; }

helm repo add cilium https://helm.cilium.io >/dev/null 2>&1 || true
helm repo add argo   https://argoproj.github.io/argo-helm >/dev/null 2>&1 || true
helm repo update cilium argo >/dev/null 2>&1 || true

echo ">> render Cilium (CNI)"
helm template cilium cilium/cilium --version "$CILIUM_VERSION" \
  -n kube-system -f "$REPO/infra/talos/cilium-values.yaml" > "$OUT/cilium.yaml"

echo ">> render Argo CD"
helm template argocd argo/argo-cd --version "$ARGOCD_VERSION" \
  -n argocd > "$OUT/argocd.yaml"

# CRS resources are ConfigMaps whose data is the manifests CAPI applies to each cluster.
mk_cm() { kubectl create configmap "$1" -n "$NS" --from-file="$2=$3" \
            --dry-run=client -o yaml; }

echo ">> build + apply bootstrap ConfigMaps + CRS to the hub"
{
  mk_cm cilium-bootstrap cilium.yaml  "$OUT/cilium.yaml";  echo '---'
  mk_cm argocd-bootstrap argocd.yaml  "$OUT/argocd.yaml";  echo '---'
  mk_cm argocd-root-app  root-app.yaml "$REPO/infra/gitops/root-app.yaml"
} > "$OUT/crs-configmaps.yaml"

kubectl apply -f "$OUT/crs-configmaps.yaml"
kubectl apply -f "$HERE/clusterresourceset.yaml"
echo "DONE — label spoke Clusters with oya.io/bootstrap=true to receive the bootstrap."
