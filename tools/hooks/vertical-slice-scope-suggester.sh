#!/usr/bin/env bash
# tools/hooks/vertical-slice-scope-suggester.sh
#
# Trigger:  Claude Code PreToolUse(Write|Edit)
# Purpose:  When a new microservices/<name>/ directory is being created outside
#           the current PR's declared vertical slice, remind the agent to consider
#           whether this belongs in the current PR or a follow-up vertical.
# Behavior: Reads file path from $TOOL_INPUT or stdin. Checks against
#           .omc/state/current-pr-vertical.txt if present. Prints a scope note.
#           Agent retains full autonomy.
# Non-blocking guarantee: exits 0 always.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Extract file path from tool input
FILE_PATH=""
if [ -n "${TOOL_INPUT:-}" ]; then
    if command -v jq >/dev/null 2>&1; then
        FILE_PATH=$(echo "$TOOL_INPUT" | jq -r '.path // .file_path // ""' 2>/dev/null || echo "")
    else
        FILE_PATH="$TOOL_INPUT"
    fi
fi

if [ -z "$FILE_PATH" ] && [ ! -t 0 ]; then
    STDIN_CONTENT=$(cat 2>/dev/null || true)
    if command -v jq >/dev/null 2>&1; then
        FILE_PATH=$(echo "$STDIN_CONTENT" | jq -r '.path // .file_path // ""' 2>/dev/null || echo "$STDIN_CONTENT")
    else
        FILE_PATH="$STDIN_CONTENT"
    fi
fi

if [ -z "$FILE_PATH" ]; then
    exit 0
fi

# Only act on microservices/<name>/* paths
if ! echo "$FILE_PATH" | grep -qE '^microservices/[^/]+/' 2>/dev/null; then
    exit 0
fi

# Extract the µservice name from the path
# Extract ms name: strip leading 'microservices/' then everything from second '/' onward
_tmp="${FILE_PATH#microservices/}"
MS_NAME="${_tmp%%/*}"

# Check current declared vertical if state file exists
VERTICAL_FILE="$REPO_ROOT/.omc/state/current-pr-vertical.txt"
if [ -f "$VERTICAL_FILE" ]; then
    CURRENT_VERTICAL=$(tr -d '[:space:]' < "$VERTICAL_FILE" 2>/dev/null || true)
    if [ -n "$CURRENT_VERTICAL" ] && [ "$MS_NAME" != "$CURRENT_VERTICAL" ]; then
        echo "ℹ [vertical-slice-scope-suggester] Creating microservices/$MS_NAME/ but current PR vertical is '$CURRENT_VERTICAL'." >&2
        echo "ℹ  Per ADR-0217 (vertical-slice rollout), consider:" >&2
        echo "ℹ    - Does this new µservice belong in this PR, or a follow-up vertical-slice PR?" >&2
        echo "ℹ    - If intentional cross-vertical work, update .omc/state/current-pr-vertical.txt." >&2
        echo "ℹ  Continuing as requested — agent decides." >&2
    fi
else
    # No vertical declared; gentle nudge for any new µservice
    echo "ℹ [vertical-slice-scope-suggester] Creating microservices/$MS_NAME/ — no PR vertical declared." >&2
    echo "ℹ  Per ADR-0217, consider recording the current vertical in .omc/state/current-pr-vertical.txt." >&2
    echo "ℹ  Continuing as requested." >&2
fi

exit 0
