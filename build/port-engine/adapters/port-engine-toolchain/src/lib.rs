//! # port-engine-toolchain — receipt `toolchain_digest` binder (W0-B Slice 9).
//!
//! Digests the hermetic dual-home toolchain corpus (`build/toolchains/**` mirrored under
//! `src/corpus/*.txt`). Filenames avoid nesting a `BUCK` path (buck2 srcs globs exclude those).
//! Cell remap (`.buckconfig` `toolchains = build/toolchains`) remains PARKED outside the
//! `roots.build` envelope; this adapter binds the dual-home *bytes* so the receipt axis is
//! content-addressed without rewriting the live buck cell.
#![forbid(unsafe_code)]

use port_engine_api::Digest;
use port_engine_hash::digest_bytes;

/// Fail-closed readiness gate. `true` once Slice 9 toolchain axis binding is present.
pub const fn w0_ready() -> bool {
    true
}

/// Dual-home logical paths in stable sort order (relative to `build/toolchains/`).
pub const CORPUS_PATHS: [&str; 4] = [
    "BUCK",
    "cache/BUCK",
    "cache/OWNERS",
    "cache/defs.bzl",
];

// Package-local mirrors (`.txt` so buck2 srcs include them; logical paths stay dual-home).
const CORPUS_BUCK: &str = include_str!("corpus/toolchains.buck.txt");
const CORPUS_CACHE_BUCK: &str = include_str!("corpus/cache.buck.txt");
const CORPUS_CACHE_OWNERS: &str = include_str!("corpus/cache.owners.txt");
const CORPUS_CACHE_DEFS: &str = include_str!("corpus/cache.defs.bzl.txt");

/// Stable admission preimage: each `path\\0content\\0` in [`CORPUS_PATHS`] order.
#[must_use]
pub fn toolchain_preimage() -> Vec<u8> {
    let entries: [(&str, &str); 4] = [
        (CORPUS_PATHS[0], CORPUS_BUCK),
        (CORPUS_PATHS[1], CORPUS_CACHE_BUCK),
        (CORPUS_PATHS[2], CORPUS_CACHE_OWNERS),
        (CORPUS_PATHS[3], CORPUS_CACHE_DEFS),
    ];
    let mut out = Vec::new();
    for (path, content) in entries {
        out.extend_from_slice(path.as_bytes());
        out.push(0);
        out.extend_from_slice(content.as_bytes());
        out.push(0);
    }
    out
}

/// Content digest of the dual-home toolchain corpus (`sha256:<hex>`).
#[must_use]
pub fn toolchain_digest() -> Digest {
    digest_bytes(&toolchain_preimage())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice9_claims_toolchain_readiness() {
        assert!(w0_ready());
    }

    #[test]
    fn toolchain_digest_matches_known_dual_home_preimage() {
        let d = toolchain_digest();
        assert_eq!(
            d.0,
            "sha256:419e00d0e9c4d25f07431224dc50f89083d772adb9c59751a9a7d78c28f01cbd"
        );
        assert_eq!(d, toolchain_digest());
    }

    #[test]
    fn corpus_mirrors_are_nonempty() {
        assert!(!CORPUS_BUCK.is_empty());
        assert!(!CORPUS_CACHE_BUCK.is_empty());
        assert!(!CORPUS_CACHE_OWNERS.is_empty());
        assert!(!CORPUS_CACHE_DEFS.is_empty());
    }
}
