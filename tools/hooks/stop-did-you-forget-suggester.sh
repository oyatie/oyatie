#!/usr/bin/env bash
# tools/hooks/stop-did-you-forget-suggester.sh
#
# Trigger:  Claude Code Stop
# Purpose:  Before session ends, check for common incomplete-work signals and
#           suggest next steps to the agent. Advisory only — never blocks.
# Behavior: Runs three fast checks (each with a 5-second budget):
#           1. Dirty Rust workspace (uncommitted .rs changes, detected via git status)
#           2. ADR-NNNN references in recent diff without corresponding files
#           3. Lane regression markers in registry/quality/lanes.yaml
#           Prints suggestions to stderr; exits 0 either way.
# Non-blocking guarantee: exits 0 always; all checks abort gracefully on timeout.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

FOUND_SUGGESTIONS=0

# Helper: print a suggestion
suggest() {
    echo "ℹ [stop-hook] $*" >&2
    FOUND_SUGGESTIONS=1
}

# Check 1: Dirty Rust files (5-second budget)
DIRTY_RS=$(timeout 5 git -C "$REPO_ROOT" status --porcelain 2>/dev/null | grep '\.rs$' | head -5 || true)
if [ -n "$DIRTY_RS" ]; then
    suggest "Uncommitted Rust changes detected. Consider: buck2 build //...[check] before closing (cargo retired — Buck2 takeover)."
fi

# Check 2: Orphan ADR references in recent diff (5-second budget)
RECENT_DIFF=$(timeout 5 git -C "$REPO_ROOT" diff HEAD 2>/dev/null || true)
if [ -n "$RECENT_DIFF" ]; then
    ORPHAN_ADRS=$(echo "$RECENT_DIFF" | grep -oE 'ADR-[0-9]{4}' | sort -u | while read -r adr; do
        num="${adr#ADR-}"
        # Check if any file matching docs/decisions/ADR-NNNN-*.md exists
        matches=$(find "$REPO_ROOT/docs/decisions" -name "ADR-${num}-*.md" 2>/dev/null | head -1)
        if [ -z "$matches" ]; then
            echo "$adr"
        fi
    done || true)
    if [ -n "$ORPHAN_ADRS" ]; then
        suggest "Orphan ADR references in diff (no docs/decisions/ADR-NNNN-*.md found): $ORPHAN_ADRS"
        suggest "Consider creating ADR stubs or verifying the ADR number is correct."
    fi
fi

# Check 3: Lane regression markers (5-second budget)
LANES_FILE="$REPO_ROOT/registry/quality/lanes.yaml"
if [ -f "$LANES_FILE" ]; then
    REGRESSIONS=$(timeout 5 grep -n 'status:.*fail\|status:.*FAIL\|regression' "$LANES_FILE" 2>/dev/null | head -5 || true)
    if [ -n "$REGRESSIONS" ]; then
        suggest "Possible lane regressions in registry/quality/lanes.yaml:"
        echo "$REGRESSIONS" | while IFS= read -r line; do
            suggest "  $line"
        done
    fi
fi

if [ "$FOUND_SUGGESTIONS" -eq 0 ]; then
    echo "ℹ [stop-hook] No open concerns detected. Good session." >&2
fi

exit 0
