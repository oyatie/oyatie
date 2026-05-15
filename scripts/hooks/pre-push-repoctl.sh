#!/usr/bin/env bash
# Purpose: local-developer git pre-push hook. Thin wrapper that forwards to
# `oya-dev-cli repoctl pre-push`. Installed by developers as
# `.git/hooks/pre-push`. No CI binding.
# Scheduled-replacement: delete in favor of `cargo run -p oya-dev-cli --bin
# repoctl -- pre-push` invoked directly via `oya-dev-cli hook install` (see
# `evidence/audits/shell-python-replacement-audit-2026-05-15.md` row B-12).
set -euo pipefail
cargo run -p oya-dev-cli --bin repoctl -- pre-push "$@"
