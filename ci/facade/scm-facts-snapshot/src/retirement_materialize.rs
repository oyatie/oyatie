//! Materialize history-only retirement facts from a Git object source.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::{
    CONTROL_PLANE_PATH, PROTECTED_BASE_REF, RECEIPT_ROOT, ReceiptStage, RetirementControlPlane,
    RetirementMaterializationContext, RetirementObjectSource, build_equivalence_index,
    canonical_value_sha256, classify_stage, closure_preparation_link, control_entry_value,
    coverage_scope, entries_by_path, expected_receipt_paths, find_linked_preparation, input_fact,
    parse_closed_json, receipt_baseline, receipt_for_stage, receipt_root_inventory,
    require_predecessor_baseline, require_regular, sha256_digest, validate_control_plane,
    validate_event_identity, validate_oid, validate_predecessor_inputs, validate_receipt_identity,
    validate_receipt_population, validate_selector_coverage,
};

pub(crate) fn materialize_history_only_retirement_facts(
    source: &impl RetirementObjectSource,
    context: &RetirementMaterializationContext<'_>,
) -> Result<Value, String> {
    if context.control_plane_path != CONTROL_PLANE_PATH {
        return Err(format!(
            "retirement control-plane path must be {CONTROL_PLANE_PATH}"
        ));
    }

    for (label, requested) in [
        (
            "requested protected base commit",
            context.protected_base_commit,
        ),
        ("requested evaluated commit", context.evaluated_commit),
        ("requested subject commit", context.subject_commit),
    ] {
        validate_oid(requested, label)?;
    }
    let candidate = source.resolve_commit(context.evaluated_commit)?;
    let head = source.resolve_commit("HEAD")?;
    if candidate != head {
        return Err(format!(
            "retirement candidate {candidate} is not exact HEAD {head}"
        ));
    }
    let protected = source.resolve_commit(context.protected_base_commit)?;
    let subject = source.resolve_commit(context.subject_commit)?;
    if protected != context.protected_base_commit
        || candidate != context.evaluated_commit
        || subject != context.subject_commit
    {
        return Err(
            "requested retirement event identity must equal resolved commit identity".to_owned(),
        );
    }
    validate_event_identity(
        source,
        context.scm_event_name,
        context.scm_event_ref,
        context.scm_event_base_ref,
        &protected,
        &candidate,
        &subject,
    )?;
    let first_parent = source.first_parent(&candidate)?;
    if protected != first_parent {
        return Err(format!(
            "retirement protected base {protected} is not candidate first parent {first_parent}"
        ));
    }
    if !source.is_ancestor(&protected, &candidate)? {
        return Err("retirement protected base is not an ancestor of candidate".to_owned());
    }
    let protected_tree = source.tree_for_commit(&protected)?;
    let candidate_tree = source.tree_for_commit(&candidate)?;
    let protected_entries = entries_by_path(source.tree_entries(&protected)?)?;
    let candidate_entries = entries_by_path(source.tree_entries(&candidate)?)?;

    let candidate_control_entry = candidate_entries
        .get(CONTROL_PLANE_PATH)
        .ok_or_else(|| "candidate retirement control plane is absent".to_owned())?;
    require_regular(
        candidate_control_entry,
        "candidate retirement control plane",
    )?;
    let candidate_control_bytes = source.read_blob(&candidate_control_entry.oid)?;
    let control_plane: RetirementControlPlane = parse_closed_json(&candidate_control_bytes)?;
    validate_control_plane(&control_plane)?;

    let predecessor = source.resolve_commit(&control_plane.predecessor_snapshot.commit_oid)?;
    if predecessor != control_plane.predecessor_snapshot.commit_oid {
        return Err("retirement predecessor commit is not canonical".to_owned());
    }
    let predecessor_tree = source.tree_for_commit(&predecessor)?;
    if predecessor_tree != control_plane.predecessor_snapshot.tree_oid {
        return Err("retirement predecessor commit/tree binding does not match Git".to_owned());
    }
    if !source.is_ancestor(&predecessor, &protected)? {
        return Err("retirement predecessor is not an ancestor of protected base".to_owned());
    }
    let predecessor_entries = entries_by_path(source.tree_entries(&predecessor)?)?;
    let predecessor_input_bodies =
        validate_predecessor_inputs(source, &control_plane, &predecessor_entries)?;
    validate_selector_coverage(&control_plane, &predecessor_entries, "predecessor")?;
    validate_selector_coverage(&control_plane, &protected_entries, "protected")?;
    validate_selector_coverage(&control_plane, &candidate_entries, "candidate")?;

    let protected_control = protected_entries.get(CONTROL_PLANE_PATH);
    let bootstrap = protected_control.is_none();
    let protected_control_bytes = match protected_control {
        None => None,
        Some(entry) => {
            require_regular(entry, "protected retirement control plane")?;
            let bytes = source.read_blob(&entry.oid)?;
            if bytes != candidate_control_bytes || entry.oid != candidate_control_entry.oid {
                return Err(
                    "protected and candidate retirement control planes are not immutable-identical"
                        .to_owned(),
                );
            }
            Some(bytes)
        }
    };

    let stages = control_plane
        .entries
        .iter()
        .map(|entry| classify_stage(entry, &protected_entries, &candidate_entries))
        .collect::<Result<Vec<_>, _>>()?;
    let active_receipt_population = validate_receipt_population(&stages)?;
    if bootstrap && active_receipt_population {
        return Err("retirement bootstrap may not add receipts".to_owned());
    }
    let equivalence_index = active_receipt_population
        .then(|| {
            build_equivalence_index(
                source,
                &predecessor_input_bodies,
                &protected_entries,
                &candidate_entries,
            )
        })
        .transpose()?
        .unwrap_or_default();

    let protected_receipt_inventory = receipt_root_inventory(&protected_entries);
    let candidate_receipt_inventory = receipt_root_inventory(&candidate_entries);
    let expected_receipt_paths = expected_receipt_paths(&control_plane);
    let unexpected_protected_receipt_paths = protected_receipt_inventory
        .difference(&expected_receipt_paths)
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_candidate_receipt_paths = candidate_receipt_inventory
        .difference(&expected_receipt_paths)
        .cloned()
        .collect::<Vec<_>>();

    let control_plane_entries = control_plane
        .entries
        .iter()
        .map(control_entry_value)
        .collect::<Result<Vec<_>, _>>()?;
    let control_plane_entry_hashes = control_plane
        .entries
        .iter()
        .zip(control_plane_entries.iter())
        .map(|(entry, value)| {
            Ok(json!({
                "scope_ref": entry.scope_ref,
                "sha256": canonical_value_sha256(value)?,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let protected_control_sha = protected_control_bytes.as_deref().map(sha256_digest);
    let candidate_control_sha = sha256_digest(&candidate_control_bytes);
    let mut receipts = Vec::new();
    let mut object_facts = Vec::new();
    let mut scopes = Vec::new();
    let mut all_required_paths = BTreeSet::new();
    let mut protected_preparations = Vec::new();

    if active_receipt_population {
        for (entry, stage) in control_plane.entries.iter().zip(stages.iter().copied()) {
            let (receipt_path, receipt_entry) =
                receipt_for_stage(stage, entry, &protected_entries, &candidate_entries)?;
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
                        &protected_entries,
                        &candidate_entries,
                        &equivalence_index,
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
                &predecessor,
                &predecessor_tree,
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

            let (preparation_path, preparation_blob, predecessor_context) = match stage {
                ReceiptStage::PreparedNew => (
                    Value::Null,
                    Value::Null,
                    json!({
                        "source": "control-plane-predecessor",
                        "commit_oid": predecessor,
                        "tree_oid": predecessor_tree,
                        "receipt_path": Value::Null,
                        "receipt_blob_oid": Value::Null,
                    }),
                ),
                ReceiptStage::ClosureNew => {
                    let (linked_path, linked_blob) = closure_preparation_link(&receipt)?;
                    if linked_path != entry.preparation_receipt_path {
                        return Err(format!(
                            "closure {receipt_path} links unexpected preparation path"
                        ));
                    }
                    let protected_preparation = protected_entries
                        .get(&entry.preparation_receipt_path)
                        .ok_or_else(|| {
                            "closure is missing protected preparation receipt".to_owned()
                        })?;
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
                        &predecessor,
                        &predecessor_tree,
                        &entry.preparation_receipt_path,
                    )?;
                    protected_preparations.push(json!({
                        "receipt_path": entry.preparation_receipt_path,
                        "receipt_blob_oid": protected_preparation.oid,
                        "baseline_commit_oid": commit,
                        "baseline_tree_oid": tree,
                    }));
                    (
                        json!(entry.preparation_receipt_path),
                        json!(protected_preparation.oid),
                        json!({
                            "source": "protected-preparation-receipt",
                            "commit_oid": commit,
                            "tree_oid": tree,
                            "receipt_path": entry.preparation_receipt_path,
                            "receipt_blob_oid": protected_preparation.oid,
                        }),
                    )
                }
                ReceiptStage::ClosedCarried => {
                    let (path, blob) = closure_preparation_link(&receipt)?;
                    if path != entry.preparation_receipt_path {
                        return Err(format!(
                            "closure {receipt_path} links unexpected preparation path"
                        ));
                    }
                    let (commit, tree) =
                        find_linked_preparation(source, &protected, path, blob, entry)?;
                    require_predecessor_baseline(
                        &commit,
                        &tree,
                        &predecessor,
                        &predecessor_tree,
                        path,
                    )?;
                    protected_preparations.push(json!({
                        "receipt_path": path,
                        "receipt_blob_oid": blob,
                        "baseline_commit_oid": commit,
                        "baseline_tree_oid": tree,
                    }));
                    (
                        json!(path),
                        json!(blob),
                        json!({
                            "source": "linked-preparation-history",
                            "commit_oid": commit,
                            "tree_oid": tree,
                            "receipt_path": path,
                            "receipt_blob_oid": blob,
                        }),
                    )
                }
                ReceiptStage::Dormant => unreachable!("dormant stage has no receipt facts"),
            };

            let protected_receipt = protected_entries.get(receipt_path);
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
                &predecessor_entries,
                &protected_entries,
                &candidate_entries,
            ));
        }
    }

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

    let protected_receipt_paths = protected_receipt_inventory
        .intersection(&expected_receipt_paths)
        .cloned()
        .collect::<Vec<_>>();
    let candidate_receipt_paths = candidate_receipt_inventory
        .intersection(&expected_receipt_paths)
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
                "required_retired_paths": all_required_paths,
            },
            "retirement_receipt_object_facts": object_facts,
            "protected_scm_context": {
                "protected_base_ref": PROTECTED_BASE_REF,
                "protected_base_commit_oid": protected,
                "protected_base_tree_oid": protected_tree,
                "evaluated_commit_oid": candidate,
                "evaluated_tree_oid": candidate_tree,
                "subject_commit_oid": subject,
                "subject_tree_oid": source.tree_for_commit(&subject)?,
                "scm_event_name": context.scm_event_name,
                "subject_relationship": if context.scm_event_name == "pull_request" { "pull-request-head" } else { "evaluated-self" },
                "protected_base_is_ancestor_of_evaluated": true,
                "protected_base_is_evaluated_first_parent": true,
                "subject_is_evaluated_second_parent": context.scm_event_name == "pull_request",
                "predecessor_commit_oid": predecessor,
                "predecessor_tree_oid": predecessor_tree,
                "predecessor_commit_exists": true,
                "predecessor_tree_exists": true,
                "predecessor_commit_tree_bound": true,
                "predecessor_is_ancestor_of_protected_base": true,
                "protected_preparation_receipts": protected_preparations,
            },
            "retirement_control_plane_context": {
                "control_plane_path": CONTROL_PLANE_PATH,
                "receipt_root": RECEIPT_ROOT,
                "bootstrap": bootstrap,
                "protected_control_plane_blob_oid": protected_control.map_or(Value::Null, |entry| json!(entry.oid)),
                "protected_control_plane_sha256": protected_control_sha,
                "protected_control_plane_byte_count": protected_control_bytes.as_ref().map(|bytes| bytes.len() as u64),
                "candidate_control_plane_blob_oid": candidate_control_entry.oid,
                "candidate_control_plane_sha256": candidate_control_sha,
                "candidate_control_plane_byte_count": candidate_control_bytes.len() as u64,
                "control_plane_entries": control_plane_entries,
                "control_plane_entry_hashes": control_plane_entry_hashes,
                "protected_receipt_root_paths": protected_receipt_inventory,
                "candidate_receipt_root_paths": candidate_receipt_inventory,
                "unexpected_protected_receipt_paths": unexpected_protected_receipt_paths,
                "unexpected_candidate_receipt_paths": unexpected_candidate_receipt_paths,
            },
        }
    }))
}

