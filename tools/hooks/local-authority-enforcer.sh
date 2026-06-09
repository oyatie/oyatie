#!/usr/bin/env bash
# local-authority-enforcer (PreToolUse:Bash)
#
# Blocks retired local authority surfaces before they become muscle memory in
# agent sessions. The live merge authority is the `oya-ci-required` status
# produced by the cloud-ci/oya-ci pipeline. Local commands may provide
# shift-left evidence, but local VCS wrappers must not coordinate work or
# decide merge readiness.

set -uo pipefail

payload="$(cat)"

# Extract the Bash command from Claude/Codex hook JSON. Keep this dependency-free
# because hooks run on the hot path before every shell command.
cmd="$(printf '%s' "$payload" \
  | tr '\n' ' ' \
  | sed -nE 's/.*"command"[[:space:]]*:[[:space:]]*"(([^"\\]|\\.)*)".*/\1/p' \
  | head -n 1 \
  | sed -E 's/\\n/ /g; s/\\t/ /g; s/\\r/ /g; s/\\"/"/g; s/\\\\/\\/g')"

if [ -z "$cmd" ]; then
  exit 0
fi

# Retired VCS wrappers: ADR-0363 moved coordination to plain git + protected PR.
retired_vcs='(^|[;&|(]|[[:space:]])(\./bin/|bin/)?oya[[:space:]]+(git|vcs)([[:space:]]|$)'

if printf '%s' "$cmd" | grep -Eq "$retired_vcs"; then
  {
    echo "🚫 BLOCKED: retired local VCS authority surface detected: oya git/oya vcs."
    echo "Use plain git for local source control, then open a PR against dev."
    echo "Merge readiness is decided by the cloud-ci/oya-ci pipeline status 'oya-ci-required', not by a local wrapper."
  } >&2
  exit 2
fi

exit 0
