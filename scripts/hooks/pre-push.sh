#!/usr/bin/env bash
# Canonical local pre-push hook: shift-left evidence only.
#
# Protected-branch merge readiness is NOT decided by this hook. It is decided by
# the cloud-ci/oya-ci pipeline status named `oya-ci-required` on the candidate
# commit. This hook may block a local push when the installed verifier reports a
# problem, but its output is advisory local evidence, not merge authority.
#
# Accepted form:
#   - Installed binary: `oya verify --ci-required --include-deferred`
#
# Deliberately NOT accepted:
#   - `cargo run -p oya-dev-cli ...` fallback. Buck2 is the canonical build
#     substrate, and local dev-cli execution must not become hidden authority.
set -euo pipefail

if command -v oya >/dev/null 2>&1; then
  exec oya verify --ci-required --include-deferred "$@"
fi

printf '%s\n' "pre-push: SKIP local verifier; 'oya' binary not found on PATH." >&2
printf '%s\n' "pre-push: not authoritative — protected-branch authority remains the cloud-ci/oya-ci 'oya-ci-required' status." >&2
exit 0
