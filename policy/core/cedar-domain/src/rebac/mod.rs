//! ReBAC tuple-store port vocabulary.
//!
//! This module models the Zanzibar/OpenFGA-style relationship tuple surface
//! without binding the domain crate to any storage engine or serving path.  A
//! tuple is tenant-scoped and rendered within that scope as
//! `object#relation@subject`; the subject can be either a concrete object
//! (`user:alice`) or another userset (`group:platform#member`).
//! Zookie and snapshot tokens are opaque policy/tuple-store consistency
//! vocabulary: callers may carry and echo them, but ordering belongs to the
//! tuple store implementation.

mod query;
mod rewrite;
mod token;
mod tuple;
mod validate;

pub use query::{RebacTuplePage, RebacTupleQuery, RebacTupleStore, RebacTupleStoreError};
pub use rewrite::UsersetRewrite;
pub use token::{RebacReadSnapshot, ResolvedRebacSnapshot, SnapshotToken, Zookie};
pub use tuple::{RebacObjectRef, RebacRelation, RebacSubjectRef, RebacTenantScope, RebacTuple};

use std::fmt;

/// Validation failure for ReBAC tuple vocabulary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebacTupleValidationError {
    EmptyField { field: &'static str },
    InvalidToken { field: &'static str, value: String },
    InvalidCanonicalTuple { detail: String },
    EmptyRewrite { kind: &'static str },
}

impl fmt::Display for RebacTupleValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField { field } => write!(f, "{field} must not be empty"),
            Self::InvalidToken { field, value } => {
                write!(f, "{field} contains invalid characters: {value:?}")
            }
            Self::InvalidCanonicalTuple { detail } => {
                write!(f, "invalid ReBAC canonical tuple: {detail}")
            }
            Self::EmptyRewrite { kind } => write!(f, "{kind} userset rewrite must not be empty"),
        }
    }
}

impl std::error::Error for RebacTupleValidationError {}
