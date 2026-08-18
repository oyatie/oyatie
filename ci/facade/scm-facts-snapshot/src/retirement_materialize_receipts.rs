//! Collect per-receipt object facts for an active retirement population.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::super::{
    ControlPlaneEntry, PROTECTED_BASE_REF, ReceiptStage, RetirementObjectSource,
    canonical_value_sha256, closure_preparation_link, control_entry_value, coverage_scope,
    find_linked_preparation, input_fact, parse_closed_json, receipt_baseline, receipt_for_stage,
    require_predecessor_baseline, require_regular, required_value_string, sha256_digest,
    validate_oid, validate_receipt_identity,
};
use super::resolve::ResolvedRetirement;

pub(super) struct CollectedReceiptFacts {
    pub(super) receipts: Vec<Value>,
    pub(super) object_facts: Vec<Value>,
    pub(super) scopes: Vec<Value>,
    pub(super) all_required_paths: BTreeSet<String>,
    pub(super) protected_preparations: Vec<Value>,
}

pub(super) fn collect_receipt_facts(
    source: &impl RetirementObjectSource,
    resolved: &ResolvedRetirement,
) -> Result<CollectedReceiptFacts, String> {
    let mut receipts = Vec::new();
    let mut object_facts = Vec::new();
    let mut scopes = Vec::new();
    let mut all_required_paths = BTreeSet::new();
    let mut protected_preparations = Vec::new();

    if resolved.active_receipt_population {
        for (entry, stage) in resolved
            .control_plane
            .entries
            .iter()
            .zip(resolved.stages.iter().copied())
        {
            let (receipt_path, receipt_entry) = receipt_for_stage(
                stage,
                entry,
                &resolved.protected_entries,
                &resolved.candidate_entries,
            )?;
            require_regular(receipt_entry, "retirement receipt")?;
            let receipt_bytes = source.read_blob(&receipt_entry.oid)?;
            let candidate_receipt_sha256 = sha256_digest(&receipt_bytes);
            let receipt: Value = parse_closed_json(&receipt_bytes)?;
            validate_receipt_identity(stage, entry, receipt_path, &receipt)?;
            let artifact_id = required_value_string(receipt.get("artifact_id"), "artifact_id")?;

            let input_facts = entry
                .selectors
                .iter()
                .flat_map(|selector| selector.expected_inputs.iter())
                .map(|input| {
                    all_required_paths.insert(input.path.clone());
                    input_fact(
                        source,
                        input,
                        &resolved.protected_entries,
                        &resolved.candidate_entries,
                        &resolved.equivalence_index,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;

            let control_entry = control_entry_value(entry)?;
            let baseline = receipt
                .get("baseline")
                .and_then(Value::as_object)
                .ok_or_else(|| format!("receipt {receipt_path} has no baseline object"))?;
            let baseline_commit =
                required_value_string(baseline.get("commit_oid"), "baseline.commit_oid")?;
            let baseline_tree =
                required_value_string(baseline.get("tree_oid"), "baseline.tree_oid")?;
            validate_oid(baseline_commit, "receipt baseline commit")?;
            validate_oid(baseline_tree, "receipt baseline tree")?;
            require_predecessor_baseline(
                baseline_commit,
                baseline_tree,
                &resolved.predecessor,
                &resolved.predecessor_tree,
                receipt_path,
            )?;
            receipts.push(json!({
                "receipt_path": receipt_path,
                "artifact_id": artifact_id,
                "scope_ref": entry.scope_ref,
                "receipt_state": stage.as_str(),
                "candidate_receipt_blob_oid": receipt_entry.oid,
                "candidate_receipt_sha256": candidate_receipt_sha256,
                "baseline_commit_oid": baseline_commit,
                "baseline_tree_oid": baseline_tree,
            }));

            let (preparation_path, preparation_blob, predecessor_context) = preparation_for_stage(
                source,
                stage,
                entry,
                &receipt,
                receipt_path,
                resolved,
                &mut protected_preparations,
            )?;

            let protected_receipt = resolved.protected_entries.get(receipt_path);
            let protected_receipt_sha256 = protected_receipt
                .map(|entry| {
                    source
                        .read_blob(&entry.oid)
                        .map(|bytes| sha256_digest(&bytes))
                })
                .transpose()?;
            object_facts.push(json!({
                "artifact_id": artifact_id,
                "receipt_path": receipt_path,
                "protected_base_ref": PROTECTED_BASE_REF,
                "receipt_state": stage.as_str(),
                "scope_ref": entry.scope_ref,
                "scope_type": entry.scope_type,
                "baseline_commit_oid": baseline_commit,
                "baseline_tree_oid": baseline_tree,
                "protected_receipt_blob_oid": protected_receipt.map_or(Value::Null, |entry| json!(entry.oid)),
                "candidate_receipt_blob_oid": receipt_entry.oid,
                "protected_registry_row_sha256": protected_receipt_sha256,
                "candidate_registry_row_sha256": candidate_receipt_sha256,
                "retired_inputs": input_facts,
                "preparation_receipt_path": preparation_path,
                "protected_preparation_receipt_blob_oid": preparation_blob,
                "predecessor_context": predecessor_context,
                "control_plane_entry": control_entry,
                "control_plane_entry_sha256": canonical_value_sha256(&control_entry_value(entry)?)?,
            }));
            scopes.push(coverage_scope(
                entry,
                &resolved.predecessor_entries,
                &resolved.protected_entries,
                &resolved.candidate_entries,
            ));
        }
    }

    Ok(CollectedReceiptFacts {
        receipts,
        object_facts,
        scopes,
        all_required_paths,
        protected_preparations,
    })
}

fn preparation_for_stage(
    source: &impl RetirementObjectSource,
    stage: ReceiptStage,
    entry: &ControlPlaneEntry,
    receipt: &Value,
    receipt_path: &str,
    resolved: &ResolvedRetirement,
    protected_preparations: &mut Vec<Value>,
) -> Result<(Value, Value, Value), String> {
    match stage {
        ReceiptStage::PreparedNew => Ok((
            Value::Null,
            Value::Null,
            json!({
                "source": "control-plane-predecessor",
                "commit_oid": resolved.predecessor,
                "tree_oid": resolved.predecessor_tree,
                "receipt_path": Value::Null,
                "receipt_blob_oid": Value::Null,
            }),
        )),
        ReceiptStage::ClosureNew => {
            let (linked_path, linked_blob) = closure_preparation_link(receipt)?;
            if linked_path != entry.preparation_receipt_path {
                return Err(format!(
                    "closure {receipt_path} links unexpected preparation path"
                ));
            }
            let protected_preparation = resolved
                .protected_entries
                .get(&entry.preparation_receipt_path)
                .ok_or_else(|| "closure is missing protected preparation receipt".to_owned())?;
            require_regular(protected_preparation, "protected preparation receipt")?;
            if linked_blob != protected_preparation.oid {
                return Err(format!(
                    "closure {receipt_path} links unexpected protected preparation blob"
                ));
            }
            let preparation_bytes = source.read_blob(&protected_preparation.oid)?;
            let preparation: Value = parse_closed_json(&preparation_bytes)?;
            validate_receipt_identity(
                ReceiptStage::PreparedNew,
                entry,
                &entry.preparation_receipt_path,
                &preparation,
            )?;
            let (commit, tree) = receipt_baseline(&preparation)?;
            require_predecessor_baseline(
                &commit,
                &tree,
                &resolved.predecessor,
                &resolved.predecessor_tree,
                &entry.preparation_receipt_path,
            )?;
            protected_preparations.push(json!({
                "receipt_path": entry.preparation_receipt_path,
                "receipt_blob_oid": protected_preparation.oid,
                "baseline_commit_oid": commit,
                "baseline_tree_oid": tree,
            }));
            Ok((
                json!(entry.preparation_receipt_path),
                json!(protected_preparation.oid),
                json!({
                    "source": "protected-preparation-receipt",
                    "commit_oid": commit,
                    "tree_oid": tree,
                    "receipt_path": entry.preparation_receipt_path,
                    "receipt_blob_oid": protected_preparation.oid,
                }),
            ))
        }
        ReceiptStage::ClosedCarried => {
            let (path, blob) = closure_preparation_link(receipt)?;
            if path != entry.preparation_receipt_path {
                return Err(format!(
                    "closure {receipt_path} links unexpected preparation path"
                ));
            }
            let (commit, tree) =
                find_linked_preparation(source, &resolved.protected, path, blob, entry)?;
            require_predecessor_baseline(
                &commit,
                &tree,
                &resolved.predecessor,
                &resolved.predecessor_tree,
                path,
            )?;
            protected_preparations.push(json!({
                "receipt_path": path,
                "receipt_blob_oid": blob,
                "baseline_commit_oid": commit,
                "baseline_tree_oid": tree,
            }));
            Ok((
                json!(path),
                json!(blob),
                json!({
                    "source": "linked-preparation-history",
                    "commit_oid": commit,
                    "tree_oid": tree,
                    "receipt_path": path,
                    "receipt_blob_oid": blob,
                }),
            ))
        }
        ReceiptStage::Dormant => unreachable!("dormant stage has no receipt facts"),
    }
}
