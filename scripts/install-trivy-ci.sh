#!/usr/bin/env bash
set -euo pipefail

# Compatibility shim: CI Trivy installation is Rust-owned.
exec cargo run -p oya-dev-cli -- supply-chain install-trivy "$@"
