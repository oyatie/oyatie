#!/usr/bin/env bash
# fleet-preflight.sh — non-mutating ADR-0375 fleet-bootstrap readiness preflight.
#
# Default mode is safe for local developer machines: it performs static shape
# checks and runs render-only Helm checks when Helm is present. Cloud CI should
# set FLEET_PREFLIGHT_STRICT=1 so missing render tools fail the pipeline.
#
#   make fleet-preflight
#   FLEET_PREFLIGHT_STRICT=1 FLEET_PREFLIGHT_OUT="$ARTIFACT_DIR/fleet-preflight" make fleet-preflight
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUT_DIR="${FLEET_PREFLIGHT_OUT:-$HERE/_out/fleet-preflight}"
STRICT="${FLEET_PREFLIGHT_STRICT:-0}"
REPORT="$OUT_DIR/report.txt"
CUT_LINES="$OUT_DIR/hardware-gated-cut-lines.txt"
STATUS=0

mkdir -p "$OUT_DIR"
: > "$REPORT"

log() { printf '%s\n' "$*" | tee -a "$REPORT"; }
ok() { log "OK   $*"; }
warn() { log "WARN $*"; }
fail() { log "FAIL $*"; STATUS=1; }

require_file() {
  local path="$1" label="$2"
  if [ -f "$ROOT/$path" ]; then ok "$label: $path"; else fail "$label missing: $path"; fi
}

require_grep() {
  local pattern="$1" path="$2" label="$3"
  if grep -Eq -- "$pattern" "$ROOT/$path"; then ok "$label"; else fail "$label (pattern not found in $path)"; fi
}

run_optional_helm() {
  local label="$1"; shift
  if command -v helm >/dev/null 2>&1; then
    log "RUN  $label: $*"
    if "$@" >> "$REPORT" 2>&1; then ok "$label"; else fail "$label failed"; fi
  elif [ "$STRICT" = "1" ]; then
    fail "$label requires helm in FLEET_PREFLIGHT_STRICT=1"
  else
    warn "$label skipped because helm is not installed; set FLEET_PREFLIGHT_STRICT=1 in cloud-ci to make this required"
  fi
}

log "ADR-0375 fleet-bootstrap readiness preflight (non-mutating)"
log "repo: $ROOT"
log "out : $OUT_DIR"
log "mode: $([ "$STRICT" = "1" ] && printf strict || printf local)"
log ""

require_file "Makefile" "operator Makefile"
require_file "infra/talos/installation-media/gen-media.sh" "Talos media generator"
require_file "infra/talos/installation-media/presets.yaml" "Talos media preset catalog"
require_file "infra/capi/clusterctl.yaml" "clusterctl provider pin file"
require_file "infra/capi/init.sh" "CAPI init entrypoint"
require_file "infra/capi/crs/render.sh" "CRS render entrypoint"
require_file "infra/capi/clusters/Chart.yaml" "spoke Helm chart"
require_file "infra/capi/clusters/values-example.yaml" "spoke render example values"
require_file "infra/gitops/Chart.yaml" "GitOps app-of-apps Helm chart"
require_file "infra/gitops/values.yaml" "GitOps app-of-apps values"
require_file "infra/gitops/root-app.yaml" "GitOps root Application"
log ""

log "Static shell syntax checks"
for script in \
  "infra/talos/installation-media/gen-media.sh" \
  "infra/capi/init.sh" \
  "infra/capi/crs/render.sh"; do
  if bash -n "$ROOT/$script"; then ok "bash -n $script"; else fail "bash -n $script"; fi
done
log ""

log "Talos installation-media command-shape checks"
require_grep "control-plane\)" "infra/talos/installation-media/gen-media.sh" "control-plane preset branch exists"
require_grep "node\)" "infra/talos/installation-media/gen-media.sh" "node preset branch exists"
require_grep "--embedded-config-path" "infra/talos/installation-media/gen-media.sh" "control-plane ISO bakes config offline"
require_grep "talos.config=" "infra/talos/installation-media/gen-media.sh" "node ISO fetches config via talos.config"
require_grep "control-plane:" "infra/talos/installation-media/presets.yaml" "control-plane preset declared"
require_grep "node:" "infra/talos/installation-media/presets.yaml" "node preset declared"
log "SHAPE Talos control-plane media: CONTROLPLANE_ENDPOINT=https://<cp-vip>:6443 infra/talos/installation-media/gen-media.sh control-plane"
log "SHAPE Talos node media: CONFIG_URL=https://join.oyatie.dev/config infra/talos/installation-media/gen-media.sh node"
log ""

log "CAPI provider pin/init command-shape checks"
require_grep "bootstrap-components.yaml" "infra/capi/clusterctl.yaml" "Talos bootstrap provider release URL declared"
require_grep "control-plane-components.yaml" "infra/capi/clusterctl.yaml" "Talos control-plane provider release URL declared"
require_grep "talos:v0\.6\.12" "infra/capi/init.sh" "CABPT pin in init.sh"
require_grep "talos:v0\.5\.13" "infra/capi/init.sh" "CACPPT pin in init.sh"
require_grep "oci:v0\.24\.0,aws:v2\.11\.1,metal3:v1\.13\.0" "infra/capi/init.sh" "infra provider pins in init.sh"
require_grep "clusterctl init" "infra/capi/init.sh" "clusterctl init entrypoint exists"
log "SHAPE CAPI init: KUBECONFIG=<hub-control-plane> infra/capi/init.sh"
log ""

log "CRS render/template shape checks"
require_grep "crds\.install=false" "infra/capi/crs/render.sh" "Argo CD controller render excludes oversized CRDs"
require_grep "--dry-run=client" "infra/capi/crs/render.sh" "CRS ConfigMap assembly is client-side renderable"
require_grep "kind: ClusterResourceSet" "infra/capi/crs/clusterresourceset.yaml" "ClusterResourceSet templates declared"
require_grep "argocdCrdManifests" "infra/capi/clusters/values.yaml" "Argo CD CRD URLs are declared for Talos extraManifests"
require_grep "cluster/extraManifests" "infra/capi/clusters/templates/clusters.yaml" "Talos extraManifests carries Argo CD CRD URLs"
if command -v helm >/dev/null 2>&1; then
  ARGOCD_CHART_VERSION=$(sed -n 's/^ARGOCD_VERSION=.*:-\([^}\"]*\).*/\1/p' "$ROOT/infra/capi/crs/render.sh")
  ARGOCD_APP_VERSION=$(helm show chart argo-cd --repo https://argoproj.github.io/argo-helm --version "$ARGOCD_CHART_VERSION" 2>> "$REPORT" \
    | /usr/bin/awk '/^appVersion:/{gsub(/"/,"",$2); print $2}')
  ARGOCD_CRD_VERSIONS=$(
    { grep -Eo 'argoproj/argo-cd/v[0-9]+\.[0-9]+\.[0-9]+' "$ROOT/infra/capi/clusters/values.yaml" || true; } \
      | sed 's#.*argo-cd/##' \
      | sort -u \
      | tr '\n' ' ' \
      | sed 's/[[:space:]]*$//'
  )
  if [ -n "$ARGOCD_CHART_VERSION" ] && [ -n "$ARGOCD_APP_VERSION" ] && [ "$ARGOCD_CRD_VERSIONS" = "$ARGOCD_APP_VERSION" ]; then
    ok "Argo CD chart $ARGOCD_CHART_VERSION appVersion $ARGOCD_APP_VERSION matches Talos extraManifests CRD URLs"
  else
    fail "Argo CD chart appVersion/extraManifests drift (chart=${ARGOCD_CHART_VERSION:-unknown}, appVersion=${ARGOCD_APP_VERSION:-unknown}, crdUrls=${ARGOCD_CRD_VERSIONS:-none})"
  fi
elif [ "$STRICT" = "1" ]; then
  fail "Argo CD chart appVersion/extraManifests alignment requires helm in FLEET_PREFLIGHT_STRICT=1"
else
  warn "Argo CD chart appVersion/extraManifests alignment skipped because helm is not installed; render.sh enforces it before CRS apply"
fi
if command -v helm >/dev/null 2>&1 && command -v kubectl >/dev/null 2>&1; then
  log "RUN  CRS dry render: infra/capi/crs/render.sh --dry-run"
  if CRS_RENDER_OUT="$OUT_DIR/crs" bash "$ROOT/infra/capi/crs/render.sh" --dry-run >> "$REPORT" 2>&1; then
    ok "CRS dry render wrote $OUT_DIR/crs"
  else
    fail "CRS dry render failed"
  fi
elif [ "$STRICT" = "1" ]; then
  fail "CRS dry render requires helm and kubectl in FLEET_PREFLIGHT_STRICT=1"
else
  warn "CRS dry render skipped because helm or kubectl is missing; static shape checks still ran"
fi
log ""

log "Spoke CAPI Helm render shape"
require_grep "kind: TalosControlPlane" "infra/capi/clusters/templates/clusters.yaml" "TalosControlPlane template present"
require_grep "kind: MachineDeployment" "infra/capi/clusters/templates/clusters.yaml" "MachineDeployment template present"
require_grep "oya\.io/bootstrap" "infra/capi/clusters/templates/clusters.yaml" "CRS bootstrap label rendered on clusters"
run_optional_helm "helm lint infra/capi/clusters" helm lint "$ROOT/infra/capi/clusters"
if command -v helm >/dev/null 2>&1; then
  log "RUN  helm template oya-spokes"
  if helm template oya-spokes "$ROOT/infra/capi/clusters" -f "$ROOT/infra/capi/clusters/values-example.yaml" > "$OUT_DIR/spokes.yaml" 2>> "$REPORT"; then
    ok "spoke render wrote $OUT_DIR/spokes.yaml"
  else
    fail "spoke render failed"
  fi
fi
log ""

log "Per-cell GitOps app-of-apps render shape"
require_grep "kind: Application" "infra/gitops/templates/applications.yaml" "Argo CD Application template present"
require_grep "argocd\.argoproj\.io/sync-wave" "infra/gitops/templates/applications.yaml" "sync-wave annotations rendered"
require_grep "root" "infra/gitops/root-app.yaml" "root Application present"
run_optional_helm "helm lint infra/gitops" helm lint "$ROOT/infra/gitops"
if command -v helm >/dev/null 2>&1; then
  log "RUN  helm template oya-platform"
  if helm template oya-platform "$ROOT/infra/gitops" > "$OUT_DIR/gitops-apps.yaml" 2>> "$REPORT"; then
    ok "GitOps render wrote $OUT_DIR/gitops-apps.yaml"
  else
    fail "GitOps render failed"
  fi
fi
log ""

cat > "$CUT_LINES" <<'CUTLINES'
ADR-0375 hardware-gated cut lines preserved by fleet-preflight:

1. Talos media build/write/boot is hardware-gated. The preflight records the
   command shape but does not run talosctl, docker, curl, imager, dd, or boot nodes.
2. CAPI provider install is hub-kubeconfig-gated. The preflight records the
   `KUBECONFIG=<hub> infra/capi/init.sh` shape but does not run clusterctl init.
3. CRS bootstrap is cluster-mutating when applied. The preflight allows only
   `infra/capi/crs/render.sh --dry-run` and never runs kubectl apply.
4. Spoke CAPI resources are cluster-mutating when applied. The preflight may run
   `helm template` to produce `spokes.yaml` but never pipes it to kubectl apply.
5. Per-cell GitOps app-of-apps is render-only here. Argo CD live reconciliation
   waits for a real hub/spoke cluster and future fleet-bootstrap run evidence.
6. `make install` remains Cloudflare-edge only (`install: apply`); fleet bootstrap
   is a separate ADR-0375 path, not part of OpenTofu edge install.

Future live bootstrap evidence expected once a hub kubeconfig exists:
- control-plane Talos ISO checksum + custody/shred evidence (not the ISO itself)
- node Talos Image Factory schematic id + checksum
- clusterctl init provider readiness: `kubectl get providers -A` and `clusterctl describe cluster <name>`
- CRS rendered artifacts and `kubectl apply` audit-chain row against the hub
- spoke Helm render artifact plus server-side dry-run/apply transcript
- per-cell Argo CD root/app-of-apps render plus Argo CD sync health transcript
CUTLINES
ok "hardware-gated cut-line artifact wrote $CUT_LINES"
log ""

log "Cloud-CI/readiness pipeline entrypoint"
log "  FLEET_PREFLIGHT_STRICT=1 FLEET_PREFLIGHT_OUT=\$ARTIFACT_DIR/fleet-preflight make fleet-preflight"
log "Expected artifacts: report.txt, hardware-gated-cut-lines.txt, and when Helm is present: crs/, spokes.yaml, gitops-apps.yaml"
log ""

if [ "$STATUS" -ne 0 ]; then
  log "fleet-preflight FAILED"
  exit "$STATUS"
fi

log "fleet-preflight passed without cluster/cloud mutation"
