#!/usr/bin/env bash
# build-and-push-cloud-intelligence.sh — build, sign, and push the cloud-intelligence
# container image to registry.oyatie.dev.
#
# Usage (from repo root):
#   ./scripts/build/build-and-push-cloud-intelligence.sh
#
# Prerequisites:
#   - docker (or nerdctl) on PATH
#   - cosign on PATH (keyless OIDC signing per ADR-0181; cluster cosign-key secret
#     mounted at /workspace/oya-ci/cosign-key for key-based fallback in CI)
#   - push access to registry.oyatie.dev (oya-registry namespace on Talos)
#
# ADR-0039: supply-chain-security (cosign + SBOM + Trivy)
# ADR-0181: container-image-promotion-pipeline (dev-tier signing)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHART_YAML="${REPO_ROOT}/microservices/cloud-intelligence/iac/k8s/helm/Chart.yaml"
DOCKERFILE="${REPO_ROOT}/microservices/cloud-intelligence/Dockerfile"
REGISTRY="registry.oyatie.dev"
IMAGE_REPO="oya-cloud-intelligence"

# ── Resolve version from Chart.yaml ──────────────────────────────────────────
if [[ ! -f "${CHART_YAML}" ]]; then
  echo "ERROR: Chart.yaml not found at ${CHART_YAML}" >&2
  exit 1
fi
VERSION="$(grep '^version:' "${CHART_YAML}" | head -1 | awk '{print $2}' | tr -d '"')"
if [[ -z "${VERSION}" ]]; then
  echo "ERROR: Could not extract version from ${CHART_YAML}" >&2
  exit 1
fi
IMAGE_TAG="${REGISTRY}/${IMAGE_REPO}:${VERSION}"
echo "==> Building image: ${IMAGE_TAG}"

# ── Select container runtime ──────────────────────────────────────────────────
if command -v nerdctl &>/dev/null; then
  DOCKER_CMD="nerdctl"
elif command -v docker &>/dev/null; then
  DOCKER_CMD="docker"
else
  echo "ERROR: neither docker nor nerdctl found on PATH" >&2
  exit 1
fi

# ── Build ─────────────────────────────────────────────────────────────────────
echo "==> Running: ${DOCKER_CMD} build"
"${DOCKER_CMD}" build \
  -f "${DOCKERFILE}" \
  -t "${IMAGE_TAG}" \
  "${REPO_ROOT}"

# ── Push ──────────────────────────────────────────────────────────────────────
echo "==> Pushing: ${IMAGE_TAG}"
"${DOCKER_CMD}" push "${IMAGE_TAG}"

# ── Resolve digest ────────────────────────────────────────────────────────────
echo "==> Resolving digest"
DIGEST="$("${DOCKER_CMD}" inspect --format='{{index .RepoDigests 0}}' "${IMAGE_TAG}" 2>/dev/null \
  | sed 's/.*@//' \
  || true)"

if [[ -z "${DIGEST}" ]]; then
  # Fallback: pull the manifest digest from the registry
  DIGEST="$("${DOCKER_CMD}" manifest inspect "${IMAGE_TAG}" 2>/dev/null \
    | grep -o '"digest":"sha256:[^"]*"' | head -1 | sed 's/"digest":"//;s/"//' \
    || true)"
fi

if [[ -z "${DIGEST}" ]]; then
  echo "WARNING: Could not resolve digest automatically. Run:" >&2
  echo "  ${DOCKER_CMD} inspect --format='{{index .RepoDigests 0}}' ${IMAGE_TAG}" >&2
  DIGEST="sha256:<run-inspect-to-get-digest>"
fi

# ── Cosign sign (ADR-0181 dev-tier keyless) ───────────────────────────────────
echo "==> Signing image with cosign (ADR-0181)"
if command -v cosign &>/dev/null; then
  # Prefer keyless OIDC (Sigstore Fulcio) — no long-lived key material (ADR-0043).
  # For local/offline use, fall back to the cluster cosign key if present.
  COSIGN_KEY="${COSIGN_KEY_PATH:-/workspace/oya-ci/cosign-key}"
  if [[ -f "${COSIGN_KEY}" ]]; then
    cosign sign --key "${COSIGN_KEY}" "${REGISTRY}/${IMAGE_REPO}@${DIGEST}"
  else
    cosign sign "${REGISTRY}/${IMAGE_REPO}@${DIGEST}"
  fi
else
  echo "WARNING: cosign not found — image not signed. Install cosign before promoting." >&2
  echo "  https://docs.sigstore.dev/cosign/system_config/installation/" >&2
fi

# ── Output ────────────────────────────────────────────────────────────────────
echo ""
echo "==> Image published successfully"
echo "    image:  ${IMAGE_TAG}"
echo "    digest: ${DIGEST}"
echo ""
echo "==> Copy the following line into microservices/cloud-intelligence/iac/k8s/helm/values.yaml:"
echo "    digest: \"${DIGEST}\""
