#!/usr/bin/env bash
# oya/ci-controller/iac/oci/push-and-sign.sh
#
# Push the assembled OCI Image Layout to the in-cluster registry and
# cosign-sign it by digest.
#
# LINUX CI ONLY.  Do NOT run on darwin; the binary inside the layout is
# Mach-O (host arch) and not deployable to aarch64-linux containers.
#
# Prerequisites (installed in the CI pod):
#   - crane  (gcr.io/go-containerregistry/crane; pinned bootstrap tool)
#   - cosign (sigstore; pinned to a verified version)
#
# Usage:
#   push-and-sign.sh <oci-layout-dir> [<tag>]
#
# Arguments:
#   oci-layout-dir   Path to the assembled OCI Image Layout (output of
#                    `buck2 build //oya/ci-controller/iac/oci:controller-oci`).
#   tag              Image tag (default: git short SHA of HEAD).
#
# Environment:
#   REGISTRY         Override registry (default: registry.oya-registry.svc.cluster.local:5000)
#   REPOSITORY       Override repository name (default: oya-ci-controller)
#   COSIGN_KEY       Path or k8s:// ref for cosign signing key
#                    (default: k8s://oya-ci/cosign-key — projected by ESO from OpenBao)
#   DRY_RUN          Set to "1" to print commands without executing (default: "")
#
# Outputs:
#   Prints the pushed image digest (sha256:...) to stdout on success.
#   On success also writes the digest to iac/oci/last-pushed-digest.txt so
#   the Helm values.yaml image.digest field can be updated by the CI crier.
#
# ADR references:
#   ADR-0146  distroless-nonroot base image
#   ADR-0181  container image promotion pipeline (cosign-sign by digest)
#   ADR-0514  buck2-native OCI (retire BuildKit/Dockerfile)

set -euo pipefail

# ── Argument parsing ──────────────────────────────────────────────────────────

OCI_LAYOUT_DIR="${1:-}"
if [[ -z "$OCI_LAYOUT_DIR" ]]; then
  echo "ERROR: missing oci-layout-dir argument" >&2
  echo "Usage: $0 <oci-layout-dir> [<tag>]" >&2
  exit 1
fi
if [[ ! -f "$OCI_LAYOUT_DIR/oci-layout" ]]; then
  echo "ERROR: $OCI_LAYOUT_DIR does not look like an OCI Image Layout (missing oci-layout marker)" >&2
  exit 1
fi

GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
TAG="${2:-${GIT_SHA}}"

REGISTRY="${REGISTRY:-registry.oya-registry.svc.cluster.local:5000}"
REPOSITORY="${REPOSITORY:-oya-ci-controller}"
COSIGN_KEY="${COSIGN_KEY:-k8s://oya-ci/cosign-key}"
DRY_RUN="${DRY_RUN:-}"

IMAGE_REF="${REGISTRY}/${REPOSITORY}:${TAG}"

# ── OS guard ─────────────────────────────────────────────────────────────────

OS="$(uname -s)"
if [[ "$OS" == "Darwin" ]]; then
  echo "ERROR: push-and-sign.sh must run on Linux (CI pod on aarch64-linux Talos node)." >&2
  echo "       On darwin the assembled binary is Mach-O and cannot be deployed." >&2
  echo "       Assemble on darwin for layout inspection; push only from Linux CI." >&2
  exit 1
fi

# ── Tool presence ─────────────────────────────────────────────────────────────

for tool in crane cosign; do
  if ! command -v "$tool" &>/dev/null; then
    echo "ERROR: $tool not found on PATH" >&2
    exit 1
  fi
done

# ── Helper ────────────────────────────────────────────────────────────────────

run() {
  if [[ -n "$DRY_RUN" ]]; then
    echo "[DRY_RUN] $*"
  else
    "$@"
  fi
}

# ── Push ─────────────────────────────────────────────────────────────────────

echo "==> Pushing OCI layout to ${IMAGE_REF}"
run crane push "${OCI_LAYOUT_DIR}" "${IMAGE_REF}" \
  --insecure \
  --platform linux/arm64

# Resolve the pushed digest (sha256:<hex>) from the registry.
echo "==> Resolving pushed digest"
PUSHED_DIGEST=$(crane digest "${IMAGE_REF}" --insecure 2>/dev/null || true)
if [[ -z "$PUSHED_DIGEST" ]]; then
  echo "ERROR: could not resolve digest for ${IMAGE_REF}" >&2
  exit 1
fi
IMAGE_WITH_DIGEST="${REGISTRY}/${REPOSITORY}@${PUSHED_DIGEST}"
echo "    digest: ${PUSHED_DIGEST}"

# ── Cosign sign ───────────────────────────────────────────────────────────────
#
# Sign by DIGEST (not tag) per ADR-0181 and the Kyverno ClusterPolicy
# verify-oya-registry-images-signed (infra/kyverno/policies/verify-image-signed.yaml).
# Key is projected by ESO from OpenBao (oya/ci/cosign-key).

echo "==> Cosign signing ${IMAGE_WITH_DIGEST}"
run cosign sign \
  --key "${COSIGN_KEY}" \
  --yes \
  "${IMAGE_WITH_DIGEST}"

# ── Emit digest for Helm values update ────────────────────────────────────────

DIGEST_FILE="$(dirname "$0")/last-pushed-digest.txt"
if [[ -z "$DRY_RUN" ]]; then
  echo "${PUSHED_DIGEST}" > "${DIGEST_FILE}"
  echo "==> Wrote digest to ${DIGEST_FILE}"
fi

echo "==> Done. Pushed and signed: ${IMAGE_WITH_DIGEST}"
echo "${PUSHED_DIGEST}"
