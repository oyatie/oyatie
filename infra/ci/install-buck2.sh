#!/usr/bin/env bash
# Install the digest-pinned Buck2 release used by the canonical cloud-ci bridge.
# Also ensures rustup exists on GitHub-hosted runners (baked ARC images already have it).
set -euo pipefail

# Hosted GHA + large monorepo: concurrent PR jobs hit Linux inotify defaults and buck2 fails
# with "OS file watch limit reached" / DaemonStateData (fleet-wide reds on 1566–1572).
# Raise watches early so every job that sources this installer can start buck2d. Best-effort:
# non-root / restricted runners skip without failing the install.
if [ "$(uname -s)" = "Linux" ]; then
  if command -v sysctl >/dev/null 2>&1; then
    if [ "$(id -u)" -eq 0 ]; then
      sysctl -w fs.inotify.max_user_watches=524288 >/dev/null 2>&1 || true
      sysctl -w fs.inotify.max_user_instances=1024 >/dev/null 2>&1 || true
    elif command -v sudo >/dev/null 2>&1; then
      sudo -n sysctl -w fs.inotify.max_user_watches=524288 >/dev/null 2>&1 || true
      sudo -n sysctl -w fs.inotify.max_user_instances=1024 >/dev/null 2>&1 || true
    fi
  fi
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    echo "buck2-preflight: fs.inotify max_user_watches=$(sysctl -n fs.inotify.max_user_watches 2>/dev/null || echo unknown) max_user_instances=$(sysctl -n fs.inotify.max_user_instances 2>/dev/null || echo unknown)"
  fi
fi

# W1 hosted runners: ubuntu-latest has no rustup; ARC images ship it under /opt/rust.
# Always pin RUSTUP_HOME for later steps that use set -u and pass --env RUSTUP_HOME=${RUSTUP_HOME}
# (ARC bakes RUSTUP_HOME=/opt/rust/rustup; GHA needs an explicit default or the var is unbound).
if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none --profile minimal
  # shellcheck disable=SC1091
  source "${HOME}/.cargo/env"
  if [ -n "${GITHUB_PATH:-}" ]; then
    echo "${HOME}/.cargo/bin" >> "${GITHUB_PATH}"
  fi
  if [ -n "${GITHUB_ENV:-}" ]; then
    echo "PATH=${HOME}/.cargo/bin:${PATH}" >> "${GITHUB_ENV}"
  fi
  export PATH="${HOME}/.cargo/bin:${PATH}"
fi
if [ -z "${RUSTUP_HOME:-}" ]; then
  export RUSTUP_HOME="${HOME}/.rustup"
fi
# Windows Git Bash HOME is /c/Users/... . If that POSIX form is exported via GITHUB_ENV,
# later native steps (pwsh + Buck2 hermetic env + msvc rustup) treat it as a relative path
# and resolve it as D:/c/Users/... — missing manifests, soft platform-smoke red.
# Convert to mixed Windows form (C:/Users/...) so pre-provision and Buck2 share one tree.
case "$(uname -s)" in
  MINGW* | MSYS*)
    if command -v cygpath >/dev/null 2>&1; then
      if rustup_home_native="$(cygpath -m -- "${RUSTUP_HOME}" 2>/dev/null)" \
        && [ -n "${rustup_home_native}" ]; then
        export RUSTUP_HOME="${rustup_home_native}"
      fi
    fi
    ;;
esac
if [ -n "${GITHUB_ENV:-}" ]; then
  echo "RUSTUP_HOME=${RUSTUP_HOME}" >> "${GITHUB_ENV}"
fi

BUCK2_RELEASE="${BUCK2_RELEASE:-2026-07-15}"
BUCK2_INSTALL_DIR="${BUCK2_INSTALL_DIR:-}"
windows_github_path=0

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)
    BUCK2_INSTALL_DIR="${BUCK2_INSTALL_DIR:-/tmp/oya-ci-buck2-${BUCK2_RELEASE}}"
    BUCK2_ASSET="${BUCK2_ASSET-buck2-x86_64-unknown-linux-gnu.zst}"
    BUCK2_SHA256="${BUCK2_SHA256-ecc3d807dd0b0feff1a423688bd598263b8339d223e685578a87196456c19d95}"
    BUCK2_BINARY_NAME="buck2"
    ;;
  Linux-aarch64 | Linux-arm64)
    # Hosted ubuntu-24.04-arm and lab ARC are both aarch64. Same digest-pinned adapter
    # edge as x86_64: release tag selects the asset, SHA-256 pins the bytes.
    BUCK2_INSTALL_DIR="${BUCK2_INSTALL_DIR:-/tmp/oya-ci-buck2-${BUCK2_RELEASE}}"
    BUCK2_ASSET="${BUCK2_ASSET-buck2-aarch64-unknown-linux-gnu.zst}"
    BUCK2_SHA256="${BUCK2_SHA256-e239bf72f40a7987db9024eb6d5e325642f6496c589dec6be54c1008d2618a19}"
    BUCK2_BINARY_NAME="buck2"
    ;;
  Darwin-arm64 | Darwin-aarch64)
    # GitHub macos-latest is Apple Silicon. Digests from facebook/buck2 2026-07-15 release API.
    BUCK2_INSTALL_DIR="${BUCK2_INSTALL_DIR:-/tmp/oya-ci-buck2-${BUCK2_RELEASE}}"
    BUCK2_ASSET="${BUCK2_ASSET-buck2-aarch64-apple-darwin.zst}"
    BUCK2_SHA256="${BUCK2_SHA256-088cacc72c400fa438be4052c36782f56b2af86287aadf13ece5e9772d72455c}"
    BUCK2_BINARY_NAME="buck2"
    ;;
  Darwin-x86_64)
    # macos-*-large / macos-15-intel class (Intel). Same pin discipline as other arms.
    BUCK2_INSTALL_DIR="${BUCK2_INSTALL_DIR:-/tmp/oya-ci-buck2-${BUCK2_RELEASE}}"
    BUCK2_ASSET="${BUCK2_ASSET-buck2-x86_64-apple-darwin.zst}"
    BUCK2_SHA256="${BUCK2_SHA256-46cc4bb1372ea3110c099240e05176bb9eff003e7e38233c1bb2ef268449dbb3}"
    BUCK2_BINARY_NAME="buck2"
    ;;
  MINGW*-x86_64 | MSYS*-x86_64)
    windows_github_path=1
    if [ -z "${BUCK2_INSTALL_DIR}" ]; then
      if [ -z "${RUNNER_TEMP:-}" ] || ! command -v cygpath >/dev/null 2>&1; then
        echo "Windows pinned Buck2 installation requires RUNNER_TEMP and cygpath." >&2
        exit 1
      fi
      if ! runner_temp_posix="$(cygpath -u -- "${RUNNER_TEMP}")" || [ -z "${runner_temp_posix}" ]; then
        echo "Failed to convert RUNNER_TEMP to a Git Bash path for Windows Buck2 installation." >&2
        exit 1
      fi
      BUCK2_INSTALL_DIR="${runner_temp_posix}/oya-ci-buck2-${BUCK2_RELEASE}"
    fi
    BUCK2_ASSET="${BUCK2_ASSET-buck2-x86_64-pc-windows-msvc.exe.zst}"
    BUCK2_SHA256="${BUCK2_SHA256-719324109a8c5f9f95d9f1f6895ec500505eebcc466b193fa46e05f243276e59}"
    BUCK2_BINARY_NAME="buck2.exe"
    ;;
  MINGW*-ARM64 | MINGW*-aarch64 | MSYS*-ARM64 | MSYS*-aarch64)
    # windows-11-arm hosted class (when available to the account).
    windows_github_path=1
    if [ -z "${BUCK2_INSTALL_DIR}" ]; then
      if [ -z "${RUNNER_TEMP:-}" ] || ! command -v cygpath >/dev/null 2>&1; then
        echo "Windows pinned Buck2 installation requires RUNNER_TEMP and cygpath." >&2
        exit 1
      fi
      if ! runner_temp_posix="$(cygpath -u -- "${RUNNER_TEMP}")" || [ -z "${runner_temp_posix}" ]; then
        echo "Failed to convert RUNNER_TEMP to a Git Bash path for Windows Buck2 installation." >&2
        exit 1
      fi
      BUCK2_INSTALL_DIR="${runner_temp_posix}/oya-ci-buck2-${BUCK2_RELEASE}"
    fi
    BUCK2_ASSET="${BUCK2_ASSET-buck2-aarch64-pc-windows-msvc.exe.zst}"
    BUCK2_SHA256="${BUCK2_SHA256-9939fda2913e27fcda30a70e8d51833523d7083b9f56459626fa5bc161f10a86}"
    BUCK2_BINARY_NAME="buck2.exe"
    ;;
  *)
    if [ "${OYA_CI_ALLOW_AMBIENT_BUCK2:-}" = "1" ] && command -v buck2 >/dev/null 2>&1; then
      echo "Using ambient buck2 only because OYA_CI_ALLOW_AMBIENT_BUCK2=1 was set." >&2
      buck2 --version
      exit 0
    fi
    echo "Unsupported host for default pinned Buck2 install ($(uname -s)-$(uname -m)); set OYA_CI_ALLOW_AMBIENT_BUCK2=1 for local advisory use." >&2
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
  local deadline now owner_pid
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
    now="$(date +%s)"
    if [ "${now}" -ge "${deadline}" ]; then
      if [ -z "${owner_pid}" ] && rmdir -- "${lock_dir}" 2>/dev/null; then
        continue
      fi
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
  github_path_entry="${content_dir}"
  if [ "${windows_github_path}" -eq 1 ]; then
    if ! github_path_entry="$(cygpath -w -- "${content_dir}")" || [ -z "${github_path_entry}" ]; then
      echo "Failed to convert the Windows Buck2 installation path for GITHUB_PATH." >&2
      exit 1
    fi
    case "${github_path_entry}" in
      [A-Za-z]:\\*|\\\\*) ;;
      *)
        echo "cygpath did not produce a native Windows path for GITHUB_PATH." >&2
        exit 1
        ;;
    esac
  fi
  echo "${github_path_entry}" >> "${GITHUB_PATH}"
fi
