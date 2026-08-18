const MASTERPLAN_EVIDENCE_SET_ID: &str = "masterplan-retired-surfaces-history-only-retirement-v1";
const MASTERPLAN_PREPARATION_ID: &str = "masterplan-retired-surfaces-retirement-preparation";
const MASTERPLAN_PREPARATION_PATH: &str =
    "evidence/history-only-retirement/masterplan-retired-surfaces-preparation.json";
const MASTERPLAN_CLOSURE_ID: &str = "masterplan-retired-surfaces-retirement-closure";
const MASTERPLAN_CLOSURE_PATH: &str =
    "evidence/history-only-retirement/masterplan-retired-surfaces-closure.json";

const ADR_0363_EVIDENCE_SET_ID: &str = "adr-0363-amended-agentic-vcs-history-only-retirement-v1";
const ADR_0363_PREPARATION_ID: &str = "adr-0363-amended-agentic-vcs-retirement-preparation";
const ADR_0363_PREPARATION_PATH: &str =
    "evidence/history-only-retirement/adr-0363-amended-agentic-vcs-preparation.json";
const ADR_0363_CLOSURE_ID: &str = "adr-0363-amended-agentic-vcs-retirement-closure";
const ADR_0363_CLOSURE_PATH: &str =
    "evidence/history-only-retirement/adr-0363-amended-agentic-vcs-closure.json";

const ADR_0388_EVIDENCE_SET_ID: &str = "adr-0388-transient-ideas-history-only-retirement-v1";
const ADR_0388_PREPARATION_ID: &str = "adr-0388-transient-ideas-retirement-preparation";
const ADR_0388_PREPARATION_PATH: &str =
    "evidence/history-only-retirement/adr-0388-transient-ideas-preparation.json";
const ADR_0388_CLOSURE_ID: &str = "adr-0388-transient-ideas-retirement-closure";
const ADR_0388_CLOSURE_PATH: &str =
    "evidence/history-only-retirement/adr-0388-transient-ideas-closure.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RetirementControlPlane {
    #[serde(rename = "$schema")]
    schema: String,
    schema_version: u64,
    canonical_name: String,
    planning_state: String,
    dispatch_authorized: bool,
    receipt_root: String,
    predecessor_snapshot: CommitTree,
    entries: Vec<ControlPlaneEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CommitTree {
    commit_oid: String,
    tree_oid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ControlPlaneEntry {
    evidence_set_id: String,
    scope_ref: String,
    scope_type: String,
    selectors: Vec<ControlSelector>,
    preparation_artifact_id: String,
    preparation_receipt_path: String,
    closure_artifact_id: String,
    closure_receipt_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ControlSelector {
    selector_type: String,
    selector: String,
    expected_inputs: Vec<ExpectedInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ExpectedInput {
    path: String,
    mode: String,
    predecessor_blob_oid: String,
    sha256: String,
    byte_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TreeEntry {
    mode: String,
    kind: String,
    oid: String,
    path: String,
}

impl TreeEntry {
    fn is_regular_blob(&self) -> bool {
        self.mode == "100644" && self.kind == "blob"
    }
}
