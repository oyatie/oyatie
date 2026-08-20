#!/usr/bin/env bash
# Assemble what the engine emits into ONE crate, shaped the way a reviewer expects to read it.
#
# The review gates are two of the engine's five acceptance conditions, and a reviewer can only judge
# what they are handed. This script exists because the hand-assembled version produced FALSE
# FINDINGS twice, and both were reported as engine defects before being traced back here:
#
#   - concatenating every emitted module into one file made `ContextKey` collide with itself across
#     `chi` and `chi/middleware`, which read as nine duplicate-definition errors;
#   - nesting each package's single module inside a directory of the same name produced
#     `uuid::uuid`, which clippy calls `module_inception` and a reviewer called a Go shape.
#
# Neither was the engine. A gate is evidence about the thing it measures only after it is evidence
# about itself.
#
# So: a package that emits ONE module becomes `src/<pkg>.rs`. A package that emits several becomes
# `src/<pkg>/mod.rs` plus a submodule each, with the module sharing the package's name inlined into
# `mod.rs` rather than nested beneath itself.
#
# Usage:  review-bundle.sh <snapshot-dir> <out-dir> <package>...
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
snapshots="${1:?usage: review-bundle.sh <snapshot-dir> <out-dir> <package>...}"
out="${2:?usage: review-bundle.sh <snapshot-dir> <out-dir> <package>...}"
shift 2

rm -rf "$out"
mkdir -p "$out/src"
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT

for package in "$@"; do
  mkdir -p "$stage/$package"
  (cd "$root/../.." && cargo run -q -p port-engine-app -- port "$snapshots/$package.json" 2>/dev/null) \
    | sed '/^port=/d' \
    | awk -v dir="$stage/$package" '
        /^\/\/ [A-Za-z0-9_]+\.rs$/ { name = substr($2, 1, length($2) - 3); file = dir "/" name ".rs"; next }
        file { print > file }
      '

  modules=("$stage/$package"/*.rs)
  if [ "${#modules[@]}" -eq 1 ]; then
    cp "${modules[0]}" "$out/src/$package.rs"
    continue
  fi

  mkdir -p "$out/src/$package"
  declarations=""
  inlined=""
  for module in "${modules[@]}"; do
    name="$(basename "$module" .rs)"
    if [ "$name" = "$package" ]; then
      inlined="$module"
    else
      cp "$module" "$out/src/$package/$name.rs"
      declarations+="pub mod $name;"$'\n'
    fi
  done
  { printf '%s\n' "$declarations"; [ -n "$inlined" ] && cat "$inlined"; } > "$out/src/$package/mod.rs"
done

# `forbid`, not `deny`: nothing the engine emits may opt back out.
{
  echo '#![forbid(unsafe_code)]'
  echo
  for package in "$@"; do echo "pub mod $package;"; done
} > "$out/src/lib.rs"

echo "bundle: $out/src/lib.rs"
