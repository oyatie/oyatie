#!/usr/bin/env bash
# Regenerate every crate's `sources.rs`, the embedded manifest the engine-identity axis hashes.
#
# `engine_digest` is only meaningful if the manifest is the WHOLE engine: a source file nobody lists
# changes emitted bytes with no axis movement, which is the `Unexplained` case the kernel calls RED
# except that nothing would ever report it. `the_manifest_is_the_whole_engine` fences that, and this
# is what answers the fence.
#
# Run it whenever a source file is ADDED, RENAMED or REMOVED. Editing one needs nothing: the
# manifest embeds contents through `include_str!`, so an edit already moves the digest.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

for manifest in "$root"/*/*/src/sources.rs; do
  crate="$(dirname "$manifest")"
  {
    cat <<'HEADER'
//! This crate's own sources, embedded so the engine-identity axis can hash them.
//!
//! GENERATED; regenerate rather than edit. Only files this crate OWNS are listed — the facade
//! assembles the whole-engine digest, because the facade is the one place the whole engine is
//! legitimately visible without inverting the dependency direction.

/// Crate-relative path and contents of each source this crate owns, sorted by path.
pub const CRATE_SOURCES: &[(&str, &str)] = &[
HEADER
    # Sorted by path so the manifest -- and therefore the digest -- does not depend on readdir
    # order. `sources.rs` lists itself: it is a source of this crate like any other.
    (cd "$crate" && find . -name '*.rs' | sed 's|^\./||' | LC_ALL=C sort) \
      | while read -r file; do
          printf '    ("%s", include_str!("%s")),\n' "$file" "$file"
        done
    echo '];'
  } > "$manifest.next"
  mv "$manifest.next" "$manifest"
  echo "regen ${crate#"$root/"}/sources.rs"
done
