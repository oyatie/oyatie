#!/usr/bin/env bash
# Deterministic GitHub Actions bootstrap for the temporary lane-unlocker.
# Keep this serialized before any Buck2 fanout so rustup never races inside
# concurrently executing Buck2 actions on a fresh runner.
set -euo pipefail

: "${BUCK2_RELEASE:=2026-06-01}"
: "${RUSTUP_CONCURRENT_DOWNLOADS:=1}"
export RUSTUP_CONCURRENT_DOWNLOADS

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

install_system_tools() {
  sudo apt-get update
  sudo apt-get install -y --no-install-recommends ca-certificates curl zstd
}

rust_toolchain_channel() {
  awk -F '"' '/^[[:space:]]*channel[[:space:]]*=/ { channel = $2 } END { print channel }' rust-toolchain.toml
}

bootstrap_rust() {
  require_cmd rustup
  local toolchain
  toolchain="$(rust_toolchain_channel)"
  if [[ -z "$toolchain" ]]; then
    echo "rust-toolchain.toml must declare [toolchain].channel" >&2
    exit 1
  fi

  rustup set profile minimal
  rustup toolchain install "$toolchain" \
    --profile minimal \
    --component rustfmt,clippy,llvm-tools-preview \
    --target x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu

  local active_toolchain
  active_toolchain="$(rustup show active-toolchain | awk '{ toolchain = $1 } END { print toolchain }')"
  if [[ -z "$active_toolchain" ]]; then
    echo "rustup did not report an active toolchain" >&2
    exit 1
  fi

  rustup component add --toolchain "$active_toolchain" rustfmt clippy llvm-tools-preview
  rustup target add --toolchain "$active_toolchain" x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

  rustup show
  rustc --version --verbose
  cargo --version
  clippy-driver --version

  local host sysroot llvm_bin
  host="$(rustc -vV | awk '/^host: / { host = $2 } END { print host }')"
  sysroot="$(rustc --print sysroot)"
  llvm_bin="$sysroot/lib/rustlib/$host/bin"
  test -x "$llvm_bin/llvm-profdata"
  test -x "$llvm_bin/llvm-cov"
  "$llvm_bin/llvm-profdata" --version
  "$llvm_bin/llvm-cov" --version
  rustc --print=cfg --target=aarch64-unknown-linux-gnu >/dev/null
}

bootstrap_buck2() {
  if command -v buck2 >/dev/null 2>&1; then
    buck2 --version
    return 0
  fi

  local buck2_arch
  case "$(uname -m)" in
    x86_64) buck2_arch="x86_64-unknown-linux-gnu" ;;
    aarch64|arm64) buck2_arch="aarch64-unknown-linux-gnu" ;;
    *) echo "unsupported Buck2 runner architecture: $(uname -m)" >&2; exit 1 ;;
  esac

  curl -fsSL -o /tmp/buck2.zst "https://github.com/facebook/buck2/releases/download/${BUCK2_RELEASE}/buck2-${buck2_arch}.zst"
  zstd -d -c /tmp/buck2.zst > /tmp/buck2
  sudo install -m 0755 /tmp/buck2 /usr/local/bin/buck2
  buck2 --version
}

install_system_tools
bootstrap_rust
bootstrap_buck2
