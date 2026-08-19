//! An admitted snapshot: the model, bound to the fleet pin, with its digests verified.

use port_engine_api::{Declaration, Digest, SourceModel, UnitId};
use port_engine_frontend_go::GoSourceModel;

/// An admitted bootstrap snapshot bound to the fleet pin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedSnapshot {
    /// Fleet pin (peeled commit) bound at admission.
    pub(crate) pin: String,
    /// SHA-256 digest of the byte-identical raw snapshot artifact.
    pub(crate) artifact_digest: Digest,
    /// Verified semantic digest claimed inside the artifact.
    pub(crate) model_digest: Digest,
    /// Decoded SourceModel (identity + order only).
    pub(crate) model: GoSourceModel,
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

    fn declarations(&self, unit: &UnitId) -> Option<Vec<Declaration>> {
        self.model.declarations_for(unit)
    }
}
