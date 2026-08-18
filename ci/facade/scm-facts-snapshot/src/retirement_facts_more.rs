//! Receipt-link, hash, and tree-inventory helpers.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{
    ControlPlaneEntry, RECEIPT_ROOT, ReceiptStage, RetirementControlPlane, RetirementObjectSource,
    TreeEntry, parse_closed_json, required_value_string, validate_oid, validate_receipt_identity,
    validate_repo_path,
};

pub(crate) fn closure_preparation_link(receipt: &Value) -> Result<(&str, &str), String> {
    let preparation = receipt
        .get("protected_preparation")
        .and_then(Value::as_object)
        .ok_or_else(|| "closure receipt has no protected_preparation".to_owned())?;
    let path = required_value_string(
        preparation.get("receipt_path"),
        "protected_preparation.receipt_path",
    )?;
    let blob = required_value_string(
        preparation.get("receipt_blob_oid"),
        "protected_preparation.receipt_blob_oid",
    )?;
    validate_repo_path(path)?;
    validate_oid(blob, "protected preparation blob")?;
    Ok((path, blob))
}

pub(crate) fn receipt_baseline(receipt: &Value) -> Result<(String, String), String> {
    let baseline = receipt
        .get("baseline")
        .and_then(Value::as_object)
        .ok_or_else(|| "preparation receipt has no baseline".to_owned())?;
    let commit = required_value_string(baseline.get("commit_oid"), "baseline.commit_oid")?;
    let tree = required_value_string(baseline.get("tree_oid"), "baseline.tree_oid")?;
    validate_oid(commit, "receipt baseline commit")?;
    validate_oid(tree, "receipt baseline tree")?;
    Ok((commit.to_owned(), tree.to_owned()))
}

pub(crate) fn require_predecessor_baseline(
    commit_oid: &str,
    tree_oid: &str,
    predecessor_commit_oid: &str,
    predecessor_tree_oid: &str,
    receipt_path: &str,
) -> Result<(), String> {
    if commit_oid != predecessor_commit_oid || tree_oid != predecessor_tree_oid {
        return Err(format!(
            "receipt {receipt_path} baseline is not the immutable control-plane predecessor"
        ));
    }
    Ok(())
}

pub(crate) fn find_linked_preparation(
    source: &impl RetirementObjectSource,
    protected_commit: &str,
    path: &str,
    blob_oid: &str,
    control: &ControlPlaneEntry,
) -> Result<(String, String), String> {
    let blob_bytes = source.read_blob(blob_oid)?;
    let preparation: Value = parse_closed_json(&blob_bytes)?;
    validate_receipt_identity(ReceiptStage::PreparedNew, control, path, &preparation)?;
    let baseline = receipt_baseline(&preparation)?;
    for commit in source.commits_touching_path(protected_commit, path)? {
        let entries = entries_by_path(source.tree_entries(&commit)?)?;
        if entries
            .get(path)
            .is_some_and(|entry| entry.oid == blob_oid && entry.is_regular_blob())
        {
            return Ok(baseline);
        }
    }
    Err(format!(
        "linked preparation object {blob_oid} at {path} is not reachable in protected history"
    ))
}

pub(crate) fn control_entry_value(entry: &ControlPlaneEntry) -> Result<Value, String> {
    serde_json::to_value(entry)
        .map_err(|error| format!("serialize retirement control-plane entry: {error}"))
}

pub(crate) fn canonical_value_sha256(value: &Value) -> Result<String, String> {
    let bytes = semantic_canonical_json(value)?;
    Ok(sha256_digest(bytes.as_bytes()))
}

pub(crate) fn semantic_canonical_json(value: &Value) -> Result<String, String> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => Ok(value.to_string()),
        Value::String(_) => serde_json::to_string(value)
            .map_err(|error| format!("canonicalize retirement string: {error}")),
        Value::Array(values) => values
            .iter()
            .map(semantic_canonical_json)
            .collect::<Result<Vec<_>, _>>()
            .map(|values| format!("[{}]", values.join(","))),
        Value::Object(values) => {
            let ordered = values.iter().collect::<BTreeMap<_, _>>();
            let fields = ordered
                .into_iter()
                .map(|(key, value)| {
                    Ok(format!(
                        "{}:{}",
                        serde_json::to_string(key)
                            .map_err(|error| format!("canonicalize retirement key: {error}"))?,
                        semantic_canonical_json(value)?
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(format!("{{{}}}", fields.join(",")))
        }
    }
}

pub(crate) fn sha256_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

pub(crate) fn entries_by_path(
    entries: Vec<TreeEntry>,
) -> Result<BTreeMap<String, TreeEntry>, String> {
    let mut result = BTreeMap::new();
    for entry in entries {
        validate_repo_path(&entry.path)?;
        if result.insert(entry.path.clone(), entry).is_some() {
            return Err("Git tree contains a duplicate path".to_owned());
        }
    }
    Ok(result)
}

pub(crate) fn receipt_root_inventory(entries: &BTreeMap<String, TreeEntry>) -> BTreeSet<String> {
    let prefix = format!("{RECEIPT_ROOT}/");
    entries
        .keys()
        .filter(|path| path.starts_with(&prefix))
        .cloned()
        .collect()
}

pub(crate) fn expected_receipt_paths(control: &RetirementControlPlane) -> BTreeSet<String> {
    control
        .entries
        .iter()
        .flat_map(|entry| {
            [
                entry.preparation_receipt_path.clone(),
                entry.closure_receipt_path.clone(),
            ]
        })
        .collect()
}

pub(crate) fn require_regular(entry: &TreeEntry, label: &str) -> Result<(), String> {
    if entry.is_regular_blob() {
        Ok(())
    } else {
        Err(format!(
            "{label} {} must be exact 100644 blob, found {} {}",
            entry.path, entry.mode, entry.kind
        ))
    }
}
