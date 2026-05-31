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
#   COSIGN_PASSWORD  Passphrase for the cosign private key (projected by ESO
#                    alongside COSIGN_KEY from OpenBao oya/ci/cosign-key).
#                    Required in non-DRY_RUN mode when the key is password-protected.
#                    Source from the ESO-projected secret mount, e.g.:
#                      export COSIGN_PASSWORD="$(cat /var/run/secrets/cosign/password)"
#                    For unencrypted keys set COSIGN_PASSWORD="" explicitly.
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

# COSIGN_PASSWORD: passphrase for the ESO-projected cosign private key.
# Source from the projected secret before calling this script, e.g.:
#   export COSIGN_PASSWORD="$(cat /var/run/secrets/cosign/password)"
# Exported here so cosign reads it from the environment (non-interactive CI).
# Tolerate unset for unencrypted keys (empty string is valid for cosign).
export COSIGN_PASSWORD="${COSIGN_PASSWORD:-}"

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

# ── COSIGN_PASSWORD assertion ─────────────────────────────────────────────────
#
# The ESO ExternalSecret projects a `password` property alongside the private
# key from OpenBao oya/ci/cosign-key.  Without COSIGN_PASSWORD exported, cosign
# prompts for a passphrase on stdin and a non-interactive CI step hangs.
# In non-DRY_RUN mode, assert the variable is set (empty string is accepted for
# unencrypted keys; the key type determines whether it is used).
if [[ -z "$DRY_RUN" ]]; then
  if [[ ! -v COSIGN_PASSWORD ]]; then
    echo "ERROR: COSIGN_PASSWORD is not set. Export it before calling this script:" >&2
    echo "  export COSIGN_PASSWORD=\"\$(cat /var/run/secrets/cosign/password)\"" >&2
    echo "  For unencrypted keys: export COSIGN_PASSWORD=\"\"" >&2
    exit 1
  fi
fi

# ── Push ─────────────────────────────────────────────────────────────────────
#
# crane push takes a tarball (docker save / OCI tar) or a directory for
# OCI Image Layout directories.  The go-containerregistry crane push command
# accepts an OCI Image Layout directory directly (not just tarballs) when the
# path is a directory — the library detects the format.  However, `--platform`
# is not a valid flag for `crane push` (platform is a pull/index concern);
# drop it here.  The layout produced by oya-oci-assemble is already
# single-platform (linux/arm64 config from the base + the app layer).
#
# If a future crane version breaks OCI-layout-dir push, the fallback is:
#   tar -cf - -C "${OCI_LAYOUT_DIR}" . | crane push - "${IMAGE_REF}" --insecure
# (piped tarball form; crane push reads a tar from stdin when given "-").

echo "==> Pushing OCI layout to ${IMAGE_REF}"
run crane push "${OCI_LAYOUT_DIR}" "${IMAGE_REF}" \
  --insecure

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
# COSIGN_PASSWORD is already exported above.

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
