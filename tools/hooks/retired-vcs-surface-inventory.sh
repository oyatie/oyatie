#!/usr/bin/env bash
# Inventory `oya vcs <verb>` policy-ratchet invocations during the oya-git cutover.
# This is intentionally non-blocking; task #38 owns the full drop-in git surface.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

PATTERN='oya[[:space:]]+vcs[[:space:]]+(claim|work|verify|done|status|symbols|queue|watch|promote)'
EXCLUDES=(
  ':(exclude).git/**'
  ':(exclude).claude/worktrees/**'
  ':(exclude)target/**'
  ':(exclude)evidence/**'
)

matches="$(git grep -n -E "$PATTERN" -- . "${EXCLUDES[@]}" || true)"

if [ -z "$matches" ]; then
  echo "oya-git cutover inventory: no oya vcs policy-ratchet invocations found"
  exit 0
fi

count="$(printf '%s\n' "$matches" | wc -l | tr -d ' ')"
echo "::notice title=Oya git cutover inventory::Found ${count} current oya vcs policy-ratchet invocation(s). Keep ratchet semantics unchanged; migrate plain git/drop-in docs toward oya git."
printf '%s\n' "$matches" | sed -n '1,100p'
if [ "$count" -gt 100 ]; then
  echo "... truncated $((count - 100)) additional match(es)"
fi

exit 0
