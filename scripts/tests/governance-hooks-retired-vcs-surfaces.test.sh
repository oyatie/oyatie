#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

primer_output="$tmpdir/primer.out"
bash "$repo_root/tools/hooks/userprompt-canonical-primer.sh" >"$primer_output"
grep -q 'plain git for VCS work' "$primer_output"
grep -q './bin/oya verify --ci-required' "$primer_output"
grep -q 'oya gate run-all' "$primer_output"
if grep -Eq 'oya git for|policy ratchet compatibility|current policy ratchet' "$primer_output"; then
  echo "primer still advertises retired governance hook surfaces" >&2
  exit 1
fi

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

inventory_output="$tmpdir/inventory.out"
bash "$repo_root/tools/hooks/retired-vcs-surface-inventory.sh" >"$inventory_output"
grep -q 'retired VCS surface inventory: no oya git/oya vcs invocations found' "$inventory_output"

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
