#!/usr/bin/env bash
set -euo pipefail

# Compatibility entrypoint only. The merge-safety logic lives in Rust so Buck2
# can exercise the same implementation without shell parsing or jq dependency.
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
src="$repo_root/scripts/check-sequential-pr-merge-conflicts.rs"
bin="${TMPDIR:-/tmp}/oyatie-check-sequential-pr-merge-conflicts-$$"
trap 'rm -f "$bin"' EXIT

rustc --edition=2024 -D warnings "$src" -o "$bin"
exec "$bin" "$@"
