#!/usr/bin/env bash
# local-authority-enforcer (PreToolUse:Bash)
#
# Blocks retired local authority surfaces before they become muscle memory in
# agent sessions. The live merge authority is the single required context
# `presubmit`, produced by GitHub Actions per ADR-0515 (ci is the
# shadow/future runner). Local source control uses plain git; retired local
# authority wrappers must not coordinate work or decide merge readiness.

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

# Strip quoted string arguments before pattern-matching so that retired tool
# names appearing inside message text (git commit -m "...oya gate...", gh pr
# comment --body "...", omc team api send-message --input '{"body":"..."}') do
# not produce false positives.  Tradeoff: quoted invocations such as
# bash -c "oya gate" are no longer caught here — acceptable because hooks are
# the last-stop safety net per enforcement-layering doctrine (AMENDMENT 5); the
# canonical CI gates remain the authority.
cmd_stripped="$(printf '%s' "$cmd" | sed -E "s/'[^']*'//g; s/\"[^\"]*\"//g")"

# Retired authority wrappers: D-CLOUD-NATIVE/D-CICD-AUTHORITY route source
# control to plain git + protected PR and route gate verdicts to the single
# required context `presubmit`, produced by GitHub Actions per ADR-0515
# (ci is the shadow/future runner).
retired_authority='(^|[;&|(]|[[:space:]])(\./bin/|bin/)?oya[[:space:]]+(git|vcs|gate|verify|check|submit)([[:space:]]|$)'

if printf '%s' "$cmd_stripped" | grep -Eq "$retired_authority"; then
  {
    echo "🚫 BLOCKED: retired local authority surface detected."
    echo "Use plain git for local source control and Buck2/cloud-ci targets for local confidence."
    echo "Merge readiness is decided only by the single required context 'presubmit' (produced by GitHub Actions per ADR-0515; ci is the shadow/future runner), not by local oya wrappers."
  } >&2
  exit 2
fi

exit 0
