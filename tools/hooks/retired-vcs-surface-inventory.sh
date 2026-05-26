#!/usr/bin/env bash
# Inventory command-shaped retired `oya git` / `oya vcs` invocations in active
# hook/control surfaces after ADR-0363. This is intentionally non-blocking; CI
# gates remain the enforcement surface.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

PATTERN='(^|[^[:alnum:]_./-])(\./bin/)?oya[[:space:]]+(git[[:space:]]+(status|add|commit|push|pull|fetch|checkout|switch|branch|diff|log|show|rev-parse|grep|merge|rebase|restore|reset|tag|stash|remote|ls-files)|vcs[[:space:]]+(claim|work|verify|done|status|symbols|queue|watch|promote))([[:space:];|&)]|$)'
EXCLUDES=(
  ':(exclude).git/**'
  ':(exclude).claude/worktrees/**'
  ':(exclude)target/**'
  ':(exclude)evidence/**'
)

SEARCH_PATHS=(
  .claude/settings.json
  .codex/hooks.json
  .gemini/settings.json
  .hermes/hooks.json
  tools/hooks
  tools/hook-bootstrap/install.sh
)

existing_paths=()
for path in "${SEARCH_PATHS[@]}"; do
  if [ -e "$path" ]; then
    existing_paths+=("$path")
  fi
done

if [ "${#existing_paths[@]}" -eq 0 ]; then
  echo "retired VCS surface inventory: no active hook/control surfaces found"
  exit 0
fi

matches="$(git grep -n -E "$PATTERN" -- "${existing_paths[@]}" "${EXCLUDES[@]}" || true)"

if [ -z "$matches" ]; then
  echo "retired VCS surface inventory: no oya git/oya vcs invocations found"
  exit 0
fi

count="$(printf '%s\n' "$matches" | wc -l | tr -d ' ')"
echo "::notice title=Retired VCS surface inventory::Found ${count} retired oya git/oya vcs invocation(s). Use plain git plus PR/Jenkins and oya gate/verify for governance."
printf '%s\n' "$matches" | sed -n '1,100p'
if [ "$count" -gt 100 ]; then
  echo "... truncated $((count - 100)) additional match(es)"
fi

exit 0
