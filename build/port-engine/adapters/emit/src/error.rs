//! Typed refusals from canary emit and materialize.

use std::fmt;

use crate::{CANARY_OUT_DIRNAME, CANARY_RULE_SUFFIX};

/// Validate `out_dir` is allowlisted for single-file canary materialize.
///
/// # Errors
/// [`EmitError::PathRefused`] when basename is wrong, path has `..`, or points at the corpus root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmitError {
    /// No canary region present in the emit tree.
    MissingCanary,
    /// More than one canary region — bulk/ambiguous emit refused.
    AmbiguousCanary {
        /// How many canary-shaped regions were found.
        count: usize,
    },
    /// Bytes do not match the embedded golden.
    GoldenMismatch {
        /// Digest of the emitted canary.
        actual: String,
        /// Digest of the golden.
        expected: String,
        /// UTF-8 lossy spelling of emitted bytes (for golden authoring).
        actual_utf8: String,
    },
    /// Destination path escapes the canary-out allowlist.
    PathRefused {
        /// Why the path was refused.
        detail: String,
    },
    /// Filesystem IO failed.
    Io {
        /// OS detail.
        detail: String,
    },
}

impl fmt::Display for EmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCanary => {
                write!(f, "canary emit: no `{CANARY_RULE_SUFFIX}` region in tree")
            }
            Self::AmbiguousCanary { count } => write!(
                f,
                "canary emit: expected exactly one canary region, found {count}"
            ),
            Self::GoldenMismatch {
                actual,
                expected,
                actual_utf8,
            } => write!(
                f,
                "canary emit golden mismatch: actual `{actual}`, expected `{expected}`, bytes={actual_utf8:?}"
            ),
            Self::PathRefused { detail } => write!(f, "canary emit path refused: {detail}"),
            Self::Io { detail } => write!(f, "canary emit io failed: {detail}"),
        }
    }
}

impl std::error::Error for EmitError {}
