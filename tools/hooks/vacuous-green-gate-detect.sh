#!/usr/bin/env bash
# tools/hooks/vacuous-green-gate-detect.sh
#
# Trigger:  Claude Code PostToolUse(Edit|Write) where target is registry/quality/lanes.yaml
#           or under libs/check-*/
# Purpose:  Detect potential vacuous-green test patterns: gates that pass with zero
#           assertions or trivially true bodies. Advisory measurement, not enforcement.
# Behavior: Greps the edited file for known vacuous-green heuristics:
#           assert!(true), Ok(()) as sole body, tests with zero assertions.
#           Prints a warning citing ADR-0221 mistake M-06. Advisory only.
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

# Only act on lanes.yaml or check-* crate files
if ! echo "$FILE_PATH" | grep -qE 'registry/quality/lanes\.yaml|libs/check-' 2>/dev/null; then
    exit 0
fi

ABS_PATH="$REPO_ROOT/$FILE_PATH"
if [[ "$FILE_PATH" = /* ]]; then
    ABS_PATH="$FILE_PATH"
fi
if [ ! -f "$ABS_PATH" ]; then
    exit 0
fi

ISSUES=""

# Heuristic 1: assert!(true) — trivially passing assertion
if grep -q 'assert!(true)' "$ABS_PATH" 2>/dev/null; then
    ISSUES="$ISSUES assert!(true)"
fi

# Heuristic 2: Ok(()) as the only expression in a function body (vacuous test body)
if grep -qE '^\s*Ok\(\(\)\)\s*$' "$ABS_PATH" 2>/dev/null; then
    ISSUES="$ISSUES Ok(())-only-body"
fi

# Heuristic 3: #[test] functions with no assert/expect calls (zero-assertion tests)
if grep -q '#\[test\]' "$ABS_PATH" 2>/dev/null; then
    if ! grep -qE 'assert|expect|panic|Err' "$ABS_PATH" 2>/dev/null; then
        ISSUES="$ISSUES zero-assertion-tests"
    fi
fi

if [ -n "$ISSUES" ]; then
    echo "ℹ [vacuous-green-gate-detect] Possible vacuous-green pattern in $FILE_PATH:${ISSUES}." >&2
    echo "ℹ  Per ADR-0221 mistake M-06, lanes must fail on intended-failure inputs to be honest." >&2
    echo "ℹ  Ensure tests exercise actual validator logic and assert on real expected outputs." >&2
fi

exit 0
