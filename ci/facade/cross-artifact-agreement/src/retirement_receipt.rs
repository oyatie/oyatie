//! Pure, declared-input validation for history-only retirement receipts.
//!
//! This module never reads Git, the filesystem, process state, or ambient time.
//! Its callers provide the receipt, candidate-authored retirement facts, and a separate
//! `protected_scm_context` materialized at the Git trust boundary. That boundary keeps a
//! receipt check hermetic and prevents candidate data from defining both sides of a proof.
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
    "protected_preparation",
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
const PROTECTED_PREPARATION_FIELDS: &[&str] = &["receipt_path", "receipt_blob_oid"];
const FACT_FIELDS: &[&str] = &[
    "artifact_id",
    "receipt_path",
    "protected_base_ref",
    "receipt_state",
    "scope_ref",
    "scope_type",
    "baseline_commit_oid",
    "baseline_tree_oid",
    "protected_receipt_blob_oid",
    "candidate_receipt_blob_oid",
    "protected_registry_row_sha256",
    "candidate_registry_row_sha256",
    "retired_inputs",
    "preparation_receipt_path",
    "protected_preparation_receipt_blob_oid",
];
const FACT_INPUT_FIELDS: &[&str] = &[
    "path",
    "predecessor_blob_oid",
    "sha256",
    "byte_count",
    "protected_path_exists",
    "protected_path_kind",
    "protected_blob_oid",
    "protected_sha256",
    "protected_byte_count",
    "protected_mode",
    "candidate_path_exists",
    "candidate_path_kind",
    "candidate_blob_oid",
    "candidate_sha256",
    "candidate_byte_count",
    "candidate_mode",
    "candidate_new_equivalent_paths",
    "candidate_equivalent_paths",
];
const COVERAGE_FIELDS: &[&str] = &[
    "protected_base_ref",
    "protected_receipt_paths",
    "candidate_receipt_paths",
    "carried_receipt_paths",
    "new_receipt_paths",
    "scopes",
    "required_retired_paths",
];
const PROTECTED_SCM_CONTEXT_FIELDS: &[&str] = &[
    "protected_base_ref",
    "protected_base_commit_oid",
    "protected_base_tree_oid",
    "candidate_commit_oid",
    "candidate_tree_oid",
    "protected_base_is_ancestor_of_candidate",
    "prepared_receipt_paths",
    "protected_preparation_receipts",
];
const PROTECTED_PREPARATION_RECEIPT_FIELDS: &[&str] = &["receipt_path", "receipt_blob_oid"];
const SCM_FACT_FIELDS: &[&str] = &[
    "retirement_receipt_coverage",
    "retirement_receipt_object_facts",
    "protected_scm_context",
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
    let authority_decisions = string_set(
        authority.get("decisions"),
        "authority.decisions",
        &mut findings,
    );
    let expected_authority = scope.as_deref().and_then(scope_authority);
    if authority_decisions.is_empty() {
        fail(&mut findings, "authority.decisions.empty");
    } else if expected_authority
        .map(|authority| authority_decisions != BTreeSet::from([authority.to_owned()]))
        .unwrap_or(true)
    {
        fail(&mut findings, "authority.scope_binding");
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
    let effects = child(receipt, "effects");
    closed_object(effects, EFFECT_FIELDS, "effects", &mut findings);

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
    let state = fact.get("receipt_state").and_then(Value::as_str);
    if !matches!(
        state,
        Some("prepared-new" | "closure-new" | "carried" | "closed-carried")
    ) {
        fail(&mut findings, "object_fact.receipt_state");
    }
    let closure = state == Some("closure-new");
    if closure {
        if receipt.get("promoted_commit_oid").is_some()
            || receipt.get("postmerge_success").is_some()
        {
            fail(&mut findings, "closure_claim_ceiling");
        }
        exact(
            receipt,
            "artifact_type",
            "history-only-retirement-closure-receipt",
            &mut findings,
        );
        let preparation = child(receipt, "protected_preparation");
        closed_object(
            preparation,
            PROTECTED_PREPARATION_FIELDS,
            "protected_preparation",
            &mut findings,
        );
        let preparation_path = preparation
            .get("receipt_path")
            .and_then(Value::as_str)
            .filter(|path| repo_path(path));
        let preparation_blob = oid(
            preparation,
            "receipt_blob_oid",
            "protected_preparation.receipt_blob_oid",
            &mut findings,
        );
        if preparation_path.is_none()
            || fact.get("preparation_receipt_path").and_then(Value::as_str) != preparation_path
            || fact
                .get("protected_preparation_receipt_blob_oid")
                .and_then(Value::as_str)
                != preparation_blob.as_deref()
        {
            fail(&mut findings, "closure_preparation_link");
        }
    } else {
        exact(
            receipt,
            "artifact_type",
            "migration-closure-receipt",
            &mut findings,
        );
        if receipt.get("protected_preparation").is_some()
            || fact.get("preparation_receipt_path").is_some()
            || fact.get("protected_preparation_receipt_blob_oid").is_some()
        {
            fail(&mut findings, "closure_preparation_link");
        }
    }
    let _ = oid(
        fact,
        "candidate_receipt_blob_oid",
        "object_fact.candidate_receipt_blob_oid",
        &mut findings,
    );
    if !sha256(
        fact.get("candidate_registry_row_sha256")
            .and_then(Value::as_str),
    ) {
        fail(&mut findings, "object_fact.candidate_registry_row_sha256");
    }
    if matches!(state, Some("prepared-new" | "closure-new")) {
        if !fact
            .get("protected_receipt_blob_oid")
            .is_some_and(Value::is_null)
            || !fact
                .get("protected_registry_row_sha256")
                .is_some_and(Value::is_null)
        {
            fail(&mut findings, "prepared_new_protected_absent");
        }
    } else {
        let _ = oid(
            fact,
            "protected_receipt_blob_oid",
            "object_fact.protected_receipt_blob_oid",
            &mut findings,
        );
        if !sha256(
            fact.get("protected_registry_row_sha256")
                .and_then(Value::as_str),
        ) {
            fail(&mut findings, "object_fact.protected_registry_row_sha256");
        }
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
    if matches!(state, Some("carried" | "closed-carried"))
        && (fact.get("protected_receipt_blob_oid") != fact.get("candidate_receipt_blob_oid")
            || fact.get("protected_registry_row_sha256")
                != fact.get("candidate_registry_row_sha256"))
    {
        fail(&mut findings, "object_fact.immutable_carried_binding");
    }
    validate_state_claims(receipt, verification, effects, state, &mut findings);
    validate_inputs(receipt, fact, scope.as_deref(), state, &mut findings);
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
    closed_object(scm_facts, SCM_FACT_FIELDS, "scm_facts", &mut findings);
    let Some(records) = corpus.get("receipts").and_then(Value::as_array) else {
        fail(&mut findings, "retirement_receipts.receipts");
        return findings;
    };
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

/// Validate declared candidate-epoch partitioning and exact per-scope coverage.
/// Preparation binds only materialized candidate commit/tree facts. A later closure
/// is a distinct candidate receipt linked to a protected preparation blob; it never
/// mutates or copies that protected preparation.
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
    let protected_context = child(facts, "protected_scm_context");
    closed_object(
        protected_context,
        PROTECTED_SCM_CONTEXT_FIELDS,
        "protected_scm_context",
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
    let prepared = path_set(
        coverage.get("new_receipt_paths"),
        "new_receipt_paths",
        &mut findings,
    );
    let protected_preparations = protected_preparation_receipts(
        protected_context.get("protected_preparation_receipts"),
        &mut findings,
    );
    let protected_preparation_paths = protected_preparations
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let candidate_commit = oid(
        protected_context,
        "candidate_commit_oid",
        "protected_scm_context.candidate_commit_oid",
        &mut findings,
    );
    let candidate_tree = oid(
        protected_context,
        "candidate_tree_oid",
        "protected_scm_context.candidate_tree_oid",
        &mut findings,
    );
    let protected_commit = oid(
        protected_context,
        "protected_base_commit_oid",
        "protected_scm_context.protected_base_commit_oid",
        &mut findings,
    );
    let protected_tree = oid(
        protected_context,
        "protected_base_tree_oid",
        "protected_scm_context.protected_base_tree_oid",
        &mut findings,
    );
    if protected_context.get("protected_base_ref") != coverage.get("protected_base_ref")
        || protected_context
            .get("protected_base_is_ancestor_of_candidate")
            .and_then(Value::as_bool)
            != Some(true)
        || (!prepared.is_empty()
            && (candidate_commit == protected_commit || candidate_tree == protected_tree))
    {
        fail(&mut findings, "protected_scm_context.binding");
    }
    if protected
        != carried
            .union(&protected_preparation_paths)
            .cloned()
            .collect()
        || !carried.is_subset(&candidate)
        || !prepared.is_disjoint(&carried)
        || candidate != carried.union(&prepared).cloned().collect()
        || protected.intersection(&prepared).next().is_some()
    {
        fail(&mut findings, "receipt_path_partition");
    }
    if !protected_preparation_paths.is_disjoint(&candidate) {
        fail(&mut findings, "protected_preparation_immutable");
    }
    let required = validate_scopes(coverage, &mut findings);
    if required.is_empty() && (!candidate.is_empty() || !protected.is_empty()) {
        fail(&mut findings, "retirement_receipt_coverage.scopes.empty");
    }
    let object_facts = facts
        .get("retirement_receipt_object_facts")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    if object_facts.is_empty() && !candidate.is_empty() {
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
    let prepared_fact_paths = object_facts
        .iter()
        .filter(|fact| fact.get("receipt_state").and_then(Value::as_str) == Some("prepared-new"))
        .filter_map(|fact| fact.get("receipt_path").and_then(Value::as_str))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if path_set(
        protected_context.get("prepared_receipt_paths"),
        "protected_scm_context.prepared_receipt_paths",
        &mut findings,
    ) != prepared_fact_paths
    {
        fail(
            &mut findings,
            "protected_scm_context.prepared_receipt_paths",
        );
    }
    let mut carried_covered: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut prepared_covered: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut closed_preparation_paths = BTreeSet::new();
    let receipt_artifacts = receipts
        .iter()
        .filter_map(|receipt| receipt.get("artifact_id").and_then(Value::as_str))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if receipt_artifacts.len() != receipts.len() || receipt_artifacts != fact_artifacts {
        fail(&mut findings, "receipt_object_fact_artifact_set");
    }
    let receipt_scope_keys = receipts
        .iter()
        .filter_map(|receipt| receipt.get("scope_ref").and_then(Value::as_str))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let declared_scope_keys = required.keys().cloned().collect::<BTreeSet<_>>();
    if receipt_scope_keys != declared_scope_keys {
        fail(&mut findings, "receipt_scope_key_set");
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
        let state = fact.get("receipt_state").and_then(Value::as_str);
        match (path, state) {
            (Some(path), Some("carried" | "closed-carried")) if carried.contains(path) => {
                if fact.get("protected_base_ref") != coverage.get("protected_base_ref")
                    || fact.get("scope_ref") != receipt.get("scope_ref")
                    || fact.get("scope_type").and_then(Value::as_str)
                        != receipt
                            .get("scope_ref")
                            .and_then(Value::as_str)
                            .and_then(scope_type)
                    || fact.get("baseline_commit_oid") != receipt.pointer("/baseline/commit_oid")
                    || fact.get("baseline_tree_oid") != receipt.pointer("/baseline/tree_oid")
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
            (Some(path), Some("prepared-new")) if prepared.contains(path) => {
                if receipt
                    .pointer("/baseline/commit_oid")
                    .and_then(Value::as_str)
                    != protected_commit.as_deref()
                    || receipt
                        .pointer("/baseline/tree_oid")
                        .and_then(Value::as_str)
                        != protected_tree.as_deref()
                    || fact.get("baseline_commit_oid").and_then(Value::as_str)
                        != protected_commit.as_deref()
                    || fact.get("baseline_tree_oid").and_then(Value::as_str)
                        != protected_tree.as_deref()
                {
                    fail(&mut findings, "prepared_protected_baseline_binding");
                }
                let scope = receipt
                    .get("scope_ref")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                prepared_covered
                    .entry(scope.to_owned())
                    .or_default()
                    .extend(input_paths(receipt, &mut findings));
            }
            (Some(path), Some("closure-new")) if prepared.contains(path) => {
                let preparation = child(receipt, "protected_preparation");
                let preparation_path = preparation.get("receipt_path").and_then(Value::as_str);
                let preparation_blob = preparation.get("receipt_blob_oid").and_then(Value::as_str);
                if preparation_path
                    .and_then(|path| protected_preparations.get(path))
                    .map(String::as_str)
                    != preparation_blob
                    || receipt
                        .pointer("/baseline/commit_oid")
                        .and_then(Value::as_str)
                        != candidate_commit.as_deref()
                    || receipt
                        .pointer("/baseline/tree_oid")
                        .and_then(Value::as_str)
                        != candidate_tree.as_deref()
                    || fact.get("baseline_commit_oid").and_then(Value::as_str)
                        != candidate_commit.as_deref()
                    || fact.get("baseline_tree_oid").and_then(Value::as_str)
                        != candidate_tree.as_deref()
                {
                    fail(&mut findings, "closure_preparation_link");
                }
                if !preparation_path
                    .is_some_and(|path| closed_preparation_paths.insert(path.to_owned()))
                {
                    fail(&mut findings, "closure_preparation_ambiguous");
                }
                let scope = receipt
                    .get("scope_ref")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                prepared_covered
                    .entry(scope.to_owned())
                    .or_default()
                    .extend(input_paths(receipt, &mut findings));
            }
            _ => fail(&mut findings, "receipt_epoch_classification"),
        }
    }
    if closed_preparation_paths != protected_preparation_paths {
        fail(&mut findings, "closure_preparation_ambiguous");
    }
    for (scope, expected) in required {
        let carried_paths = carried_covered.get(&scope).cloned().unwrap_or_default();
        let prepared_paths = prepared_covered.get(&scope).cloned().unwrap_or_default();
        if carried_paths.is_empty() == prepared_paths.is_empty()
            || (!carried_paths.is_empty() && carried_paths != expected.0)
            || (!prepared_paths.is_empty() && prepared_paths != expected.0)
        {
            fail(
                &mut findings,
                &format!("scope_bidirectional_coverage.{scope}"),
            );
        }
    }
    findings
}

fn validate_state_claims(
    receipt: &Value,
    verification: &Value,
    effects: &Value,
    state: Option<&str>,
    findings: &mut BTreeSet<Finding>,
) {
    let prepared = state == Some("prepared-new");
    let expected_status = if prepared {
        "prepared-for-history-only-retirement"
    } else {
        "history-only-retired-nonauthoritative"
    };
    exact(receipt, "status", expected_status, findings);
    for key in EFFECT_FIELDS {
        if effects
            .get(*key)
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            fail(findings, &format!("effects.{key}"));
        }
    }
    if prepared {
        if verification
            .as_object()
            .is_none_or(|verification| !verification.is_empty())
        {
            fail(findings, "prepared_new.verification_contract");
        }
        for (key, value) in [
            ("repository_effect", "prepared"),
            ("runtime_effect", "none"),
            ("roadmap_effect", "none"),
            ("planning_hold_effect", "HOLD(Planning)"),
        ] {
            if effects.get(key).and_then(Value::as_str) != Some(value) {
                fail(findings, &format!("prepared_new.effects.{key}"));
            }
        }
        return;
    }
    if path_set(
        verification.get("expected_absent_paths"),
        "verification_contract.expected_absent_paths",
        findings,
    ) != input_paths(receipt, findings)
    {
        fail(findings, "verification_contract.expected_absent_paths");
    }
    if verification
        .get("expected_tracked_readable_archive_directory_count")
        .and_then(Value::as_u64)
        != Some(0)
    {
        fail(
            findings,
            "verification_contract.expected_tracked_readable_archive_directory_count",
        );
    }
    if string_set(
        verification.get("required_gates"),
        "verification_contract.required_gates",
        findings,
    ) != REQUIRED_GATES
        .iter()
        .map(|value| (*value).to_owned())
        .collect()
    {
        fail(findings, "verification_contract.required_gates.exact_set");
    }
    if state == Some("closure-new") {
        for (key, value) in [
            ("repository_effect", "history-only"),
            ("runtime_effect", "none"),
            ("roadmap_effect", "none"),
            ("planning_hold_effect", "HOLD(Planning)"),
        ] {
            if effects.get(key).and_then(Value::as_str) != Some(value) {
                fail(findings, "closure_claim_ceiling");
            }
        }
    } else if effects.get("repository_effect").and_then(Value::as_str) != Some("history-only") {
        fail(findings, "closure_repository_effect");
    }
}

fn validate_inputs(
    receipt: &Value,
    fact: &Value,
    scope: Option<&str>,
    state: Option<&str>,
    findings: &mut BTreeSet<Finding>,
) {
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
        let expected_disposition = if state == Some("prepared-new") {
            "prepared-for-history-only-retirement"
        } else {
            "retired-git-history-only"
        };
        if input.get("disposition").and_then(Value::as_str) != Some(expected_disposition) {
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
        let successor_refs = string_set(
            input.get("successor_refs"),
            &format!("retired_inputs[{index}].successor_refs"),
            findings,
        );
        if scope_authority(scope.unwrap_or(""))
            .map(|authority| successor_refs == BTreeSet::from([authority.to_owned()]))
            != Some(true)
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
        let protected_matches = fact_input
            .get("protected_path_exists")
            .and_then(Value::as_bool)
            == Some(true)
            && fact_input
                .get("protected_path_kind")
                .and_then(Value::as_str)
                == Some("regular")
            && fact_input.get("protected_blob_oid") == input.get("predecessor_blob_oid")
            && fact_input.get("protected_sha256") == input.get("sha256")
            && fact_input.get("protected_byte_count") == input.get("byte_count")
            && fact_input.get("protected_mode").and_then(Value::as_str) == Some("100644");
        if !protected_matches {
            fail(
                findings,
                &format!("retired_inputs[{index}].protected_body_binding"),
            );
        }
        let no_equivalent_copy = path_set(
            fact_input.get("candidate_new_equivalent_paths"),
            "candidate_new_equivalent_paths",
            findings,
        )
        .is_empty()
            && path_set(
                fact_input.get("candidate_equivalent_paths"),
                "candidate_equivalent_paths",
                findings,
            )
            .is_empty();
        let candidate_matches = fact_input
            .get("candidate_path_exists")
            .and_then(Value::as_bool)
            == Some(true)
            && fact_input
                .get("candidate_path_kind")
                .and_then(Value::as_str)
                == Some("regular")
            && fact_input.get("candidate_blob_oid") == input.get("predecessor_blob_oid")
            && fact_input.get("candidate_sha256") == input.get("sha256")
            && fact_input.get("candidate_byte_count") == input.get("byte_count")
            && fact_input.get("candidate_mode").and_then(Value::as_str) == Some("100644");
        let candidate_absent = fact_input
            .get("candidate_path_exists")
            .and_then(Value::as_bool)
            == Some(false)
            && fact_input
                .get("candidate_path_kind")
                .is_some_and(Value::is_null)
            && fact_input
                .get("candidate_blob_oid")
                .is_some_and(Value::is_null)
            && fact_input
                .get("candidate_sha256")
                .is_some_and(Value::is_null)
            && fact_input
                .get("candidate_byte_count")
                .is_some_and(Value::is_null)
            && fact_input.get("candidate_mode").is_some_and(Value::is_null);
        if !no_equivalent_copy
            || (state == Some("prepared-new") && !candidate_matches)
            || (matches!(state, Some("closure-new" | "carried" | "closed-carried"))
                && !candidate_absent)
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
        if declared != protected {
            fail(findings, &format!("scopes[{i}].required_retired_paths"));
        }
        if !union.is_disjoint(&declared) {
            fail(findings, "scope_retired_path_overlap");
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
        !value.is_empty()
            && value
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
fn scope_type(scope: &str) -> Option<&'static str> {
    match scope {
        "artifact:masterplan" => Some("masterplan-retired-surfaces"),
        // ADR-0363 is the existing amended retirement authority; this remains
        // distinct from ADR-0388's transient-idea scope.
        "ADR-0363" => Some("amended-agentic-vcs-retirement"),
        "ADR-0388" => Some("transient-ideas"),
        _ => None,
    }
}
fn scope_authority(scope: &str) -> Option<&'static str> {
    match scope {
        "artifact:masterplan" => Some("/specs/masterplan.json"),
        "ADR-0363" => Some("ADR-0363"),
        "ADR-0388" => Some("ADR-0388"),
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
fn protected_preparation_receipts(
    value: Option<&Value>,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, String> {
    let Some(receipts) = value.and_then(Value::as_array) else {
        fail(findings, "protected_preparation_receipts");
        return BTreeMap::new();
    };
    let mut result = BTreeMap::new();
    for (index, receipt) in receipts.iter().enumerate() {
        closed_object(
            receipt,
            PROTECTED_PREPARATION_RECEIPT_FIELDS,
            &format!("protected_preparation_receipts[{index}]"),
            findings,
        );
        let path = receipt
            .get("receipt_path")
            .and_then(Value::as_str)
            .filter(|path| repo_path(path));
        let blob = oid(
            receipt,
            "receipt_blob_oid",
            &format!("protected_preparation_receipts[{index}].receipt_blob_oid"),
            findings,
        );
        if let (Some(path), Some(blob)) = (path, blob) {
            if result.insert(path.to_owned(), blob).is_some() {
                fail(findings, "protected_preparation_receipts");
            }
        } else {
            fail(findings, "protected_preparation_receipts");
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
        let authority = scope_authority(scope).expect("known test scope");
        json!({"$schema":"https://json-schema.org/draft/2020-12/schema","artifact_id":id,"artifact_type":"migration-closure-receipt","status":"history-only-retired-nonauthoritative","recorded_at":"2026-07-22","scope_ref":scope,"authority":{"decisions":[authority],"planning_state":"HOLD(Planning)","dispatch_authorized":false,"completion_claims_promoted":0},"baseline":{"commit_oid":commit,"tree_oid":tree},"retired_inputs":[{"path":path,"predecessor_blob_oid":BLOB,"sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","byte_count":1,"successor_refs":[authority],"disposition":"retired-git-history-only"}],"provenance":{"content_store":"authorized Git object history only","readable_tracked_copy_retained":false,"readable_archive_directory_retained":false,"tombstone_content_retained":false,"receipt_reproduces_retired_content":false},"verification_contract":{"expected_absent_paths":[path],"expected_tracked_readable_archive_directory_count":0,"required_gates":REQUIRED_GATES},"effects":{"repository_effect":"history-only","runtime_effect":"none","roadmap_effect":"none","planning_hold_effect":"HOLD(Planning)"}})
    }
    fn prepared_receipt(id: &str, scope: &str, commit: &str, tree: &str, path: &str) -> Value {
        let mut receipt = receipt(id, scope, commit, tree, path);
        receipt["status"] = json!("prepared-for-history-only-retirement");
        receipt["retired_inputs"][0]["disposition"] = json!("prepared-for-history-only-retirement");
        receipt["verification_contract"] = json!({});
        receipt["effects"]["repository_effect"] = json!("prepared");
        receipt
    }
    fn fact(
        id: &str,
        receipt_path: &str,
        scope: &str,
        state: &str,
        commit: &str,
        tree: &str,
        path: &str,
    ) -> Value {
        let prepared = state == "prepared-new";
        json!({"artifact_id":id,"receipt_path":receipt_path,"protected_base_ref":"origin/dev","receipt_state":state,"scope_ref":scope,"scope_type":scope_type(scope).unwrap(),"baseline_commit_oid":commit,"baseline_tree_oid":tree,"protected_receipt_blob_oid":if prepared { Value::Null } else { json!(BLOB) },"candidate_receipt_blob_oid":BLOB,"protected_registry_row_sha256":if prepared { Value::Null } else { json!("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa") },"candidate_registry_row_sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","retired_inputs":[{"path":path,"predecessor_blob_oid":BLOB,"sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","byte_count":1,"protected_path_exists":true,"protected_path_kind":"regular","protected_blob_oid":BLOB,"protected_sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","protected_byte_count":1,"protected_mode":"100644","candidate_path_exists":prepared,"candidate_path_kind":if prepared { json!("regular") } else { Value::Null },"candidate_blob_oid":if prepared { json!(BLOB) } else { Value::Null },"candidate_sha256":if prepared { json!("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa") } else { Value::Null },"candidate_byte_count":if prepared { json!(1) } else { Value::Null },"candidate_mode":if prepared { json!("100644") } else { Value::Null },"candidate_new_equivalent_paths":[],"candidate_equivalent_paths":[]} ]})
    }
    fn protected_scm_context(prepared_receipt_paths: &[&str]) -> Value {
        json!({"protected_base_ref":"origin/dev","protected_base_commit_oid":OLD,"protected_base_tree_oid":OLD_TREE,"candidate_commit_oid":NEW,"candidate_tree_oid":NEW_TREE,"protected_base_is_ancestor_of_candidate":true,"prepared_receipt_paths":prepared_receipt_paths,"protected_preparation_receipts":[]})
    }
    fn closure_receipt(
        id: &str,
        scope: &str,
        path: &str,
        preparation_receipt_path: &str,
        preparation_receipt_blob_oid: &str,
    ) -> Value {
        let mut receipt = receipt(id, scope, NEW, NEW_TREE, path);
        receipt["artifact_type"] = json!("history-only-retirement-closure-receipt");
        receipt["protected_preparation"] = json!({
            "receipt_path": preparation_receipt_path,
            "receipt_blob_oid": preparation_receipt_blob_oid,
        });
        receipt
    }
    fn closure_fact(
        id: &str,
        receipt_path: &str,
        scope: &str,
        path: &str,
        preparation_receipt_path: &str,
        preparation_receipt_blob_oid: &str,
    ) -> Value {
        let mut fact = fact(id, receipt_path, scope, "closure-new", NEW, NEW_TREE, path);
        fact["protected_receipt_blob_oid"] = Value::Null;
        fact["protected_registry_row_sha256"] = Value::Null;
        fact["preparation_receipt_path"] = json!(preparation_receipt_path);
        fact["protected_preparation_receipt_blob_oid"] = json!(preparation_receipt_blob_oid);
        fact
    }
    fn protected_scm_context_with_preparation(
        preparation_receipt_path: &str,
        preparation_receipt_blob_oid: &str,
    ) -> Value {
        let mut context = protected_scm_context(&[]);
        context["protected_preparation_receipts"] = json!([{
            "receipt_path": preparation_receipt_path,
            "receipt_blob_oid": preparation_receipt_blob_oid,
        }]);
        context
    }
    #[test]
    fn carried_protected_idea_and_new_repository_authority_receipt_are_independently_admitted() {
        let idea_path = "evidence/ideas-receipt.json";
        let repo_path = "evidence/repository-receipt.json";
        let idea = receipt(
            "idea-receipt",
            "ADR-0388",
            OLD,
            OLD_TREE,
            "docs/idea-old.md",
        );
        let repo = prepared_receipt(
            "repository-receipt",
            "artifact:masterplan",
            OLD,
            OLD_TREE,
            "docs/masterplan-old.md",
        );
        let idea_selector = json!({"selector_type":"exact","selector":"docs/idea-old.md","protected_paths":["docs/idea-old.md"],"candidate_paths":[],"removed_paths":["docs/idea-old.md"],"surviving_paths":[],"candidate_only_paths":[],"external_assertion":"not-applicable"});
        let repo_selector = json!({"selector_type":"exact","selector":"docs/masterplan-old.md","protected_paths":["docs/masterplan-old.md"],"candidate_paths":["docs/masterplan-old.md"],"removed_paths":[],"surviving_paths":["docs/masterplan-old.md"],"candidate_only_paths":[],"external_assertion":"not-applicable"});
        let facts = json!({"retirement_receipt_coverage":{"protected_base_ref":"origin/dev","protected_receipt_paths":[idea_path],"candidate_receipt_paths":[idea_path,repo_path],"carried_receipt_paths":[idea_path],"new_receipt_paths":[repo_path],"scopes":[{"scope_ref":"ADR-0388","scope_type":"transient-ideas","selectors":[idea_selector],"required_retired_paths":["docs/idea-old.md"]},{"scope_ref":"artifact:masterplan","scope_type":"masterplan-retired-surfaces","selectors":[repo_selector],"required_retired_paths":["docs/masterplan-old.md"]}],"required_retired_paths":["docs/idea-old.md","docs/masterplan-old.md"]},"retirement_receipt_object_facts":[fact("idea-receipt",idea_path,"ADR-0388","carried",OLD,OLD_TREE,"docs/idea-old.md"),fact("repository-receipt",repo_path,"artifact:masterplan","prepared-new",OLD,OLD_TREE,"docs/masterplan-old.md")],"protected_scm_context":protected_scm_context(&[repo_path])});
        let findings = evaluate_history_only_retirement_receipt_coverage(
            &[idea.clone(), repo.clone()],
            &facts,
        );
        assert!(findings.is_empty(), "{findings:?}");

        let corpus = json!({"receipts":[
            {"receipt_path":idea_path,"receipt":idea},
            {"receipt_path":repo_path,"receipt":repo}
        ],"scm_facts":facts});
        let corpus_findings = evaluate_history_only_retirement_receipts(&corpus);
        assert!(corpus_findings.is_empty(), "{corpus_findings:?}");

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
        rebound["scm_facts"]["retirement_receipt_object_facts"][0]["candidate_registry_row_sha256"] =
            json!("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
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
        let mut facts = json!({"retirement_receipt_object_facts":[fact("idea-receipt","evidence/ideas-receipt.json","ADR-0388","carried",OLD,OLD_TREE,"docs/old.md")]});
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
    fn declared_state_facts_reject_false_green_mutations() {
        // 13 prepared-new mutations plus 6 for each carried epoch: 25 false-green
        // regressions over declared facts, with no fixture elevated to authority.
        let prepared_claim =
            prepared_receipt("state-receipt", "ADR-0363", OLD, OLD_TREE, "docs/old.md");
        let receipt = receipt("state-receipt", "ADR-0363", OLD, OLD_TREE, "docs/old.md");
        let prepared = fact(
            "state-receipt",
            "evidence/state-receipt.json",
            "ADR-0363",
            "prepared-new",
            OLD,
            OLD_TREE,
            "docs/old.md",
        );
        assert!(
            evaluate_history_only_retirement_receipt(
                "evidence/state-receipt.json",
                &prepared_claim,
                &json!({"retirement_receipt_object_facts":[prepared.clone()]})
            )
            .is_empty()
        );

        for (pointer, value) in [
            ("/protected_receipt_blob_oid", json!(BLOB)),
            (
                "/protected_registry_row_sha256",
                json!("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            ),
            ("/retired_inputs/0/protected_path_kind", json!("symlink")),
            ("/retired_inputs/0/protected_blob_oid", json!(NEW)),
            ("/retired_inputs/0/protected_mode", json!("160000")),
            ("/retired_inputs/0/candidate_path_exists", json!(false)),
            ("/retired_inputs/0/candidate_path_kind", json!("symlink")),
            ("/retired_inputs/0/candidate_blob_oid", json!(NEW)),
            (
                "/retired_inputs/0/candidate_sha256",
                json!("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            ),
            ("/retired_inputs/0/candidate_byte_count", json!(2)),
            ("/retired_inputs/0/candidate_mode", json!("120000")),
            (
                "/retired_inputs/0/candidate_new_equivalent_paths",
                json!(["docs/renamed.md"]),
            ),
            (
                "/retired_inputs/0/candidate_equivalent_paths",
                json!(["docs/pre-existing-copy.md"]),
            ),
        ] {
            let mut mutated = prepared.clone();
            *mutated.pointer_mut(pointer).expect("test fixture path") = value;
            assert!(
                !evaluate_history_only_retirement_receipt(
                    "evidence/state-receipt.json",
                    &prepared_claim,
                    &json!({"retirement_receipt_object_facts":[mutated]})
                )
                .is_empty(),
                "prepared-new mutation {pointer} was accepted"
            );
        }

        for state in ["carried", "closed-carried"] {
            let carried = fact(
                "state-receipt",
                "evidence/state-receipt.json",
                "ADR-0363",
                state,
                OLD,
                OLD_TREE,
                "docs/old.md",
            );
            assert!(
                evaluate_history_only_retirement_receipt(
                    "evidence/state-receipt.json",
                    &receipt,
                    &json!({"retirement_receipt_object_facts":[carried.clone()]})
                )
                .is_empty()
            );
            for (pointer, value) in [
                ("/protected_receipt_blob_oid", json!(NEW)),
                (
                    "/protected_registry_row_sha256",
                    json!(
                        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    ),
                ),
                ("/retired_inputs/0/candidate_path_exists", json!(true)),
                ("/retired_inputs/0/candidate_path_kind", json!("regular")),
                (
                    "/retired_inputs/0/candidate_equivalent_paths",
                    json!(["docs/resurrected.md"]),
                ),
            ] {
                let mut mutated = carried.clone();
                *mutated.pointer_mut(pointer).expect("test fixture path") = value;
                assert!(
                    !evaluate_history_only_retirement_receipt(
                        "evidence/state-receipt.json",
                        &receipt,
                        &json!({"retirement_receipt_object_facts":[mutated]})
                    )
                    .is_empty(),
                    "{state} mutation {pointer} was accepted"
                );
            }
        }
    }

    #[test]
    fn carry_only_coverage_allows_no_prepared_receipts_and_rejects_scope_contamination() {
        let carry_receipt = receipt("carry-only", "ADR-0363", OLD, OLD_TREE, "docs/old.md");
        let receipt_path = "evidence/carry-only.json";
        let selector = json!({"selector_type":"exact","selector":"docs/old.md","protected_paths":["docs/old.md"],"candidate_paths":[],"removed_paths":["docs/old.md"],"surviving_paths":[],"candidate_only_paths":[],"external_assertion":"not-applicable"});
        let facts = json!({"retirement_receipt_coverage":{"protected_base_ref":"origin/dev","protected_receipt_paths":[receipt_path],"candidate_receipt_paths":[receipt_path],"carried_receipt_paths":[receipt_path],"new_receipt_paths":[],"scopes":[{"scope_ref":"ADR-0363","scope_type":"amended-agentic-vcs-retirement","selectors":[selector.clone()],"required_retired_paths":["docs/old.md"]}],"required_retired_paths":["docs/old.md"]},"retirement_receipt_object_facts":[fact("carry-only",receipt_path,"ADR-0363","closed-carried",OLD,OLD_TREE,"docs/old.md")],"protected_scm_context":protected_scm_context(&[])});
        assert!(
            evaluate_history_only_retirement_receipt_coverage(&[carry_receipt], &facts).is_empty()
        );

        let mut contaminated = facts;
        contaminated["retirement_receipt_coverage"]["scopes"] = json!([
            {"scope_ref":"ADR-0363","scope_type":"amended-agentic-vcs-retirement","selectors":[selector.clone()],"required_retired_paths":["docs/old.md"]},
            {"scope_ref":"ADR-0388","scope_type":"transient-ideas","selectors":[selector],"required_retired_paths":["docs/old.md"]}
        ]);
        assert!(
            evaluate_history_only_retirement_receipt_coverage(
                &[receipt(
                    "carry-only",
                    "ADR-0363",
                    OLD,
                    OLD_TREE,
                    "docs/old.md"
                )],
                &contaminated
            )
            .contains(&Finding::new(
                RETIREMENT_RECEIPT_CODE,
                "scope_retired_path_overlap"
            ))
        );
    }

    #[test]
    fn preparation_and_scope_authority_contradictions_fail_closed() {
        let prepared = prepared_receipt(
            "prepared-authority",
            "ADR-0363",
            OLD,
            OLD_TREE,
            ".omc/legacy-a.md",
        );
        let prepared_fact = fact(
            "prepared-authority",
            "evidence/prepared-authority.json",
            "ADR-0363",
            "prepared-new",
            OLD,
            OLD_TREE,
            ".omc/legacy-a.md",
        );
        assert!(
            evaluate_history_only_retirement_receipt(
                "evidence/prepared-authority.json",
                &prepared,
                &json!({"retirement_receipt_object_facts":[prepared_fact.clone()]})
            )
            .is_empty()
        );

        for (pointer, value) in [
            ("/status", json!("history-only-retired-nonauthoritative")),
            (
                "/status",
                json!("prepared-for-history-only-retirement-success"),
            ),
            (
                "/retired_inputs/0/disposition",
                json!("retired-git-history-only"),
            ),
            (
                "/retired_inputs/0/disposition",
                json!("prepared-for-history-only-retirement-closed"),
            ),
            ("/effects/repository_effect", json!("history-only")),
            ("/effects/runtime_effect", json!("success")),
            ("/effects/roadmap_effect", json!("closed")),
            ("/effects/planning_hold_effect", json!("deleted")),
            ("/authority/decisions", json!(["ADR-0388"])),
            ("/retired_inputs/0/successor_refs", json!(["ADR-0388"])),
        ] {
            let mut contradicted = prepared.clone();
            *contradicted.pointer_mut(pointer).expect("fixture path") = value;
            assert!(
                !evaluate_history_only_retirement_receipt(
                    "evidence/prepared-authority.json",
                    &contradicted,
                    &json!({"retirement_receipt_object_facts":[prepared_fact.clone()]})
                )
                .is_empty(),
                "prepared contradiction {pointer} was accepted"
            );
        }
        let mut absent_claim = prepared.clone();
        absent_claim["verification_contract"]["expected_absent_paths"] =
            json!([".omc/legacy-a.md"]);
        assert!(
            !evaluate_history_only_retirement_receipt(
                "evidence/prepared-authority.json",
                &absent_claim,
                &json!({"retirement_receipt_object_facts":[prepared_fact.clone()]})
            )
            .is_empty()
        );
        for verification_claim in [
            json!({"expected_absent_paths":[".omc/legacy-a.md"]}),
            json!({"expected_tracked_readable_archive_directory_count":0}),
            json!({"required_gates":[RETIREMENT_RECEIPT_VALIDATOR]}),
            json!({"required_gate_claim":"success"}),
        ] {
            let mut contradicted = prepared.clone();
            contradicted["verification_contract"] = verification_claim;
            assert!(
                !evaluate_history_only_retirement_receipt(
                    "evidence/prepared-authority.json",
                    &contradicted,
                    &json!({"retirement_receipt_object_facts":[prepared_fact.clone()]})
                )
                .is_empty()
            );
        }

        let selector = json!({"selector_type":"exact","selector":".omc/legacy-a.md","protected_paths":[".omc/legacy-a.md"],"candidate_paths":[".omc/legacy-a.md"],"removed_paths":[],"surviving_paths":[".omc/legacy-a.md"],"candidate_only_paths":[],"external_assertion":"not-applicable"});
        let coverage = json!({"retirement_receipt_coverage":{"protected_base_ref":"origin/dev","protected_receipt_paths":[],"candidate_receipt_paths":["evidence/prepared-authority.json"],"carried_receipt_paths":[],"new_receipt_paths":["evidence/prepared-authority.json"],"scopes":[{"scope_ref":"ADR-0363","scope_type":"amended-agentic-vcs-retirement","selectors":[selector],"required_retired_paths":[".omc/legacy-a.md"]}],"required_retired_paths":[".omc/legacy-a.md"]},"retirement_receipt_object_facts":[prepared_fact],"protected_scm_context":protected_scm_context(&["evidence/prepared-authority.json"])});
        assert!(
            evaluate_history_only_retirement_receipt_coverage(
                std::slice::from_ref(&prepared),
                &coverage
            )
            .is_empty()
        );

        let mut baseline_lockstep_receipt = prepared.clone();
        baseline_lockstep_receipt["baseline"]["commit_oid"] = json!(NEW);
        baseline_lockstep_receipt["baseline"]["tree_oid"] = json!(NEW_TREE);
        let mut baseline_lockstep_facts = coverage.clone();
        baseline_lockstep_facts["retirement_receipt_object_facts"][0]["baseline_commit_oid"] =
            json!(NEW);
        baseline_lockstep_facts["retirement_receipt_object_facts"][0]["baseline_tree_oid"] =
            json!(NEW_TREE);
        assert!(
            !evaluate_history_only_retirement_receipt_coverage(
                std::slice::from_ref(&baseline_lockstep_receipt),
                &baseline_lockstep_facts
            )
            .is_empty(),
            "prepared receipt and fact baselines must bind to protected SCM context"
        );

        let mut candidate_rebound = coverage.clone();
        candidate_rebound["protected_scm_context"]["candidate_commit_oid"] = json!(OLD);
        assert!(
            evaluate_history_only_retirement_receipt_coverage(
                std::slice::from_ref(&prepared),
                &candidate_rebound
            )
            .contains(&Finding::new(
                RETIREMENT_RECEIPT_CODE,
                "protected_scm_context.binding"
            ))
        );
        let mut ancestry_rebound = coverage.clone();
        ancestry_rebound["protected_scm_context"]["protected_base_is_ancestor_of_candidate"] =
            json!(false);
        assert!(
            evaluate_history_only_retirement_receipt_coverage(
                std::slice::from_ref(&prepared),
                &ancestry_rebound
            )
            .contains(&Finding::new(
                RETIREMENT_RECEIPT_CODE,
                "protected_scm_context.binding"
            ))
        );
        let mut candidate_self_proof = coverage.clone();
        candidate_self_proof["retirement_receipt_coverage"]["candidate_commit_oid"] = json!(OLD);
        candidate_self_proof["retirement_receipt_coverage"]["candidate_tree_oid"] = json!(OLD_TREE);
        candidate_self_proof["retirement_receipt_object_facts"][0]["candidate_commit_oid"] =
            json!(OLD);
        candidate_self_proof["retirement_receipt_object_facts"][0]["candidate_tree_oid"] =
            json!(OLD_TREE);
        candidate_self_proof["retirement_receipt_object_facts"][0]["protected_base_is_ancestor_of_candidate"] =
            json!(false);
        assert!(
            !evaluate_history_only_retirement_receipt_coverage(
                std::slice::from_ref(&prepared),
                &candidate_self_proof
            )
            .is_empty(),
            "candidate-authored lockstep fields must not define the protected context"
        );

        let mut missing_scope = coverage.clone();
        missing_scope["retirement_receipt_coverage"]["scopes"] = json!([]);
        assert!(
            evaluate_history_only_retirement_receipt_coverage(
                std::slice::from_ref(&prepared),
                &missing_scope
            )
            .contains(&Finding::new(
                RETIREMENT_RECEIPT_CODE,
                "receipt_scope_key_set"
            ))
        );
    }

    #[test]
    fn empty_artifact_identifier_fails_closed() {
        let mut prepared = prepared_receipt(
            "prepared-empty-id",
            "ADR-0363",
            OLD,
            OLD_TREE,
            "docs/old.md",
        );
        prepared["artifact_id"] = json!("");
        let mut object_fact = fact(
            "prepared-empty-id",
            "evidence/prepared-empty-id.json",
            "ADR-0363",
            "prepared-new",
            OLD,
            OLD_TREE,
            "docs/old.md",
        );
        object_fact["artifact_id"] = json!("");
        assert!(
            evaluate_history_only_retirement_receipt(
                "evidence/prepared-empty-id.json",
                &prepared,
                &json!({"retirement_receipt_object_facts":[object_fact]})
            )
            .contains(&Finding::new(RETIREMENT_RECEIPT_CODE, "artifact_id"))
        );
    }

    #[test]
    fn dormant_empty_corpus_is_valid_but_any_present_row_fails_closed() {
        let empty = json!({
            "receipts": [],
            "scm_facts": {
                "retirement_receipt_coverage": {
                    "protected_base_ref": "origin/dev",
                    "protected_receipt_paths": [],
                    "candidate_receipt_paths": [],
                    "carried_receipt_paths": [],
                    "new_receipt_paths": [],
                    "scopes": [],
                    "required_retired_paths": []
                },
                "retirement_receipt_object_facts": [],
                "protected_scm_context": protected_scm_context(&[])
            }
        });
        assert!(
            evaluate_history_only_retirement_receipts(&empty).is_empty(),
            "the dormant no-receipt state must be valid"
        );

        let mut present = empty;
        present["scm_facts"]["retirement_receipt_coverage"]["candidate_receipt_paths"] =
            json!(["evidence/unbacked.json"]);
        assert!(
            !evaluate_history_only_retirement_receipts(&present).is_empty(),
            "any declared receipt path requires a complete receipt row"
        );
    }

    #[test]
    fn protected_preparation_closes_only_through_a_distinct_linked_closure_receipt() {
        const PREPARATION_PATH: &str = "evidence/prepared-authority.json";
        const CLOSURE_PATH: &str = "evidence/closure-authority.json";
        const PREPARATION_BLOB: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let closure = closure_receipt(
            "closure-authority",
            "ADR-0363",
            ".omc/legacy-a.md",
            PREPARATION_PATH,
            PREPARATION_BLOB,
        );
        let closure_object_fact = closure_fact(
            "closure-authority",
            CLOSURE_PATH,
            "ADR-0363",
            ".omc/legacy-a.md",
            PREPARATION_PATH,
            PREPARATION_BLOB,
        );
        let corpus = json!({
            "receipts": [{"receipt_path": CLOSURE_PATH, "receipt": closure}],
            "scm_facts": {
                "retirement_receipt_coverage": {
                    "protected_base_ref": "origin/dev",
                    "protected_receipt_paths": [PREPARATION_PATH],
                    "candidate_receipt_paths": [CLOSURE_PATH],
                    "carried_receipt_paths": [],
                    "new_receipt_paths": [CLOSURE_PATH],
                    "scopes": [{
                        "scope_ref": "ADR-0363",
                        "scope_type": "amended-agentic-vcs-retirement",
                        "selectors": [{
                            "selector_type": "exact",
                            "selector": ".omc/legacy-a.md",
                            "protected_paths": [".omc/legacy-a.md"],
                            "candidate_paths": [],
                            "removed_paths": [".omc/legacy-a.md"],
                            "surviving_paths": [],
                            "candidate_only_paths": [],
                            "external_assertion": "not-applicable"
                        }],
                        "required_retired_paths": [".omc/legacy-a.md"]
                    }],
                    "required_retired_paths": [".omc/legacy-a.md"]
                },
                "retirement_receipt_object_facts": [closure_object_fact],
                "protected_scm_context": protected_scm_context_with_preparation(
                    PREPARATION_PATH,
                    PREPARATION_BLOB
                )
            }
        });
        assert!(
            evaluate_history_only_retirement_receipts(&corpus).is_empty(),
            "a protected preparation must close through a separate linked receipt"
        );

        for (pointer, value) in [
            (
                "/receipts/0/receipt/protected_preparation/receipt_blob_oid",
                json!("cccccccccccccccccccccccccccccccccccccccc"),
            ),
            (
                "/receipts/0/receipt/protected_preparation/receipt_path",
                json!("evidence/missing-preparation.json"),
            ),
        ] {
            let mut mutated = corpus.clone();
            *mutated.pointer_mut(pointer).expect("fixture path") = value;
            assert!(
                evaluate_history_only_retirement_receipts(&mutated).contains(&Finding::new(
                    RETIREMENT_RECEIPT_CODE,
                    "closure_preparation_link"
                )),
                "closure mutation {pointer} must fail at closure_preparation_link"
            );
        }

        let mut missing_link = corpus.clone();
        missing_link["receipts"][0]["receipt"]["protected_preparation"] = json!({});
        assert!(
            evaluate_history_only_retirement_receipts(&missing_link).contains(&Finding::new(
                RETIREMENT_RECEIPT_CODE,
                "closure_preparation_link"
            ))
        );

        for (pointer, value) in [
            ("promoted_commit_oid", json!(NEW)),
            ("postmerge_success", json!(true)),
            ("effects.runtime_effect", json!("success")),
        ] {
            let mut mutated = corpus.clone();
            let target = &mut mutated["receipts"][0]["receipt"];
            if let Some((parent, key)) = pointer.rsplit_once('.') {
                target[parent][key] = value;
            } else {
                target[pointer] = value;
            }
            assert!(
                evaluate_history_only_retirement_receipts(&mutated).contains(&Finding::new(
                    RETIREMENT_RECEIPT_CODE,
                    "closure_claim_ceiling"
                )),
                "closure claim {pointer} must remain below the promotion ceiling"
            );
        }

        let mut copied_preparation = corpus.clone();
        copied_preparation["scm_facts"]["retirement_receipt_coverage"]["candidate_receipt_paths"] =
            json!([PREPARATION_PATH, CLOSURE_PATH]);
        copied_preparation["scm_facts"]["retirement_receipt_coverage"]["new_receipt_paths"] =
            json!([PREPARATION_PATH, CLOSURE_PATH]);
        assert!(
            evaluate_history_only_retirement_receipts(&copied_preparation).contains(&Finding::new(
                RETIREMENT_RECEIPT_CODE,
                "protected_preparation_immutable"
            ))
        );

        let mut duplicate = corpus.clone();
        duplicate["receipts"]
            .as_array_mut()
            .expect("array")
            .push(json!({
                "receipt_path": "evidence/closure-authority-duplicate.json",
                "receipt": closure_receipt(
                    "closure-authority-duplicate",
                    "ADR-0363",
                    ".omc/legacy-a.md",
                    PREPARATION_PATH,
                    PREPARATION_BLOB
                )
            }));
        duplicate["scm_facts"]["retirement_receipt_coverage"]["candidate_receipt_paths"] =
            json!([CLOSURE_PATH, "evidence/closure-authority-duplicate.json"]);
        duplicate["scm_facts"]["retirement_receipt_coverage"]["new_receipt_paths"] =
            json!([CLOSURE_PATH, "evidence/closure-authority-duplicate.json"]);
        duplicate["scm_facts"]["retirement_receipt_object_facts"]
            .as_array_mut()
            .expect("array")
            .push(closure_fact(
                "closure-authority-duplicate",
                "evidence/closure-authority-duplicate.json",
                "ADR-0363",
                ".omc/legacy-a.md",
                PREPARATION_PATH,
                PREPARATION_BLOB,
            ));
        assert!(
            evaluate_history_only_retirement_receipts(&duplicate).contains(&Finding::new(
                RETIREMENT_RECEIPT_CODE,
                "closure_preparation_ambiguous"
            ))
        );
    }

    #[test]
    fn empty_contract_and_malformed_protected_bindings_fail_closed() {
        let empty = json!({"receipts":[],"scm_facts":{"retirement_receipt_coverage":{"protected_base_ref":"origin/dev","current_epoch_commit_oid":NEW,"current_epoch_tree_oid":NEW_TREE,"protected_receipt_paths":[],"candidate_receipt_paths":[],"carried_receipt_paths":[],"new_receipt_paths":[],"scopes":[],"required_retired_paths":[]},"retirement_receipt_object_facts":[]}});
        let findings = evaluate_history_only_retirement_receipts(&empty);
        assert!(findings.contains(&Finding::new(
            RETIREMENT_RECEIPT_CODE,
            "retirement_receipt_coverage.unknown_field.current_epoch_commit_oid"
        )));

        let receipt = receipt("idea-receipt", "ADR-0388", OLD, OLD_TREE, "docs/old.md");
        let mut facts = json!({"retirement_receipt_object_facts":[fact("idea-receipt","evidence/ideas-receipt.json","ADR-0388","carried",OLD,OLD_TREE,"docs/old.md")]});
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
