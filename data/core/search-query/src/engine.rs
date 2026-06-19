//! Search engine preference value object.
//!
//! Per M02-P09 (search merge-variant delta 1): backport of the `SearchEngine`
//! discriminant from the planned `oya-search-query-kernel/src/types.rs`.
//! Placed here because `oya-search-query-domain` is the live domain that owns
//! query-plan identity; concrete adapters (pgroonga / tantivy) are downstream.

#![forbid(unsafe_code)]

/// Selects which physical search engine services a `SearchQuery`.
///
/// - `Pgroonga` — PostgreSQL full-text index (pgroonga extension); best for
///   pack-provided morphology via the bundled tokenizer.
/// - `Tantivy` — on-disk Lucene-style index; lower SQL overhead, good for
///   large generic corpora.
/// - `Auto` — try `Pgroonga` first; fall back to `Tantivy` on error.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SearchEngine {
    Pgroonga,
    Tantivy,
    Auto,
}

impl SearchEngine {
    /// Returns `true` if this engine value requires a running pgroonga index.
    pub fn needs_pgroonga(self) -> bool {
        matches!(self, Self::Pgroonga | Self::Auto)
    }

    /// Returns `true` if this engine value may use the Tantivy on-disk index.
    pub fn needs_tantivy(self) -> bool {
        matches!(self, Self::Tantivy | Self::Auto)
    }
}

#[cfg(test)]
mod tests {
    #![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

    use super::*;

    #[test]
    fn pgroonga_needs_pgroonga_not_tantivy() {
        assert!(SearchEngine::Pgroonga.needs_pgroonga());
        assert!(!SearchEngine::Pgroonga.needs_tantivy());
    }

    #[test]
    fn tantivy_needs_tantivy_not_pgroonga() {
        assert!(SearchEngine::Tantivy.needs_tantivy());
        assert!(!SearchEngine::Tantivy.needs_pgroonga());
    }

    #[test]
    fn auto_needs_both() {
        assert!(SearchEngine::Auto.needs_pgroonga());
        assert!(SearchEngine::Auto.needs_tantivy());
    }

    #[test]
    fn variants_are_ordered() {
        // Pgroonga < Tantivy < Auto (Ord derives left-to-right declaration order)
        assert!(SearchEngine::Pgroonga < SearchEngine::Tantivy);
        assert!(SearchEngine::Tantivy < SearchEngine::Auto);
    }

    #[test]
    fn clone_and_copy_are_consistent() {
        let e = SearchEngine::Pgroonga;
        let c = e;
        assert_eq!(e, c);
        assert_eq!(e.clone(), c);
    }
}
