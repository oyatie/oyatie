#!/usr/bin/env bash
# render.sh — produce the ClusterResourceSet bootstrap ConfigMaps (Cilium + Argo CD + root app)
# and apply them + the CRS to the MANAGEMENT cluster. CAPI then copies them into every cluster
# labelled oya.io/bootstrap=true at provision time.
#
# Needs: helm, kubectl, KUBECONFIG=<control-plane>. Run once on the control plane (or in CI) after `clusterctl init`.
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

echo ">> render Argo CD (namespace + controllers ONLY — CRDs are NOT bundled)"
# CRITICAL: a CRS ConfigMap is hard-capped at 1 MiB (etcd). The full Argo CD
# chart renders ~1.82 MiB and the `applicationsets.argoproj.io` CRD ALONE is
# ~1.32 MiB — so CRDs CANNOT travel through a CRS ConfigMap (not even split).
# Therefore: this ConfigMap carries ns + controllers only (`crds.install=false`,
# ~106 KiB, safely under cap); the Argo CD CRDs land on each spoke via Talos
# `cluster.extraManifests` (URL-fetched at bootstrap, bypassing the cap) — set
# in the spoke chart's defaults.argocdCrdManifests. The root Application CR
# ships via the separate Reconcile CRS, which retries until those CRDs exist.
{
  printf 'apiVersion: v1\nkind: Namespace\nmetadata:\n  name: argocd\n---\n'
  helm template argocd argo/argo-cd --version "$ARGOCD_VERSION" \
    --set crds.install=false -n argocd
} > "$OUT/argocd.yaml"

# Size guard: fail loudly if the controller render ever creeps over the cap.
ARGOCD_BYTES=$(/usr/bin/wc -c < "$OUT/argocd.yaml" | tr -d ' ')
if [ "$ARGOCD_BYTES" -ge 1048576 ]; then
  echo "ERROR: argocd render is $ARGOCD_BYTES bytes (>= 1 MiB ConfigMap cap). Trim values or move more to extraManifests." >&2
  exit 1
fi
echo "   argocd controller render: $ARGOCD_BYTES bytes (cap 1048576) — OK"

# The spoke `extraManifests` URLs MUST carry the CRDs for THIS chart's app
# version. Echo it so any drift between the chart pin and the CRD URLs is caught.
ARGOCD_APP_VERSION=$(helm show chart argo/argo-cd --version "$ARGOCD_VERSION" 2>/dev/null \
  | /usr/bin/awk '/^appVersion:/{gsub(/"/,"",$2); print $2}')
echo "   argo-cd chart $ARGOCD_VERSION -> appVersion ${ARGOCD_APP_VERSION:-<unknown>}"
if [ -z "${ARGOCD_APP_VERSION:-}" ]; then
  echo "ERROR: could not resolve argo-cd chart appVersion for chart $ARGOCD_VERSION" >&2
  exit 1
fi
ARGOCD_CRD_VERSIONS=$(
  { grep -Eo 'argoproj/argo-cd/v[0-9]+\.[0-9]+\.[0-9]+' "$REPO/infra/capi/clusters/values.yaml" || true; } \
    | sed 's#.*argo-cd/##' \
    | sort -u \
    | tr '\n' ' ' \
    | sed 's/[[:space:]]*$//'
)
if [ "$ARGOCD_CRD_VERSIONS" != "$ARGOCD_APP_VERSION" ]; then
  echo "ERROR: infra/capi/clusters/values.yaml defaults.argocdCrdManifests references Argo CD version(s) [${ARGOCD_CRD_VERSIONS:-none}], expected $ARGOCD_APP_VERSION for chart $ARGOCD_VERSION" >&2
  exit 1
fi
echo "   argo-cd CRD URLs match chart appVersion $ARGOCD_APP_VERSION"

# CRS resources are ConfigMaps whose data is the manifests CAPI applies to each cluster.
mk_cm() { kubectl create configmap "$1" -n "$NS" --from-file="$2=$3" \
            --dry-run=client -o yaml; }

echo ">> build + apply bootstrap ConfigMaps + CRS to the control plane"
{
  mk_cm cilium-bootstrap cilium.yaml  "$OUT/cilium.yaml";  echo '---'
  mk_cm argocd-bootstrap argocd.yaml  "$OUT/argocd.yaml";  echo '---'
  mk_cm argocd-root-app  root-app.yaml "$REPO/infra/gitops/root-app.yaml"
} > "$OUT/crs-configmaps.yaml"

sed -E "s/^([[:space:]]*)namespace: default$/\\1namespace: ${NS}/" \
  "$HERE/clusterresourceset.yaml" > "$OUT/clusterresourceset.yaml"

kubectl apply -f "$OUT/crs-configmaps.yaml"
kubectl apply -f "$OUT/clusterresourceset.yaml"
echo "DONE — label spoke Clusters with oya.io/bootstrap=true to receive the bootstrap."
echo "     Ordering: cilium + argocd install land via the ApplyOnce CRS; the root"
echo "     Application lands via the Reconcile CRS, which retries until Argo CD's"
echo "     Application CRD is established (no CR-before-CRD race)."
