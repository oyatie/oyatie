#!/usr/bin/env bash
set -euo pipefail
cargo run -p oya-tooling-cli-dev-runtime --bin repoctl -- pre-push "$@"
