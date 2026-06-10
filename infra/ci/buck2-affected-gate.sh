#!/bin/sh
# buck2-native affected-only CI gate.
#
# Builds + tests the reverse-dependency closure of the PR's changed files —
# the hyperscaler "affected targets" pattern (Google/Meta), buck2-native via
# `uquery owner()` + `rdeps()`. Replaces the cargo-era `oya verify --affected`.
# No oya-dev-cli dependency.
#
# Usage:  buck2-affected-gate.sh <base-ref> [head-ref]
#         base-ref  — the merge-base anchor (e.g. origin/dev)
#         head-ref  — the tip to diff (default: HEAD)
#
# The 1-arg form (buck2-affected-gate.sh origin/dev) diffs the current
# checkout: HEAD is the PR checkout in the GitHub Actions runner, so omitting
# head-ref is the default invocation.
#
# The 2-arg form (buck2-affected-gate.sh origin/dev origin/pr-N) is used by
# the controller Job, where the working tree is trunk (dev) and the PR ref
# is fetched as data via `git fetch origin refs/pull/N/head:refs/remotes/origin/pr-N`.
#
# Exit 0 = pass (incl. non-Rust / no-affected PRs); non-zero = build/test failure.
set -eu

BASE="${1:-origin/dev}"
HEAD_REF="${2:-HEAD}"
BUCK2="${BUCK2:-buck2}"

echo "buck2-affected-gate: start (pwd=$(pwd) base=$BASE head-ref=$HEAD_REF resolved=$(git rev-parse --short "$HEAD_REF" 2>&1))"
echo "buck2-affected-gate: .buckconfig=$(test -f .buckconfig && echo present || echo MISSING) HOME=${HOME:-unset} buck2=$($BUCK2 --version 2>&1 | head -1)"
if ! git rev-parse --verify --quiet "$BASE" >/dev/null 2>&1; then
  echo "buck2-affected-gate: FATAL base ref '$BASE' does not resolve in this checkout"
  echo "  remotes: $(git remote 2>&1)  | refs: $(git for-each-ref --format='%(refname)' refs/remotes 2>&1 | paste -sd' ' -)"
  exit 1
fi
if ! git rev-parse --verify --quiet "$HEAD_REF" >/dev/null 2>&1; then
  echo "buck2-affected-gate: FATAL head ref '$HEAD_REF' does not resolve in this checkout"
  echo "  remotes: $(git remote 2>&1)  | refs: $(git for-each-ref --format='%(refname)' refs/remotes 2>&1 | paste -sd' ' -)"
  exit 1
fi
if ! MERGE_BASE=$(git merge-base "$HEAD_REF" "$BASE" 2>&1); then
  echo "buck2-affected-gate: FATAL merge-base $HEAD_REF $BASE failed (need full history): $MERGE_BASE"
  exit 1
fi
CHANGED=$(git diff --name-only "$MERGE_BASE" "$HEAD_REF")
if [ -z "$CHANGED" ]; then
  echo "buck2-affected-gate: no changed files vs $BASE ($HEAD_REF) -> PASS"
  exit 0
fi
echo "buck2-affected-gate: $(printf '%s\n' "$CHANGED" | wc -l | tr -d ' ') changed file(s) vs $BASE..${HEAD_REF} (merge-base $MERGE_BASE)"

# Classify. Only docs/non-graph files (e.g. .md/.yaml/.json outside crates) may
# legitimately map to no target. A *.rs / Cargo.toml / buck-graph file MUST map to
# a target — FAIL CLOSED if it doesn't (never silently pass a Rust change unbuilt).
RUST_REL=$(printf '%s\n' "$CHANGED" | grep -E '\.rs$|/Cargo\.toml$|^Cargo\.(toml|lock)$|^\.buckconfig$|(^|/)BUCK$|^toolchains/|^third-party/' || true)
if [ -z "$RUST_REL" ]; then
  echo "buck2-affected-gate: no Rust/buck-graph files changed -> NoRust PASS"
  exit 0
fi

# owner() resolution — batched to minimise buck2 daemon round-trips.
#
# Strategy:
#   1. BUCK files: no owner() result by design (they ARE the package definition).
#      Run a small per-file pass to expand each to its package target pattern.
#      (One buck2 uquery per BUCK file — these are typically 0-1 files per PR.)
#   2. Non-BUCK Rust/graph files: build ONE "owner('f1') union owner('f2') union ..."
#      expression and run a single buck2 uquery call for all files at once.
#      owner() takes file-path strings, not target-set placeholders, so %Ss/@argfile
#      cannot be used here — the union expression is the correct single-call form.
#      A uquery ERROR (non-zero exit) FAILS the gate — it is NOT 'no owner'.
#      (The false-pass bug was: 2>/dev/null||true swallowed buck2 errors.)

OWNERS=""

# ── Pass 1: BUCK files → package target pattern (unchanged semantics, separate pass) ──
BUCK_FILES=$(printf '%s\n' "$RUST_REL" | grep -E '(^|/)BUCK$' || true)
for f in $BUCK_FILES; do
  [ -e "$f" ] || continue
  d=$(dirname "$f")
  case "$d" in
    third-party)   pat="third-party//:" ;;
    third-party/*) pat="third-party//${d#third-party/}:" ;;
    toolchains)    pat="toolchains//:" ;;
    toolchains/*)  pat="toolchains//${d#toolchains/}:" ;;
    .)             pat="//:" ;;
    *)             pat="//$d:" ;;
  esac
  if ! o=$("$BUCK2" uquery "$pat" 2>/tmp/uqerr); then
    echo "buck2-affected-gate: FATAL buck2 uquery '$pat' (BUCK pkg for $f) errored:"; sed 's/^/    /' /tmp/uqerr; exit 1
  fi
  [ -n "$o" ] && OWNERS="$OWNERS $o"
done

# ── Pass 2: non-BUCK files → ONE batched uquery call via union-of-owner() expression ──
# Build: owner('f1') union owner('f2') union ... and run as a single buck2 uquery invocation.
# This replaces N serial daemon round-trips (one per file) with a single round-trip.
NON_BUCK_FILES=$(printf '%s\n' "$RUST_REL" | grep -vE '(^|/)BUCK$' || true)
NON_BUCK_EXISTING=$(printf '%s\n' "$NON_BUCK_FILES" | while read -r f; do [ -e "$f" ] && printf '%s\n' "$f"; done)
if [ -n "$NON_BUCK_EXISTING" ]; then
  OWNER_EXPR=$(printf '%s\n' "$NON_BUCK_EXISTING" | \
    awk 'NR==1{printf "owner('"'"'%s'"'"')", $0; next} {printf " union owner('"'"'%s'"'"')", $0}')
  if ! o=$("$BUCK2" uquery "$OWNER_EXPR" 2>/tmp/uqerr); then
    echo "buck2-affected-gate: FATAL buck2 uquery owner() errored:"; sed 's/^/    /' /tmp/uqerr; exit 1
  fi
  [ -n "$o" ] && OWNERS="$OWNERS $o"
fi

OWNERS=$(printf '%s\n' $OWNERS | sed '/^$/d' | sort -u)
if [ -z "$OWNERS" ]; then
  echo "buck2-affected-gate: FATAL Rust/buck files changed but NO owning target found (refusing to false-pass):"
  printf '    %s\n' $RUST_REL
  exit 1
fi
echo "buck2-affected-gate: $(printf '%s\n' "$OWNERS" | wc -l | tr -d ' ') owning target(s)"

# Affected = changed targets + reverse-dep closure. rdeps error also FAILS closed.
# Pass owners via @argfile + the %Ss set placeholder, NOT an inline set(...): a change
# to a large BUCK package (e.g. third-party/BUCK owns 1689 targets) overflows the inline
# query string and buck2 errors out (uquery RC=3, no build attempted) — which silently
# blocked landing ANY third-party change. @argfile + %Ss handles an arbitrary set size
# (verified: 1689 owners -> 1919 affected). One owner per line.
printf '%s\n' $OWNERS | sed '/^$/d' > /tmp/gate-owners.txt
if ! AFFECTED=$("$BUCK2" uquery 'rdeps(//..., %Ss)' @/tmp/gate-owners.txt 2>/tmp/rqerr); then
  echo "buck2-affected-gate: FATAL rdeps query errored:"; sed 's/^/    /' /tmp/rqerr; exit 1
fi
N=$(printf '%s\n' "$AFFECTED" | sed '/^$/d' | wc -l | tr -d ' ')
echo "buck2-affected-gate: $N affected target(s) (owners + reverse-dep closure)"
if [ "$N" = "0" ]; then echo "buck2-affected-gate: FATAL owners found but rdeps empty (query problem)"; exit 1; fi

# Build then test the affected set. @- reads the newline-delimited target list
# from stdin, avoiding ARG_MAX limits on large closures.
printf '%s\n' "$AFFECTED" | sed '/^$/d' > /tmp/affected-targets.txt
echo "=== buck2 build (affected) ==="
"$BUCK2" build @/tmp/affected-targets.txt
echo "=== buck2 test (affected) ==="
"$BUCK2" test @/tmp/affected-targets.txt
echo "buck2-affected-gate: PASS"
