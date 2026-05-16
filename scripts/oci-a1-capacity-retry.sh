#!/usr/bin/env sh
# Compatibility shim: OCI A1 retry logic is Rust-owned.
set -eu
REPO_ROOT="${OYA_REPO_ROOT:-$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)}"
if command -v oya >/dev/null 2>&1; then
  exec oya ops oci-a1-capacity-retry "$@"
fi
OYA_BIN="${OYA_BIN:-$REPO_ROOT/target/debug/oya}"
if [ -x "$OYA_BIN" ]; then
  exec "$OYA_BIN" ops oci-a1-capacity-retry "$@"
fi
exec cargo run -p oya-dev-cli --manifest-path "$REPO_ROOT/Cargo.toml" -- ops oci-a1-capacity-retry "$@"
