use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerProseManifest {
    pub schema: String,
    pub repository: OwnerProseRepositoryBinding,
    pub producer: OwnerProseProducer,
    pub owner: String,
    pub sources: Vec<OwnerProseSource>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerProseRepositoryBinding {
    pub identity: String,
    pub source: OwnerProseRevisionBinding,
    pub candidate: OwnerProseRevisionBinding,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerProseRevisionBinding {
    pub commit: String,
    pub tree: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerProseProducer {
    pub identity: String,
    pub schema: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerProseSource {
    pub path: String,
    pub sha256: String,
    pub claims: Vec<OwnerProseClaim>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerProseClaim {
    pub id: String,
    pub start: usize,
    pub end: usize,
    pub sha256: String,
    pub classification: OwnerProseClassification,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_reference: Option<OwnerProseWorkReference>,
    #[serde(default)]
    pub projections: Vec<OwnerProseProjection>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum OwnerProseClassification {
    #[serde(rename = "accepted-current")]
    AcceptedCurrent,
    #[serde(rename = "proposal/work")]
    ProposalWork,
    #[serde(rename = "historical/rejected")]
    HistoricalRejected,
    #[serde(rename = "Unknown")]
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerProseWorkReference {
    pub system: String,
    pub locator: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerProseProjection {
    pub path: String,
    pub start: usize,
    pub end: usize,
    pub sha256: String,
    pub consumer: OwnerProseNativeConsumer,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OwnerProseNativeConsumer {
    RustCompiler,
    RustTest,
    Runtime,
    Admission,
    CedarPolicyEngine,
    ProtobufCompiler,
    Reconciler,
    SloController,
    Cargo,
    Buck,
    OwnershipEnforcement,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct QualifiedOwnerProseView {
    pub(super) schema: String,
    pub(super) repository: OwnerProseRepositoryBinding,
    pub(super) producer: OwnerProseProducer,
    pub(super) qualifier: OwnerProseProducer,
    pub(super) owner: String,
    pub(super) input_manifest_sha256: String,
    pub(super) source_digests: Vec<OwnerProsePathDigest>,
    pub(super) candidate_digests: Vec<OwnerProsePathDigest>,
    pub(super) claims: Vec<QualifiedOwnerProseClaim>,
}

impl QualifiedOwnerProseView {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn repository(&self) -> &OwnerProseRepositoryBinding {
        &self.repository
    }

    pub fn producer(&self) -> &OwnerProseProducer {
        &self.producer
    }

    pub fn qualifier(&self) -> &OwnerProseProducer {
        &self.qualifier
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn input_manifest_sha256(&self) -> &str {
        &self.input_manifest_sha256
    }

    pub fn source_digests(&self) -> &[OwnerProsePathDigest] {
        &self.source_digests
    }

    pub fn candidate_digests(&self) -> &[OwnerProsePathDigest] {
        &self.candidate_digests
    }

    pub fn claims(&self) -> &[QualifiedOwnerProseClaim] {
        &self.claims
    }

    pub(crate) fn authorized_deletions(&self) -> BTreeSet<String> {
        self.source_digests
            .iter()
            .map(|digest| digest.path.clone())
            .collect()
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerProsePathDigest {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualifiedOwnerProseClaim {
    pub source_path: String,
    pub id: String,
    pub start: usize,
    pub end: usize,
    pub sha256: String,
    pub classification: OwnerProseClassification,
    pub work_reference: Option<OwnerProseWorkReference>,
    pub projections: Vec<OwnerProseProjection>,
}
