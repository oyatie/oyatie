#!/usr/bin/env bash
# local-authority-enforcer (PreToolUse:Bash)
#
# Blocks retired local authority surfaces before they become muscle memory in
# agent sessions. The live merge authority is the `oya-ci-required` status
# produced by the cloud-ci/oya-ci pipeline. Local source control uses plain
# git; local oya gate/verify/check wrappers must not coordinate work or decide
# merge readiness.

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

# Retired authority wrappers: D-CLOUD-NATIVE/D-CICD-AUTHORITY route source
# control to plain git + protected PR and route gate verdicts to the
# cloud-ci/oya-ci pipeline's single required context.
retired_authority='(^|[;&|(]|[[:space:]])(\./bin/|bin/)?oya[[:space:]]+(git|vcs|gate|verify|check|submit)([[:space:]]|$)'

if printf '%s' "$cmd" | grep -Eq "$retired_authority"; then
  {
    echo "🚫 BLOCKED: retired local authority surface detected."
    echo "Use plain git for local source control and Buck2/cloud-ci targets for local confidence."
    echo "Merge readiness is decided only by the cloud-ci/oya-ci pipeline status 'oya-ci-required', not by local oya wrappers."
  } >&2
  exit 2
fi

exit 0
