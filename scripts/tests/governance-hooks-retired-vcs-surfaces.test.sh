#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

plain_git_output="$tmpdir/plain-git.out"
TOOL_INPUT='{"command":"git status --short"}' \
  bash "$repo_root/tools/hooks/stale-tool-suggester.sh" >"$plain_git_output" 2>&1
if [ -s "$plain_git_output" ]; then
  echo "plain git should not trigger stale-tool-suggester" >&2
  cat "$plain_git_output" >&2
  exit 1
fi

retired_git_output="$tmpdir/retired-git.out"
TOOL_INPUT='{"command":"oya git status --short"}' \
  bash "$repo_root/tools/hooks/stale-tool-suggester.sh" >"$retired_git_output" 2>&1
grep -q 'Retired VCS surface detected' "$retired_git_output"
grep -q 'Use plain git for VCS work' "$retired_git_output"

retired_vcs_output="$tmpdir/retired-vcs.out"
TOOL_INPUT='{"command":"oya vcs status"}' \
  bash "$repo_root/tools/hooks/stale-tool-suggester.sh" >"$retired_vcs_output" 2>&1
grep -q 'Retired VCS surface detected' "$retired_vcs_output"
grep -q 'Use plain git for VCS work' "$retired_vcs_output"

context_registration_pattern='SessionStart|UserPromptSubmit|BeforeAgent|oya-session-context|oya-canonical-primer'
if rg -n "$context_registration_pattern" \
  "$repo_root/.codex/hooks.json" \
  "$repo_root/.claude/settings.json" \
  "$repo_root/.gemini/settings.json" \
  "$repo_root/tools/hook-bootstrap/install.sh"; then
  echo "context-injection hooks should not be registered by managed hook configs" >&2
  exit 1
fi

for retired_context_hook in session-start-context-inject.sh userprompt-canonical-primer.sh; do
  retired_output="$tmpdir/$retired_context_hook.out"
  bash "$repo_root/tools/hooks/$retired_context_hook" >"$retired_output"
  if [ -s "$retired_output" ]; then
    echo "$retired_context_hook compatibility stub should not inject prompt context" >&2
    cat "$retired_output" >&2
    exit 1
  fi
done

if rg -n \
  'Preferred drop-in surface: oya git|policy ratchet compatibility|policy-ratchet|route through `oya git`|Top-level subcommands: git|oya-git cutover|migrate plain git/drop-in docs toward oya git' \
  "$repo_root/tools/hooks" \
  "$repo_root/tools/hook-bootstrap" \
  "$repo_root/.codex/hooks.json" \
  "$repo_root/.claude/settings.json" \
  "$repo_root/.gemini/settings.json"; then
  echo "active governance hook surface still contains stale VCS guidance" >&2
  exit 1
fi

echo "governance hook retired VCS surface tests passed"
