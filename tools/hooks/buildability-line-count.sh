#!/usr/bin/env bash
# tools/hooks/buildability-line-count.sh
#
# Trigger:  Claude Code PostToolUse(Write) where target is microservices/<ms>/docs/* file
# Purpose:  After writing a doc file under a µservice, count substantive lines and
#           encourage expansion if below the ADR-0212 buildability bar. Positive ack
#           when bar is met.
# Behavior: Counts non-blank, non-comment lines. <50 lines = suggestion to expand.
#           >=50 lines = positive acknowledgement. Advisory only.
# Non-blocking guarantee: exits 0 always.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

FILE_PATH=""
if [ -n "${TOOL_INPUT:-}" ]; then
    if command -v jq >/dev/null 2>&1; then
        FILE_PATH=$(echo "$TOOL_INPUT" | jq -r '.path // .file_path // ""' 2>/dev/null || echo "")
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

# Only act on microservices/<ms>/docs/* paths
if ! echo "$FILE_PATH" | grep -qE '^microservices/[^/]+/docs/' 2>/dev/null; then
    exit 0
fi

ABS_PATH="$REPO_ROOT/$FILE_PATH"
if [[ "$FILE_PATH" = /* ]]; then
    ABS_PATH="$FILE_PATH"
fi
if [ ! -f "$ABS_PATH" ]; then
    exit 0
fi

# Count non-blank, non-comment lines (lines not starting with #, //, or being empty)
SUBSTANTIVE=$(grep -cvE '^\s*$|^\s*#|^\s*//' "$ABS_PATH" 2>/dev/null || echo 0)

if [ "$SUBSTANTIVE" -lt 50 ]; then
    echo "ℹ [buildability-line-count] $FILE_PATH has $SUBSTANTIVE substantive lines (bar: 50+)." >&2
    echo "ℹ  Per ADR-0212 buildability doctrine, consider expanding this document." >&2
    echo "ℹ  Stubs and placeholders reduce the µservice's overall artifact quality." >&2
else
    echo "✓ [buildability-line-count] Buildability bar met for $FILE_PATH ($SUBSTANTIVE substantive lines)." >&2
fi

exit 0
