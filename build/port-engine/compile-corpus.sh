#!/usr/bin/env bash
# Compile what the engine emits for REAL third-party Go packages.
#
# `survey` reports how many declarations TRANSLATED. That is not the same claim as "the output
# compiles", and the two were measured apart for a long time: the engine's compile proof runs on the
# hermetic fixture corpus only, so nothing ever compiled the output for an actual repository. The
# first time this script ran, three of nine packages did not compile, and one of them had been made
# worse that morning by a rule that raised coverage.
#
# A rule is not finished when its refusal leaves the histogram. It is finished when the output still
# compiles. Run this after every rule.
#
# Usage:  compile-corpus.sh <snapshot-dir> [package ...]
# The snapshot directory holds <package>.json files produced by the Go extractor. Corpora are cloned
# to scratch and never into the repository, so the path is an argument rather than a default.
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
snapshots="${1:?usage: compile-corpus.sh <snapshot-dir> [package ...]}"
shift

# rustc writes its temporaries beside the output path, so both have to be somewhere writable —
# `-o /dev/null` fails with "couldn't create a temp dir", which reads exactly like a compile error
# and is not one. That cost a wrong reading of this table once already.
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

packages=("$@")
if [ ${#packages[@]} -eq 0 ]; then
  for file in "$snapshots"/*.json; do
    packages+=("$(basename "$file" .json)")
  done
fi

failed=0
printf '%-14s %s\n' PACKAGE RESULT
for package in "${packages[@]}"; do
  snapshot="$snapshots/$package.json"
  [ -f "$snapshot" ] || { printf '%-14s %s\n' "$package" "no snapshot"; continue; }

  # ONE FILE PER MODULE. `port` emits a `// <name>.rs` banner per source package, and those are
  # separate Rust modules -- concatenating them into one file makes every name the two packages
  # share collide. chi is two packages (`chi` and `chi/middleware`), and reading their fourteen
  # duplicate-definition errors as an engine defect was wrong: the engine had emitted them apart.
  crate="$work/$package"
  rm -rf "$crate"; mkdir -p "$crate"
  (cd "$root/../.." && cargo run -q -p port-engine-app -- port "$snapshot" 2>/dev/null) \
    | sed '/^port=/d' \
    | awk -v dir="$crate" '
        /^\/\/ [A-Za-z0-9_]+\.rs$/ { name = substr($2, 1, length($2) - 3); file = dir "/" name ".rs"; next }
        file { print > file }
      '

  # `forbid` rather than `deny`: the emitted crate may not opt back out, and nothing the engine
  # emits is allowed to try.
  # The module list is built BEFORE lib.rs is opened. A redirect creates its file first, so a glob
  # evaluated inside the redirect sees lib.rs and declares `pub mod lib;` -- a crate root that
  # declares itself.
  declarations='#![forbid(unsafe_code)]'
  # EMITTED NOTHING IS NOT COMPILING, and this table said it was. A package whose port fails --
  # a snapshot that will not admit, a pipeline error -- writes no modules, and an empty crate
  # compiles perfectly. Three of thirteen packages were passing that way, and the headline
  # "13/13 compile" counted them. A gate that cannot tell success from absence measures nothing.
  if ! ls "$crate"/*.rs > /dev/null 2>&1; then
    printf '%-14s %s\n' "$package" "emitted nothing"
    failed=$((failed + 1))
    continue
  fi

  for module in "$crate"/*.rs; do
    [ -e "$module" ] || continue
    declarations+=$'\n'"pub mod $(basename "$module" .rs);"
  done
  printf '%s\n' "$declarations" > "$crate/lib.rs"

  # Dead code is EXPECTED and is not a defect: the engine emits only what it can prove, so a
  # translated helper whose only caller refused is unused through no fault of its own.
  #
  # `unused` is NOT suppressed, and used to be. It hid three real translator artifacts — an import
  # emitted into modules that never name it, and a loop binding nobody reads — which a reviewer
  # found by running the compiler without the flag this script was passing. An expected warning
  # class is one thing; a blanket group that happens to contain it is another.
  # `-D warnings` because that is the gate the engine is actually held to. Running without it
  # measured a weaker claim: `pub const K: PrivateType` is a WARNING, and a warning under this
  # policy is a build failure -- so the table said "compiles" for a crate that does not.
  # CLIPPY, not just rustc. The gate the engine is held to is both, each with `--deny=warnings`,
  # and clippy sees a whole class rustc does not: a manual range test, a byte string written as an
  # array of byte literals, a tab inside a doc comment. Those are exactly the shapes a reviewer
  # names as "translated", so measuring without them measured the wrong thing.
  checker=rustc
  command -v clippy-driver > /dev/null && checker=clippy-driver
  errors=$("$checker" --edition 2021 --crate-type lib --emit=metadata \
             -A dead_code -D warnings \
             -o "$work/$package.rmeta" "$crate/lib.rs" 2>&1 \
           | grep -cE '^(error|warning)')
  if [ "$errors" -eq 0 ]; then
    printf '%-14s %s\n' "$package" "compiles"
  else
    printf '%-14s %s\n' "$package" "$errors errors"
    failed=$((failed + 1))
  fi
done

exit $((failed > 0))
