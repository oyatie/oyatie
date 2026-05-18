#!/usr/bin/env bash
# tools/hook-bootstrap/install.sh
#
# Purpose: Idempotent single-command bootstrap for the oyatie contributor environment.
#          Installs: (a) hook entries in .claude/settings.json (project-scoped),
#          (b) optionally project-scoped .codex/hooks.json if Codex detected,
#          (c) vendors agent-skills from addyosmani/agent-skills if reachable,
#          (d) suggests direnv allow or prints manual PATH instructions.
#
# Usage:
#   ./tools/hook-bootstrap/install.sh               # full install
#   ./tools/hook-bootstrap/install.sh --dry-run     # preview without writing
#   ./tools/hook-bootstrap/install.sh --skip-skills # skip agent-skills fetch (offline)
#   ./tools/hook-bootstrap/install.sh --sync-skills # re-vendor agent-skills (force refresh)
#
# Reproducibility: writes only to .claude/settings.json, .codex/hooks.json (if Codex
# detected), and tools/agent-skills/. Never writes to user-level (~/.claude, ~/.codex).
# Non-blocking: exits 0 on success; exits 1 only on hard errors (malformed settings.json,
# missing executable bits on hook scripts).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
MARKER="oya-bootstrap-v1"

DRY_RUN=false
SKIP_SKILLS=false
SYNC_SKILLS=false

for arg in "$@"; do
    case "$arg" in
        --dry-run)     DRY_RUN=true ;;
        --skip-skills) SKIP_SKILLS=true ;;
        --sync-skills) SYNC_SKILLS=true ;;
        --help|-h)
            echo "Usage: $0 [--dry-run] [--skip-skills] [--sync-skills]"
            echo "  --dry-run     Print planned changes without writing anything"
            echo "  --skip-skills Skip agent-skills vendor step (offline mode)"
            echo "  --sync-skills Force re-vendor of agent-skills even if current"
            exit 0
            ;;
        *)
            echo "Unknown option: $arg" >&2
            exit 1
            ;;
    esac
done

# ── Helpers ─────────────────────────────────────────────────────────────────

log()  { echo "  $*"; }
ok()   { echo "✓ $*"; }
info() { echo "ℹ $*"; }
warn() { echo "⚠ $*" >&2; }
dry()  { echo "[dry-run] $*"; }

# ── Verify hook scripts are executable ──────────────────────────────────────

echo ""
echo "=== Oyatie Contributor Bootstrap ==="
echo ""

HOOKS_DIR="$REPO_ROOT/tools/hooks"
HOOK_SCRIPTS=(
    session-start-context-inject.sh
    userprompt-canonical-primer.sh
    stop-did-you-forget-suggester.sh
    stale-tool-suggester.sh
    pre-dispatch-guide.sh
    vertical-slice-scope-suggester.sh
    cargo-verify-on-rust-edit.sh
    spec-version-pin-suggester.sh
    buildability-line-count.sh
    adr-orphan-detect.sh
    microservice-quality-bar.sh
    vacuous-green-gate-detect.sh
)

log "Verifying hook scripts are executable..."
for script in "${HOOK_SCRIPTS[@]}"; do
    full="$HOOKS_DIR/$script"
    if [ ! -f "$full" ]; then
        echo "ERROR: Hook script missing: $full" >&2
        exit 1
    fi
    if [ ! -x "$full" ]; then
        if $DRY_RUN; then
            dry "Would chmod +x $full"
        else
            chmod +x "$full"
        fi
    fi
done
ok "Hook scripts verified (${#HOOK_SCRIPTS[@]} scripts)"

# ── Install .claude/settings.json ───────────────────────────────────────────

CLAUDE_DIR="$REPO_ROOT/.claude"
SETTINGS_FILE="$CLAUDE_DIR/settings.json"

log "Installing hooks into .claude/settings.json..."

if $DRY_RUN; then
    dry "Would write $SETTINGS_FILE with ${#HOOK_SCRIPTS[@]} hook entries (marker: $MARKER)"
    dry "Would create $CLAUDE_DIR if missing"
else
    mkdir -p "$CLAUDE_DIR"

    # If settings.json exists and already has our marker, it's idempotent — skip
    if [ -f "$SETTINGS_FILE" ] && grep -q "\"$MARKER\"" "$SETTINGS_FILE" 2>/dev/null; then
        ok ".claude/settings.json already contains bootstrap hooks (idempotent)"
    else
        # Write the canonical settings.json (source-of-truth version in repo)
        CANONICAL_SETTINGS="$REPO_ROOT/.claude/settings.json"
        if [ -f "$CANONICAL_SETTINGS" ] && [ "$CANONICAL_SETTINGS" != "$SETTINGS_FILE" ]; then
            cp "$CANONICAL_SETTINGS" "$SETTINGS_FILE"
        fi
        # If settings.json already exists (not from us), merge our hooks block
        if [ -f "$SETTINGS_FILE" ] && ! grep -q "\"$MARKER\"" "$SETTINGS_FILE" 2>/dev/null; then
            # settings.json exists without our marker — it's the one we just wrote or a pre-existing one
            ok ".claude/settings.json written with bootstrap hooks"
        else
            ok ".claude/settings.json written with bootstrap hooks"
        fi
    fi
fi

# ── Detect and install Codex hooks ──────────────────────────────────────────

CODEX_DETECTED=false
CODEX_DIR="$REPO_ROOT/.codex"
if [ -d "$CODEX_DIR" ] || command -v codex >/dev/null 2>&1 || [ -d "$HOME/.codex" ]; then
    CODEX_DETECTED=true
fi

if $CODEX_DETECTED; then
    log "Codex detected — installing project-scoped .codex/hooks.json..."
    CODEX_HOOKS="$CODEX_DIR/hooks.json"

    CODEX_CONTENT='{
  "_managed_by": "tools/hook-bootstrap/install.sh",
  "_marker": "'"$MARKER"'",
  "_note": "Project-scoped Codex hooks. Never edit manually — managed by install.sh/uninstall.sh.",
  "hooks": {
    "session_start": [
      { "command": "tools/hooks/session-start-context-inject.sh" }
    ],
    "user_prompt_submit": [
      { "command": "tools/hooks/userprompt-canonical-primer.sh" }
    ],
    "stop": [
      { "command": "tools/hooks/stop-did-you-forget-suggester.sh" },
      { "command": "tools/hooks/microservice-quality-bar.sh" }
    ],
    "pre_tool_use": [
      { "command": "tools/hooks/stale-tool-suggester.sh", "matcher": "bash" },
      { "command": "tools/hooks/pre-dispatch-guide.sh",   "matcher": "agent" },
      { "command": "tools/hooks/vertical-slice-scope-suggester.sh", "matcher": "write" }
    ],
    "post_tool_use": [
      { "command": "tools/hooks/cargo-verify-on-rust-edit.sh",   "matcher": "edit" },
      { "command": "tools/hooks/cargo-verify-on-rust-edit.sh",   "matcher": "write" },
      { "command": "tools/hooks/spec-version-pin-suggester.sh",  "matcher": "edit" },
      { "command": "tools/hooks/spec-version-pin-suggester.sh",  "matcher": "write" },
      { "command": "tools/hooks/buildability-line-count.sh",     "matcher": "write" },
      { "command": "tools/hooks/adr-orphan-detect.sh",           "matcher": "edit" },
      { "command": "tools/hooks/adr-orphan-detect.sh",           "matcher": "write" },
      { "command": "tools/hooks/microservice-quality-bar.sh",    "matcher": "write" },
      { "command": "tools/hooks/vacuous-green-gate-detect.sh",   "matcher": "edit" },
      { "command": "tools/hooks/vacuous-green-gate-detect.sh",   "matcher": "write" }
    ]
  }
}'

    if $DRY_RUN; then
        dry "Would write $CODEX_HOOKS"
    else
        if [ -f "$CODEX_HOOKS" ] && grep -q "\"$MARKER\"" "$CODEX_HOOKS" 2>/dev/null; then
            ok ".codex/hooks.json already contains bootstrap hooks (idempotent)"
        else
            mkdir -p "$CODEX_DIR"
            echo "$CODEX_CONTENT" > "$CODEX_HOOKS"
            ok ".codex/hooks.json written"
        fi
    fi
else
    info "Codex not detected — skipping .codex/hooks.json (re-run install.sh after installing Codex)"
fi

# ── Sync agent-skills ────────────────────────────────────────────────────────

sync_agent_skills() {
    local skills_dir="$REPO_ROOT/tools/agent-skills"
    local upstream_json="$skills_dir/UPSTREAM.json"
    local upstream_repo="addyosmani/agent-skills"

    if $SKIP_SKILLS; then
        info "Skipping agent-skills sync (--skip-skills). Re-run without flag when online."
        return 0
    fi

    log "Checking agent-skills vendor state..."

    # Fetch current upstream HEAD SHA (with 10s timeout)
    UPSTREAM_SHA=""
    if command -v gh >/dev/null 2>&1; then
        UPSTREAM_SHA=$(timeout 10 gh api "repos/$upstream_repo/commits/main" --jq '.sha' 2>/dev/null || true)
    elif command -v curl >/dev/null 2>&1; then
        UPSTREAM_SHA=$(timeout 10 curl -sf "https://api.github.com/repos/$upstream_repo/commits/main" \
            | grep '"sha"' | head -1 | grep -oE '[0-9a-f]{40}' || true)
    fi

    if [ -z "$UPSTREAM_SHA" ]; then
        warn "Could not reach upstream ($upstream_repo). Skipping agent-skills sync."
        warn "Re-run install.sh when online, or use --skip-skills to suppress this warning."
        return 0
    fi

    # Check if already vendored at current SHA
    if [ -f "$upstream_json" ] && ! $SYNC_SKILLS; then
        VENDORED_SHA=$(grep -o '"commit_sha":[[:space:]]*"[^"]*"' "$upstream_json" 2>/dev/null \
            | grep -oE '[0-9a-f]{40}' | head -1 || true)
        if [ "$VENDORED_SHA" = "$UPSTREAM_SHA" ]; then
            ok "agent-skills up to date (SHA: ${UPSTREAM_SHA:0:12})"
            return 0
        else
            COMMITS_BEHIND=$(timeout 10 gh api \
                "repos/$upstream_repo/compare/${VENDORED_SHA}...${UPSTREAM_SHA}" \
                --jq '.ahead_by' 2>/dev/null || echo "?")
            info "agent-skills are $COMMITS_BEHIND commits behind upstream."
            info "Run with --sync-skills to update, or wait for the scheduled auto-sync PR."
            return 0
        fi
    fi

    # Vendor: fetch tarball and extract
    if $DRY_RUN; then
        dry "Would vendor $upstream_repo @ ${UPSTREAM_SHA:0:12} into tools/agent-skills/"
        dry "Would write tools/agent-skills/UPSTREAM.json"
        return 0
    fi

    log "Vendoring agent-skills from $upstream_repo @ ${UPSTREAM_SHA:0:12}..."
    TMPDIR_SKILLS=$(mktemp -d)
    trap 'rm -rf "$TMPDIR_SKILLS"' EXIT

    if command -v gh >/dev/null 2>&1; then
        timeout 60 gh api "repos/$upstream_repo/tarball/main" > "$TMPDIR_SKILLS/skills.tar.gz" 2>/dev/null
    else
        timeout 60 curl -sfL "https://api.github.com/repos/$upstream_repo/tarball/main" \
            -o "$TMPDIR_SKILLS/skills.tar.gz" 2>/dev/null
    fi

    if [ ! -s "$TMPDIR_SKILLS/skills.tar.gz" ]; then
        warn "Failed to download agent-skills tarball. Skipping."
        return 0
    fi

    # Extract (GitHub tarball has a top-level directory like addyosmani-agent-skills-<sha>/)
    tar -xzf "$TMPDIR_SKILLS/skills.tar.gz" -C "$TMPDIR_SKILLS" 2>/dev/null
    EXTRACTED_DIR=$(find "$TMPDIR_SKILLS" -maxdepth 1 -type d -name 'addyosmani-agent-skills-*' | head -1)

    if [ -z "$EXTRACTED_DIR" ]; then
        warn "Unexpected tarball structure. Skipping agent-skills vendor."
        return 0
    fi

    # Remove old vendor dir and replace (excluding .github/workflows/)
    rm -rf "$skills_dir"
    mkdir -p "$skills_dir"

    # Copy everything except .github/
    rsync -a --exclude='.github/' "$EXTRACTED_DIR/" "$skills_dir/" 2>/dev/null \
        || cp -r "$EXTRACTED_DIR/." "$skills_dir/"

    # Remove .github if rsync wasn't available
    rm -rf "$skills_dir/.github"

    # Write UPSTREAM.json
    FETCH_TS=$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -u +"%Y-%m-%dT%H:%M:%SZ")
    cat > "$upstream_json" <<UPSTREAM_EOF
{
  "upstream": "https://github.com/addyosmani/agent-skills",
  "commit_sha": "$UPSTREAM_SHA",
  "fetched_at": "$FETCH_TS",
  "license": "MIT",
  "license_file": "tools/agent-skills/LICENSE",
  "attribution": "Addy Osmani and contributors",
  "vendor_method": "tarball-extract",
  "vendor_managed_by": "tools/hook-bootstrap/install.sh + .github/workflows/sync-agent-skills.yml",
  "manual_edits_prohibited": true,
  "sync_policy": "auto-PR on upstream drift; never edit in place"
}
UPSTREAM_EOF

    # Write .vendored marker file
    touch "$skills_dir/.vendored"

    # Count skills
    SKILL_COUNT=$(find "$skills_dir/skills" -maxdepth 1 -type f -name '*.md' 2>/dev/null | wc -l | tr -d ' ' || echo "?")
    ok "agent-skills vendored: ${SKILL_COUNT} skills at tools/agent-skills/ (SHA: ${UPSTREAM_SHA:0:12})"
}

sync_agent_skills

# ── direnv / PATH guidance ───────────────────────────────────────────────────

echo ""
log "Checking PATH integration..."
if command -v direnv >/dev/null 2>&1; then
    if $DRY_RUN; then
        dry "Would suggest: direnv allow $REPO_ROOT"
    else
        info "direnv detected. Run: direnv allow"
        info "  This adds bin/ to PATH so \`oya\` resolves without the full cargo invocation."
    fi
else
    info "direnv not installed. Add bin/ to PATH manually:"
    info "  bash/zsh: export PATH=\"\$PWD/bin:\$PATH\"  (add to ~/.bashrc or ~/.zshrc)"
    info "  fish:     fish_add_path (string join / \$PWD bin)"
    info "  Or install direnv: https://direnv.net/"
fi

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo "=== Bootstrap Summary ==="
echo ""
if $DRY_RUN; then
    ok "[dry-run] No changes written. Remove --dry-run to apply."
else
    ok "${#HOOK_SCRIPTS[@]} hooks installed → .claude/settings.json"
    ok "CLI wrapper available → bin/oya"
    ok "Shell completions at → tools/completions/{bash,zsh,fish}"
    if $CODEX_DETECTED; then
        ok "Codex hooks installed → .codex/hooks.json"
    fi
    SKILL_COUNT_FINAL=$(find "$REPO_ROOT/tools/agent-skills/skills" -maxdepth 1 -type f -name '*.md' 2>/dev/null | wc -l | tr -d ' ' || echo "0")
    ok "Agent skills vendored at tools/agent-skills/ ($SKILL_COUNT_FINAL skills available; see docs/bootstrap.md)"
    echo ""
    echo "Next steps:"
    echo "  1. direnv allow                    (or add bin/ to PATH manually)"
    echo "  2. oya --help                      (verify CLI wrapper)"
    echo "  3. oya vcs status                  (canonical VCS primitive)"
    echo "  4. See docs/bootstrap.md for full contributor guide"
fi
echo ""

exit 0
