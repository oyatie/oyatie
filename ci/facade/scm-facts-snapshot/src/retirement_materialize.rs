//! Materialize history-only retirement facts from a Git object source.

use serde_json::{Value, json};

use super::{
    CONTROL_PLANE_PATH, PROTECTED_BASE_REF, RECEIPT_ROOT, RetirementMaterializationContext,
    RetirementObjectSource,
};

#[path = "retirement_materialize_receipts.rs"]
mod receipts;
#[path = "retirement_materialize_resolve.rs"]
mod resolve;

use receipts::collect_receipt_facts;
use resolve::{ResolvedRetirement, resolve_retirement_materialization};

pub(crate) fn materialize_history_only_retirement_facts(
    source: &impl RetirementObjectSource,
    context: &RetirementMaterializationContext<'_>,
) -> Result<Value, String> {
    let resolved = resolve_retirement_materialization(source, context)?;
    let collected = collect_receipt_facts(source, &resolved)?;
    assemble_retirement_facts(source, context, resolved, collected)
}

fn assemble_retirement_facts(
    source: &impl RetirementObjectSource,
    context: &RetirementMaterializationContext<'_>,
    resolved: ResolvedRetirement,
    collected: receipts::CollectedReceiptFacts,
) -> Result<Value, String> {
    let mut receipts = collected.receipts;
    let mut object_facts = collected.object_facts;
    let mut scopes = collected.scopes;
    let mut protected_preparations = collected.protected_preparations;
    receipts.sort_by(|left, right| {
        left.get("receipt_path")
            .and_then(Value::as_str)
            .cmp(&right.get("receipt_path").and_then(Value::as_str))
    });
    object_facts.sort_by(|left, right| {
        left.get("receipt_path")
            .and_then(Value::as_str)
            .cmp(&right.get("receipt_path").and_then(Value::as_str))
    });
    scopes.sort_by(|left, right| {
        left.get("scope_ref")
            .and_then(Value::as_str)
            .cmp(&right.get("scope_ref").and_then(Value::as_str))
    });
    protected_preparations.sort_by(|left, right| {
        left.get("receipt_path")
            .and_then(Value::as_str)
            .cmp(&right.get("receipt_path").and_then(Value::as_str))
    });

    let protected_receipt_paths = resolved
        .protected_receipt_inventory
        .intersection(&resolved.expected_receipt_paths)
        .cloned()
        .collect::<Vec<_>>();
    let candidate_receipt_paths = resolved
        .candidate_receipt_inventory
        .intersection(&resolved.expected_receipt_paths)
        .cloned()
        .collect::<Vec<_>>();
    let carried_receipt_paths = protected_receipt_paths
        .iter()
        .filter(|path| candidate_receipt_paths.contains(path))
        .cloned()
        .collect::<Vec<_>>();
    let new_receipt_paths = candidate_receipt_paths
        .iter()
        .filter(|path| !protected_receipt_paths.contains(path))
        .cloned()
        .collect::<Vec<_>>();

    Ok(json!({
        "receipts": receipts,
        "scm_facts": {
            "retirement_receipt_coverage": {
                "protected_base_ref": PROTECTED_BASE_REF,
                "protected_receipt_paths": protected_receipt_paths,
                "candidate_receipt_paths": candidate_receipt_paths,
                "carried_receipt_paths": carried_receipt_paths,
                "new_receipt_paths": new_receipt_paths,
                "scopes": scopes,
                "required_retired_paths": collected.all_required_paths,
            },
            "retirement_receipt_object_facts": object_facts,
            "protected_scm_context": {
                "protected_base_ref": PROTECTED_BASE_REF,
                "protected_base_commit_oid": resolved.protected,
                "protected_base_tree_oid": resolved.protected_tree,
                "evaluated_commit_oid": resolved.candidate,
                "evaluated_tree_oid": resolved.candidate_tree,
                "subject_commit_oid": resolved.subject,
                "subject_tree_oid": source.tree_for_commit(&resolved.subject)?,
                "scm_event_name": context.scm_event_name,
                "subject_relationship": if context.scm_event_name == "pull_request" { "pull-request-head" } else { "evaluated-self" },
                "protected_base_is_ancestor_of_evaluated": true,
                // Computed, never asserted. Relaxing the check to accept a first parent that
                // advanced past the recorded base means this can legitimately be false, and a
                // hardcoded `true` would make the receipt state something untrue.
                "protected_base_is_evaluated_first_parent": resolved.protected_base_is_evaluated_first_parent,
                "subject_is_evaluated_second_parent": context.scm_event_name == "pull_request",
                "predecessor_commit_oid": resolved.predecessor,
                "predecessor_tree_oid": resolved.predecessor_tree,
                "predecessor_commit_exists": true,
                "predecessor_tree_exists": true,
                "predecessor_commit_tree_bound": true,
                "predecessor_is_ancestor_of_protected_base": true,
                "protected_preparation_receipts": protected_preparations,
            },
            "retirement_control_plane_context": {
                "control_plane_path": CONTROL_PLANE_PATH,
                "receipt_root": RECEIPT_ROOT,
                "bootstrap": resolved.bootstrap,
                "protected_control_plane_blob_oid": resolved.protected_control_oid.clone().map_or(Value::Null, |oid| json!(oid)),
                "protected_control_plane_sha256": resolved.protected_control_sha,
                "protected_control_plane_byte_count": resolved.protected_control_bytes.as_ref().map(|bytes| bytes.len() as u64),
                "candidate_control_plane_blob_oid": resolved.candidate_control_oid,
                "candidate_control_plane_sha256": resolved.candidate_control_sha,
                "candidate_control_plane_byte_count": resolved.candidate_control_bytes.len() as u64,
                "control_plane_entries": resolved.control_plane_entries,
                "control_plane_entry_hashes": resolved.control_plane_entry_hashes,
                "protected_receipt_root_paths": resolved.protected_receipt_inventory,
                "candidate_receipt_root_paths": resolved.candidate_receipt_inventory,
                "unexpected_protected_receipt_paths": resolved.unexpected_protected_receipt_paths,
                "unexpected_candidate_receipt_paths": resolved.unexpected_candidate_receipt_paths,
            },
        }
    }))
}
