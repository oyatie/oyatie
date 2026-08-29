//! Fail-closed expansion failures.
//!
//! Every variant denies. An expansion that cannot complete is never reported
//! as "no grant found" — the caller cannot distinguish an absent grant from an
//! unread one, so both must refuse.

use std::fmt;

use policy_cedar_domain::rebac::RebacTupleStoreError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpansionError {
    /// No rewrite is configured for this object type and relation. Deny by
    /// omission: an unconfigured relation grants nothing.
    UndefinedRelation {
        object_type: String,
        relation: String,
    },
    /// The rewrite tree nested deeper than the configured bound.
    DepthExceeded { limit: u32 },
    /// The walk read more tuples than the configured bound allowed.
    TupleBudgetExceeded { limit: usize },
    /// One tupleset paged further than the configured bound allowed. Distinct
    /// from the tuple bound so an operator can tell a wide relation from a
    /// store that is not terminating its pagination.
    PageBudgetExceeded { limit: usize },
    /// The store refused or failed. Never treated as an absent grant.
    Store(RebacTupleStoreError),
}

impl fmt::Display for ExpansionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndefinedRelation {
                object_type,
                relation,
            } => write!(f, "no rewrite configured for {object_type}#{relation}"),
            Self::DepthExceeded { limit } => {
                write!(f, "expansion exceeded the depth bound of {limit}")
            }
            Self::TupleBudgetExceeded { limit } => {
                write!(f, "expansion exceeded the tuple-read bound of {limit}")
            }
            Self::PageBudgetExceeded { limit } => {
                write!(f, "one tupleset exceeded the page bound of {limit}")
            }
            Self::Store(error) => write!(f, "tuple store: {error}"),
        }
    }
}

impl std::error::Error for ExpansionError {}

impl From<RebacTupleStoreError> for ExpansionError {
    fn from(error: RebacTupleStoreError) -> Self {
        Self::Store(error)
    }
}
