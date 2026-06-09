#!/usr/bin/env bash
# Canonical local pre-push hook: shift-left evidence only.
#
# Protected-branch merge readiness is NOT decided by this hook. It is decided by
# the cloud-ci/oya-ci pipeline status named `oya-ci-required` on the candidate
# commit. This hook may block a local push when the installed verifier reports a
# problem, but its output is advisory local evidence, not merge authority.
#
# This hook deliberately does not execute local oya verifier/gate/dev-cli
# wrappers. Buck2/cloud-ci targets are the local confidence path, and the
# protected `oya-ci-required` status is the merge authority.
set -euo pipefail

printf '%s\n' "pre-push: SKIP retired local oya verifier/gate wrappers." >&2
printf '%s\n' "pre-push: not authoritative — protected-branch authority remains the cloud-ci/oya-ci 'oya-ci-required' status." >&2
exit 0
