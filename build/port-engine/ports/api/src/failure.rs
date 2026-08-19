//! The source's failure convention.
//!
//! Every real package in the source language returns failure as a trailing result rather than as a
//! distinct type, so an engine that cannot see that convention cannot port anything real. What it
//! needs to see is small: which source type carries the convention, what the target should carry
//! instead, and how the source spells the absent value it compares against.
//!
//! Deliberately NOT here: `Result`, `Ok`, `Err`. Those are target spellings, and the face that
//! renders the target owns them. Putting them in the pack would make a second language pair
//! re-declare the target's own vocabulary, which is the thing the neutral seam exists to prevent.

/// How the source spells failure, so the engine can recognise it without knowing the language.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailureConvention {
    /// Source type identity of the failure value — the type a trailing result has when the
    /// function can fail.
    pub source_type: String, // data_class: INTERNAL_ONLY
    /// Target type the failure value becomes.
    pub target_type: String, // data_class: INTERNAL_ONLY
    /// How the source spells the ABSENT failure value, which is what a success returns and what a
    /// check compares against.
    pub absent: String, // data_class: INTERNAL_ONLY
}

impl FailureConvention {
    /// Whether a source type identity is the failure type.
    #[must_use]
    pub fn is_failure(&self, identity: &str) -> bool {
        identity == self.source_type
    }
}
