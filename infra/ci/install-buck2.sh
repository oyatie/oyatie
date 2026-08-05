#!/usr/bin/env bash
# Install the digest-pinned Buck2 release used by the canonical cloud-ci bridge.
set -euo pipefail

BUCK2_RELEASE="${BUCK2_RELEASE:-2026-06-01}"
BUCK2_INSTALL_DIR="${BUCK2_INSTALL_DIR:-/tmp/oya-ci-buck2-${BUCK2_RELEASE}}"
BUCK2_BINARY="${BUCK2_INSTALL_DIR}/buck2"

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

if [ -x "${BUCK2_BINARY}" ]; then
  if "${BUCK2_BINARY}" --version >/dev/null 2>&1; then
    echo "Using cached buck2 at ${BUCK2_BINARY}."
    if [ -n "${GITHUB_PATH:-}" ]; then
      echo "${BUCK2_INSTALL_DIR}" >> "${GITHUB_PATH}"
    fi
    exec "${BUCK2_BINARY}" "$@"
  fi

  echo "Cached buck2 binary at ${BUCK2_BINARY} is unusable; redownloading." >&2
fi

curl -fsSL "https://github.com/facebook/buck2/releases/download/${BUCK2_RELEASE}/${BUCK2_ASSET}" -o "${BUCK2_INSTALL_DIR}/${BUCK2_ASSET}"
echo "${BUCK2_SHA256}  ${BUCK2_INSTALL_DIR}/${BUCK2_ASSET}" | sha256sum -c -
zstd -f -d "${BUCK2_INSTALL_DIR}/${BUCK2_ASSET}" -o "${BUCK2_BINARY}"
chmod +x "${BUCK2_BINARY}"

if [ -n "${GITHUB_PATH:-}" ]; then
  echo "${BUCK2_INSTALL_DIR}" >> "${GITHUB_PATH}"
fi

"${BUCK2_BINARY}" --version
