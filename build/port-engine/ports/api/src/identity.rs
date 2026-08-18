//! Stable identities, and the language pair they are addressed under.
//!
//! Every one is an opaque newtype over a `String`. The engine COMPARES these and never interprets
//! them: a unit id is not a path, a digest is not an algorithm, and a language slug is not a
//! language. That is what lets a second language pair be a second directory of rule data over the
//! same engine rather than a second engine.

use crate::error::PortError;

/// The source→target language pair a [`RulePack`] is authored for.
///
/// This is DATA, not a type parameter: the rule namespace is `specs/port-rules/lang/<pair>/**`,
/// so a second pair is a second directory of rule data over the same engine, never a second
/// engine. Both fields are opaque slugs — the kernel compares them and never interprets them.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LanguagePair {
    /// Slug of the language being read (matches [`SourceModel::language`]).
    pub source: String, // data_class: INTERNAL_ONLY
    /// Slug of the language being emitted (matches [`TargetIr::target_language`]).
    pub target: String, // data_class: INTERNAL_ONLY
}

/// True for the bytes a [`LanguagePair`] slug may contain: ASCII lowercase alphanumeric, `_`, and
/// `+` (so `c++` stays spellable).
const fn is_slug_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'+'
}

impl LanguagePair {
    /// The `<pair>` path segment of the rule namespace, e.g. `source-target`.
    ///
    /// FAIL-CLOSED, and the reason is an addressing collision rather than tidiness. The segment is
    /// the two slugs joined by `-`, so if a slug may itself contain `-` the join is not injective:
    /// `("a-b", "c")` and `("a", "b-c")` both render `a-b-c`, and once this value addresses
    /// `specs/port-rules/lang/<pair>` the two pairs select the SAME rule namespace. One of them is
    /// then reading or overwriting the other's rules with no error anywhere. Refusing the
    /// ambiguity here is the only place it is cheap: after the join the information is gone.
    ///
    /// The rule is that neither slug may be empty or carry a byte outside [`is_slug_byte`], the
    /// grammar of ONE portable path component. That is what the ADR fixes the segment to be —
    /// a single component of the form `<source>-<target>` — and the grammar is derived from that
    /// USE rather than from the separator collision alone. A slug of `a/b` renders `a/b-c`, which
    /// is two components, not one; a slug of `..` or a leading `/` is worse than a wrong name,
    /// because `Path::join` documents that an absolute operand REPLACES the receiver, so the
    /// namespace root would be discarded rather than descended from. Refusing the whole class here
    /// costs one predicate; enumerating the hostile bytes costs a review round each.
    ///
    /// # Errors
    /// [`PortError::AmbiguousLanguagePair`] when either slug is empty or carries a byte the
    /// component grammar does not admit (the separator among them).
    pub fn slug(&self) -> Result<String, PortError> {
        for slug in [&self.source, &self.target] {
            if slug.is_empty() || !slug.bytes().all(is_slug_byte) {
                return Err(PortError::AmbiguousLanguagePair {
                    source: self.source.clone(),
                    target: self.target.clone(),
                });
            }
        }
        Ok(format!("{}{PAIR_SEPARATOR}{}", self.source, self.target))
    }
}

/// The byte joining the two slugs of a [`LanguagePair::slug`], and therefore the byte neither slug
/// may contain.
pub const PAIR_SEPARATOR: char = '-';

/// The join is injective only while the separator sits OUTSIDE the slug grammar. Asserted at
/// compile time rather than argued in prose, so widening [`is_slug_byte`] to admit `-` fails the
/// build instead of silently making two pairs address one rule namespace.
const _: () = assert!(
    !is_slug_byte(PAIR_SEPARATOR as u8),
    "the separator must sit outside the slug grammar or the join stops being injective"
);

/// A stable identity for one translatable unit of the source model.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UnitId(pub String); // data_class: INTERNAL_ONLY

/// A stable identity for one rule in a [`RulePack`].
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RuleId(pub String); // data_class: INTERNAL_ONLY

/// A stable identity for one emitted region (the ADR-0597 registered regenerable region).
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RegionId(pub String); // data_class: INTERNAL_ONLY

/// An opaque content digest. The kernel COMPARES digests and never computes one — hashing is an
/// adapter concern, and keeping it out of here is what lets the receipt seam stay pure.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Digest(pub String); // data_class: INTERNAL_ONLY
