//! The decoded Go source model: artifact bytes in, identity and declarations out.
//!
//! No Go toolchain, ever. ADR-0638 D3 puts the extractor out of band and leaves this side
//! consuming artifacts only.

use std::collections::BTreeSet;

use port_engine_api::{Declaration, Digest, UnitId};

use crate::convert::convert_declarations;
use crate::error::SnapshotError;
use crate::vocabulary::{
    PRODUCER_BOOTSTRAP_GO, PRODUCER_OWNED_RUST, SCHEMA_VERSION_DECLARATIONS,
    SCHEMA_VERSION_IDENTITY_ONLY,
};
use crate::wire::SnapshotDocument;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceModel {
    schema_version: u32,
    language: String,
    build_config: String,
    snapshot_digest: Digest,
    units: Vec<UnitId>,
    /// Parallel to [`Self::units`] — ADR-0638 D3 package→producer map (one producer per package).
    producers: Vec<String>,
    /// Parallel to [`Self::units`] — declaration tree per package. Empty for a v0 artifact.
    declarations: Vec<Vec<Declaration>>,
}

impl GoSourceModel {
    /// Decode snapshot JSON bytes into an unadmitted Go model.
    ///
    /// # Errors
    /// [`SnapshotError`] on parse failure, schema violation, unknown producer, or duplicate unit.
    pub fn decode(bytes: &[u8]) -> Result<Self, SnapshotError> {
        let text = std::str::from_utf8(bytes).map_err(|err| SnapshotError::Parse {
            detail: format!("utf-8: {err}"),
        })?;
        Self::decode_str(text)
    }

    /// Decode snapshot JSON from an in-memory string (test hook and future adapter input).
    ///
    /// # Errors
    /// [`SnapshotError`] on parse failure, schema violation, unknown producer, or duplicate unit.
    pub fn decode_str(json: &str) -> Result<Self, SnapshotError> {
        let doc: SnapshotDocument =
            serde_json::from_str(json).map_err(|err| SnapshotError::Parse {
                detail: err.to_string(),
            })?;
        if doc.language.is_empty() {
            return Err(SnapshotError::Schema { field: "language" });
        }
        if doc.snapshot_digest.is_empty() {
            return Err(SnapshotError::Schema {
                field: "snapshot_digest",
            });
        }
        if doc.schema_version != SCHEMA_VERSION_IDENTITY_ONLY
            && doc.schema_version != SCHEMA_VERSION_DECLARATIONS
        {
            return Err(SnapshotError::UnknownSchemaVersion {
                actual: doc.schema_version,
            });
        }

        let mut units = Vec::with_capacity(doc.packages.len());
        let mut producers = Vec::with_capacity(doc.packages.len());
        let mut declarations = Vec::with_capacity(doc.packages.len());
        let mut seen = BTreeSet::new();
        for pkg in doc.packages {
            if pkg.unit_id.is_empty() || pkg.unit_id.contains('\0') {
                return Err(SnapshotError::Schema {
                    field: "packages.unit_id",
                });
            }
            if pkg.producer != PRODUCER_BOOTSTRAP_GO && pkg.producer != PRODUCER_OWNED_RUST {
                return Err(SnapshotError::UnknownProducer {
                    actual: pkg.producer,
                });
            }
            if !seen.insert(pkg.unit_id.clone()) {
                return Err(SnapshotError::DuplicateUnit {
                    unit_id: pkg.unit_id,
                });
            }
            // A v0 artifact carrying declarations is a version lie, not a bonus. Accepting it
            // would mean the version field says one thing about the payload while the payload
            // says another, and every later reader has to guess which one to believe.
            if doc.schema_version == SCHEMA_VERSION_IDENTITY_ONLY && !pkg.declarations.is_empty() {
                return Err(SnapshotError::VersionPayloadMismatch {
                    detail: "schema_version 0 carries declarations",
                });
            }
            declarations.push(convert_declarations(&pkg.unit_id, &pkg.declarations)?);
            units.push(UnitId(pkg.unit_id));
            producers.push(pkg.producer);
        }
        Ok(Self {
            schema_version: doc.schema_version,
            language: doc.language,
            build_config: doc.build_config,
            snapshot_digest: Digest(doc.snapshot_digest),
            units,
            producers,
            declarations,
        })
    }

    /// Source-language slug claimed by the decoded snapshot.
    #[must_use]
    pub fn language(&self) -> &str {
        &self.language
    }

    /// The build configuration the snapshot was extracted FOR, or empty for one written before the
    /// field existed.
    #[must_use]
    pub fn build_config(&self) -> &str {
        &self.build_config
    }

    /// Semantic digest claimed by the decoded snapshot.
    #[must_use]
    pub fn snapshot_digest(&self) -> Digest {
        self.snapshot_digest.clone()
    }

    /// Units in decoded snapshot order.
    #[must_use]
    pub fn units(&self) -> Vec<UnitId> {
        self.units.clone()
    }

    /// Envelope version this model was decoded from.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Producer identity for `unit`, if present in the snapshot map.
    #[must_use]
    pub fn producer_for(&self, unit: &UnitId) -> Option<&str> {
        self.units
            .iter()
            .position(|u| u == unit)
            .map(|idx| producers_at(self, idx))
    }

    /// Declaration tree for `unit`, or `None` when the model does not carry that unit.
    #[must_use]
    pub fn declarations_for(&self, unit: &UnitId) -> Option<Vec<Declaration>> {
        self.units
            .iter()
            .position(|u| u == unit)
            .map(|idx| self.declarations[idx].clone())
    }
}

fn producers_at(model: &GoSourceModel, idx: usize) -> &str {
    &model.producers[idx]
}
