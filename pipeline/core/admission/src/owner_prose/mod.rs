//! Exact-revision qualification for frozen owner-prose migration input.

mod claims;
mod manifest;
mod qualify;
mod validation;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use manifest::{
    OwnerProseClaim, OwnerProseClassification, OwnerProseManifest, OwnerProseNativeConsumer,
    OwnerProsePathDigest, OwnerProseProducer, OwnerProseProjection, OwnerProseRepositoryBinding,
    OwnerProseRevisionBinding, OwnerProseSource, OwnerProseWorkReference, QualifiedOwnerProseClaim,
    QualifiedOwnerProseView,
};
pub use qualify::qualify_owner_prose;

pub const OWNER_PROSE_CLASSIFICATION_SCHEMA: &str = "oyatie.owner-prose-classification.v1";
pub const OWNER_PROSE_CLASSIFIER_IDENTITY: &str = "pipeline-owner-prose-classifier";
pub const OWNER_PROSE_PRODUCER_SCHEMA: &str = "oyatie.owner-prose-classifier.v1";
pub const OWNER_PROSE_QUALIFIER_IDENTITY: &str = "pipeline-owner-prose-qualifier";
pub const OWNER_PROSE_QUALIFIER_SCHEMA: &str = "oyatie.owner-prose-qualifier.v1";
pub const OWNER_PROSE_QUALIFIED_VIEW_SCHEMA: &str = "oyatie.owner-prose-qualified-view.v1";

pub(crate) const OWNER_PROSE_NAMES: [&str; 4] = ["ADR.md", "PLAN.md", "PRD.md", "SPEC.md"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerProseRevision {
    Source,
    Candidate,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OwnerProseRefusalKind {
    ManifestInvalid,
    SchemaMismatch,
    RepositoryBindingMismatch,
    ProducerInvalid,
    OwnerInvalid,
    SourceSetMismatch,
    SourceUnavailable,
    RepositoryReadFailed,
    SourceDigestMismatch,
    ClaimCoverageMismatch,
    ClaimIdentityInvalid,
    DuplicateClassification,
    ClaimDigestMismatch,
    UnknownClassification,
    WorkReferenceInvalid,
    ProjectionCountMismatch,
    ProjectionTargetInvalid,
    ProjectionUnavailable,
    ProjectionDigestMismatch,
    DuplicateProjection,
    AtomicDeletionIncomplete,
}

impl OwnerProseRefusalKind {
    pub fn semantic_name(&self) -> &'static str {
        match self {
            Self::ManifestInvalid => "manifest-invalid",
            Self::SchemaMismatch => "schema-mismatch",
            Self::RepositoryBindingMismatch => "repository-binding-mismatch",
            Self::ProducerInvalid => "producer-invalid",
            Self::OwnerInvalid => "owner-invalid",
            Self::SourceSetMismatch => "source-set-mismatch",
            Self::SourceUnavailable => "source-unavailable",
            Self::RepositoryReadFailed => "repository-read-failed",
            Self::SourceDigestMismatch => "source-digest-mismatch",
            Self::ClaimCoverageMismatch => "claim-coverage-mismatch",
            Self::ClaimIdentityInvalid => "claim-identity-invalid",
            Self::DuplicateClassification => "duplicate-classification",
            Self::ClaimDigestMismatch => "claim-digest-mismatch",
            Self::UnknownClassification => "unknown-classification",
            Self::WorkReferenceInvalid => "work-reference-invalid",
            Self::ProjectionCountMismatch => "projection-count-mismatch",
            Self::ProjectionTargetInvalid => "projection-target-invalid",
            Self::ProjectionUnavailable => "projection-unavailable",
            Self::ProjectionDigestMismatch => "projection-digest-mismatch",
            Self::DuplicateProjection => "duplicate-projection",
            Self::AtomicDeletionIncomplete => "atomic-deletion-incomplete",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerProseRefusal {
    pub kind: OwnerProseRefusalKind,
    pub subject: String,
    pub detail: String,
}

impl OwnerProseRefusal {
    pub(crate) fn new(
        kind: OwnerProseRefusalKind,
        subject: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
            detail: detail.into(),
        }
    }

    pub fn message(&self) -> String {
        format!(
            "{}: {}: {}",
            self.kind.semantic_name(),
            self.subject,
            self.detail
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnerProseQualification {
    Ready(Box<QualifiedOwnerProseView>),
    Unknown(Vec<OwnerProseRefusal>),
}

impl OwnerProseQualification {
    pub(crate) fn unknown(mut refusals: Vec<OwnerProseRefusal>) -> Self {
        refusals.sort();
        refusals.dedup();
        Self::Unknown(refusals)
    }
}

pub fn owner_prose_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn canonical_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}
