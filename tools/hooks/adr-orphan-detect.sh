#!/usr/bin/env bash
# tools/hooks/adr-orphan-detect.sh
#
# Trigger:  Claude Code PostToolUse(Edit|Write) where target is .md or .json
# Purpose:  After editing a markdown or JSON file, scan for ADR-NNNN references and
#           check whether the corresponding docs/decisions/ADR-NNNN-*.md file exists.
#           Report orphaned references as advisory output.
# Behavior: Greps the edited file for ADR-NNNN patterns. For each unique reference,
#           checks docs/decisions/. Prints orphan list to stderr. Advisory only.
# Non-blocking guarantee: exits 0 always.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# WHY .tool_input.*: real Claude Code / Codex deliver hook input as JSON on STDIN
# with the edited path nested under tool_input ({"tool_input":{"file_path":"..."}}).
# The flat .path/.file_path keys are kept for the TOOL_INPUT env path used by the CI
# governance harness only. See code.claude.com/docs hooks + developers.openai.com/codex/hooks.
FILE_PATH=""
if [ -n "${TOOL_INPUT:-}" ]; then
    if command -v jq >/dev/null 2>&1; then
        FILE_PATH=$(echo "$TOOL_INPUT" | jq -r '.tool_input.file_path // .tool_input.path // .path // .file_path // ""' 2>/dev/null || echo "")
    fi
fi
if [ -z "$FILE_PATH" ] && [ ! -t 0 ]; then
    STDIN_CONTENT=$(cat 2>/dev/null || true)
    if command -v jq >/dev/null 2>&1; then
        FILE_PATH=$(echo "$STDIN_CONTENT" | jq -r '.tool_input.file_path // .tool_input.path // .path // .file_path // ""' 2>/dev/null || echo "$STDIN_CONTENT")
    else
        FILE_PATH="$STDIN_CONTENT"
    fi
fi

if [ -z "$FILE_PATH" ]; then
    exit 0
fi

# Only act on .md and .json files
if ! echo "$FILE_PATH" | grep -qiE '\.(md|json)$' 2>/dev/null; then
    exit 0
fi

ABS_PATH="$REPO_ROOT/$FILE_PATH"
if [[ "$FILE_PATH" = /* ]]; then
    ABS_PATH="$FILE_PATH"
fi
if [ ! -f "$ABS_PATH" ]; then
    exit 0
fi

# Extract unique ADR-NNNN references (10-second budget when GNU timeout exists).
if command -v timeout >/dev/null 2>&1; then
    ADR_REFS=$(timeout 10 grep -oE 'ADR-[0-9]{4}' "$ABS_PATH" 2>/dev/null | sort -u || true)
else
    ADR_REFS=$(grep -oE 'ADR-[0-9]{4}' "$ABS_PATH" 2>/dev/null | sort -u || true)
fi

if [ -z "$ADR_REFS" ]; then
    exit 0
fi

DECISIONS_DIR="$REPO_ROOT/docs/decisions"
ORPHANS=""

while IFS= read -r adr_ref; do
    num="${adr_ref#ADR-}"
    match=$(find "$DECISIONS_DIR" -maxdepth 1 -name "ADR-${num}-*.md" 2>/dev/null | head -1 || true)
    if [ -z "$match" ]; then
        ORPHANS="$ORPHANS $adr_ref"
    fi
done <<< "$ADR_REFS"

if [ -n "$ORPHANS" ]; then
    echo "ℹ [adr-orphan-detect] Orphan ADR references in $FILE_PATH (no docs/decisions/ADR-NNNN-*.md found):" >&2
    for orphan in $ORPHANS; do
        echo "ℹ    $orphan" >&2
    done
    echo "ℹ  Consider creating ADR stubs or verifying the ADR numbers are correct." >&2
fi

exit 0
