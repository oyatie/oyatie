//! Control-plane types and constants for history-only retirement facts.

use std::io::{Cursor, Read};

use serde::{Deserialize, Serialize};

pub(crate) const CONTROL_PLANE_PATH: &str = "registry/history-only-retirement/control-plane.json";
/// Canonical untracked generated-facts path, exposed for the integration contract.
pub const GENERATED_FACTS_PATH: &str =
    "ci/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json";

pub(crate) const CONTROL_PLANE_SCHEMA: &str =
    "https://docs.oyatie.com/schemas/history-only-retirement-control-plane.schema.json";
pub(crate) const CONTROL_PLANE_NAME: &str = "history-only-retirement-control-plane";
pub(crate) const RECEIPT_ROOT: &str = "evidence/history-only-retirement";
pub(crate) const PROTECTED_BASE_REF: &str = "origin/dev";
pub(crate) const CAT_FILE_HEADER_LIMIT: usize = 128;
pub(crate) const MASTERPLAN_EVIDENCE_SET_ID: &str = "masterplan-retired-surfaces-history-only-retirement-v1";
pub(crate) const MASTERPLAN_PREPARATION_ID: &str = "masterplan-retired-surfaces-retirement-preparation";
pub(crate) const MASTERPLAN_PREPARATION_PATH: &str =
    "evidence/history-only-retirement/masterplan-retired-surfaces-preparation.json";
pub(crate) const MASTERPLAN_CLOSURE_ID: &str = "masterplan-retired-surfaces-retirement-closure";
pub(crate) const MASTERPLAN_CLOSURE_PATH: &str =
    "evidence/history-only-retirement/masterplan-retired-surfaces-closure.json";

pub(crate) const ADR_0363_EVIDENCE_SET_ID: &str = "adr-0363-amended-agentic-vcs-history-only-retirement-v1";
pub(crate) const ADR_0363_PREPARATION_ID: &str = "adr-0363-amended-agentic-vcs-retirement-preparation";
pub(crate) const ADR_0363_PREPARATION_PATH: &str =
    "evidence/history-only-retirement/adr-0363-amended-agentic-vcs-preparation.json";
pub(crate) const ADR_0363_CLOSURE_ID: &str = "adr-0363-amended-agentic-vcs-retirement-closure";
pub(crate) const ADR_0363_CLOSURE_PATH: &str =
    "evidence/history-only-retirement/adr-0363-amended-agentic-vcs-closure.json";

pub(crate) const ADR_0388_EVIDENCE_SET_ID: &str = "adr-0388-transient-ideas-history-only-retirement-v1";
pub(crate) const ADR_0388_PREPARATION_ID: &str = "adr-0388-transient-ideas-retirement-preparation";
pub(crate) const ADR_0388_PREPARATION_PATH: &str =
    "evidence/history-only-retirement/adr-0388-transient-ideas-preparation.json";
pub(crate) const ADR_0388_CLOSURE_ID: &str = "adr-0388-transient-ideas-retirement-closure";
pub(crate) const ADR_0388_CLOSURE_PATH: &str =
    "evidence/history-only-retirement/adr-0388-transient-ideas-closure.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetirementControlPlane {
    #[serde(rename = "$schema")]
    pub(crate) schema: String,
    pub(crate) schema_version: u64,
    pub(crate) canonical_name: String,
    pub(crate) planning_state: String,
    pub(crate) dispatch_authorized: bool,
    pub(crate) receipt_root: String,
    pub(crate) predecessor_snapshot: CommitTree,
    pub(crate) entries: Vec<ControlPlaneEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CommitTree {
    pub(crate) commit_oid: String,
    pub(crate) tree_oid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ControlPlaneEntry {
    pub(crate) evidence_set_id: String,
    pub(crate) scope_ref: String,
    pub(crate) scope_type: String,
    pub(crate) selectors: Vec<ControlSelector>,
    pub(crate) preparation_artifact_id: String,
    pub(crate) preparation_receipt_path: String,
    pub(crate) closure_artifact_id: String,
    pub(crate) closure_receipt_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ControlSelector {
    pub(crate) selector_type: String,
    pub(crate) selector: String,
    pub(crate) expected_inputs: Vec<ExpectedInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExpectedInput {
    pub(crate) path: String,
    pub(crate) mode: String,
    pub(crate) predecessor_blob_oid: String,
    pub(crate) sha256: String,
    pub(crate) byte_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TreeEntry {
    pub(crate) mode: String,
    pub(crate) kind: String,
    pub(crate) oid: String,
    pub(crate) path: String,
}

impl TreeEntry {
    fn is_regular_blob(&self) -> bool {
        self.mode == "100644" && self.kind == "blob"
    }
}

/// Streaming visitor for one bounded Git blob body.
pub type BlobVisitor<'a> = dyn FnMut(&str, u64, &mut dyn Read) -> Result<(), String> + 'a;

pub(crate) trait RetirementObjectSource {
    fn resolve_commit(&self, revision: &str) -> Result<String, String>;
    fn tree_for_commit(&self, commit_oid: &str) -> Result<String, String>;
    fn first_parent(&self, commit_oid: &str) -> Result<String, String>;
    fn parents(&self, commit_oid: &str) -> Result<Vec<String>, String>;
    fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool, String>;
    fn tree_entries(&self, commit_oid: &str) -> Result<Vec<TreeEntry>, String>;
    fn read_blob(&self, blob_oid: &str) -> Result<Vec<u8>, String>;
    /// Visit the requested blobs without requiring callers to retain their bodies.
    ///
    /// Sources with an efficient streaming object protocol should override this. The
    /// default keeps test doubles and non-Git sources correct while preserving the
    /// bounded-memory contract for callers.
    fn visit_blobs(&self, blob_oids: &[String], visit: &mut BlobVisitor<'_>) -> Result<(), String> {
        for blob_oid in blob_oids {
            let bytes = self.read_blob(blob_oid)?;
            let size = bytes.len() as u64;
            let mut reader = Cursor::new(bytes);
            visit(blob_oid, size, &mut reader)?;
        }
        Ok(())
    }
    fn commits_touching_path(&self, commit_oid: &str, path: &str) -> Result<Vec<String>, String>;
}

