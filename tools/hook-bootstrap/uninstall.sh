#!/usr/bin/env bash
# tools/hook-bootstrap/uninstall.sh
#
# Purpose: Reverse everything tools/hook-bootstrap/install.sh installed.
#          Removes hook entries from .claude/settings.json, removes .codex/hooks.json,
#          .gemini/settings.json, removes PATH_add bin from .envrc if we added it,
#          and removes legacy .hermes/hooks.json artifacts left by pre-ADR-0335 install.sh
#          (Hermes CLI support retired per ADR-0335 Wave 15I + ADR-0247 D-10; the
#          uninstall block remains so users with legacy installs can clean up).
#          Preserves agent-skills by default (useful even without hooks).
#
# Usage:
#   ./tools/hook-bootstrap/uninstall.sh           # interactive uninstall
#   ./tools/hook-bootstrap/uninstall.sh --dry-run # preview without writing
#
# Safety: only removes entries bearing the "oya-bootstrap-v1" marker.
#         Never touches user-level settings (~/.claude, ~/.codex, ~/.gemini).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
MARKER="oya-bootstrap-v1"

DRY_RUN=false

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=true ;;
        --help|-h)
            echo "Usage: $0 [--dry-run]"
            echo "  --dry-run  Print planned removals without writing anything"
            exit 0
            ;;
        *)
            echo "Unknown option: $arg" >&2
            exit 1
            ;;
    esac
done

# ── Helpers ─────────────────────────────────────────────────────────────────

ok()   { echo "✓ $*"; }
info() { echo "ℹ $*"; }
dry()  { echo "[dry-run] $*"; }
warn() { echo "⚠ $*" >&2; }

echo ""
echo "=== Oyatie Contributor Bootstrap — Uninstall ==="
echo ""

REMOVED_COUNT=0

# ── Remove .claude/settings.json hooks ──────────────────────────────────────

SETTINGS_FILE="$REPO_ROOT/.claude/settings.json"
if [ -f "$SETTINGS_FILE" ] && grep -q "\"$MARKER\"" "$SETTINGS_FILE" 2>/dev/null; then
    if $DRY_RUN; then
        dry "Would remove hooks bearing marker '$MARKER' from $SETTINGS_FILE"
        dry "  (removes all hook entries with \"marker\": \"$MARKER\")"
    else
        if command -v python3 >/dev/null 2>&1; then
            python3 - "$SETTINGS_FILE" "$MARKER" <<'PYEOF'
import json, sys
path, marker = sys.argv[1], sys.argv[2]
with open(path) as f:
    data = json.load(f)
if "hooks" not in data:
    sys.exit(0)
for event, entries in list(data["hooks"].items()):
    data["hooks"][event] = [e for e in entries if e.get("marker") != marker]
    if not data["hooks"][event]:
        del data["hooks"][event]
if not data["hooks"]:
    del data["hooks"]
with open(path, "w") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
PYEOF
            ok "Removed bootstrap hooks from .claude/settings.json"
        else
            warn "python3 not available; cannot surgically remove hooks."
            warn "To uninstall manually: remove entries with \"marker\": \"$MARKER\" from $SETTINGS_FILE"
        fi
    fi
    REMOVED_COUNT=$((REMOVED_COUNT + 1))
else
    info ".claude/settings.json has no bootstrap hooks to remove (already clean)"
fi

# ── Remove .codex/hooks.json if we created it ───────────────────────────────

CODEX_HOOKS="$REPO_ROOT/.codex/hooks.json"
if [ -f "$CODEX_HOOKS" ] && grep -q "\"$MARKER\"" "$CODEX_HOOKS" 2>/dev/null; then
    if $DRY_RUN; then
        dry "Would remove $CODEX_HOOKS (created by install.sh; contains marker '$MARKER')"
    else
        rm -f "$CODEX_HOOKS"
        # Remove .codex/ dir if now empty
        rmdir "$REPO_ROOT/.codex" 2>/dev/null || true
        ok "Removed .codex/hooks.json"
    fi
    REMOVED_COUNT=$((REMOVED_COUNT + 1))
else
    info ".codex/hooks.json not present or not managed by bootstrap (skipping)"
fi

# ── Remove .gemini/settings.json if we created it ───────────────────────────

GEMINI_SETTINGS="$REPO_ROOT/.gemini/settings.json"
if [ -f "$GEMINI_SETTINGS" ] && grep -q "\"$MARKER\"" "$GEMINI_SETTINGS" 2>/dev/null; then
    if $DRY_RUN; then
        dry "Would remove $GEMINI_SETTINGS (created by install.sh; contains marker '$MARKER')"
    else
        rm -f "$GEMINI_SETTINGS"
        rm -f "$GEMINI_SETTINGS.oya-bootstrap-example" 2>/dev/null || true
        # Remove .gemini/ dir if now empty (preserve if upstream commands/ etc. live there)
        rmdir "$REPO_ROOT/.gemini" 2>/dev/null || true
        ok "Removed .gemini/settings.json"
    fi
    REMOVED_COUNT=$((REMOVED_COUNT + 1))
else
    info ".gemini/settings.json not present or not managed by bootstrap (skipping)"
fi

# ── Legacy cleanup: remove .hermes/hooks.json if a pre-ADR-0335 install.sh created it
# (Hermes CLI support retired per ADR-0335 Wave 15I + ADR-0247 D-10; this block remains
# so users with legacy installs can clean up; new installs no longer create this file)

HERMES_HOOKS="$REPO_ROOT/.hermes/hooks.json"
if [ -f "$HERMES_HOOKS" ] && grep -q "\"$MARKER\"" "$HERMES_HOOKS" 2>/dev/null; then
    if $DRY_RUN; then
        dry "Would remove $HERMES_HOOKS (created by install.sh; contains marker '$MARKER')"
    else
        rm -f "$HERMES_HOOKS"
        rm -f "$HERMES_HOOKS.oya-bootstrap-example" 2>/dev/null || true
        rmdir "$REPO_ROOT/.hermes" 2>/dev/null || true
        ok "Removed .hermes/hooks.json"
    fi
    REMOVED_COUNT=$((REMOVED_COUNT + 1))
else
    info ".hermes/hooks.json not present or not managed by bootstrap (skipping)"
fi

# ── Remove symlinks to vendored commands + skills ───────────────────────────
#
# Remove only symlinks that point back at tools/agent-skills/. User-authored
# files at .<agent>/commands/<name> or .<agent>/skills/ are preserved.

remove_agent_symlink() {
    local label="$1"
    local link="$2"           # path to symlink
    local expected_prefix="$3"  # required substring of readlink target

    if [ -L "$link" ]; then
        local tgt
        tgt="$(readlink "$link")"
        case "$tgt" in
            *"$expected_prefix"*)
                if $DRY_RUN; then
                    dry "Would remove symlink $link → $tgt ($label)"
                else
                    rm -f "$link"
                    ok "Removed $label symlink: $link"
                fi
                REMOVED_COUNT=$((REMOVED_COUNT + 1))
                ;;
            *)
                info "$link is a symlink but not to our vendored path — preserving"
                ;;
        esac
    fi
}

# Per-file command symlinks (Claude + Gemini)
for f in "$REPO_ROOT"/.claude/commands/*.md "$REPO_ROOT"/.gemini/commands/*.toml; do
    [ -L "$f" ] || continue
    remove_agent_symlink "command" "$f" "tools/agent-skills"
done

# Per-agent skills directory symlinks
for agent in claude codex gemini hermes; do
    remove_agent_symlink "$agent-skills-dir" "$REPO_ROOT/.${agent}/skills" "tools/agent-skills/skills"
done

# Clean up empty .<agent>/commands/ dirs if we created them
# (rmdir silently no-ops on non-empty or non-existent; || true catches both)
for d in "$REPO_ROOT"/.claude/commands "$REPO_ROOT"/.gemini/commands; do
    rmdir "$d" 2>/dev/null || true
done

# ── Remove PATH_add bin from .envrc if we added it ──────────────────────────

ENVRC="$REPO_ROOT/.envrc"
if [ -f "$ENVRC" ] && grep -q 'PATH_add bin' "$ENVRC" 2>/dev/null; then
    if $DRY_RUN; then
        dry "Would remove 'PATH_add bin' line from $ENVRC"
    else
        # Remove the PATH_add bin line (and its comment if directly preceding)
        TMP_ENVRC=$(mktemp)
        grep -v 'PATH_add bin' "$ENVRC" > "$TMP_ENVRC" || true
        mv "$TMP_ENVRC" "$ENVRC"
        ok "Removed 'PATH_add bin' from .envrc"
    fi
    REMOVED_COUNT=$((REMOVED_COUNT + 1))
else
    info ".envrc has no 'PATH_add bin' entry to remove"
fi

# ── Prompt about agent-skills ────────────────────────────────────────────────

SKILLS_DIR="$REPO_ROOT/tools/agent-skills"
if [ -f "$SKILLS_DIR/.vendored" ] || [ -d "$SKILLS_DIR" ]; then
    echo ""
    if $DRY_RUN; then
        dry "Would prompt: Remove vendored tools/agent-skills/? [y/N]"
        dry "  Default: preserve (agent-skills are useful even without hooks)"
    else
        printf "Remove vendored tools/agent-skills/? [y/N] "
        read -r REMOVE_SKILLS </dev/tty 2>/dev/null || REMOVE_SKILLS="N"
        case "$REMOVE_SKILLS" in
            y|Y|yes|YES)
                rm -rf "$SKILLS_DIR"
                ok "Removed tools/agent-skills/"
                REMOVED_COUNT=$((REMOVED_COUNT + 1))
                ;;
            *)
                info "Preserving tools/agent-skills/ (useful even without bootstrap hooks)"
                ;;
        esac
    fi
fi

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo "=== Uninstall Summary ==="
echo ""
if $DRY_RUN; then
    ok "[dry-run] No changes written. Remove --dry-run to apply."
else
    if [ "$REMOVED_COUNT" -gt 0 ]; then
        ok "$REMOVED_COUNT component(s) removed"
        info "bin/oya and tools/hooks/ remain in the repo (VCS-tracked; not removed by bootstrap)"
        info "To reinstall: ./tools/hook-bootstrap/install.sh"
    else
        info "Nothing to remove — bootstrap was not installed or already clean"
    fi
fi
echo ""

exit 0
