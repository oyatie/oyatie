//! Shared retirement-facts test doubles and constructors.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use super::{
    ADR_0363_CLOSURE_ID, ADR_0363_CLOSURE_PATH, ADR_0363_EVIDENCE_SET_ID, ADR_0363_PREPARATION_ID,
    ADR_0363_PREPARATION_PATH, ADR_0388_CLOSURE_ID, ADR_0388_CLOSURE_PATH, ADR_0388_EVIDENCE_SET_ID,
    ADR_0388_PREPARATION_ID, ADR_0388_PREPARATION_PATH, CONTROL_PLANE_NAME, CONTROL_PLANE_PATH,
    CONTROL_PLANE_SCHEMA, CommitTree, ControlPlaneEntry, ControlSelector, ExpectedInput,
    MASTERPLAN_CLOSURE_ID, MASTERPLAN_CLOSURE_PATH, MASTERPLAN_EVIDENCE_SET_ID,
    MASTERPLAN_PREPARATION_ID, MASTERPLAN_PREPARATION_PATH, RECEIPT_ROOT, RetirementControlPlane,
    RetirementObjectSource, TreeEntry, sha256_digest,
};

pub(crate) const PREDECESSOR: &str = "1111111111111111111111111111111111111111";
pub(crate) const PREDECESSOR_TREE: &str = "2222222222222222222222222222222222222222";
pub(crate) const PROTECTED: &str = "3333333333333333333333333333333333333333";
pub(crate) const PROTECTED_TREE: &str = "4444444444444444444444444444444444444444";
pub(crate) const CANDIDATE: &str = "5555555555555555555555555555555555555555";
pub(crate) const CANDIDATE_TREE: &str = "6666666666666666666666666666666666666666";
pub(crate) const OTHER_COMMIT: &str = "7777777777777777777777777777777777777777";
#[derive(Clone)]
pub(crate) struct FakeSource {
    pub(crate) commits: BTreeMap<String, String>,
    pub(crate) first_parent: String,
    pub(crate) parents: Vec<String>,
    pub(crate) ancestry: BTreeSet<(String, String)>,
    pub(crate) trees: BTreeMap<String, Vec<TreeEntry>>,
    pub(crate) blobs: BTreeMap<String, Vec<u8>>,
    pub(crate) read_counts: RefCell<BTreeMap<String, usize>>,
    pub(crate) history: BTreeMap<String, Vec<String>>,
}

impl RetirementObjectSource for FakeSource {
    fn resolve_commit(&self, revision: &str) -> Result<String, String> {
        self.commits
            .get(revision)
            .cloned()
            .ok_or_else(|| format!("missing commit {revision}"))
    }
    fn tree_for_commit(&self, commit_oid: &str) -> Result<String, String> {
        self.commits
            .get(&format!("tree:{commit_oid}"))
            .cloned()
            .ok_or_else(|| "missing tree".to_owned())
    }
    fn first_parent(&self, _commit_oid: &str) -> Result<String, String> {
        Ok(self.first_parent.clone())
    }
    fn parents(&self, _commit_oid: &str) -> Result<Vec<String>, String> {
        Ok(self.parents.clone())
    }
    fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool, String> {
        Ok(self
            .ancestry
            .contains(&(ancestor.to_owned(), descendant.to_owned())))
    }
    fn tree_entries(&self, commit_oid: &str) -> Result<Vec<TreeEntry>, String> {
        self.trees
            .get(commit_oid)
            .cloned()
            .ok_or_else(|| format!("missing entries {commit_oid}"))
    }
    fn read_blob(&self, blob_oid: &str) -> Result<Vec<u8>, String> {
        *self
            .read_counts
            .borrow_mut()
            .entry(blob_oid.to_owned())
            .or_default() += 1;
        self.blobs
            .get(blob_oid)
            .cloned()
            .ok_or_else(|| format!("missing blob {blob_oid}"))
    }
    fn commits_touching_path(
        &self,
        _commit_oid: &str,
        path: &str,
    ) -> Result<Vec<String>, String> {
        Ok(self.history.get(path).cloned().unwrap_or_default())
    }
}

pub(crate) fn oid(byte: u8) -> String {
    format!("{byte:040x}")
}

pub(crate) fn input(path: &str, bytes: &[u8], blob_oid: String) -> ExpectedInput {
    ExpectedInput {
        path: path.to_owned(),
        mode: "100644".to_owned(),
        predecessor_blob_oid: blob_oid,
        sha256: sha256_digest(bytes),
        byte_count: bytes.len() as u64,
    }
}

pub(crate) fn control_plane() -> RetirementControlPlane {
    let bodies = [
        ("docs/ROADMAP.md", b"roadmap".as_slice()),
        (".omc/ultragoal/OWNERS", b"owners".as_slice()),
        (
            ".omc/ultragoal/TEAMMATE-PREAMBLE.md",
            b"preamble".as_slice(),
        ),
        (".omc/ultragoal/friction-ledger.jsonl", b"ledger".as_slice()),
        (".omc/ultragoal/premise.txt", b"premise".as_slice()),
        (
            "docs/ideas/archive/cloud-intelligence-bedrock-on-talos-2026-05-28.md",
            b"idea-a".as_slice(),
        ),
        (
            "docs/ideas/archive/cloud-intelligence-v1-pipeline-2026-05-28.md",
            b"idea-b".as_slice(),
        ),
        (
            "docs/ideas/archive/n-lane-parallel-safety-and-unified-devops-console-2026-05-28.md",
            b"idea-c".as_slice(),
        ),
    ];
    RetirementControlPlane {
        schema: CONTROL_PLANE_SCHEMA.to_owned(),
        schema_version: 1,
        canonical_name: CONTROL_PLANE_NAME.to_owned(),
        planning_state: "HOLD(Planning)".to_owned(),
        dispatch_authorized: false,
        receipt_root: RECEIPT_ROOT.to_owned(),
        predecessor_snapshot: CommitTree {
            commit_oid: PREDECESSOR.to_owned(),
            tree_oid: PREDECESSOR_TREE.to_owned(),
        },
        entries: vec![
            ControlPlaneEntry {
                selectors: vec![ControlSelector {
                    selector_type: "exact".to_owned(),
                    selector: "docs/ROADMAP.md".to_owned(),
                    expected_inputs: vec![input(bodies[0].0, bodies[0].1, oid(10))],
                }],
                ..fixed_entry(
                    "artifact:masterplan",
                    "masterplan-retired-surfaces",
                    MASTERPLAN_EVIDENCE_SET_ID,
                    MASTERPLAN_PREPARATION_ID,
                    MASTERPLAN_PREPARATION_PATH,
                    MASTERPLAN_CLOSURE_ID,
                    MASTERPLAN_CLOSURE_PATH,
                )
            },
            ControlPlaneEntry {
                selectors: bodies[1..5]
                    .iter()
                    .enumerate()
                    .map(|(index, (path, bytes))| ControlSelector {
                        selector_type: "exact".to_owned(),
                        selector: (*path).to_owned(),
                        expected_inputs: vec![input(path, bytes, oid(11 + index as u8))],
                    })
                    .collect(),
                ..fixed_entry(
                    "ADR-0363",
                    "amended-agentic-vcs-retirement",
                    ADR_0363_EVIDENCE_SET_ID,
                    ADR_0363_PREPARATION_ID,
                    ADR_0363_PREPARATION_PATH,
                    ADR_0363_CLOSURE_ID,
                    ADR_0363_CLOSURE_PATH,
                )
            },
            ControlPlaneEntry {
                selectors: vec![ControlSelector {
                    selector_type: "glob".to_owned(),
                    selector: "docs/ideas/archive/**".to_owned(),
                    expected_inputs: bodies[5..]
                        .iter()
                        .enumerate()
                        .map(|(index, (path, bytes))| input(path, bytes, oid(20 + index as u8)))
                        .collect(),
                }],
                ..fixed_entry(
                    "ADR-0388",
                    "transient-ideas",
                    ADR_0388_EVIDENCE_SET_ID,
                    ADR_0388_PREPARATION_ID,
                    ADR_0388_PREPARATION_PATH,
                    ADR_0388_CLOSURE_ID,
                    ADR_0388_CLOSURE_PATH,
                )
            },
        ],
    }
}

pub(crate) fn fixture() -> FakeSource {
    let control = control_plane();
    let control_bytes = to_canonical_json(&serde_json::to_value(&control).unwrap())
        .unwrap()
        .into_bytes();
    let control_oid = oid(90);
    let mut predecessor_entries = Vec::new();
    let mut blobs = BTreeMap::from([(control_oid.clone(), control_bytes)]);
    for input in control
        .entries
        .iter()
        .flat_map(|entry| entry.selectors.iter())
        .flat_map(|selector| selector.expected_inputs.iter())
    {
        let bytes = input.path.rsplit('/').next().unwrap().as_bytes().to_vec();
        // Replace the synthetic input bytes with bytes matching its declared digest.
        let declared = match input.path.as_str() {
            "docs/ROADMAP.md" => b"roadmap".to_vec(),
            ".omc/ultragoal/OWNERS" => b"owners".to_vec(),
            ".omc/ultragoal/TEAMMATE-PREAMBLE.md" => b"preamble".to_vec(),
            ".omc/ultragoal/friction-ledger.jsonl" => b"ledger".to_vec(),
            ".omc/ultragoal/premise.txt" => b"premise".to_vec(),
            path if path.contains("bedrock") => b"idea-a".to_vec(),
            path if path.contains("v1-pipeline") => b"idea-b".to_vec(),
            _ => b"idea-c".to_vec(),
        };
        let _ = bytes;
        blobs.insert(input.predecessor_blob_oid.clone(), declared);
        predecessor_entries.push(TreeEntry {
            mode: "100644".to_owned(),
            kind: "blob".to_owned(),
            oid: input.predecessor_blob_oid.clone(),
            path: input.path.clone(),
        });
    }
    let candidate_entries = vec![TreeEntry {
        mode: "100644".to_owned(),
        kind: "blob".to_owned(),
        oid: control_oid,
        path: CONTROL_PLANE_PATH.to_owned(),
    }];
    FakeSource {
        commits: BTreeMap::from([
            ("HEAD".to_owned(), CANDIDATE.to_owned()),
            (CANDIDATE.to_owned(), CANDIDATE.to_owned()),
            (PROTECTED.to_owned(), PROTECTED.to_owned()),
            (PREDECESSOR.to_owned(), PREDECESSOR.to_owned()),
            (format!("tree:{CANDIDATE}"), CANDIDATE_TREE.to_owned()),
            (format!("tree:{PROTECTED}"), PROTECTED_TREE.to_owned()),
            (format!("tree:{PREDECESSOR}"), PREDECESSOR_TREE.to_owned()),
        ]),
        first_parent: PROTECTED.to_owned(),
        parents: vec![PROTECTED.to_owned()],
        ancestry: BTreeSet::from([
            (PROTECTED.to_owned(), CANDIDATE.to_owned()),
            (PREDECESSOR.to_owned(), PROTECTED.to_owned()),
        ]),
        trees: BTreeMap::from([
            (PREDECESSOR.to_owned(), predecessor_entries),
            (PROTECTED.to_owned(), Vec::new()),
            (CANDIDATE.to_owned(), candidate_entries),
        ]),
        blobs,
        read_counts: RefCell::new(BTreeMap::new()),
        history: BTreeMap::new(),
    }
}


use super::RetirementMaterializationContext;

pub(crate) fn context() -> RetirementMaterializationContext<'static> {
    RetirementMaterializationContext {
        control_plane_path: super::CONTROL_PLANE_PATH,
        protected_base_commit: PROTECTED,
        evaluated_commit: CANDIDATE,
        scm_event_name: "push",
        scm_event_ref: "refs/heads/dev",
        scm_event_base_ref: "refs/heads/dev",
        subject_commit: CANDIDATE,
    }
}
