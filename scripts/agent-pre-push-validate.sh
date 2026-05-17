#!/usr/bin/env bash
# agent-pre-push-validate.sh
#
# Shift-left validator. Run BEFORE git push from any agent worktree.
# Mirrors the CI required-checks set so failures surface locally first,
# avoiding round-trips through GitHub Actions.
#
# Usage:
#   bash scripts/agent-pre-push-validate.sh [--touched-crate NAME]...
#
# Exit codes:
#   0 = all gates passed; safe to push
#   1 = a gate failed; do not push without fixing
#
# Gates (mirror PR-required checks):
#   * Working tree is clean (no spurious deletions from worktree drift, staged or unstaged)
#   * rustfmt 1.95.0 (edition 2024) — must produce no diff on touched .rs files
#   * cargo check --workspace (mirrors CI cargo-check job)
#   * cargo clippy --workspace --all-targets -D warnings (mirrors CI)
#   * cargo nextest run --workspace --no-fail-fast (mirrors CI)
#   * evidence/multispectrum/<change_id>.json ↔ evidence/audit-chain.jsonl pairing
#   * Cargo.toml diff has no NEW workspace-level deps (warn; explicit override = --allow-deps)

set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT" || exit 1

RUSTFMT_BIN="${HOME}/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustfmt"
EDITION="2024"
TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target-validate}"
ALLOW_DEPS=0
TOUCHED_CRATES=()

while [ $# -gt 0 ]; do
    case "$1" in
        --touched-crate) TOUCHED_CRATES+=("$2"); shift 2 ;;
        --allow-deps) ALLOW_DEPS=1; shift ;;
        *) echo "unknown arg: $1" >&2; exit 1 ;;
    esac
done

FAIL=0
fail() { echo "::FAIL:: $*" >&2; FAIL=1; }
pass() { echo "::PASS:: $*"; }
warn() { echo "::WARN:: $*" >&2; }

# ── Baseline: verify origin/dev is resolvable ──────────────────────────
if ! git rev-parse --verify origin/dev >/dev/null 2>&1; then
    fail "origin/dev cannot be resolved — fetch origin first (git fetch origin dev)"
    echo "VALIDATION FAILED — fix above issues before pushing."
    exit 1
fi

# ── Gate 1: working-tree cleanliness ───────────────────────────────────
# Match both staged deletions (^D  ) and unstaged deletions (^ D ).
DELETED=$(git status --porcelain | grep -E '^(D | D)' | awk '{print $NF}' || true)
if [ -n "$DELETED" ]; then
    fail "working tree has deletions (worktree-drift?): $(echo "$DELETED" | tr '\n' ' ')"
    echo "  Fix: git checkout HEAD -- <each-file>"
fi

# ── Gate 2: rustfmt on changed .rs files ───────────────────────────────
# Union of staged and branch-level changed .rs files.
STAGED_RS=$(git diff --name-only --cached --diff-filter=AMR 2>/dev/null | grep '\.rs$' || true)
BRANCH_RS=$(git diff --name-only --diff-filter=AMR origin/dev..HEAD 2>/dev/null | grep '\.rs$' || true)
CHANGED_RS=$(printf '%s\n%s\n' "$STAGED_RS" "$BRANCH_RS" | grep -v '^$' | sort -u)

if [ -n "$CHANGED_RS" ]; then
    DIFFS=0
    while IFS= read -r F; do
        [ -f "$F" ] || continue
        if [ -x "$RUSTFMT_BIN" ]; then
            # Direct binary invocation — unambiguous single executable.
            if ! "$RUSTFMT_BIN" --edition "$EDITION" --check "$F" >/dev/null 2>&1; then
                DIFFS=$((DIFFS+1))
                echo "  not formatted: $F"
            fi
        else
            warn "rustfmt 1.95.0 binary not found at $RUSTFMT_BIN; falling back to cargo fmt --check"
            # Fallback: rustfmt flags (--edition, --check) go after the -- separator.
            if ! cargo fmt -- --edition "$EDITION" --check "$F" >/dev/null 2>&1; then
                DIFFS=$((DIFFS+1))
                echo "  not formatted: $F"
            fi
        fi
    done <<< "$CHANGED_RS"
    if [ "$DIFFS" -gt 0 ]; then
        fail "rustfmt produced $DIFFS diffs — run: rustfmt --edition $EDITION <files>"
    else
        pass "rustfmt clean ($(printf '%s\n' "$CHANGED_RS" | wc -l | tr -d ' ') file(s))"
    fi
fi

# ── Crate inference: from both .rs files AND Cargo.toml/build.rs changes ──
# Covers: Cargo.toml, build.rs, and *.rs under crates/* or tools/*.
if [ "${#TOUCHED_CRATES[@]}" -eq 0 ]; then
    # All changed files (not just .rs) to detect Cargo.toml/build.rs changes.
    STAGED_ALL=$(git diff --name-only --cached --diff-filter=AMR 2>/dev/null || true)
    BRANCH_ALL=$(git diff --name-only --diff-filter=AMR origin/dev..HEAD 2>/dev/null || true)
    CHANGED_ALL=$(printf '%s\n%s\n' "$STAGED_ALL" "$BRANCH_ALL" | grep -v '^$' | sort -u)
    while IFS= read -r F; do
        C=$(echo "$F" | grep -oE '^(crates|tools)/[^/]+' | head -1 | sed 's|^\(crates\|tools\)/||')
        [ -n "$C" ] && TOUCHED_CRATES+=("$C")
    done <<< "$CHANGED_ALL"
fi
# Dedup
if [ "${#TOUCHED_CRATES[@]}" -gt 0 ]; then
    TOUCHED_CRATES=($(printf '%s\n' "${TOUCHED_CRATES[@]}" | sort -u))
fi

# ── Gate 3: cargo check --workspace ────────────────────────────────────
# Mirrors CI cargo-check job (pr-tests.yml, job cargo-check).
# --all-targets includes tests/examples/benches to match CI scope exactly.
echo "Running cargo check --workspace --all-targets..."
if ! CARGO_TARGET_DIR="$TARGET_DIR" cargo check --workspace --all-targets 2>&1; then
    fail "cargo check --workspace --all-targets failed"
else
    pass "cargo check --workspace --all-targets clean"
fi

# ── Gate 4: clippy --workspace ─────────────────────────────────────────
# Mirrors CI: cargo clippy --workspace --all-targets --keep-going -- -D warnings
echo "Running cargo clippy --workspace..."
CLIPPY_OUT=$(CARGO_TARGET_DIR="$TARGET_DIR" cargo clippy --workspace --all-targets --keep-going -- -D warnings 2>&1)
CLIPPY_RC=$?
if [ $CLIPPY_RC -ne 0 ]; then
    ERR=$(echo "$CLIPPY_OUT" | grep -E '^error' | head -5)
    fail "clippy --workspace: $ERR"
else
    pass "clippy --workspace clean"
fi

# ── Gate 5: nextest --workspace ────────────────────────────────────────
# Mirrors CI: cargo nextest run --workspace --no-fail-fast
echo "Running cargo nextest run --workspace..."
NEXTEST_OUT=$(CARGO_TARGET_DIR="$TARGET_DIR" cargo nextest run --workspace --no-fail-fast 2>&1)
NEXTEST_RC=$?
if [ $NEXTEST_RC -ne 0 ]; then
    fail "nextest --workspace failed: $(echo "$NEXTEST_OUT" | tail -5)"
else
    pass "nextest --workspace clean"
fi

# ── Gate 6: evidence/multispectrum ↔ audit-chain pairing ───────────────
# Union of newly committed and currently staged evidence files.
COMMITTED_EVIDENCE=$(git diff --name-only --diff-filter=A origin/dev..HEAD 2>/dev/null | grep '^evidence/multispectrum/.*\.json$' || true)
STAGED_EVIDENCE=$(git diff --name-only --cached --diff-filter=AM 2>/dev/null | grep '^evidence/multispectrum/.*\.json$' || true)
NEW_EVIDENCE=$(printf '%s\n%s\n' "$COMMITTED_EVIDENCE" "$STAGED_EVIDENCE" | grep -v '^$' | sort -u)

if [ -n "$NEW_EVIDENCE" ]; then
    if [ ! -f evidence/audit-chain.jsonl ]; then
        fail "evidence/multispectrum files present but evidence/audit-chain.jsonl is missing"
    else
        while IFS= read -r F; do
            [ -f "$F" ] || continue
            # Pass path via sys.argv to avoid shell injection from crafted filenames.
            CID=$(python3 - "$F" <<'PYEOF' 2>/dev/null
import json, sys
try:
    print(json.load(open(sys.argv[1])).get('change_id', ''))
except Exception:
    pass
PYEOF
)
            if [ -z "$CID" ]; then
                fail "evidence file $(basename "$F") has no readable change_id — fix or remove it"
            elif ! grep -qF "\"$CID\"" evidence/audit-chain.jsonl; then
                fail "evidence/multispectrum/$(basename "$F") has change_id=$CID but no matching row in evidence/audit-chain.jsonl"
            else
                pass "audit-chain row matches: $CID"
            fi
        done <<< "$NEW_EVIDENCE"
    fi
fi

# ── Gate 7: no new workspace deps (warn unless --allow-deps) ───────────
# Include both committed and staged Cargo.toml changes.
# Restrict to dependency-section lines only (e.g. [dependencies], [workspace.dependencies]).
COMMITTED_DEPS=$(git diff origin/dev..HEAD -- Cargo.toml 2>/dev/null | grep -E '^\+[a-zA-Z0-9_-]+ ?=.*\{' || true)
STAGED_DEPS=$(git diff --cached -- Cargo.toml 2>/dev/null | grep -E '^\+[a-zA-Z0-9_-]+ ?=.*\{' || true)
DEPS_DIFF=$(printf '%s\n%s\n' "$COMMITTED_DEPS" "$STAGED_DEPS" | grep -v '^$' | sort -u)
if [ -n "$DEPS_DIFF" ]; then
    if [ "$ALLOW_DEPS" -eq 1 ]; then
        warn "workspace Cargo.toml adds dependency lines (allowed via --allow-deps):"
        echo "$DEPS_DIFF"
    else
        fail "workspace Cargo.toml adds dependency lines (use --allow-deps to override):"
        echo "$DEPS_DIFF"
    fi
fi

# ── Summary ────────────────────────────────────────────────────────────
echo
if [ "$FAIL" -eq 0 ]; then
    echo "ALL GATES PASSED — safe to push."
    exit 0
else
    echo "VALIDATION FAILED — fix above issues before pushing."
    exit 1
fi
