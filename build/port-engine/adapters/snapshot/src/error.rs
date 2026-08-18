//! Typed refusals from snapshot admission.

use std::fmt;

use port_engine_api::Digest;
use port_engine_frontend_go::{PRODUCER_BOOTSTRAP_GO, SnapshotError};
use port_engine_source_pin::PinError;

/// Typed refusal from snapshot admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmitError {
    /// Snapshot decode / producer validation failed.
    Snapshot(SnapshotError),
    /// Fleet pin could not load.
    Pin(PinError),
    /// The two extractor passes did not produce byte-identical snapshots.
    SnapshotMismatch {
        /// SHA-256 digest of the first raw snapshot artifact.
        first: Digest,
        /// SHA-256 digest of the second raw snapshot artifact.
        second: Digest,
    },
    /// Claimed `snapshot_digest` does not match the stable preimage hash.
    DigestMismatch {
        /// Digest claimed in the artifact.
        claimed: String,
        /// Digest computed from the admission preimage.
        computed: String,
    },
    /// Snapshot language is not the bootstrap Go pair source.
    Language {
        /// Language found on the artifact.
        actual: String,
    },
    /// A producer is not authorized during bootstrap admission.
    ProducerNotAuthorized {
        /// Unit whose producer is premature.
        unit: String,
        /// Producer identity found on the artifact.
        actual: String,
    },
}

impl fmt::Display for AdmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(err) => write!(f, "snapshot admit decode failed: {err}"),
            Self::Pin(err) => write!(f, "snapshot admit pin failed: {err}"),
            Self::SnapshotMismatch { first, second } => write!(
                f,
                "snapshot extractor passes differ: first `{}`, second `{}`",
                first.0, second.0
            ),
            Self::DigestMismatch { claimed, computed } => write!(
                f,
                "snapshot admit digest mismatch: claimed `{claimed}`, computed `{computed}`"
            ),
            Self::Language { actual } => write!(
                f,
                "snapshot admit language must be `go` for bootstrap admission, got `{actual}`"
            ),
            Self::ProducerNotAuthorized { unit, actual } => write!(
                f,
                "snapshot admit producer for unit `{unit}` must be `{PRODUCER_BOOTSTRAP_GO}` before \
                 front-end equivalence, got `{actual}`"
            ),
        }
    }
}

impl std::error::Error for AdmitError {}
