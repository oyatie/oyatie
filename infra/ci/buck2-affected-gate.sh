#!/bin/sh
# buck2-native affected-only CI gate.
#
# Builds + tests the reverse-dependency closure of the PR's changed files —
# the hyperscaler "affected targets" pattern (Google/Meta), buck2-native via
# `uquery owner()` + `rdeps()`. Replaces the cargo-era `oya verify --affected`.
# No oya-dev-cli dependency.
#
# Usage:  buck2-affected-gate.sh [base-ref]   (default base: origin/dev)
# Exit 0 = pass (incl. non-Rust / no-affected PRs); non-zero = build/test failure.
set -eu

BASE="${1:-origin/dev}"
BUCK2="${BUCK2:-buck2}"

echo "buck2-affected-gate: start (pwd=$(pwd) base=$BASE head=$(git rev-parse --short HEAD 2>&1))"
if ! git rev-parse --verify --quiet "$BASE" >/dev/null 2>&1; then
  echo "buck2-affected-gate: FATAL base ref '$BASE' does not resolve in this checkout"
  echo "  remotes: $(git remote 2>&1)  | refs: $(git for-each-ref --format='%(refname)' refs/remotes 2>&1 | paste -sd' ' -)"
  exit 1
fi
if ! MERGE_BASE=$(git merge-base HEAD "$BASE" 2>&1); then
  echo "buck2-affected-gate: FATAL merge-base HEAD $BASE: $MERGE_BASE"
  exit 1
fi
CHANGED=$(git diff --name-only "$MERGE_BASE" HEAD)
if [ -z "$CHANGED" ]; then
  echo "buck2-affected-gate: no changed files vs $BASE -> PASS"
  exit 0
fi
echo "buck2-affected-gate: $(printf '%s\n' "$CHANGED" | wc -l | tr -d ' ') changed file(s) vs $BASE ($MERGE_BASE)"

# owner() of each changed file that buck2 actually tracks. Files owned by no
# target (docs, .md, CI yaml) contribute nothing -> a docs-only PR yields an
# empty owner set and passes without building anything.
OWNERS=""
for f in $CHANGED; do
  [ -e "$f" ] || continue
  o=$("$BUCK2" uquery "owner('$f')" 2>/dev/null || true)
  [ -n "$o" ] && OWNERS="$OWNERS $o"
done
OWNERS=$(printf '%s\n' $OWNERS | sort -u)
if [ -z "$OWNERS" ]; then
  echo "buck2-affected-gate: no buck2 targets own the changed files (non-Rust change) -> PASS"
  exit 0
fi

# Affected = the changed targets + everything that depends on them (rdeps closure).
OWNER_SET=$(printf '%s\n' $OWNERS | paste -sd' ' -)
AFFECTED=$("$BUCK2" uquery "rdeps(//..., set($OWNER_SET))" 2>/dev/null || true)
N=$(printf '%s\n' "$AFFECTED" | sed '/^$/d' | wc -l | tr -d ' ')
echo "buck2-affected-gate: $N affected target(s) (owners + reverse-dep closure)"
if [ "$N" = "0" ]; then echo "PASS (no affected targets)"; exit 0; fi

# Build then test the affected set. @- reads the newline-delimited target list
# from stdin, avoiding ARG_MAX limits on large closures.
printf '%s\n' "$AFFECTED" | sed '/^$/d' > /tmp/affected-targets.txt
echo "=== buck2 build (affected) ==="
"$BUCK2" build @/tmp/affected-targets.txt
echo "=== buck2 test (affected) ==="
"$BUCK2" test @/tmp/affected-targets.txt
echo "buck2-affected-gate: PASS"
