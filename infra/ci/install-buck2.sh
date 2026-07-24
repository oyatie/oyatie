#!/usr/bin/env bash
# Install the digest-pinned Buck2 release used by the canonical cloud-ci bridge.
set -euo pipefail

BUCK2_RELEASE="${BUCK2_RELEASE:-2026-06-01}"
BUCK2_INSTALL_DIR="${BUCK2_INSTALL_DIR:-/tmp/oya-ci-buck2-${BUCK2_RELEASE}}"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)
    BUCK2_ASSET="${BUCK2_ASSET:-buck2-x86_64-unknown-linux-gnu.zst}"
    BUCK2_SHA256="${BUCK2_SHA256:-4dd9ae54c87fdcf795101074f8788232af55523885135d5e3358c77365993555}"
    ;;
  *)
    if [ "${OYA_CI_ALLOW_AMBIENT_BUCK2:-}" = "1" ] && command -v buck2 >/dev/null 2>&1; then
      echo "Using ambient buck2 only because OYA_CI_ALLOW_AMBIENT_BUCK2=1 was set." >&2
      buck2 --version
      exit 0
    fi
    echo "Unsupported host for default pinned Buck2 install; set OYA_CI_ALLOW_AMBIENT_BUCK2=1 for local advisory use." >&2
    exit 1
    ;;
esac

mkdir -p "${BUCK2_INSTALL_DIR}"
asset_path="${BUCK2_INSTALL_DIR}/${BUCK2_ASSET}"
binary_path="${BUCK2_INSTALL_DIR}/buck2"
asset_temp=""
binary_temp=""

cleanup_partials() {
  [ -z "${asset_temp}" ] || rm -f -- "${asset_temp}"
  [ -z "${binary_temp}" ] || rm -f -- "${binary_temp}"
}
trap cleanup_partials EXIT

# Cache-hit fast path (ADR-0556 D5 QW-4: the tool binary is a digest-pinned INPUT, not a build
# output — warm-eligible velocity). If the compressed release asset is already present (e.g.
# restored by actions/cache) and its bytes match the pinned SHA-256, skip the network download.
# A present-but-mismatching asset is discarded and re-downloaded.
if [ -f "${asset_path}" ] \
  && echo "${BUCK2_SHA256}  ${asset_path}" | sha256sum -c - >/dev/null 2>&1; then
  echo "buck2 release asset cache hit (SHA-256 verified): ${asset_path} — skipping download." >&2
else
  rm -f -- "${asset_path}"
  asset_temp="$(mktemp "${asset_path}.part.XXXXXX")"
  curl --retry 8 --retry-all-errors --retry-max-time 180 --connect-timeout 20 --max-time 60 -fsSL "https://github.com/facebook/buck2/releases/download/${BUCK2_RELEASE}/${BUCK2_ASSET}" -o "${asset_temp}"
  echo "${BUCK2_SHA256}  ${asset_temp}" | sha256sum -c -
  mv -f -- "${asset_temp}" "${asset_path}"
  asset_temp=""
fi

# Integrity is non-negotiable (ADR-0556: the SHA check is the integrity anchor that makes the
# warm path admissible). The pinned-digest verification ALWAYS runs on the exact bytes about to
# be decompressed and executed — cached and fresh paths alike — and the executable is ALWAYS
# re-derived from those verified bytes (never trusted as a loose cached binary).
echo "${BUCK2_SHA256}  ${asset_path}" | sha256sum -c -
binary_temp="$(mktemp "${binary_path}.part.XXXXXX")"
zstd -d -f "${asset_path}" -o "${binary_temp}"
chmod +x "${binary_temp}"
mv -f -- "${binary_temp}" "${binary_path}"
binary_temp=""

if [ -n "${GITHUB_PATH:-}" ]; then
  echo "${BUCK2_INSTALL_DIR}" >> "${GITHUB_PATH}"
fi

"${binary_path}" --version
