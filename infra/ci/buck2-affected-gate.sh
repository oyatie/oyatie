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
# The 1-arg form (buck2-affected-gate.sh origin/dev) is BACKWARD-COMPATIBLE:
# HEAD is the PR checkout in the Jenkins bridge path, so omitting head-ref
# keeps the existing Jenkins call unchanged.
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

# owner() per Rust-relevant file. A uquery ERROR (non-zero exit) is a buck2/graph
# failure and FAILS the gate — it is NOT 'no owner'. (This was the false-pass bug:
# 2>/dev/null||true swallowed buck2 errors and passed everything.)
OWNERS=""
for f in $RUST_REL; do
  [ -e "$f" ] || continue
  if ! o=$("$BUCK2" uquery "owner('$f')" 2>/tmp/uqerr); then
    echo "buck2-affected-gate: FATAL buck2 uquery owner('$f') errored:"; sed 's/^/    /' /tmp/uqerr; exit 1
  fi
  # A BUCK file is the package DEFINITION, not a target source, so owner() finds
  # nothing for it. A BUCK change can add/alter/remove any target in that package,
  # so the affected set is ALL targets in the package -> expand to the package
  # target pattern (cell-qualified) rather than treating it as a 'no owner' FATAL.
  if [ -z "$o" ]; then
    case "$f" in
      */BUCK|BUCK)
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
        ;;
    esac
  fi
  [ -n "$o" ] && OWNERS="$OWNERS $o"
done
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

# The temporary GitHub lane-unlocker must stay deterministic and fast. Optional
# external toolchain proof archives are validated by dedicated cutover lanes, not
# by every affected PR, because Buck2 materializes http_archive targets eagerly
# when a toolchain BUCK file changes and hosted runners time out on external
# HEAD requests. Set OYA_AFFECTED_GATE_INCLUDE_OPTIONAL_TOOLCHAIN_ARCHIVES=1 to
# include these proof targets explicitly. Do not filter normal Rust crates.
printf '%s\n' "$AFFECTED" | sed '/^$/d' > /tmp/affected-targets.unfiltered.txt
if [ "${OYA_AFFECTED_GATE_INCLUDE_OPTIONAL_TOOLCHAIN_ARCHIVES:-0}" != "1" ]; then
  grep -v -E '^(toolchains//cxx/clang_hermetic:(aarch64-musl-sysroot|aarch64-linux-sysroot|llvm-18-aarch64-linux|clang-dist-aarch64-linux|cxx_hermetic_linux|cxx_hermetic_linux_musl))$' /tmp/affected-targets.unfiltered.txt > /tmp/affected-targets.txt || true
  FILTERED=$(( $(wc -l < /tmp/affected-targets.unfiltered.txt | tr -d ' ') - $(wc -l < /tmp/affected-targets.txt | tr -d ' ') ))
  if [ "$FILTERED" != "0" ]; then
    echo "buck2-affected-gate: filtered $FILTERED optional external toolchain proof target(s); set OYA_AFFECTED_GATE_INCLUDE_OPTIONAL_TOOLCHAIN_ARCHIVES=1 to include"
  fi
else
  cp /tmp/affected-targets.unfiltered.txt /tmp/affected-targets.txt
fi
if [ ! -s /tmp/affected-targets.txt ]; then
  echo "buck2-affected-gate: all affected targets were optional external toolchain proof targets -> PASS"
  exit 0
fi

# Build then test the affected set. @- reads the newline-delimited target list
# from stdin, avoiding ARG_MAX limits on large closures.
echo "=== buck2 build (affected) ==="
"$BUCK2" build @/tmp/affected-targets.txt
echo "=== buck2 test (affected) ==="
"$BUCK2" test @/tmp/affected-targets.txt
echo "buck2-affected-gate: PASS"
