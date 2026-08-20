//! Control-plane and selector validation for history-only retirement facts.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    ADR_0363_CLOSURE_ID, ADR_0363_CLOSURE_PATH, ADR_0363_EVIDENCE_SET_ID, ADR_0363_PREPARATION_ID,
    ADR_0363_PREPARATION_PATH, ADR_0388_CLOSURE_ID, ADR_0388_CLOSURE_PATH,
    ADR_0388_EVIDENCE_SET_ID, ADR_0388_PREPARATION_ID, ADR_0388_PREPARATION_PATH,
    CONTROL_PLANE_NAME, CONTROL_PLANE_SCHEMA, ControlPlaneEntry, MASTERPLAN_CLOSURE_ID,
    MASTERPLAN_CLOSURE_PATH, MASTERPLAN_EVIDENCE_SET_ID, MASTERPLAN_PREPARATION_ID,
    MASTERPLAN_PREPARATION_PATH, RECEIPT_ROOT, RetirementControlPlane, RetirementObjectSource,
    TreeEntry, require_regular, selector_matches_path, sha256_digest, validate_oid,
    validate_repo_path, validate_sha256,
};

pub(crate) fn validate_control_plane(control: &RetirementControlPlane) -> Result<(), String> {
    if control.schema != CONTROL_PLANE_SCHEMA
        || control.schema_version != 1
        || control.canonical_name != CONTROL_PLANE_NAME
        || control.planning_state != "HOLD(Planning)"
        || control.dispatch_authorized
        || control.receipt_root != RECEIPT_ROOT
    {
        return Err("retirement control-plane header is not canonical HOLD".to_owned());
    }
    validate_oid(
        &control.predecessor_snapshot.commit_oid,
        "retirement predecessor commit",
    )?;
    validate_oid(
        &control.predecessor_snapshot.tree_oid,
        "retirement predecessor tree",
    )?;
    if control.entries.len() != 3 {
        return Err("retirement control plane must contain exactly three entries".to_owned());
    }
    let expected = [
        fixed_entry(
            "artifact:masterplan",
            "masterplan-retired-surfaces",
            MASTERPLAN_EVIDENCE_SET_ID,
            MASTERPLAN_PREPARATION_ID,
            MASTERPLAN_PREPARATION_PATH,
            MASTERPLAN_CLOSURE_ID,
            MASTERPLAN_CLOSURE_PATH,
        ),
        fixed_entry(
            "ADR-0363",
            "amended-agentic-vcs-retirement",
            ADR_0363_EVIDENCE_SET_ID,
            ADR_0363_PREPARATION_ID,
            ADR_0363_PREPARATION_PATH,
            ADR_0363_CLOSURE_ID,
            ADR_0363_CLOSURE_PATH,
        ),
        fixed_entry(
            "ADR-0388",
            "transient-ideas",
            ADR_0388_EVIDENCE_SET_ID,
            ADR_0388_PREPARATION_ID,
            ADR_0388_PREPARATION_PATH,
            ADR_0388_CLOSURE_ID,
            ADR_0388_CLOSURE_PATH,
        ),
    ];
    for (entry, fixed) in control.entries.iter().zip(expected) {
        if entry.scope_ref != fixed.scope_ref
            || entry.scope_type != fixed.scope_type
            || entry.evidence_set_id != fixed.evidence_set_id
            || entry.preparation_artifact_id != fixed.preparation_artifact_id
            || entry.preparation_receipt_path != fixed.preparation_receipt_path
            || entry.closure_artifact_id != fixed.closure_artifact_id
            || entry.closure_receipt_path != fixed.closure_receipt_path
        {
            return Err(format!(
                "retirement control-plane identity mismatch for {}",
                entry.scope_ref
            ));
        }
        validate_repo_path(&entry.preparation_receipt_path)?;
        validate_repo_path(&entry.closure_receipt_path)?;
    }
    validate_fixed_selectors(control)
}

pub(crate) fn fixed_entry(
    scope_ref: &str,
    scope_type: &str,
    evidence_set_id: &str,
    preparation_artifact_id: &str,
    preparation_receipt_path: &str,
    closure_artifact_id: &str,
    closure_receipt_path: &str,
) -> ControlPlaneEntry {
    ControlPlaneEntry {
        evidence_set_id: evidence_set_id.to_owned(),
        scope_ref: scope_ref.to_owned(),
        scope_type: scope_type.to_owned(),
        selectors: Vec::new(),
        preparation_artifact_id: preparation_artifact_id.to_owned(),
        preparation_receipt_path: preparation_receipt_path.to_owned(),
        closure_artifact_id: closure_artifact_id.to_owned(),
        closure_receipt_path: closure_receipt_path.to_owned(),
    }
}

pub(crate) fn validate_fixed_selectors(control: &RetirementControlPlane) -> Result<(), String> {
    let expected: BTreeMap<&str, Vec<(&str, &str)>> = BTreeMap::from([
        ("artifact:masterplan", vec![("exact", "docs/ROADMAP.md")]),
        (
            "ADR-0363",
            vec![
                ("exact", ".omc/ultragoal/OWNERS"),
                ("exact", ".omc/ultragoal/TEAMMATE-PREAMBLE.md"),
                ("exact", ".omc/ultragoal/friction-ledger.jsonl"),
                ("exact", ".omc/ultragoal/premise.txt"),
            ],
        ),
        ("ADR-0388", vec![("glob", "docs/ideas/archive/**")]),
    ]);
    for entry in &control.entries {
        let required = expected
            .get(entry.scope_ref.as_str())
            .ok_or_else(|| format!("unknown retirement scope {}", entry.scope_ref))?;
        if entry.selectors.len() != required.len() {
            return Err(format!("selector count mismatch for {}", entry.scope_ref));
        }
        for (selector, (kind, pattern)) in entry.selectors.iter().zip(required) {
            if selector.selector_type != *kind || selector.selector != *pattern {
                return Err(format!("selector mismatch for {}", entry.scope_ref));
            }
            if selector.expected_inputs.is_empty() {
                return Err(format!(
                    "selector has no immutable inputs for {}",
                    entry.scope_ref
                ));
            }
            for input in &selector.expected_inputs {
                validate_repo_path(&input.path)?;
                if input.mode != "100644" {
                    return Err(format!(
                        "immutable retirement input {} must declare mode 100644",
                        input.path
                    ));
                }
                validate_oid(&input.predecessor_blob_oid, "predecessor blob")?;
                validate_sha256(&input.sha256)?;
            }
        }
    }
    let actual_paths = control
        .entries
        .iter()
        .flat_map(|entry| entry.selectors.iter())
        .flat_map(|selector| selector.expected_inputs.iter())
        .map(|input| input.path.as_str())
        .collect::<BTreeSet<_>>();
    let required_paths = BTreeSet::from([
        "docs/ROADMAP.md",
        ".omc/ultragoal/OWNERS",
        ".omc/ultragoal/TEAMMATE-PREAMBLE.md",
        ".omc/ultragoal/friction-ledger.jsonl",
        ".omc/ultragoal/premise.txt",
        "docs/ideas/archive/cloud-intelligence-bedrock-on-talos-2026-05-28.md",
        "docs/ideas/archive/cloud-intelligence-v1-pipeline-2026-05-28.md",
        "docs/ideas/archive/n-lane-parallel-safety-and-unified-devops-console-2026-05-28.md",
    ]);
    if actual_paths != required_paths {
        return Err("retirement immutable input population is not exact".to_owned());
    }
    Ok(())
}

pub(crate) fn validate_predecessor_inputs(
    source: &impl RetirementObjectSource,
    control: &RetirementControlPlane,
    predecessor_entries: &BTreeMap<String, TreeEntry>,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut bodies = BTreeMap::new();
    for input in control
        .entries
        .iter()
        .flat_map(|entry| entry.selectors.iter())
        .flat_map(|selector| selector.expected_inputs.iter())
    {
        let entry = predecessor_entries
            .get(&input.path)
            .ok_or_else(|| format!("retirement predecessor path {} is absent", input.path))?;
        require_regular(entry, "retirement predecessor input")?;
        if entry.mode != input.mode || entry.oid != input.predecessor_blob_oid {
            return Err(format!(
                "retirement predecessor blob mismatch for {}",
                input.path
            ));
        }
        let bytes = source.read_blob(&entry.oid)?;
        if sha256_digest(&bytes) != input.sha256 || bytes.len() as u64 != input.byte_count {
            return Err(format!(
                "retirement predecessor raw-byte binding mismatch for {}",
                input.path
            ));
        }
        if bodies.insert(input.path.clone(), bytes).is_some() {
            return Err(format!(
                "retirement predecessor input population duplicates {}",
                input.path
            ));
        }
    }
    Ok(bodies)
}

pub(crate) fn validate_selector_coverage(
    control: &RetirementControlPlane,
    entries: &BTreeMap<String, TreeEntry>,
    tree_role: &str,
) -> Result<(), String> {
    for selector in control.entries.iter().flat_map(|entry| &entry.selectors) {
        let expected = selector
            .expected_inputs
            .iter()
            .map(|input| input.path.as_str())
            .collect::<BTreeSet<_>>();
        for path in entries
            .keys()
            .filter(|path| selector_matches_path(selector, path))
        {
            if !expected.contains(path.as_str()) {
                return Err(format!(
                    "retirement selector coverage rejects unlisted {tree_role} path {path}"
                ));
            }
        }
    }
    Ok(())
}
