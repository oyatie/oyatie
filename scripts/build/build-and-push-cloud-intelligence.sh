#!/usr/bin/env bash
# Build and push cloud-intelligence through Buck2-native OCI assembly.
# Docker/BuildKit is intentionally not used; see ADR-0515 and
# specs/buck2-authority-policy.json.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

BUCK2="${BUCK2:-buck2}"
OCI_TARGET="${OYA_CLOUD_INTELLIGENCE_OCI_TARGET:-//cloud/cloud-intelligence/iac/oci:cloud-intelligence-oci}"
REGISTRY="${OYA_CLOUD_INTELLIGENCE_REGISTRY:-registry.oyatie.dev}"
IMAGE_REPO="${OYA_CLOUD_INTELLIGENCE_IMAGE_REPO:-oya-cloud-intelligence}"
CHART_YAML="${REPO_ROOT}/cloud/cloud-intelligence/iac/k8s/helm/Chart.yaml"
TAG="${OYA_CLOUD_INTELLIGENCE_TAG:-}"

if [[ -z "$TAG" ]]; then
  TAG="$(awk '/^version:/ {print $2; exit}' "$CHART_YAML" | tr -d '"')"
fi
if [[ -z "$TAG" ]]; then
  echo "ERROR: could not resolve image tag from $CHART_YAML" >&2
  exit 65
fi
command -v "$BUCK2" >/dev/null 2>&1 || { echo "ERROR: buck2 not found; build scripts must use Buck2, not Cargo or Docker" >&2; exit 127; }

echo "==> Buck2 build: ${OCI_TARGET}"
OCI_LAYOUT="$($BUCK2 build --show-full-simple-output "$OCI_TARGET" | tail -1)"
if [[ -z "$OCI_LAYOUT" || ! -d "$OCI_LAYOUT" ]]; then
  echo "ERROR: Buck2 did not return an OCI layout directory for ${OCI_TARGET}: ${OCI_LAYOUT:-<empty>}" >&2
  exit 66
fi

echo "==> Push OCI layout: ${REGISTRY}/${IMAGE_REPO}:${TAG}"
PUSH_FLAGS=()
if [[ -n "${OYA_CLOUD_INTELLIGENCE_PUSH_FLAGS:-}" ]]; then
  # Transitional operator escape hatch for flags such as --insecure. Keep the
  # build/push authority in Buck2; do not route through Python, Docker, or crane.
  read -r -a PUSH_FLAGS <<<"${OYA_CLOUD_INTELLIGENCE_PUSH_FLAGS}"
fi
"$BUCK2" run //tools/oci:oya-oci-push -- "$OCI_LAYOUT" "$REGISTRY" "$IMAGE_REPO" "$TAG" "${PUSH_FLAGS[@]}"

echo "==> Built and pushed via Buck2-native OCI"
echo "    target: ${OCI_TARGET}"
echo "    image:  ${REGISTRY}/${IMAGE_REPO}:${TAG}"
echo "    layout: ${OCI_LAYOUT}"
