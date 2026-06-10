#!/usr/bin/env bash
# tools/hooks/stale-tool-suggester.sh
#
# Trigger:  Claude Code PreToolUse(Bash)
# Purpose:  When a Bash command references retired local authority surfaces,
#           suggest the current plain git path + the single required context
#           'oya-ci-required' (produced by GitHub Actions per ADR-0515;
#           oya-ci is the shadow/future runner).
# Behavior: Reads $TOOL_INPUT (JSON with "command" field) from environment or stdin.
#           Prints a suggestion to stderr with the canonical replacement.
#           Agent decides whether to rewrite.
# Non-blocking guarantee: exits 0 always, even when stale commands are detected.

set -uo pipefail

# TOOL_INPUT env path serves only the CI governance harness
# (tools/governance/adr-0221-governance-gates.sh); real Claude Code / Codex hooks
# deliver JSON on stdin (handled by the fallback below). See code.claude.com/docs
# hooks reference + developers.openai.com/codex/hooks: no env var carries event data.
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

RETIRED_AUTHORITY_PATTERN='(^|[;&|][;&|]?[[:space:]]*|\([[:space:]]*)(\./bin/|bin/)?oya[[:space:]]+(git|vcs|gate|verify|check|submit)([[:space:]]|$)'

if printf '%s\n' "$COMMAND_TEXT" | grep -Eq "$RETIRED_AUTHORITY_PATTERN" 2>/dev/null; then
    echo "ℹ [stale-tool-suggester] Retired local authority surface detected." >&2
    echo "ℹ  Use plain git for local VCS work and Buck2/cloud-ci targets for local confidence." >&2
    echo "ℹ  Merge readiness is the single required context 'oya-ci-required' (produced by GitHub Actions per ADR-0515; oya-ci is the shadow/future runner), not local oya wrappers." >&2
    echo "ℹ  This advisory is paired with local-authority-enforcer, which blocks retired authority commands." >&2
fi

exit 0
