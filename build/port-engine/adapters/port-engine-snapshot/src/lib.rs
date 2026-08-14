//! # port-engine-snapshot — bootstrap SourceModel snapshot admission (W0-B Slice 8).
//!
//! ADR-0638 D3: the bootstrap Go extractor (`go/packages` + `go/types`) runs **out of band only**.
//! This adapter admits the resulting content-addressed snapshot artifact, binds it to the fleet
//! pin, and verifies the claimed `snapshot_digest` against a stable preimage. It MUST NEVER
//! invoke a Go toolchain (firewall inherited from `port-engine-frontend-go`).
#![forbid(unsafe_code)]

use std::fmt;

use port_engine_api::{Digest, SourceModel, UnitId};
use port_engine_frontend_go::{GoSourceModel, PRODUCER_BOOTSTRAP_GO, SnapshotError};
use port_engine_hash::digest_bytes;
use port_engine_source_pin::{PinError, load_embedded, receipt_pin};

/// Embedded OOB bootstrap snapshot fixture (hermetic; not produced in-process).
const FIXTURE_SNAPSHOT_JSON: &str = include_str!("fixture-snapshot-v0.json");

/// Fail-closed readiness gate. `true` once Slice 8 admission is present.
pub const fn w0_ready() -> bool {
    true
}

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
        }
    }
}

impl std::error::Error for AdmitError {}

/// An admitted bootstrap snapshot bound to the fleet pin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedSnapshot {
    /// Fleet pin (peeled commit) bound at admission.
    pin: String,
    /// SHA-256 digest of the byte-identical raw snapshot artifact.
    artifact_digest: Digest,
    /// Verified semantic digest claimed inside the artifact.
    model_digest: Digest,
    /// Decoded SourceModel (identity + order only).
    model: GoSourceModel,
}

impl AdmittedSnapshot {
    /// Fleet pin bound during admission.
    #[must_use]
    pub fn pin(&self) -> &str {
        &self.pin
    }

    /// Digest of the raw byte-identical artifact pair.
    #[must_use]
    pub fn artifact_digest(&self) -> &Digest {
        &self.artifact_digest
    }

    /// Verified semantic digest claimed by the decoded model.
    #[must_use]
    pub fn model_digest(&self) -> &Digest {
        &self.model_digest
    }

    /// Borrow the underlying [`SourceModel`].
    #[must_use]
    pub fn as_model(&self) -> &dyn SourceModel {
        self
    }

    /// Unit ids in deterministic order.
    #[must_use]
    pub fn units(&self) -> Vec<UnitId> {
        self.model.units()
    }

    /// Producer identity recorded for `unit`.
    #[must_use]
    pub fn producer_for(&self, unit: &UnitId) -> Option<&str> {
        self.model.producer_for(unit)
    }
}

impl SourceModel for AdmittedSnapshot {
    fn language(&self) -> &str {
        self.model.language()
    }

    fn snapshot_digest(&self) -> Digest {
        self.artifact_digest.clone()
    }

    fn units(&self) -> Vec<UnitId> {
        self.model.units()
    }
}

/// Stable admission preimage: `language\\0` then each `unit_id\\0producer\\0` in model order.
///
/// Chosen so the digest covers language + package→producer mapping without JSON canonicalization
/// debates (ADR-0638: composed artifact + mapping + schema covered by `snapshot_digest`).
#[must_use]
pub fn snapshot_preimage(language: &str, units_and_producers: &[(&str, &str)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(language.as_bytes());
    out.push(0);
    for (unit, producer) in units_and_producers {
        out.extend_from_slice(unit.as_bytes());
        out.push(0);
        out.extend_from_slice(producer.as_bytes());
        out.push(0);
    }
    out
}

/// Admit two byte-identical snapshot artifacts against the fleet pin.
///
/// # Errors
/// [`AdmitError::SnapshotMismatch`] when the two extractor passes differ, or another
/// [`AdmitError`] on decode, pin, language, or digest mismatch.
pub fn admit_reproducible_pair(
    first: &[u8],
    second: &[u8],
) -> Result<AdmittedSnapshot, AdmitError> {
    if first != second {
        return Err(AdmitError::SnapshotMismatch {
            first: digest_bytes(first),
            second: digest_bytes(second),
        });
    }
    admit_one(first, digest_bytes(first))
}

fn admit_one(bytes: &[u8], artifact_digest: Digest) -> Result<AdmittedSnapshot, AdmitError> {
    let model = GoSourceModel::decode(bytes).map_err(AdmitError::Snapshot)?;
    if model.language() != "go" {
        return Err(AdmitError::Language {
            actual: model.language().to_owned(),
        });
    }

    let units = model.units();
    let mut pairs: Vec<(String, String)> = Vec::with_capacity(units.len());
    for unit in &units {
        let producer = model
            .producer_for(unit)
            .ok_or(AdmitError::Snapshot(SnapshotError::Schema {
                field: "packages.producer",
            }))?
            .to_owned();
        pairs.push((unit.0.clone(), producer));
    }
    let refs: Vec<(&str, &str)> = pairs
        .iter()
        .map(|(u, p)| (u.as_str(), p.as_str()))
        .collect();
    let computed = digest_bytes(&snapshot_preimage(model.language(), &refs));
    let claimed = model.snapshot_digest();
    if claimed != computed {
        return Err(AdmitError::DigestMismatch {
            claimed: claimed.0,
            computed: computed.0,
        });
    }

    let pin = load_embedded().map_err(AdmitError::Pin)?;
    Ok(AdmittedSnapshot {
        pin: receipt_pin(&pin),
        artifact_digest,
        model_digest: computed,
        model,
    })
}

/// Admit the package-local OOB bootstrap fixture.
///
/// The embedded fixture has one hermetic byte source, so pairing it with itself exercises normal
/// admission without pretending that a second extractor execution occurred. External extractor
/// output must enter through [`admit_reproducible_pair`] with two independently produced artifacts.
///
/// # Errors
/// [`AdmitError`] on fixture defect.
pub fn admit_embedded_fixture() -> Result<AdmittedSnapshot, AdmitError> {
    let bytes = FIXTURE_SNAPSHOT_JSON.as_bytes();
    admit_reproducible_pair(bytes, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice8_claims_snapshot_readiness() {
        assert!(w0_ready());
    }

    #[test]
    fn embedded_fixture_admits_and_binds_pin() {
        let admitted = admit_embedded_fixture().expect("fixture must admit");
        assert!(!admitted.pin().is_empty());
        assert_eq!(
            admitted.model_digest().0,
            "sha256:b541cdee7bcb23984d8d56171e66f66fdbec027b40a06953105540d5915b33fb"
        );
        assert_eq!(
            admitted.artifact_digest(),
            &digest_bytes(FIXTURE_SNAPSHOT_JSON.as_bytes())
        );
        assert_eq!(
            admitted.as_model().snapshot_digest(),
            admitted.artifact_digest().clone()
        );
        assert_eq!(admitted.units().len(), 2);
        assert_eq!(
            admitted.producer_for(&UnitId("example.com/a".into())),
            Some(PRODUCER_BOOTSTRAP_GO)
        );
    }

    #[test]
    fn refuses_digest_mismatch() {
        let json = r#"{
  "language": "go",
  "snapshot_digest": "sha256:deadbeef",
  "packages": [
    {"unit_id": "example.com/a", "producer": "bootstrap-go-packages-go-types"}
  ]
}"#;
        let bytes = json.as_bytes();
        let err = admit_reproducible_pair(bytes, bytes).expect_err("bad digest must refuse");
        assert!(matches!(err, AdmitError::DigestMismatch { .. }));
    }

    #[test]
    fn refuses_byte_drift_between_extractor_passes() {
        let first = FIXTURE_SNAPSHOT_JSON.as_bytes();
        let mut second = first.to_vec();
        second.push(b'\n');

        let err = admit_reproducible_pair(first, &second)
            .expect_err("semantically equivalent snapshots with byte drift must refuse");
        assert_eq!(
            err,
            AdmitError::SnapshotMismatch {
                first: digest_bytes(first),
                second: digest_bytes(&second),
            }
        );
    }

    #[test]
    fn refuses_non_go_bootstrap_language() {
        let json = r#"{
  "language": "rust",
  "snapshot_digest": "sha256:unused",
  "packages": []
}"#;
        let bytes = json.as_bytes();
        let err = admit_reproducible_pair(bytes, bytes)
            .expect_err("bootstrap admission must refuse a non-Go language");
        assert_eq!(
            err,
            AdmitError::Language {
                actual: "rust".to_owned(),
            }
        );
    }

    #[test]
    fn production_never_spawns_go() {
        let src = include_str!("lib.rs");
        let production = src
            .split("#[cfg(test)]")
            .next()
            .expect("production section");
        let cmd_new = ["Command", "::", "new"].concat();
        let process_cmd = ["std", "::", "process", "::", "Command"].concat();
        assert!(!production.contains(&cmd_new));
        assert!(!production.contains(&process_cmd));
    }
}
