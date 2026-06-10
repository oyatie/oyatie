#!/usr/bin/env bash
# tools/hooks/spec-version-pin-suggester.sh
#
# Trigger:  PostToolUse(Edit|MultiEdit|Write) via .codex/hooks.json + .claude/settings.json;
#           acts on any *.yaml/*.yml/*.json edit (repo-wide, not just contracts/)
# Purpose:  After editing a contract file, check spec versions and suggest corrections
#           if OpenAPI != 3.2.0 or AsyncAPI != 3.1.0. Advisory only.
# Behavior: Greps the written file for version declarations. If wrong version found,
#           prints a correction suggestion citing spec.openapis.org / asyncapi.com.
#           If correct, silent.
# Non-blocking guarantee: exits 0 always.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Extract file path from tool input.
# WHY the .tool_input.* keys: real Claude Code (and Codex) deliver hook input as a
# JSON object on STDIN with the path nested under tool_input (PostToolUse shape:
# {"tool_input":{"file_path":"..."}}) — confirmed by code.claude.com/docs hooks
# reference + developers.openai.com/codex/hooks (both: "all hooks receive JSON on
# stdin; no env var carries the event data"). The flat .path/.file_path keys are
# retained for the TOOL_INPUT env path used only by the CI governance harness
# (tools/governance/adr-0221-governance-gates.sh), so both surfaces keep working.
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

# Only act on contract files
if [ -z "$FILE_PATH" ]; then
    exit 0
fi
if ! echo "$FILE_PATH" | grep -qiE '\.(yaml|yml|json)$' 2>/dev/null; then
    exit 0
fi

# Resolve absolute path
ABS_PATH="$REPO_ROOT/$FILE_PATH"
if [[ "$FILE_PATH" = /* ]]; then
    ABS_PATH="$FILE_PATH"
fi
if [ ! -f "$ABS_PATH" ]; then
    exit 0
fi

# Check OpenAPI version
OPENAPI_VER=$(grep -oE "openapi:[[:space:]]*['\"]?[0-9]+\.[0-9]+\.[0-9]+" "$ABS_PATH" 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || true)
if [ -n "$OPENAPI_VER" ] && [ "$OPENAPI_VER" != "3.2.0" ]; then
    echo "ℹ [spec-version-pin-suggester] Detected openapi: $OPENAPI_VER in $FILE_PATH" >&2
    echo "ℹ  As of 2026-05-18, canonical version is 3.2.0 (spec.openapis.org/oas/v3.2.0)." >&2
    echo "ℹ  Consider updating unless you have a verified source for $OPENAPI_VER." >&2
fi

# Check AsyncAPI version
ASYNCAPI_VER=$(grep -oE "asyncapi:[[:space:]]*['\"]?[0-9]+\.[0-9]+\.[0-9]+" "$ABS_PATH" 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || true)
if [ -n "$ASYNCAPI_VER" ] && [ "$ASYNCAPI_VER" != "3.1.0" ]; then
    echo "ℹ [spec-version-pin-suggester] Detected asyncapi: $ASYNCAPI_VER in $FILE_PATH" >&2
    echo "ℹ  As of 2026-05-18, canonical version is 3.1.0 (asyncapi.com/docs/reference/specification/v3.1.0)." >&2
    echo "ℹ  Consider updating unless you have a verified source for $ASYNCAPI_VER." >&2
fi

exit 0
