//! Typed refusals from snapshot decode and producer validation.

use std::fmt;

use crate::vocabulary::{
    PRODUCER_BOOTSTRAP_GO, PRODUCER_OWNED_RUST, SCHEMA_VERSION_DECLARATIONS,
    SCHEMA_VERSION_IDENTITY_ONLY,
};

/// Typed refusal from snapshot decode / producer validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotError {
    /// JSON could not be parsed.
    Parse {
        /// Parser detail (no path — adapter receives bytes only).
        detail: String,
    },
    /// Required field missing or wrong type / empty.
    Schema {
        /// Which field failed.
        field: &'static str,
    },
    /// Package producer is not one of the ADR-0638 canonical identities.
    UnknownProducer {
        /// Producer string found on a package.
        actual: String,
    },
    /// Duplicate `unit_id` — non-deterministic model shape.
    DuplicateUnit {
        /// The repeated unit id.
        unit_id: String,
    },
    /// Envelope claims a schema version this decoder does not implement.
    UnknownSchemaVersion {
        /// Version claimed by the artifact.
        actual: u32,
    },
    /// Declaration kind is outside the closed Go vocabulary.
    UnknownDeclarationKind {
        /// Unit the declaration belongs to.
        unit_id: String,
        /// Kind string found.
        actual: String,
    },
    /// Type kind is outside the closed vocabulary.
    UnknownTypeKind {
        /// Unit the type appears in.
        unit_id: String,
        /// Kind string found.
        actual: String,
    },
    /// Attribute key is outside the closed vocabulary.
    UnknownAttr {
        /// Unit the declaration belongs to.
        unit_id: String,
        /// Attribute key found.
        actual: String,
    },
    /// Flag is outside the closed flag vocabulary.
    UnknownFlag {
        /// Unit the declaration belongs to.
        unit_id: String,
        /// Flag string found.
        actual: String,
    },
    /// Two declarations share one name in a scope that has a single namespace.
    DuplicateDeclaration {
        /// Unit the declarations belong to.
        unit_id: String,
        /// The repeated name.
        name: String,
    },
    /// The envelope version and its payload disagree.
    VersionPayloadMismatch {
        /// What the version claims.
        detail: &'static str,
    },
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { detail } => {
                write!(f, "source-model snapshot JSON parse failed: {detail}")
            }
            Self::Schema { field } => {
                write!(
                    f,
                    "source-model snapshot schema missing or invalid: {field}"
                )
            }
            Self::UnknownProducer { actual } => write!(
                f,
                "source-model snapshot package producer must be `{PRODUCER_BOOTSTRAP_GO}` or `{PRODUCER_OWNED_RUST}`, got `{actual}`"
            ),
            Self::DuplicateUnit { unit_id } => {
                write!(f, "source-model snapshot has duplicate unit_id `{unit_id}`")
            }
            Self::UnknownSchemaVersion { actual } => write!(
                f,
                "source-model snapshot schema_version must be {SCHEMA_VERSION_IDENTITY_ONLY} or \
                 {SCHEMA_VERSION_DECLARATIONS}, got {actual}"
            ),
            Self::UnknownDeclarationKind { unit_id, actual } => write!(
                f,
                "source-model snapshot unit `{unit_id}` declares unknown kind `{actual}`"
            ),
            Self::UnknownFlag { unit_id, actual } => write!(
                f,
                "source-model snapshot unit `{unit_id}` carries unknown flag `{actual}`"
            ),
            Self::UnknownTypeKind { unit_id, actual } => write!(
                f,
                "source-model snapshot unit `{unit_id}` carries unknown type kind `{actual}`"
            ),
            Self::UnknownAttr { unit_id, actual } => write!(
                f,
                "source-model snapshot unit `{unit_id}` carries unknown attribute `{actual}`"
            ),
            Self::DuplicateDeclaration { unit_id, name } => write!(
                f,
                "source-model snapshot unit `{unit_id}` declares `{name}` more than once in one \
                 namespace"
            ),
            Self::VersionPayloadMismatch { detail } => {
                write!(
                    f,
                    "source-model snapshot version/payload mismatch: {detail}"
                )
            }
        }
    }
}

impl std::error::Error for SnapshotError {}
