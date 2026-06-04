#!/usr/bin/env bash
# Shift-left validator for agent worktrees. Mirrors the Buck2/cloud-ci gate shape
# without using Cargo as build/test authority.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

BUCK2="${BUCK2:-buck2}"
RUSTFMT_BIN="${RUSTFMT_BIN:-${HOME}/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustfmt}"
EDITION="${RUST_EDITION:-2024}"
BASE_REF="${OYA_CI_BASE_REF:-origin/dev}"
ALLOW_DEPS=0

while [ $# -gt 0 ]; do
  case "$1" in
    --allow-deps)
      ALLOW_DEPS=1
      shift
      ;;
    --touched-crate)
      # Retained for caller compatibility; Buck2 selects affected targets.
      shift 2
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 1
      ;;
  esac
done

FAIL=0
fail() { echo "::FAIL:: $*" >&2; FAIL=1; }
pass() { echo "::PASS:: $*"; }
warn() { echo "::WARN:: $*" >&2; }

command -v "$BUCK2" >/dev/null 2>&1 || fail "buck2 not found; install/use the Buck2 toolchain image before pushing"

if ! git rev-parse --verify "$BASE_REF" >/dev/null 2>&1; then
  if [ "$BASE_REF" = "origin/dev" ] && git rev-parse --verify github-mirror/dev >/dev/null 2>&1; then
    warn "origin/dev unavailable; using github-mirror/dev for local bridge evidence"
    BASE_REF="github-mirror/dev"
  else
    fail "$BASE_REF cannot be resolved — fetch the dev base ref or set OYA_CI_BASE_REF"
  fi
fi

DELETED=$(git status --porcelain | grep -E '^(D | D)' | awk '{print $NF}' || true)
if [ -n "$DELETED" ]; then
  fail "working tree has deletions (worktree-drift?): $(printf '%s\n' "$DELETED" | tr '\n' ' ')"
fi

if buck2 build //:buck2-authority-policy-check; then
  pass "Buck2 authority policy clean"
else
  fail "Buck2 authority policy scan failed"
fi

STAGED_RS=$(git diff --name-only --cached --diff-filter=AMR 2>/dev/null | grep '\.rs$' || true)
BRANCH_RS=$(git diff --name-only --diff-filter=AMR "$BASE_REF"..HEAD 2>/dev/null | grep '\.rs$' || true)
CHANGED_RS=$(printf '%s\n%s\n' "$STAGED_RS" "$BRANCH_RS" | grep -v '^$' | sort -u)
if [ -n "$CHANGED_RS" ]; then
  if [ ! -x "$RUSTFMT_BIN" ]; then
    fail "rustfmt binary not found at $RUSTFMT_BIN; do not fall back to Cargo fmt"
  else
    DIFFS=0
    while IFS= read -r file; do
      [ -f "$file" ] || continue
      if ! "$RUSTFMT_BIN" --edition "$EDITION" --check "$file" >/dev/null 2>&1; then
        DIFFS=$((DIFFS + 1))
        echo "  not formatted: $file"
      fi
    done <<< "$CHANGED_RS"
    if [ "$DIFFS" -gt 0 ]; then
      fail "rustfmt produced $DIFFS diffs — run rustfmt directly or through Buck2 formatting target"
    else
      pass "rustfmt clean ($(printf '%s\n' "$CHANGED_RS" | wc -l | tr -d ' ') file(s))"
    fi
  fi
fi

if command -v "$BUCK2" >/dev/null 2>&1 && git rev-parse --verify "$BASE_REF" >/dev/null 2>&1; then
  if infra/ci/buck2-affected-gate.sh "$BASE_REF" HEAD; then
    pass "Buck2 affected build/test gate clean"
  else
    fail "Buck2 affected build/test gate failed"
  fi
fi

COMMITTED_EVIDENCE=$(git diff --name-only --diff-filter=A "$BASE_REF"..HEAD 2>/dev/null | grep '^evidence/multispectrum/.*\.json$' || true)
STAGED_EVIDENCE=$(git diff --name-only --cached --diff-filter=AM 2>/dev/null | grep '^evidence/multispectrum/.*\.json$' || true)
NEW_EVIDENCE=$(printf '%s\n%s\n' "$COMMITTED_EVIDENCE" "$STAGED_EVIDENCE" | grep -v '^$' | sort -u)
if [ -n "$NEW_EVIDENCE" ]; then
  if [ ! -f evidence/audit-chain.jsonl ]; then
    fail "evidence/multispectrum files present but evidence/audit-chain.jsonl is missing"
  else
    while IFS= read -r file; do
      [ -f "$file" ] || continue
      cid=$(python3 - "$file" <<'PYEOF' 2>/dev/null
import json
import sys

try:
    with open(sys.argv[1]) as fh:
        print(json.load(fh).get("change_id", ""))
except Exception:
    pass
PYEOF
)
      if [ -z "$cid" ]; then
        fail "evidence file $(basename "$file") has no readable change_id"
      elif ! grep -qF "\"$cid\"" evidence/audit-chain.jsonl; then
        fail "evidence/multispectrum/$(basename "$file") has change_id=$cid but no matching audit-chain row"
      else
        pass "audit-chain row matches: $cid"
      fi
    done <<< "$NEW_EVIDENCE"
  fi
fi

COMMITTED_DEPS=$(git diff "$BASE_REF"..HEAD -- Cargo.toml 2>/dev/null | grep -E '^\+[a-zA-Z0-9_-]+ ?=( ?"[^"]*"|\{)' || true)
STAGED_DEPS=$(git diff --cached -- Cargo.toml 2>/dev/null | grep -E '^\+[a-zA-Z0-9_-]+ ?=( ?"[^"]*"|\{)' || true)
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

echo
if [ "$FAIL" -eq 0 ]; then
  echo "ALL BUCK2 GATES PASSED — local evidence only; cloud-ci/oya-ci required context remains the merge authority."
  exit 0
fi

echo "VALIDATION FAILED — fix above issues before pushing."
exit 1
