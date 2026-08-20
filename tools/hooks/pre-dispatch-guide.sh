#!/usr/bin/env bash
# tools/hooks/pre-dispatch-guide.sh
#
# Trigger:  Claude Code PreToolUse(Agent) or PreToolUse(Task)
# Purpose:  When an agent dispatch is too sparse (short prompt, missing evidence
#           pointers), offer a structured guidance reminder. Encouragement, not blockage.
# Behavior: Reads dispatch prompt from $TOOL_INPUT or stdin. If prompt is <200 chars
#           or lacks key structural signals, prints a suggestion to stderr.
#           Agent retains full autonomy to dispatch as-is.
# Non-blocking guarantee: exits 0 always.

set -uo pipefail

# WHY .tool_input.*: real Claude Code / Codex deliver PreToolUse:Task input as JSON on
# STDIN with the dispatch prompt nested under tool_input ({"tool_input":{"prompt":"..."}}).
# Flat .prompt/.description/.input kept for the TOOL_INPUT env fallback. See
# code.claude.com/docs hooks + developers.openai.com/codex/hooks.
PROMPT_TEXT=""
if [ -n "${TOOL_INPUT:-}" ]; then
    if command -v jq >/dev/null 2>&1; then
        PROMPT_TEXT=$(echo "$TOOL_INPUT" | jq -r '.tool_input.prompt // .tool_input.description // .prompt // .description // .input // ""' 2>/dev/null || echo "")
    else
        PROMPT_TEXT="$TOOL_INPUT"
    fi
fi

if [ -z "$PROMPT_TEXT" ] && [ ! -t 0 ]; then
    STDIN_CONTENT=$(cat 2>/dev/null || true)
    if command -v jq >/dev/null 2>&1; then
        PROMPT_TEXT=$(echo "$STDIN_CONTENT" | jq -r '.tool_input.prompt // .tool_input.description // .prompt // .description // .input // ""' 2>/dev/null || echo "$STDIN_CONTENT")
    else
        PROMPT_TEXT="$STDIN_CONTENT"
    fi
fi

if [ -z "$PROMPT_TEXT" ]; then
    exit 0
fi

CHAR_COUNT=${#PROMPT_TEXT}
MISSING_SIGNALS=""

# Check for structural signals in the prompt
if [ "$CHAR_COUNT" -lt 200 ]; then
    MISSING_SIGNALS="$MISSING_SIGNALS short-prompt(<200-chars)"
fi
if ! echo "$PROMPT_TEXT" | grep -qi 'audience\|who.*agent\|agent.*type\|executor\|architect' 2>/dev/null; then
    MISSING_SIGNALS="$MISSING_SIGNALS audience-declaration"
fi
if ! echo "$PROMPT_TEXT" | grep -qiE 'evidence/|docs/|specs/|libs/|read.*file|file.*read' 2>/dev/null; then
    MISSING_SIGNALS="$MISSING_SIGNALS evidence-pointers"
fi
if ! echo "$PROMPT_TEXT" | grep -qi 'output\|artifact\|deliver\|produce\|create\|write' 2>/dev/null; then
    MISSING_SIGNALS="$MISSING_SIGNALS output-bar"
fi

if [ -n "$MISSING_SIGNALS" ]; then
    echo "ℹ [pre-dispatch-guide] Dispatch prompt may be missing context (${MISSING_SIGNALS# })." >&2
    echo "ℹ  Consider including:" >&2
    echo "ℹ    - Audience declaration: who is the sub-agent (Executor, Architect, Reviewer)?" >&2
    echo "ℹ    - Evidence pointers: which files should the agent read first?" >&2
    echo "ℹ    - Output bar: what artifacts should be produced?" >&2
    echo "ℹ    - Version sources: where to verify spec versions (OpenAPI 3.2.0, AsyncAPI 3.1.0)?" >&2
    echo "ℹ  Continuing dispatch as requested." >&2
fi

# Persona matching: suggest vendored agent persona based on task signals
PERSONA_SUGGESTION=""
if echo "$PROMPT_TEXT" | grep -qi 'review\|audit\|inspect\|quality' 2>/dev/null; then
    PERSONA_SUGGESTION="code-reviewer"
fi
if echo "$PROMPT_TEXT" | grep -qi 'security\|vulner\|threat\|auth\|permission' 2>/dev/null; then
    PERSONA_SUGGESTION="security-auditor"
fi
if echo "$PROMPT_TEXT" | grep -qi 'test\|spec\|assert\|coverage\|tdd' 2>/dev/null; then
    PERSONA_SUGGESTION="test-engineer"
fi

if [ -n "$PERSONA_SUGGESTION" ]; then
    echo "ℹ [pre-dispatch-guide] Persona match: consider installed runtime role '$PERSONA_SUGGESTION'" >&2
    echo "ℹ  Set the explicit agent_type/subagent_type when the current agent surface supports role routing." >&2
fi

exit 0
