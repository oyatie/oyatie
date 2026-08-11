//! # port-engine-snapshot — bootstrap SourceModel snapshot admission (W0-B Slice 8).
//!
//! ADR-0638 D3: the bootstrap Go extractor (`go/packages` + `go/types`) runs **out of band only**.
//! This adapter admits the resulting content-addressed snapshot artifact, binds it to the fleet
//! pin, and verifies the claimed `snapshot_digest` against a stable preimage. It MUST NEVER
//! invoke a Go toolchain (firewall inherited from `port-engine-frontend-go`).
#![forbid(unsafe_code)]

use std::fmt;

use port_engine_api::{Digest, SourceModel, UnitId};
use port_engine_frontend_go::{GoSourceModel, SnapshotError, PRODUCER_BOOTSTRAP_GO};
use port_engine_hash::digest_bytes;
use port_engine_source_pin::{load_embedded, receipt_pin, PinError};

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
    pub pin: String,
    /// Verified content digest (`sha256:<hex>` of the stable preimage).
    pub snapshot_digest: Digest,
    /// Decoded SourceModel (identity + order only).
    pub model: GoSourceModel,
}

impl AdmittedSnapshot {
    /// Borrow the underlying [`SourceModel`].
    #[must_use]
    pub fn as_model(&self) -> &dyn SourceModel {
        &self.model
    }

    /// Unit ids in deterministic order.
    #[must_use]
    pub fn units(&self) -> Vec<UnitId> {
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

/// Admit snapshot JSON bytes against the fleet pin.
///
/// # Errors
/// [`AdmitError`] on decode, pin, language, or digest mismatch.
pub fn admit_bytes(bytes: &[u8]) -> Result<AdmittedSnapshot, AdmitError> {
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
            .ok_or_else(|| AdmitError::Snapshot(SnapshotError::Schema {
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
        snapshot_digest: computed,
        model,
    })
}

/// Admit the package-local OOB bootstrap fixture.
///
/// # Errors
/// [`AdmitError`] on fixture defect.
pub fn admit_embedded_fixture() -> Result<AdmittedSnapshot, AdmitError> {
    admit_bytes(FIXTURE_SNAPSHOT_JSON.as_bytes())
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
        assert!(!admitted.pin.is_empty());
        assert_eq!(
            admitted.snapshot_digest.0,
            "sha256:b541cdee7bcb23984d8d56171e66f66fdbec027b40a06953105540d5915b33fb"
        );
        assert_eq!(admitted.units().len(), 2);
        assert_eq!(
            admitted.model.producer_for(&UnitId("example.com/a".into())),
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
        let err = admit_bytes(json.as_bytes()).expect_err("bad digest must refuse");
        assert!(matches!(err, AdmitError::DigestMismatch { .. }));
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
