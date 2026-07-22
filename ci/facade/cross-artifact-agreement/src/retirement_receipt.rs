//! Pure, declared-input validation for history-only retirement receipts.
//!
//! This module never reads Git, the filesystem, process state, or ambient time.
//! Its callers provide the receipt, declared SCM object facts, and declared coverage
//! facts.  That boundary keeps a receipt check hermetic and prevents a current epoch
//! from silently rebinding a carried protected receipt.
//!
//! This is a dormant foundation, not a live GATE-1 control: no receipt is promoted
//! and its claim ceiling is HOLD(Planning). Test fixtures and direct callers are data,
//! never admission authority. Activation is reserved for an atomic Git-boundary
//! materializer cutover that derives actual origin/dev and candidate object hashes,
//! absence/equivalence facts, and generated-face ownership.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::Finding;

pub const RETIREMENT_RECEIPT_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/history-only-retirement-receipt";
pub const RETIREMENT_RECEIPT_CODE: &str = "history_only_retirement_receipt_invalid";

const RECEIPT_FIELDS: &[&str] = &[
    "$schema",
    "artifact_id",
    "artifact_type",
    "status",
    "recorded_at",
    "scope_ref",
    "authority",
    "baseline",
    "retired_inputs",
    "provenance",
    "verification_contract",
    "effects",
];
const AUTHORITY_FIELDS: &[&str] = &[
    "decisions",
    "planning_state",
    "dispatch_authorized",
    "completion_claims_promoted",
];
const BASELINE_FIELDS: &[&str] = &["commit_oid", "tree_oid"];
const INPUT_FIELDS: &[&str] = &[
    "path",
    "predecessor_blob_oid",
    "sha256",
    "byte_count",
    "successor_refs",
    "disposition",
];
const PROVENANCE_FIELDS: &[&str] = &[
    "content_store",
    "readable_tracked_copy_retained",
    "readable_archive_directory_retained",
    "tombstone_content_retained",
    "receipt_reproduces_retired_content",
];
const VERIFICATION_FIELDS: &[&str] = &[
    "expected_absent_paths",
    "expected_tracked_readable_archive_directory_count",
    "required_gates",
];
const EFFECT_FIELDS: &[&str] = &[
    "repository_effect",
    "runtime_effect",
    "roadmap_effect",
    "planning_hold_effect",
];
const FACT_FIELDS: &[&str] = &[
    "artifact_id",
    "receipt_path",
    "protected_base_ref",
    "receipt_epoch",
    "scope_ref",
    "scope_type",
    "baseline_commit_oid",
    "baseline_tree_oid",
    "baseline_is_ancestor_of_current_epoch",
    "protected_receipt_blob_oid",
    "candidate_receipt_blob_oid",
    "protected_registry_binding_preserved",
    "retired_inputs",
];
const FACT_INPUT_FIELDS: &[&str] = &[
    "path",
    "predecessor_blob_oid",
    "sha256",
    "byte_count",
    "candidate_path_exists",
    "candidate_new_equivalent_paths",
];
const COVERAGE_FIELDS: &[&str] = &[
    "protected_base_ref",
    "current_epoch_commit_oid",
    "current_epoch_tree_oid",
    "protected_receipt_paths",
    "candidate_receipt_paths",
    "carried_receipt_paths",
    "new_receipt_paths",
    "scopes",
    "required_retired_paths",
];
const RECEIPT_CORPUS_FIELDS: &[&str] = &["receipts", "scm_facts"];
const RECEIPT_RECORD_FIELDS: &[&str] = &["receipt_path", "receipt"];
const SCOPE_FIELDS: &[&str] = &[
    "scope_ref",
    "scope_type",
    "selectors",
    "required_retired_paths",
];
const SELECTOR_FIELDS: &[&str] = &[
    "selector_type",
    "selector",
    "protected_paths",
    "candidate_paths",
    "removed_paths",
    "surviving_paths",
    "candidate_only_paths",
    "external_assertion",
];
const FORBIDDEN_CONTENT_FIELDS: &[&str] = &[
    "content",
    "contents",
    "data",
    "source_content",
    "retired_content",
    "retired_contents",
    "body",
];
const REQUIRED_GATES: &[&str] = &[
    RETIREMENT_RECEIPT_VALIDATOR,
    "cloud-ci-staleness-reaper/readable_archive_path",
    "oya-check-doc-axis/ReadableArchiveDirectory",
];

/// Validate one receipt against only declared facts. `scm_facts` must be the
/// materialized object-fact face, never a live SCM query.
pub fn evaluate_history_only_retirement_receipt(
    receipt_path: &str,
    receipt: &Value,
    scm_facts: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    reject_content(receipt, "receipt", &mut findings);
    closed_object(receipt, RECEIPT_FIELDS, "receipt", &mut findings);
    exact(
        receipt,
        "$schema",
        "https://json-schema.org/draft/2020-12/schema",
        &mut findings,
    );
    exact(
        receipt,
        "artifact_type",
        "migration-closure-receipt",
        &mut findings,
    );
    exact(
        receipt,
        "status",
        "candidate-recorded-nonauthoritative",
        &mut findings,
    );
    required_identifier(receipt, "artifact_id", &mut findings);
    required_string(receipt, "recorded_at", &mut findings);
    let scope = required_string(receipt, "scope_ref", &mut findings);
    if scope
        .as_deref()
        .is_some_and(|scope| scope_type(scope).is_none())
    {
        fail(&mut findings, "scope_ref");
    }

    let authority = child(receipt, "authority");
    closed_object(authority, AUTHORITY_FIELDS, "authority", &mut findings);
    if !string_set(
        authority.get("decisions"),
        "authority.decisions",
        &mut findings,
    )
    .is_empty()
    {
        // Exact decision identities are intentionally data, but they must retain ADR shape.
        for decision in string_set(
            authority.get("decisions"),
            "authority.decisions",
            &mut findings,
        ) {
            if !valid_adr(&decision) {
                fail(&mut findings, "authority.decisions");
            }
        }
    } else {
        fail(&mut findings, "authority.decisions.empty");
    }
    exact(authority, "planning_state", "HOLD(Planning)", &mut findings);
    if authority
        .get("dispatch_authorized")
        .and_then(Value::as_bool)
        != Some(false)
    {
        fail(&mut findings, "authority.dispatch_authorized");
    }
    if authority
        .get("completion_claims_promoted")
        .and_then(Value::as_u64)
        != Some(0)
    {
        fail(&mut findings, "authority.completion_claims_promoted");
    }

    let baseline = child(receipt, "baseline");
    closed_object(baseline, BASELINE_FIELDS, "baseline", &mut findings);
    let commit = oid(baseline, "commit_oid", "baseline.commit_oid", &mut findings);
    let tree = oid(baseline, "tree_oid", "baseline.tree_oid", &mut findings);

    let provenance = child(receipt, "provenance");
    closed_object(provenance, PROVENANCE_FIELDS, "provenance", &mut findings);
    exact(
        provenance,
        "content_store",
        "authorized Git object history only",
        &mut findings,
    );
    for key in [
        "readable_tracked_copy_retained",
        "readable_archive_directory_retained",
        "tombstone_content_retained",
        "receipt_reproduces_retired_content",
    ] {
        if provenance.get(key).and_then(Value::as_bool) != Some(false) {
            fail(&mut findings, &format!("provenance.{key}"));
        }
    }
    let verification = child(receipt, "verification_contract");
    closed_object(
        verification,
        VERIFICATION_FIELDS,
        "verification_contract",
        &mut findings,
    );
    if path_set(
        verification.get("expected_absent_paths"),
        "verification_contract.expected_absent_paths",
        &mut findings,
    ) != input_paths(receipt, &mut findings)
    {
        fail(&mut findings, "verification_contract.expected_absent_paths");
    }
    if verification
        .get("expected_tracked_readable_archive_directory_count")
        .and_then(Value::as_u64)
        != Some(0)
    {
        fail(
            &mut findings,
            "verification_contract.expected_tracked_readable_archive_directory_count",
        );
    }
    if string_set(
        verification.get("required_gates"),
        "verification_contract.required_gates",
        &mut findings,
    ) != REQUIRED_GATES
        .iter()
        .map(|value| (*value).to_owned())
        .collect()
    {
        fail(
            &mut findings,
            "verification_contract.required_gates.exact_set",
        );
    }
    let effects = child(receipt, "effects");
    closed_object(effects, EFFECT_FIELDS, "effects", &mut findings);
    for key in EFFECT_FIELDS {
        if effects
            .get(*key)
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            fail(&mut findings, &format!("effects.{key}"));
        }
    }

    let facts = scm_facts
        .get("retirement_receipt_object_facts")
        .and_then(Value::as_array);
    let matching_facts = facts
        .map(|facts| {
            facts
                .iter()
                .filter(|fact| {
                    fact.get("receipt_path").and_then(Value::as_str) == Some(receipt_path)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if matching_facts.len() != 1 {
        fail(&mut findings, "object_fact_exactly_once");
    }
    let matching = matching_facts.first().copied();
    let Some(fact) = matching else {
        fail(&mut findings, "object_fact_missing");
        return findings;
    };
    closed_object(fact, FACT_FIELDS, "object_fact", &mut findings);
    if fact.get("protected_base_ref").and_then(Value::as_str) != Some("origin/dev") {
        fail(&mut findings, "object_fact.protected_base_ref");
    }
    for key in ["protected_receipt_blob_oid", "candidate_receipt_blob_oid"] {
        let _ = oid(fact, key, &format!("object_fact.{key}"), &mut findings);
    }
    if fact.get("artifact_id") != receipt.get("artifact_id") {
        fail(&mut findings, "object_fact.artifact_id");
    }
    if fact.get("scope_ref") != receipt.get("scope_ref")
        || fact.get("scope_type").and_then(Value::as_str) != scope.as_deref().and_then(scope_type)
    {
        fail(&mut findings, "object_fact.scope");
    }
    if fact.get("baseline_commit_oid").and_then(Value::as_str) != commit.as_deref()
        || fact.get("baseline_tree_oid").and_then(Value::as_str) != tree.as_deref()
    {
        fail(&mut findings, "object_fact.baseline");
    }
    if fact
        .get("baseline_is_ancestor_of_current_epoch")
        .and_then(Value::as_bool)
        != Some(true)
        || fact
            .get("protected_registry_binding_preserved")
            .and_then(Value::as_bool)
            != Some(true)
    {
        fail(&mut findings, "object_fact.protected_binding");
    }
    if fact
        .get("protected_receipt_blob_oid")
        .and_then(Value::as_str)
        != fact
            .get("candidate_receipt_blob_oid")
            .and_then(Value::as_str)
    {
        fail(&mut findings, "object_fact.receipt_blob_binding");
    }
    validate_inputs(receipt, fact, &mut findings);
    findings
}

/// Evaluate the path-bound receipt records carried by the fixture/gate corpus.
/// The path is a declared input, never inferred from an ambient object-fact row.
pub fn evaluate_history_only_retirement_receipts(corpus: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    closed_object(
        corpus,
        RECEIPT_CORPUS_FIELDS,
        "retirement_receipts",
        &mut findings,
    );
    let scm_facts = child(corpus, "scm_facts");
    let Some(records) = corpus.get("receipts").and_then(Value::as_array) else {
        fail(&mut findings, "retirement_receipts.receipts");
        return findings;
    };
    if records.is_empty() {
        fail(&mut findings, "retirement_receipts.receipts.empty");
    }
    let mut paths = BTreeSet::new();
    let mut receipts = Vec::with_capacity(records.len());
    for (index, record) in records.iter().enumerate() {
        closed_object(
            record,
            RECEIPT_RECORD_FIELDS,
            &format!("retirement_receipts[{index}]"),
            &mut findings,
        );
        let Some(path) = record
            .get("receipt_path")
            .and_then(Value::as_str)
            .filter(|path| repo_path(path))
        else {
            fail(
                &mut findings,
                &format!("retirement_receipts[{index}].receipt_path"),
            );
            continue;
        };
        if !paths.insert(path.to_owned()) {
            fail(&mut findings, "retirement_receipts.receipt_path_duplicate");
        }
        let receipt = child(record, "receipt");
        findings.extend(evaluate_history_only_retirement_receipt(
            path, receipt, scm_facts,
        ));
        receipts.push(receipt.clone());
    }
    findings.extend(evaluate_history_only_retirement_receipt_coverage(
        &receipts, scm_facts,
    ));
    findings
}

/// Validate epoch partitioning and exact per-scope retirement coverage. Only new
/// receipts bind the current epoch; carried receipts remain bound to their own
/// protected baseline and immutable blob/registry facts.
pub fn evaluate_history_only_retirement_receipt_coverage(
    receipts: &[Value],
    facts: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let coverage = child(facts, "retirement_receipt_coverage");
    closed_object(
        coverage,
        COVERAGE_FIELDS,
        "retirement_receipt_coverage",
        &mut findings,
    );
    let protected = path_set(
        coverage.get("protected_receipt_paths"),
        "protected_receipt_paths",
        &mut findings,
    );
    let candidate = path_set(
        coverage.get("candidate_receipt_paths"),
        "candidate_receipt_paths",
        &mut findings,
    );
    let carried = path_set(
        coverage.get("carried_receipt_paths"),
        "carried_receipt_paths",
        &mut findings,
    );
    let new = path_set(
        coverage.get("new_receipt_paths"),
        "new_receipt_paths",
        &mut findings,
    );
    if protected.is_empty()
        || candidate.is_empty()
        || carried.is_empty()
        || new.is_empty()
        || protected != carried
        || !carried.is_subset(&candidate)
        || !new.is_disjoint(&carried)
        || candidate != carried.union(&new).cloned().collect()
        || protected.intersection(&new).next().is_some()
    {
        fail(&mut findings, "receipt_path_partition");
    }
    let current_commit = oid(
        coverage,
        "current_epoch_commit_oid",
        "current_epoch_commit_oid",
        &mut findings,
    );
    let current_tree = oid(
        coverage,
        "current_epoch_tree_oid",
        "current_epoch_tree_oid",
        &mut findings,
    );
    let required = validate_scopes(coverage, &mut findings);
    if required.is_empty() {
        fail(&mut findings, "retirement_receipt_coverage.scopes.empty");
    }
    let object_facts = facts
        .get("retirement_receipt_object_facts")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    if object_facts.is_empty() {
        fail(&mut findings, "retirement_receipt_object_facts.empty");
    }
    let mut fact_artifacts = BTreeSet::new();
    for (index, fact) in object_facts.iter().enumerate() {
        closed_object(
            fact,
            FACT_FIELDS,
            &format!("retirement_receipt_object_facts[{index}]"),
            &mut findings,
        );
        let Some(artifact_id) = fact.get("artifact_id").and_then(Value::as_str) else {
            fail(&mut findings, "object_fact_artifact_id");
            continue;
        };
        if !fact_artifacts.insert(artifact_id.to_owned()) {
            fail(&mut findings, "object_fact_artifact_id_duplicate");
        }
    }
    let by_path: BTreeMap<_, _> = object_facts
        .iter()
        .filter_map(|fact| {
            fact.get("receipt_path")
                .and_then(Value::as_str)
                .map(|path| (path, fact))
        })
        .collect();
    if by_path.len() != object_facts.len()
        || by_path
            .keys()
            .map(|path| (*path).to_owned())
            .collect::<BTreeSet<_>>()
            != candidate
    {
        fail(&mut findings, "object_fact_path_set");
    }
    let mut carried_covered: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut new_covered: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let receipt_artifacts = receipts
        .iter()
        .filter_map(|receipt| receipt.get("artifact_id").and_then(Value::as_str))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if receipt_artifacts.len() != receipts.len() || receipt_artifacts != fact_artifacts {
        fail(&mut findings, "receipt_object_fact_artifact_set");
    }
    for receipt in receipts {
        let id = receipt.get("artifact_id").and_then(Value::as_str);
        let fact = id.and_then(|id| {
            object_facts
                .iter()
                .find(|fact| fact.get("artifact_id").and_then(Value::as_str) == Some(id))
        });
        let Some(fact) = fact else {
            fail(&mut findings, "receipt_object_fact_missing");
            continue;
        };
        let path = fact.get("receipt_path").and_then(Value::as_str);
        let epoch = fact.get("receipt_epoch").and_then(Value::as_str);
        match (path, epoch) {
            (Some(path), Some("carried")) if carried.contains(path) => {
                if fact
                    .get("protected_registry_binding_preserved")
                    .and_then(Value::as_bool)
                    != Some(true)
                    || fact.get("protected_receipt_blob_oid")
                        != fact.get("candidate_receipt_blob_oid")
                {
                    fail(&mut findings, "carried_protected_binding");
                }
                if fact.get("protected_base_ref") != coverage.get("protected_base_ref")
                    || fact.get("scope_ref") != receipt.get("scope_ref")
                    || fact.get("scope_type").and_then(Value::as_str)
                        != receipt
                            .get("scope_ref")
                            .and_then(Value::as_str)
                            .and_then(scope_type)
                    || fact.get("baseline_commit_oid") != receipt.pointer("/baseline/commit_oid")
                    || fact.get("baseline_tree_oid") != receipt.pointer("/baseline/tree_oid")
                    || (fact.get("baseline_commit_oid") == coverage.get("current_epoch_commit_oid")
                        && fact.get("baseline_tree_oid") == coverage.get("current_epoch_tree_oid"))
                {
                    fail(&mut findings, "carried_immutable_baseline_binding");
                }
                let scope = receipt
                    .get("scope_ref")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                carried_covered
                    .entry(scope.to_owned())
                    .or_default()
                    .extend(input_paths(receipt, &mut findings));
            }
            (Some(path), Some("new")) if new.contains(path) => {
                if receipt
                    .pointer("/baseline/commit_oid")
                    .and_then(Value::as_str)
                    != current_commit.as_deref()
                    || receipt
                        .pointer("/baseline/tree_oid")
                        .and_then(Value::as_str)
                        != current_tree.as_deref()
                {
                    fail(&mut findings, "new_current_epoch_binding");
                }
                let scope = receipt
                    .get("scope_ref")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                new_covered
                    .entry(scope.to_owned())
                    .or_default()
                    .extend(input_paths(receipt, &mut findings));
            }
            _ => fail(&mut findings, "receipt_epoch_classification"),
        }
    }
    for (scope, expected) in required {
        let carried_paths = carried_covered.get(&scope).cloned().unwrap_or_default();
        if !carried_paths.is_empty() && carried_paths != expected.0 {
            fail(&mut findings, &format!("carried_scope_coverage.{scope}"));
        }
        let new_paths = new_covered.get(&scope).cloned().unwrap_or_default();
        if (carried_paths.is_empty() || !new_paths.is_empty()) && new_paths != expected.1 {
            fail(&mut findings, &format!("new_scope_coverage.{scope}"));
        }
    }
    findings
}

fn validate_inputs(receipt: &Value, fact: &Value, findings: &mut BTreeSet<Finding>) {
    let Some(inputs) = receipt.get("retired_inputs").and_then(Value::as_array) else {
        fail(findings, "retired_inputs");
        return;
    };
    let Some(fact_inputs) = fact.get("retired_inputs").and_then(Value::as_array) else {
        fail(findings, "object_fact.retired_inputs");
        return;
    };
    if inputs.is_empty() || inputs.len() > 256 || inputs.len() != fact_inputs.len() {
        fail(findings, "retired_inputs");
    }
    let mut paths = BTreeSet::new();
    for (index, input) in inputs.iter().enumerate() {
        closed_object(
            input,
            INPUT_FIELDS,
            &format!("retired_inputs[{index}]"),
            findings,
        );
        let path = input
            .get("path")
            .and_then(Value::as_str)
            .filter(|path| repo_path(path));
        if path.is_none() || !paths.insert(path.unwrap_or_default().to_owned()) {
            fail(findings, &format!("retired_inputs[{index}].path"));
        }
        if input.get("disposition").and_then(Value::as_str) != Some("retired-git-history-only") {
            fail(findings, &format!("retired_inputs[{index}].disposition"));
        }
        if oid(
            input,
            "predecessor_blob_oid",
            &format!("retired_inputs[{index}].predecessor_blob_oid"),
            findings,
        )
        .is_none()
            || !sha256(input.get("sha256").and_then(Value::as_str))
            || input.get("byte_count").and_then(Value::as_u64).is_none()
        {
            fail(findings, &format!("retired_inputs[{index}]"));
        }
        if !string_set(
            input.get("successor_refs"),
            &format!("retired_inputs[{index}].successor_refs"),
            findings,
        )
        .iter()
        .all(|value| valid_reference(value))
        {
            fail(findings, &format!("retired_inputs[{index}].successor_refs"));
        }
        let Some(fact_input) = fact_inputs.get(index) else {
            continue;
        };
        closed_object(
            fact_input,
            FACT_INPUT_FIELDS,
            &format!("object_fact.retired_inputs[{index}]"),
            findings,
        );
        for key in ["path", "predecessor_blob_oid", "sha256", "byte_count"] {
            if input.get(key) != fact_input.get(key) {
                fail(findings, &format!("retired_inputs[{index}].{key}_mismatch"));
            }
        }
        if fact_input
            .get("candidate_path_exists")
            .and_then(Value::as_bool)
            != Some(false)
            || !path_set(
                fact_input.get("candidate_new_equivalent_paths"),
                "candidate_new_equivalent_paths",
                findings,
            )
            .is_empty()
        {
            fail(
                findings,
                &format!("retired_inputs[{index}].candidate_readable_copy"),
            );
        }
    }
}

fn validate_scopes(
    coverage: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> {
    let mut result = BTreeMap::new();
    let mut union = BTreeSet::new();
    let Some(scopes) = coverage.get("scopes").and_then(Value::as_array) else {
        fail(findings, "scopes");
        return result;
    };
    for (i, scope) in scopes.iter().enumerate() {
        closed_object(scope, SCOPE_FIELDS, &format!("scopes[{i}]"), findings);
        let reference = scope.get("scope_ref").and_then(Value::as_str).unwrap_or("");
        if scope.get("scope_type").and_then(Value::as_str) != scope_type(reference)
            || result.contains_key(reference)
        {
            fail(findings, &format!("scopes[{i}].scope"));
        }
        let mut protected = BTreeSet::new();
        let mut removed = BTreeSet::new();
        let selectors = scope
            .get("selectors")
            .and_then(Value::as_array)
            .map_or(&[][..], Vec::as_slice);
        for (j, selector) in selectors.iter().enumerate() {
            closed_object(
                selector,
                SELECTOR_FIELDS,
                &format!("scopes[{i}].selectors[{j}]"),
                findings,
            );
            let kind = selector.get("selector_type").and_then(Value::as_str);
            let selector_value = selector
                .get("selector")
                .and_then(Value::as_str)
                .unwrap_or("");
            let p = path_set(selector.get("protected_paths"), "protected_paths", findings);
            let c = path_set(selector.get("candidate_paths"), "candidate_paths", findings);
            let r = path_set(selector.get("removed_paths"), "removed_paths", findings);
            let s = path_set(selector.get("surviving_paths"), "surviving_paths", findings);
            let n = path_set(
                selector.get("candidate_only_paths"),
                "candidate_only_paths",
                findings,
            );
            if kind == Some("external") {
                if selector.get("external_assertion").and_then(Value::as_str)
                    != Some("outside-repository-authority-not-inspected")
                    || !p.is_empty()
                    || !c.is_empty()
                    || !r.is_empty()
                    || !s.is_empty()
                    || !n.is_empty()
                {
                    fail(findings, &format!("scopes[{i}].selectors[{j}].external"));
                }
                continue;
            }
            if !matches!(kind, Some("exact" | "glob"))
                || !selector_valid(kind, selector_value)
                || selector.get("external_assertion").and_then(Value::as_str)
                    != Some("not-applicable")
                || r != p.difference(&c).cloned().collect()
                || s != p.intersection(&c).cloned().collect()
                || n != c.difference(&p).cloned().collect()
                || !p
                    .union(&c)
                    .all(|path| selector_matches(kind, selector_value, path))
                || !s.is_empty()
                || !n.is_empty()
            {
                fail(findings, &format!("scopes[{i}].selectors[{j}]"));
            }
            removed.extend(r);
            protected.extend(p);
        }
        let declared = path_set(
            scope.get("required_retired_paths"),
            "required_retired_paths",
            findings,
        );
        if declared != removed {
            fail(findings, &format!("scopes[{i}].required_retired_paths"));
        }
        union.extend(declared);
        result.insert(reference.to_owned(), (protected, removed));
    }
    if path_set(
        coverage.get("required_retired_paths"),
        "required_retired_paths",
        findings,
    ) != union
    {
        fail(findings, "required_retired_paths");
    }
    result
}

fn child<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).unwrap_or(&Value::Null)
}
fn fail(findings: &mut BTreeSet<Finding>, key: &str) {
    findings.insert(Finding::new(RETIREMENT_RECEIPT_CODE, key));
}
fn closed_object(value: &Value, allowed: &[&str], prefix: &str, findings: &mut BTreeSet<Finding>) {
    match value.as_object() {
        Some(object) => {
            for key in object.keys() {
                if !allowed.contains(&key.as_str()) {
                    fail(findings, &format!("{prefix}.unknown_field.{key}"));
                }
            }
        }
        None => fail(findings, prefix),
    }
}
fn exact(value: &Value, key: &str, expected: &str, findings: &mut BTreeSet<Finding>) {
    if value.get(key).and_then(Value::as_str) != Some(expected) {
        fail(findings, key);
    }
}
fn required_string(value: &Value, key: &str, findings: &mut BTreeSet<Finding>) -> Option<String> {
    let result = value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    if result.is_none() {
        fail(findings, key);
    }
    result
}
fn required_identifier(value: &Value, key: &str, findings: &mut BTreeSet<Finding>) {
    if !value.get(key).and_then(Value::as_str).is_some_and(|value| {
        value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    }) {
        fail(findings, key);
    }
}
fn oid(
    value: &Value,
    key: &str,
    finding: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<String> {
    let result = value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .map(str::to_owned);
    if result.is_none() {
        fail(findings, finding);
    }
    result
}
fn sha256(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        value.len() == 71
            && value.starts_with("sha256:")
            && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}
fn valid_adr(value: &str) -> bool {
    value.len() == 8
        && value.starts_with("ADR-")
        && value[4..].bytes().all(|byte| byte.is_ascii_digit())
}
fn valid_reference(value: &str) -> bool {
    (valid_adr(value) || (value.starts_with('/') && value.len() <= 128)) && value.len() <= 128
}
fn scope_type(scope: &str) -> Option<&'static str> {
    match scope {
        "artifact:masterplan" => Some("masterplan-retired-surfaces"),
        "ADR-0388" => Some("transient-ideas"),
        _ => None,
    }
}
fn repo_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains("..")
        && !path.contains("//")
        && !path.starts_with("./")
}
fn path_set(
    value: Option<&Value>,
    finding: &str,
    findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    let Some(values) = value.and_then(Value::as_array) else {
        fail(findings, finding);
        return BTreeSet::new();
    };
    let mut result = BTreeSet::new();
    for value in values {
        match value.as_str().filter(|path| repo_path(path)) {
            Some(path) if result.insert(path.to_owned()) => (),
            _ => fail(findings, finding),
        }
    }
    result
}
fn string_set(
    value: Option<&Value>,
    finding: &str,
    findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    let Some(values) = value.and_then(Value::as_array) else {
        fail(findings, finding);
        return BTreeSet::new();
    };
    let mut result = BTreeSet::new();
    for value in values {
        match value.as_str().filter(|value| !value.is_empty()) {
            Some(value) if result.insert(value.to_owned()) => (),
            _ => fail(findings, finding),
        }
    }
    result
}
fn input_paths(receipt: &Value, findings: &mut BTreeSet<Finding>) -> BTreeSet<String> {
    receipt
        .get("retired_inputs")
        .and_then(Value::as_array)
        .map(|inputs| {
            inputs
                .iter()
                .filter_map(|input| input.get("path").and_then(Value::as_str))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_else(|| {
            fail(findings, "retired_inputs");
            BTreeSet::new()
        })
}
fn selector_valid(kind: Option<&str>, selector: &str) -> bool {
    match kind {
        Some("exact") => repo_path(selector),
        Some("glob") => selector.strip_suffix("/**").is_some_and(repo_path),
        _ => false,
    }
}
fn selector_matches(kind: Option<&str>, selector: &str, path: &str) -> bool {
    match kind {
        Some("exact") => path == selector,
        Some("glob") => selector.strip_suffix("/**").is_some_and(|prefix| {
            path == prefix
                || path
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with('/'))
        }),
        _ => false,
    }
}
fn reject_content(value: &Value, path: &str, findings: &mut BTreeSet<Finding>) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let child_path = format!("{path}.{key}");
                if FORBIDDEN_CONTENT_FIELDS.contains(&key.as_str()) {
                    fail(findings, &format!("{child_path}.content_embedding"));
                }
                reject_content(child, &child_path, findings);
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                reject_content(child, &format!("{path}[{index}]"), findings);
            }
        }
        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const OLD: &str = "1111111111111111111111111111111111111111";
    const OLD_TREE: &str = "2222222222222222222222222222222222222222";
    const NEW: &str = "3333333333333333333333333333333333333333";
    const NEW_TREE: &str = "4444444444444444444444444444444444444444";
    const BLOB: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    fn receipt(id: &str, scope: &str, commit: &str, tree: &str, path: &str) -> Value {
        json!({"$schema":"https://json-schema.org/draft/2020-12/schema","artifact_id":id,"artifact_type":"migration-closure-receipt","status":"candidate-recorded-nonauthoritative","recorded_at":"2026-07-22","scope_ref":scope,"authority":{"decisions":["ADR-0388"],"planning_state":"HOLD(Planning)","dispatch_authorized":false,"completion_claims_promoted":0},"baseline":{"commit_oid":commit,"tree_oid":tree},"retired_inputs":[{"path":path,"predecessor_blob_oid":BLOB,"sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","byte_count":1,"successor_refs":["ADR-0388"],"disposition":"retired-git-history-only"}],"provenance":{"content_store":"authorized Git object history only","readable_tracked_copy_retained":false,"readable_archive_directory_retained":false,"tombstone_content_retained":false,"receipt_reproduces_retired_content":false},"verification_contract":{"expected_absent_paths":[path],"expected_tracked_readable_archive_directory_count":0,"required_gates":REQUIRED_GATES},"effects":{"repository_effect":"history-only","runtime_effect":"none","roadmap_effect":"none","planning_hold_effect":"HOLD(Planning)"}})
    }
    fn fact(
        id: &str,
        receipt_path: &str,
        scope: &str,
        epoch: &str,
        commit: &str,
        tree: &str,
    ) -> Value {
        json!({"artifact_id":id,"receipt_path":receipt_path,"protected_base_ref":"origin/dev","receipt_epoch":epoch,"scope_ref":scope,"scope_type":scope_type(scope).unwrap(),"baseline_commit_oid":commit,"baseline_tree_oid":tree,"baseline_is_ancestor_of_current_epoch":true,"protected_receipt_blob_oid":BLOB,"candidate_receipt_blob_oid":BLOB,"protected_registry_binding_preserved":true,"retired_inputs":[{"path":"docs/old.md","predecessor_blob_oid":BLOB,"sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","byte_count":1,"candidate_path_exists":false,"candidate_new_equivalent_paths":[]} ]})
    }
    #[test]
    fn carried_protected_idea_and_new_repository_authority_receipt_are_independently_admitted() {
        let idea_path = "evidence/ideas-receipt.json";
        let repo_path = "evidence/repository-receipt.json";
        let idea = receipt("idea-receipt", "ADR-0388", OLD, OLD_TREE, "docs/old.md");
        let repo = receipt(
            "repository-receipt",
            "artifact:masterplan",
            NEW,
            NEW_TREE,
            "docs/old.md",
        );
        let selector = json!({"selector_type":"exact","selector":"docs/old.md","protected_paths":["docs/old.md"],"candidate_paths":[],"removed_paths":["docs/old.md"],"surviving_paths":[],"candidate_only_paths":[],"external_assertion":"not-applicable"});
        let facts = json!({"retirement_receipt_coverage":{"protected_base_ref":"origin/dev","current_epoch_commit_oid":NEW,"current_epoch_tree_oid":NEW_TREE,"protected_receipt_paths":[idea_path],"candidate_receipt_paths":[idea_path,repo_path],"carried_receipt_paths":[idea_path],"new_receipt_paths":[repo_path],"scopes":[{"scope_ref":"ADR-0388","scope_type":"transient-ideas","selectors":[selector.clone()],"required_retired_paths":["docs/old.md"]},{"scope_ref":"artifact:masterplan","scope_type":"masterplan-retired-surfaces","selectors":[selector],"required_retired_paths":["docs/old.md"]}],"required_retired_paths":["docs/old.md"]},"retirement_receipt_object_facts":[fact("idea-receipt",idea_path,"ADR-0388","carried",OLD,OLD_TREE),fact("repository-receipt",repo_path,"artifact:masterplan","new",NEW,NEW_TREE)]});
        let findings = evaluate_history_only_retirement_receipt_coverage(
            &[idea.clone(), repo.clone()],
            &facts,
        );
        assert!(findings.is_empty(), "{findings:?}");

        let corpus = json!({"receipts":[
            {"receipt_path":idea_path,"receipt":idea},
            {"receipt_path":repo_path,"receipt":repo}
        ],"scm_facts":facts});
        assert!(evaluate_history_only_retirement_receipts(&corpus).is_empty());

        let decisions = json!([{"id":"ADR-0388","status":"Accepted","in_spec":true,"in_masterplan":true,"in_roadmap":true,"supersedes":[],"superseded_by":[]}]);
        let baseline_crosswalk = json!({"decisions": decisions});
        let ordinary_crosswalk = json!({
            "decisions": decisions,
            "history_only_retirement_receipts": corpus
        });
        assert_eq!(
            crate::evaluate(&ordinary_crosswalk),
            crate::evaluate(&baseline_crosswalk),
            "an undeclared retirement field must not influence ordinary crosswalk evaluation"
        );

        let mut rebound = corpus;
        rebound["scm_facts"]["retirement_receipt_object_facts"][0]["baseline_commit_oid"] =
            json!(NEW);
        rebound["scm_facts"]["retirement_receipt_object_facts"][0]["baseline_tree_oid"] =
            json!(NEW_TREE);
        assert!(
            evaluate_history_only_retirement_receipts(&rebound)
                .iter()
                .any(|finding| finding.code == RETIREMENT_RECEIPT_CODE)
        );
    }
    #[test]
    fn content_and_candidate_equivalent_copies_fail_closed() {
        let mut receipt = receipt("idea-receipt", "ADR-0388", OLD, OLD_TREE, "docs/old.md");
        receipt["retired_inputs"][0]["content"] = json!("forbidden");
        let mut facts = json!({"retirement_receipt_object_facts":[fact("idea-receipt","evidence/ideas-receipt.json","ADR-0388","new",OLD,OLD_TREE)]});
        facts["retirement_receipt_object_facts"][0]["retired_inputs"][0]["candidate_new_equivalent_paths"] =
            json!(["docs/copy.md"]);
        assert!(
            !evaluate_history_only_retirement_receipt(
                "evidence/ideas-receipt.json",
                &receipt,
                &facts
            )
            .is_empty()
        );
    }

    #[test]
    fn empty_contract_and_malformed_protected_bindings_fail_closed() {
        let empty = json!({"receipts":[],"scm_facts":{"retirement_receipt_coverage":{"protected_base_ref":"origin/dev","current_epoch_commit_oid":NEW,"current_epoch_tree_oid":NEW_TREE,"protected_receipt_paths":[],"candidate_receipt_paths":[],"carried_receipt_paths":[],"new_receipt_paths":[],"scopes":[],"required_retired_paths":[]},"retirement_receipt_object_facts":[]}});
        let findings = evaluate_history_only_retirement_receipts(&empty);
        assert!(findings.contains(&Finding::new(
            RETIREMENT_RECEIPT_CODE,
            "retirement_receipts.receipts.empty"
        )));

        let receipt = receipt("idea-receipt", "ADR-0388", OLD, OLD_TREE, "docs/old.md");
        let mut facts = json!({"retirement_receipt_object_facts":[fact("idea-receipt","evidence/ideas-receipt.json","ADR-0388","new",OLD,OLD_TREE)]});
        facts["retirement_receipt_object_facts"][0]["protected_base_ref"] = json!("origin/main");
        facts["retirement_receipt_object_facts"][0]["protected_receipt_blob_oid"] = json!(null);
        let findings = evaluate_history_only_retirement_receipt(
            "evidence/ideas-receipt.json",
            &receipt,
            &facts,
        );
        assert!(findings.contains(&Finding::new(
            RETIREMENT_RECEIPT_CODE,
            "object_fact.protected_base_ref"
        )));
        assert!(findings.contains(&Finding::new(
            RETIREMENT_RECEIPT_CODE,
            "object_fact.protected_receipt_blob_oid"
        )));
    }
}
