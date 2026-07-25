#!/usr/bin/env bash
# Install the digest-pinned Buck2 release used by the canonical cloud-ci bridge.
set -euo pipefail

BUCK2_RELEASE="${BUCK2_RELEASE:-2026-06-01}"
BUCK2_INSTALL_DIR="${BUCK2_INSTALL_DIR:-/tmp/oya-ci-buck2-${BUCK2_RELEASE}}"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)
    BUCK2_ASSET="${BUCK2_ASSET-buck2-x86_64-unknown-linux-gnu.zst}"
    BUCK2_SHA256="${BUCK2_SHA256-4dd9ae54c87fdcf795101074f8788232af55523885135d5e3358c77365993555}"
    BUCK2_BINARY_NAME="buck2"
    ;;
  MINGW*-x86_64)
    BUCK2_ASSET="${BUCK2_ASSET-buck2-x86_64-pc-windows-msvc.exe.zst}"
    BUCK2_SHA256="${BUCK2_SHA256-b3229a6e5cce50f6561dc251bf7f20e902b20c983dcdc293adefd5bba437cae3}"
    BUCK2_BINARY_NAME="buck2.exe"
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

lock_timeout_seconds="${BUCK2_INSTALL_LOCK_TIMEOUT_SECONDS-180}"
case "${lock_timeout_seconds}" in
  ''|*[!0-9]*)
    echo "BUCK2_INSTALL_LOCK_TIMEOUT_SECONDS must be a positive integer." >&2
    exit 1
    ;;
esac
if [ "${lock_timeout_seconds}" -eq 0 ]; then
  echo "BUCK2_INSTALL_LOCK_TIMEOUT_SECONDS must be a positive integer." >&2
  exit 1
fi
case "${BUCK2_ASSET}" in
  ''|'.'|'..'|*[!0-9A-Za-z._-]*)
    echo "BUCK2_ASSET must be a safe release-asset filename." >&2
    exit 1
    ;;
esac
case "${BUCK2_SHA256}" in
  *[!0-9a-fA-F]*|'')
    echo "BUCK2_SHA256 must be exactly 64 hexadecimal characters." >&2
    exit 1
    ;;
esac
if [ "${#BUCK2_SHA256}" -ne 64 ]; then
  echo "BUCK2_SHA256 must be exactly 64 hexadecimal characters." >&2
  exit 1
fi
BUCK2_SHA256="$(printf '%s' "${BUCK2_SHA256}" | tr '[:upper:]' '[:lower:]')"

content_dir="${BUCK2_INSTALL_DIR}/sha256-${BUCK2_SHA256}"
mkdir -p "${content_dir}"
asset_path="${content_dir}/${BUCK2_ASSET}"
binary_path="${content_dir}/${BUCK2_BINARY_NAME}"
lock_path="${content_dir}/.buck2-install.lock"
lock_dir="${lock_path}.d"
asset_temp=""
binary_temp=""
mkdir_lock_held=0

cleanup_partials() {
  [ -z "${asset_temp}" ] || rm -f -- "${asset_temp}"
  [ -z "${binary_temp}" ] || rm -f -- "${binary_temp}"
}

release_mkdir_lock() {
  [ "${mkdir_lock_held}" -eq 1 ] || return 0
  rm -f -- "${lock_dir}/owner-pid"
  rmdir -- "${lock_dir}" 2>/dev/null || true
  mkdir_lock_held=0
}

cleanup_on_exit() {
  cleanup_partials
  release_mkdir_lock
}
trap cleanup_on_exit EXIT

acquire_mkdir_lock() {
  local deadline owner_pid
  deadline=$(( $(date +%s) + lock_timeout_seconds ))
  while ! mkdir "${lock_dir}" 2>/dev/null; do
    owner_pid=""
    if [ -r "${lock_dir}/owner-pid" ]; then
      owner_pid="$(cat "${lock_dir}/owner-pid" 2>/dev/null || true)"
    fi
    if [ -n "${owner_pid}" ] && ! kill -0 "${owner_pid}" 2>/dev/null; then
      rm -f -- "${lock_dir}/owner-pid"
      rmdir -- "${lock_dir}" 2>/dev/null || true
      continue
    fi
    if [ "$(date +%s)" -ge "${deadline}" ]; then
      echo "Timed out waiting for Buck2 installer lock: ${lock_path}" >&2
      exit 1
    fi
    sleep 1
  done
  mkdir_lock_held=1
  printf '%s\n' "$$" > "${lock_dir}/owner-pid"
}

if [ "${BUCK2_INSTALL_FORCE_NO_FLOCK:-}" != "1" ] && command -v flock >/dev/null 2>&1; then
  exec 9>"${lock_path}"
  if ! flock -x -w "${lock_timeout_seconds}" 9; then
    echo "Timed out waiting for Buck2 installer lock: ${lock_path}" >&2
    exit 1
  fi
else
  acquire_mkdir_lock
fi
rm -f -- "${asset_path}.part."* "${binary_path}.part."*

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
  curl --retry 8 --retry-all-errors --retry-max-time 180 --connect-timeout 20 --max-time 60 -fsSL "https://github.com/facebook/buck2/releases/download/${BUCK2_RELEASE}/${BUCK2_ASSET}" -o "${asset_temp}" 9>&-
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
zstd -d -f "${asset_path}" -o "${binary_temp}" 9>&-
chmod +x "${binary_temp}"
"${binary_temp}" --version 9>&-
mv -f -- "${binary_temp}" "${binary_path}"
binary_temp=""

if [ -n "${GITHUB_PATH:-}" ]; then
  echo "${content_dir}" >> "${GITHUB_PATH}"
fi
