#!/usr/bin/env bash
# Canonical local pre-push hook: run the Buck2-backed shift-left validator.
# `oya verify` may remain local governance evidence, but it is not CI/build
# authority and this hook must not shell out through Cargo.
set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel)"
exec "$REPO_ROOT/scripts/agent-pre-push-validate.sh" "$@"
