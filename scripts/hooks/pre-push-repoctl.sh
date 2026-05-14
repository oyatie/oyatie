#!/usr/bin/env bash
set -euo pipefail
cargo run -p oya-dev-cli --bin repoctl -- pre-push "$@"
