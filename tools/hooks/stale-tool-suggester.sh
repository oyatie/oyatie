#!/usr/bin/env bash
# tools/hooks/stale-tool-suggester.sh
#
# Trigger:  Claude Code PreToolUse(Bash)
# Purpose:  When a Bash command references retired VCS wrapper surfaces,
#           suggest the current plain git + governance gate path.
# Behavior: Reads $TOOL_INPUT (JSON with "command" field) from environment or stdin.
#           Prints a suggestion to stderr with the canonical replacement.
#           Agent decides whether to rewrite.
# Non-blocking guarantee: exits 0 always, even when stale commands are detected.

set -uo pipefail

# Try to get command from TOOL_INPUT env var (Claude Code sets this for PreToolUse hooks)
COMMAND_TEXT=""
if [ -n "${TOOL_INPUT:-}" ]; then
    # Extract command field from JSON if jq available
    if command -v jq >/dev/null 2>&1; then
        COMMAND_TEXT=$(echo "$TOOL_INPUT" | jq -r '.command // .input.command // .tool_input.command // .parameters.command // ""' 2>/dev/null || echo "")
    else
        COMMAND_TEXT="$TOOL_INPUT"
    fi
fi

# Fallback: read from stdin if no env var
if [ -z "$COMMAND_TEXT" ] && [ ! -t 0 ]; then
    STDIN_CONTENT=$(cat 2>/dev/null || true)
    if command -v jq >/dev/null 2>&1; then
        COMMAND_TEXT=$(echo "$STDIN_CONTENT" | jq -r '.command // .input.command // .tool_input.command // .parameters.command // ""' 2>/dev/null || echo "$STDIN_CONTENT")
    else
        COMMAND_TEXT="$STDIN_CONTENT"
    fi
fi

if [ -z "$COMMAND_TEXT" ]; then
    exit 0
fi

RETIRED_VCS_PATTERN='(^|[;&|][;&|]?[[:space:]]*|\([[:space:]]*)oya[[:space:]]+(git|vcs)([[:space:]]|$)'

if printf '%s\n' "$COMMAND_TEXT" | grep -Eq "$RETIRED_VCS_PATTERN" 2>/dev/null; then
    echo "ℹ [stale-tool-suggester] Retired VCS surface detected: oya git/oya vcs are retired by ADR-0363." >&2
    echo "ℹ  Use plain git for VCS work; use ./bin/oya verify --ci-required or oya gate run-all for governance gates." >&2
    echo "ℹ  This is advisory only; command semantics are not changed by the hook." >&2
fi

exit 0
