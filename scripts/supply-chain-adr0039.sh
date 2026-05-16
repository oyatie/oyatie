#!/usr/bin/env bash
set -euo pipefail

# Compatibility shim: ADR-0039 release supply-chain execution is Rust-owned.
manifest="${1:-registry/release/images.yaml}"
if [[ $# -gt 0 ]]; then
  shift
fi
exec cargo run -p oya-dev-cli -- supply-chain adr0039 --manifest "$manifest" "$@"
