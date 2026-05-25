#!/usr/bin/env bash
# infra/ci-farm/agent-image/build-and-sign.sh
# ============================================================================
# O5 — Build + cosign-sign the CI agent image BY DIGEST (ADR-0360, part O5).
#
# Builds infra/ci-farm/agent-image/Dockerfile, pushes it, captures the immutable
# DIGEST, and cosign-signs THAT digest (never a mutable tag). Signing by digest is
# load-bearing: a tag can be re-pointed after signing, so a tag signature attests
# nothing; a digest signature attests exact content. The scoped Kyverno policy
# (kyverno-verify-agent-image.yaml) then admits only digests that carry this sig.
#
# Signing modes (in preference order):
#   1. KEYLESS (preferred): cosign OIDC keyless via Fulcio/Rekor. The signer
#      identity is an OIDC token (CI workload identity), recorded transparently in
#      Rekor — no long-lived key to manage. Requires COSIGN_EXPERIMENTAL or a
#      cosign new enough to default to keyless, plus an OIDC token in CI.
#   2. KEY (fallback): cosign sign --key <ref> for environments without OIDC
#      (e.g. air-gapped colo). The matching public key is the attestor in the
#      Kyverno policy. Set COSIGN_KEY to a key ref (k8s://, file://, KMS URI…).
#
# LOCAL-vs-PRODUCTION deltas (honest):
#   - Production runs this in the trunk/postsubmit lane with a real registry
#     (registry.oyatie.dev/ci/rust-agent) and CI OIDC identity (keyless). Locally
#     you can build against a local registry and use the key fallback; the local
#     k3s profile does NOT enforce the cosign policy (see README + values-local
#     deltas) — signature ENFORCEMENT is a production-cluster property.
#   - No build/sign timing or image size is claimed here; this is the recipe.
# ----------------------------------------------------------------------------
set -euo pipefail

# --- config (override via env) ------------------------------------------------
REGISTRY="${REGISTRY:-registry.oyatie.dev}"
IMAGE_REPO="${IMAGE_REPO:-${REGISTRY}/ci/rust-agent}"
# Human-readable tag is for humans only; the SIGNATURE is on the digest below.
IMAGE_TAG="${IMAGE_TAG:-$(date +%Y%m%d)-$(git rev-parse --short HEAD 2>/dev/null || echo local)}"
IMAGE_REF="${IMAGE_REPO}:${IMAGE_TAG}"
DOCKERFILE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# --- guard: cosign must be present --------------------------------------------
if ! command -v cosign >/dev/null 2>&1; then
  echo "ERROR: cosign not found on PATH. Install sigstore/cosign before signing." >&2
  echo "       (build can proceed, but the image MUST be signed to pass admission)" >&2
  exit 127
fi

# --- guard: a builder must be present -----------------------------------------
if command -v docker >/dev/null 2>&1; then
  BUILD="docker"
elif command -v podman >/dev/null 2>&1; then
  BUILD="podman"
else
  echo "ERROR: neither docker nor podman found on PATH." >&2
  exit 127
fi

echo "==> building ${IMAGE_REF} from ${DOCKERFILE_DIR}/Dockerfile"
# --provenance/--sbom left to the builder defaults; the digest is what we sign.
"${BUILD}" build --pull -t "${IMAGE_REF}" "${DOCKERFILE_DIR}"

echo "==> pushing ${IMAGE_REF}"
"${BUILD}" push "${IMAGE_REF}"

# --- capture the immutable digest (sign THIS, never the tag) ------------------
# `crane digest` is the most reliable; fall back to the builder's inspect output.
if command -v crane >/dev/null 2>&1; then
  DIGEST="$(crane digest "${IMAGE_REF}")"
else
  DIGEST="$("${BUILD}" inspect --format '{{index .RepoDigests 0}}' "${IMAGE_REF}" | sed 's/.*@//')"
fi
if [[ -z "${DIGEST:-}" || "${DIGEST}" != sha256:* ]]; then
  echo "ERROR: failed to resolve a sha256 digest for ${IMAGE_REF} (got '${DIGEST}')." >&2
  exit 1
fi
DIGEST_REF="${IMAGE_REPO}@${DIGEST}"
echo "==> resolved digest: ${DIGEST_REF}"

# --- sign BY DIGEST -----------------------------------------------------------
if [[ -n "${COSIGN_KEY:-}" ]]; then
  # FALLBACK: explicit key. COSIGN_KEY is a cosign key ref (file://, k8s://, KMS).
  echo "==> cosign sign (KEY mode) ${DIGEST_REF}"
  cosign sign --yes --key "${COSIGN_KEY}" "${DIGEST_REF}"
else
  # PREFERRED: keyless OIDC. Requires an OIDC token (CI workload identity) and a
  # cosign that defaults to keyless (or COSIGN_EXPERIMENTAL=1 on older versions).
  echo "==> cosign sign (KEYLESS OIDC mode) ${DIGEST_REF}"
  COSIGN_EXPERIMENTAL=1 cosign sign --yes "${DIGEST_REF}"
fi

echo "==> signed. Admission will accept this digest via kyverno-verify-agent-image.yaml."
echo "    image (digest-pinned, use THIS in the agent pod template): ${DIGEST_REF}"
