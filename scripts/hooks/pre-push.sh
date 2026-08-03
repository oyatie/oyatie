#!/usr/bin/env bash
# Canonical local pre-push hook: shift-left evidence only.
#
# Protected-branch merge readiness is NOT decided by this hook. It is decided by
# the cloud-ci/oya-ci pipeline status named `oya-ci-required` on the candidate
# commit. This hook may block a local push when the installed verifier reports a
# problem, but its output is advisory local evidence, not merge authority.
#
# This hook runs the local pre-push self-verify slice only: freshness,
# generated-face settle check, and Buck2 affected-set. The protected
# `oya-ci-required` status remains the merge authority.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

base_ref="${OYA_PRE_PUSH_BASE:-origin/dev}"

printf '%s\n' "pre-push: running local self-verify (freshness + faces + affected-set) against ${base_ref}." >&2
printf '%s\n' "pre-push: advisory only — protected-branch authority remains the cloud-ci/oya-ci 'oya-ci-required' status." >&2

if command -v oya >/dev/null 2>&1; then
  exec oya verify --pre-push --base "$base_ref"
fi

exec cargo run -p oya-dev-cli --bin oya -- verify --pre-push --base "$base_ref"
