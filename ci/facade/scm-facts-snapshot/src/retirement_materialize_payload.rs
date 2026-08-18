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

