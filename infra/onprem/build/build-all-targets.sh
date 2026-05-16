#!/usr/bin/env bash
# build-all-targets.sh — build oya-ops-workspace-shell for the deploy fleet.
# Produces tarballs in $REPO/dist/ for the ops/OpenTofu deployment pipeline.
#
# Targets:
#   linux-x86_64-znver4      -> this box (Ryzen 7 7800X3D)              [build on linux]
#   linux-x86_64-znver1      -> Oracle Free Tier VM.Standard.E2.1.Micro [build on linux]
#                                (AMD EPYC 7551 Naples, Zen 1)
#   linux-aarch64-neov1      -> Oracle Free Tier VM.Standard.A1.Flex    [build on linux, cross]
#                                (Ampere Altra, Neoverse-N1)
#   darwin-aarch64-apple-m5  -> Apple Silicon M5 Max (and M5/M5 Pro/Ultra) [build on macOS only]
#                                Tuned for apple-m5 microarchitecture.
#                                Override with OYATIE_APPLE_CPU=apple-m4 if your
#                                rustc/LLVM is too old to know about m5, or use
#                                OYATIE_APPLE_CPU=native to autodetect.
#
# The darwin target is HOST-GATED — only attempted when run on macOS.
# Cross-compiling to macOS from Linux requires the Apple SDK (osxcross), which
# has licensing + maintenance overhead that isn't worth it for one target.
# Run this same script on an M-series Mac to produce the darwin tarball, then
# copy it back into dist/.
#
# Prereq once on Linux: rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
#                       sudo apt install -y gcc-aarch64-linux-gnu
# Prereq once on macOS: rustup target add aarch64-apple-darwin
#                       (Xcode CLI tools — `xcode-select --install`)
set -euo pipefail

REPO=/home/oyatie/projects/oyatie
DIST=$REPO/dist
BIN=oya-ops-workspace-shell
PKG=oya-ops-workspace-shell-runtime
VERSION=${OYATIE_VERSION:-$(git -C "$REPO" rev-parse --short HEAD)}

ensure_targets() {
  for t in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
    if ! rustup target list --installed | grep -q "^$t$"; then
      rustup target add "$t"
    fi
  done
  if ! command -v aarch64-linux-gnu-gcc >/dev/null 2>&1; then
    echo "ERROR: aarch64-linux-gnu-gcc not found. Run: sudo apt install -y gcc-aarch64-linux-gnu"
    exit 1
  fi
}

ensure_cross_linker_config() {
  local cfg="$REPO/.cargo/config.toml"
  install -d "$REPO/.cargo"
  touch "$cfg"
  if ! grep -q '\[target\.aarch64-unknown-linux-gnu\]' "$cfg"; then
    cat >> "$cfg" <<'EOF'

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
EOF
    echo "==> appended cross-linker config to $cfg"
  fi
}

build_variant() {
  local label=$1 triple=$2 cpu=$3
  local target_dir="$REPO/target/matrix/$label"
  local out_dir="$DIST/$label"

  echo
  echo "==> [$label] triple=$triple cpu=$cpu"

  RUSTFLAGS="-C target-cpu=$cpu" \
  CARGO_TARGET_DIR="$target_dir" \
    cargo build --release \
      --manifest-path "$REPO/Cargo.toml" \
      -p "$PKG" --bin "$BIN" \
      --target "$triple"

  install -d "$out_dir"
  install -m 0755 "$target_dir/$triple/release/$BIN" "$out_dir/$BIN"

  ( cd "$out_dir" && sha256sum "$BIN" > "$BIN.sha256" )

  # Capture metadata so the install script can verify it picked the right one.
  cat > "$out_dir/manifest.txt" <<EOF
binary=$BIN
version=$VERSION
label=$label
triple=$triple
target_cpu=$cpu
built_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
built_on=$(uname -srm)
rustc=$(rustc --version)
EOF

  ( cd "$DIST" && \
    tar czf "${BIN}-${VERSION}-${label}.tar.gz" -C "$label" . )
}

ensure_targets
ensure_cross_linker_config
install -d "$DIST"
rm -f "$DIST/${BIN}-"*.tar.gz

build_variant linux-x86_64-znver4    x86_64-unknown-linux-gnu  znver4
build_variant linux-x86_64-znver1    x86_64-unknown-linux-gnu  znver1
build_variant linux-aarch64-neov1    aarch64-unknown-linux-gnu neoverse-n1

# Apple Silicon — only attempted on macOS (no osxcross dance).
APPLE_CPU=${OYATIE_APPLE_CPU:-apple-m5}
if [ "$(uname -s)" = "Darwin" ]; then
  build_variant "darwin-aarch64-${APPLE_CPU}" aarch64-apple-darwin "$APPLE_CPU"
else
  echo
  echo "==> [darwin-aarch64-${APPLE_CPU}] SKIPPED: not on macOS"
  echo "    Run this script on an M-series Mac to produce the darwin tarball,"
  echo "    then copy ${BIN}-*-darwin-aarch64-${APPLE_CPU}.tar.gz back into $DIST/."
  echo "    If 'apple-m5' is unknown to your rustc, retry with:"
  echo "        OYATIE_APPLE_CPU=apple-m4  ~/build-all-targets.sh"
  echo "    or  OYATIE_APPLE_CPU=native    ~/build-all-targets.sh   (host autodetect)"
fi

echo
echo "=== artifacts ==="
ls -la "$DIST"/*.tar.gz
echo
echo "=== checksums ==="
for d in "$DIST"/linux-*/; do
  printf "%-30s  " "$(basename "$d")"
  cat "$d/$BIN.sha256"
done
echo
echo "Publish through the ops/OpenTofu deployment pipeline; direct host-copy or remote-shell deploy is not supported."
