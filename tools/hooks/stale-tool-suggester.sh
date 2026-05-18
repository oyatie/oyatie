#!/usr/bin/env bash
# tools/hooks/stale-tool-suggester.sh
#
# Trigger:  Claude Code PreToolUse(Bash)
# Purpose:  When a Bash command references retired tools (grit, rtk, icm, vox),
#           suggest the canonical oya vcs equivalent. Encouragement, not blockage.
# Behavior: Reads $TOOL_INPUT (JSON with "command" field) from environment or stdin.
#           Greps for retired tool names. If found, prints a suggestion to stderr
#           with the canonical replacement. Agent decides whether to rewrite.
# Non-blocking guarantee: exits 0 always, even when stale tools are detected.

set -uo pipefail

# Try to get command from TOOL_INPUT env var (Claude Code sets this for PreToolUse hooks)
COMMAND_TEXT=""
if [ -n "${TOOL_INPUT:-}" ]; then
    # Extract command field from JSON if jq available
    if command -v jq >/dev/null 2>&1; then
        COMMAND_TEXT=$(echo "$TOOL_INPUT" | jq -r '.command // ""' 2>/dev/null || echo "")
    else
        COMMAND_TEXT="$TOOL_INPUT"
    fi
fi

# Fallback: read from stdin if no env var
if [ -z "$COMMAND_TEXT" ] && [ ! -t 0 ]; then
    STDIN_CONTENT=$(cat 2>/dev/null || true)
    if command -v jq >/dev/null 2>&1; then
        COMMAND_TEXT=$(echo "$STDIN_CONTENT" | jq -r '.command // ""' 2>/dev/null || echo "$STDIN_CONTENT")
    else
        COMMAND_TEXT="$STDIN_CONTENT"
    fi
fi

if [ -z "$COMMAND_TEXT" ]; then
    exit 0
fi

# Check for retired tool references
RETIRED_FOUND=""
for tool in grit rtk icm vox; do
    if echo "$COMMAND_TEXT" | grep -qw "$tool" 2>/dev/null; then
        RETIRED_FOUND="$RETIRED_FOUND $tool"
    fi
done

if [ -n "$RETIRED_FOUND" ]; then
    echo "ℹ [stale-tool-suggester] Retired tool(s) detected:${RETIRED_FOUND}" >&2
    echo "ℹ  These are retired per ADR-0116. Canonical replacement:" >&2
    echo "ℹ    oya vcs <subcommand>" >&2
    echo "ℹ    cargo run --quiet -p oya-dev-cli -- vcs <subcommand>" >&2
    echo "ℹ  See tools/hooks/_canonical-primitives.md for the full primitives reference." >&2
    echo "ℹ  Continuing as requested — agent decides whether to rewrite." >&2
fi

exit 0
