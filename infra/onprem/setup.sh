#!/usr/bin/env sh
# Compatibility shim: top-level on-prem orchestration lives in Rust.
set -eu
REPO_ROOT="${OYA_REPO_ROOT:-$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)}"
if command -v oya >/dev/null 2>&1; then
  exec oya onprem install --repo-root "$REPO_ROOT" "$@"
fi
OYA_BIN="${OYA_BIN:-$REPO_ROOT/target/debug/oya}"
if [ -x "$OYA_BIN" ]; then
  exec "$OYA_BIN" onprem install --repo-root "$REPO_ROOT" "$@"
fi
exec cargo run -p oya-dev-cli --manifest-path "$REPO_ROOT/Cargo.toml" -- onprem install --repo-root "$REPO_ROOT" "$@"
