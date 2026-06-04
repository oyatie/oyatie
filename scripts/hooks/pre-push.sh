#!/usr/bin/env bash
# Canonical local pre-push hook: run the Buck2-backed shift-left validator.
# Retired `oya verify`/`oya gate` CLI surfaces must not be invoked here; reusable
# governance logic belongs in Rust libraries, Buck2 targets, and Prow/K8s jobs.
set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel)"
exec "$REPO_ROOT/scripts/agent-pre-push-validate.sh" "$@"
