//! Durable, registry-only FixupTask v2 admission.
//!
//! This boundary deliberately owns no predecessor migration semantics. It binds
//! candidate registry bytes to protected SCM facts and validates the v2 lifecycle
//! contract with no ambient repository input beyond those two declared artifacts.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::{
    CollectError, Finding, collect_fixuptask_candidate_jsonl, evaluate_fixuptasks_v2_at,
    fixuptask_v2_digest,
};

pub const GATE_ID: &str = "cloud-ci-fixuptask-v2-admission";
pub const CANDIDATE_REGISTRY_PATH: &str = "registry/fixuptasks.jsonl";
pub const PROTECTED_FACTS_PATH: &str =
    "ci/facade/scm-facts-snapshot/scm-volatile-facts.generated.json";

/// Evaluates the durable candidate registry using only its protected snapshot.
pub fn evaluate_materialized_gate(root: &Path) -> Result<BTreeSet<Finding>, CollectError> {
    let candidate_bytes = fs::read(root.join(CANDIDATE_REGISTRY_PATH))
        .map_err(|error| CollectError::Io(format!("read {CANDIDATE_REGISTRY_PATH}: {error}")))?;
    let candidate = collect_fixuptask_candidate_jsonl(root, CANDIDATE_REGISTRY_PATH)?;
    let protected_text = fs::read_to_string(root.join(PROTECTED_FACTS_PATH))
        .map_err(|error| CollectError::Io(format!("read {PROTECTED_FACTS_PATH}: {error}")))?;
    let protected: Value =
        serde_json::from_str(&protected_text).map_err(|error| CollectError::Parse {
            line: 1,
            message: format!("parse {PROTECTED_FACTS_PATH}: {error}"),
        })?;
    Ok(evaluate_admission(&protected, &candidate, &candidate_bytes))
}

/// Pure durable admission projection. The digest binds exact candidate bytes;
/// validation itself remains independent from the materializer implementation.
pub fn evaluate_admission(
    protected_facts: &Value,
    candidate: &Value,
    candidate_bytes: &[u8],
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let Some(facts) = protected_facts
        .get("fixuptask_v2_durable")
        .and_then(Value::as_object)
    else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_missing",
            "<policy>",
            "protected SCM facts must contain fixuptask_v2_durable",
        ));
        return findings;
    };
    let allowed = BTreeSet::from([
        "merge_base",
        "merge_base_tree",
        "merge_base_rows",
        "candidate_registry_digest",
        "evaluation_time",
    ]);
    if facts.len() != allowed.len() || facts.keys().any(|key| !allowed.contains(key.as_str())) {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            "<policy>",
            "durable protected facts must contain only the registry admission contract",
        ));
        return findings;
    }
    let Some(rows) = facts.get("merge_base_rows").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            "<policy>",
            "durable protected facts merge_base_rows must be an array",
        ));
        return findings;
    };
    let valid_sha = |value: Option<&str>| {
        value.is_some_and(|value| {
            value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
    };
    let digest = facts
        .get("candidate_registry_digest")
        .and_then(Value::as_str);
    if !valid_sha(facts.get("merge_base").and_then(Value::as_str))
        || !valid_sha(facts.get("merge_base_tree").and_then(Value::as_str))
        || !digest.is_some_and(|value| {
            value.starts_with("sha256:")
                && value.len() == 71
                && value[7..]
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
    {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            "<policy>",
            "durable protected facts must bind merge-base identity and candidate registry digest",
        ));
        return findings;
    }
    if digest != Some(fixuptask_v2_digest(candidate_bytes).as_str()) {
        findings.insert(Finding::new(
            "fixuptask_v2_candidate_registry_digest_mismatch",
            "<policy>",
            "protected SCM facts do not bind the exact candidate registry bytes",
        ));
        return findings;
    }
    let Some(evaluation_time) = facts.get("evaluation_time").and_then(Value::as_str) else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            "<policy>",
            "durable protected facts evaluation_time must be a timestamp",
        ));
        return findings;
    };
    findings.extend(evaluate_fixuptasks_v2_at(
        &serde_json::json!({ "rows": rows }),
        candidate,
        evaluation_time,
    ));
    findings
}
