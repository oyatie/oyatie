#!/usr/bin/env bash
# tools/hooks/microservice-quality-bar.sh
#
# Trigger:  Claude Code Stop OR PostToolUse(Write) on any microservices/<ms>/* path
# Purpose:  Count artifacts in the touched µservice and report if below the 100+
#           artifact bar per ADR-0212 (Buildability Doctrine). Advisory only.
# Behavior: Derives the µservice name from the file path (or scans recent changes
#           on Stop trigger). Counts files under microservices/<ms>/. Silent if >=100.
#           Prints advisory if <100. Budget: 10 seconds via fast `find`.
# Non-blocking guarantee: exits 0 always.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

check_ms_artifact_count() {
    local ms_name="$1"
    local ms_path="$REPO_ROOT/microservices/$ms_name"

    if [ ! -d "$ms_path" ]; then
        return 0
    fi

    local count
    count=$(timeout 10 find "$ms_path" -type f 2>/dev/null | wc -l | tr -d ' ' || echo 0)

    if [ "$count" -lt 100 ]; then
        echo "ℹ [microservice-quality-bar] $ms_name has $count artifacts ($((100 - count)) below the 100+ bar per ADR-0212)." >&2
        echo "ℹ  Buildability doctrine requires 100+ files across docs/, src/, slos/, contracts/, etc." >&2
    fi
    # Silent if >= 100 to avoid noise
}

# Extract file path from tool input (PostToolUse path)
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

if [ -n "$FILE_PATH" ]; then
    # PostToolUse mode: derive µservice from the path
    if echo "$FILE_PATH" | grep -qE '^microservices/[^/]+/' 2>/dev/null; then
        # Extract ms name: strip leading 'microservices/' then everything from second '/' onward
        _tmp="${FILE_PATH#microservices/}"
        MS_NAME="${_tmp%%/*}"
        check_ms_artifact_count "$MS_NAME"
    fi
else
    # Stop mode: check all µservices that have recent uncommitted changes
    CHANGED_MS=$(git -C "$REPO_ROOT" status --porcelain 2>/dev/null \
        | grep 'microservices/' \
        | grep -oE 'microservices/[^/]+' \
        | sed 's|microservices/||' \
        | sort -u || true)
    if [ -n "$CHANGED_MS" ]; then
        while IFS= read -r ms; do
            check_ms_artifact_count "$ms"
        done <<< "$CHANGED_MS"
    fi
fi

exit 0
