#!/usr/bin/env bash
# tools/hooks/cargo-verify-on-rust-edit.sh
#
# Trigger:  Claude Code PostToolUse(Edit|Write) where target ends in .rs
# Purpose:  After a Rust file is edited, run cargo check on the affected crate
#           and surface any compile errors as advisory output. Measurement, not enforcement.
# Behavior: Derives the crate from the edited file path. Runs `cargo check` with a
#           30-second budget. Prints errors to stderr; exits 0 either way.
#           Skip if tools/hooks/.cargo-verify-disabled flag file exists (escape hatch).
# Non-blocking guarantee: exits 0 always.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Escape hatch: skip if disabled flag exists
if [ -f "$REPO_ROOT/tools/hooks/.cargo-verify-disabled" ]; then
    exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
    exit 0
fi

# Extract file path from tool input
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

# Only act on .rs files
if [ -z "$FILE_PATH" ] || ! echo "$FILE_PATH" | grep -q '\.rs$' 2>/dev/null; then
    exit 0
fi

# Derive crate root from file path: walk up to the first Cargo.toml
ABS_PATH="$REPO_ROOT/$FILE_PATH"
# Strip leading slash if FILE_PATH is absolute
if [[ "$FILE_PATH" = /* ]]; then
    ABS_PATH="$FILE_PATH"
fi

CRATE_ROOT=""
SEARCH_DIR="$(dirname "$ABS_PATH")"
while [ "$SEARCH_DIR" != "/" ] && [ "$SEARCH_DIR" != "$REPO_ROOT" ]; do
    if [ -f "$SEARCH_DIR/Cargo.toml" ]; then
        CRATE_ROOT="$SEARCH_DIR"
        break
    fi
    SEARCH_DIR="$(dirname "$SEARCH_DIR")"
done

# Fallback to repo root if no crate Cargo.toml found
if [ -z "$CRATE_ROOT" ]; then
    CRATE_ROOT="$REPO_ROOT"
fi

echo "ℹ [cargo-verify] Running cargo check on $(basename "$CRATE_ROOT") (30s budget)..." >&2

CHECK_OUTPUT=$(timeout 30 cargo check \
    --manifest-path "$CRATE_ROOT/Cargo.toml" \
    --message-format=short \
    --quiet 2>&1 || true)

if [ -n "$CHECK_OUTPUT" ]; then
    echo "ℹ [cargo-verify] cargo check output for $(basename "$CRATE_ROOT"):" >&2
    echo "$CHECK_OUTPUT" >&2
else
    echo "ℹ [cargo-verify] cargo check clean for $(basename "$CRATE_ROOT")." >&2
fi

exit 0
