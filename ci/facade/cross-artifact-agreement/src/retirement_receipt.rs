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
use sha2::{Digest, Sha256};

use crate::{
    Finding, IDEA_ARCHIVE_TRANSITION_VALIDATOR, IdeaArchiveVerifiedClosureProjection,
    immutable_idea_archive_baseline,
};

pub const RETIREMENT_RECEIPT_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/history-only-retirement-receipt";
pub const RETIREMENT_RECEIPT_CODE: &str = "history_only_retirement_receipt_invalid";

const RETIREMENT_CONTROL_PLANE_PATH: &str = "registry/history-only-retirement/control-plane.json";

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
    "predecessor_context",
    "control_plane_entry",
    "control_plane_entry_sha256",
];
const FACT_INPUT_FIELDS: &[&str] = &[
    "path",
    "mode",
    "predecessor_blob_oid",
    "sha256",
    "byte_count",
    "predecessor_path_exists",
    "predecessor_path_kind",
    "predecessor_sha256",
    "predecessor_byte_count",
    "predecessor_mode",
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
    "evaluated_commit_oid",
    "evaluated_tree_oid",
    "subject_commit_oid",
    "subject_tree_oid",
    "scm_event_name",
    "subject_relationship",
    "protected_base_is_ancestor_of_evaluated",
    "protected_base_is_evaluated_first_parent",
    "subject_is_evaluated_second_parent",
    "predecessor_commit_oid",
    "predecessor_tree_oid",
    "predecessor_commit_exists",
    "predecessor_tree_exists",
    "predecessor_commit_tree_bound",
    "predecessor_is_ancestor_of_protected_base",
    "prepared_receipt_paths",
    "protected_preparation_receipts",
    "control_plane_entries",
];
const PREDECESSOR_CONTEXT_FIELDS: &[&str] = &[
    "source",
    "commit_oid",
    "tree_oid",
    "receipt_path",
    "receipt_blob_oid",
];
const CONTROL_PLANE_ENTRY_FIELDS: &[&str] = &[
    "evidence_set_id",
    "scope_ref",
    "scope_type",
    "selectors",
    "preparation_artifact_id",
    "preparation_receipt_path",
    "closure_artifact_id",
    "closure_receipt_path",
];
const ADR_0388_EVIDENCE_SET_ID: &str = "adr-0388-transient-ideas-history-only-retirement-v1";
const ADR_0388_PREPARATION_ARTIFACT_ID: &str = "adr-0388-transient-ideas-retirement-preparation";
const ADR_0388_PREPARATION_PATH: &str =
    "evidence/history-only-retirement/adr-0388-transient-ideas-preparation.json";
const ADR_0388_CLOSURE_ARTIFACT_ID: &str = "adr-0388-transient-ideas-retirement-closure";
const ADR_0388_CLOSURE_PATH: &str =
    "evidence/history-only-retirement/adr-0388-transient-ideas-closure.json";
const PROTECTED_PREPARATION_RECEIPT_FIELDS: &[&str] = &[
    "receipt_path",
    "receipt_blob_oid",
    "baseline_commit_oid",
    "baseline_tree_oid",
];
const SCM_FACT_FIELDS: &[&str] = &[
    "retirement_receipt_coverage",
    "retirement_receipt_object_facts",
    "protected_scm_context",
    "retirement_control_plane_context",
];
const RETIREMENT_CONTROL_PLANE_CONTEXT_FIELDS: &[&str] = &[
    "control_plane_path",
    "receipt_root",
    "bootstrap",
    "lifecycle_state",
    "protected_control_plane_blob_oid",
    "protected_control_plane_sha256",
    "protected_control_plane_byte_count",
    "candidate_control_plane_blob_oid",
    "candidate_control_plane_sha256",
    "candidate_control_plane_byte_count",
    "control_plane_entries",
    "control_plane_entry_hashes",
    "protected_receipt_root_paths",
    "candidate_receipt_root_paths",
    "unexpected_protected_receipt_paths",
    "unexpected_candidate_receipt_paths",
];
const RECEIPT_CORPUS_FIELDS: &[&str] = &["receipts", "scm_facts"];
const RECEIPT_RECORD_FIELDS: &[&str] = &["receipt_path", "receipt"];
const RECEIPT_METADATA_FIELDS: &[&str] = &[
    "receipt_path",
    "artifact_id",
    "scope_ref",
    "receipt_state",
    "candidate_receipt_blob_oid",
    "candidate_receipt_sha256",
    "baseline_commit_oid",
    "baseline_tree_oid",
];
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
    "predecessor_paths",
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
    IDEA_ARCHIVE_TRANSITION_VALIDATOR,
];

/// A candidate receipt supplied separately from the generated metadata face.
///
/// The materializer owns the blob identity; callers provide the exact source bytes
/// and their parsed JSON document.  Keeping this out of the generated face prevents
/// retired-content bodies from becoming a second generated-artifact authority.
#[derive(Clone, Debug)]
pub struct RawHistoryOnlyRetirementReceipt<'a> {
    pub receipt_path: &'a str,
    pub bytes: &'a [u8],
    pub document: &'a Value,
}

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
        Some("prepared-new" | "closure-new" | "closed-carried")
    ) {
        fail(&mut findings, "object_fact.receipt_state");
    }
    validate_predecessor_context(fact, state, receipt, &mut findings);
    validate_control_plane_entry(fact, scope.as_deref(), &mut findings);
    validate_fixed_receipt_identity(receipt_path, receipt, fact, state, &mut findings);
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
            || (state != Some("closed-carried")
                && (fact.get("preparation_receipt_path").is_some()
                    || fact.get("protected_preparation_receipt_blob_oid").is_some()))
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
    if state == Some("closed-carried")
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
    validate_adr_0388_e4_bindings(&receipts, scm_facts, &mut findings);
    findings
}

/// Evaluate the canonical generated facts face together with separately loaded,
/// path-bound candidate receipt documents.  The generated `receipts` array is
/// metadata-only: it never carries a raw receipt body.
pub fn evaluate_history_only_retirement_facts(
    facts: &Value,
    raw_receipts: &[RawHistoryOnlyRetirementReceipt<'_>],
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    closed_object(
        facts,
        RECEIPT_CORPUS_FIELDS,
        "history_only_retirement_facts",
        &mut findings,
    );
    let scm_facts = child(facts, "scm_facts");
    closed_object(scm_facts, SCM_FACT_FIELDS, "scm_facts", &mut findings);
    let Some(metadata) = facts.get("receipts").and_then(Value::as_array) else {
        fail(&mut findings, "history_only_retirement_facts.receipts");
        return findings;
    };

    let mut metadata_by_path = BTreeMap::new();
    for (index, row) in metadata.iter().enumerate() {
        closed_object(
            row,
            RECEIPT_METADATA_FIELDS,
            &format!("history_only_retirement_facts.receipts[{index}]"),
            &mut findings,
        );
        let Some(path) = row
            .get("receipt_path")
            .and_then(Value::as_str)
            .filter(|path| repo_path(path))
        else {
            fail(&mut findings, "history_only_retirement_facts.receipt_path");
            continue;
        };
        if metadata_by_path.insert(path, row).is_some() {
            fail(
                &mut findings,
                "history_only_retirement_facts.receipt_path_duplicate",
            );
        }
        required_identifier(row, "artifact_id", &mut findings);
        required_string(row, "scope_ref", &mut findings);
        if !matches!(
            row.get("receipt_state").and_then(Value::as_str),
            Some("prepared-new" | "closure-new" | "closed-carried")
        ) {
            fail(&mut findings, "history_only_retirement_facts.receipt_state");
        }
        let _ = oid(
            row,
            "candidate_receipt_blob_oid",
            "history_only_retirement_facts.candidate_receipt_blob_oid",
            &mut findings,
        );
        if !sha256(row.get("candidate_receipt_sha256").and_then(Value::as_str)) {
            fail(
                &mut findings,
                "history_only_retirement_facts.candidate_receipt_sha256",
            );
        }
        let _ = oid(
            row,
            "baseline_commit_oid",
            "history_only_retirement_facts.baseline_commit_oid",
            &mut findings,
        );
        let _ = oid(
            row,
            "baseline_tree_oid",
            "history_only_retirement_facts.baseline_tree_oid",
            &mut findings,
        );
    }

    let mut raw_by_path = BTreeMap::new();
    for raw in raw_receipts {
        if !repo_path(raw.receipt_path) || raw_by_path.insert(raw.receipt_path, raw).is_some() {
            fail(
                &mut findings,
                "history_only_retirement_facts.raw_receipt_path",
            );
        }
    }
    if metadata_by_path.keys().copied().collect::<BTreeSet<_>>()
        != raw_by_path.keys().copied().collect::<BTreeSet<_>>()
    {
        fail(
            &mut findings,
            "history_only_retirement_facts.raw_receipt_path_set",
        );
    }

    let mut documents = Vec::with_capacity(raw_receipts.len());
    for (path, metadata) in metadata_by_path {
        let Some(raw) = raw_by_path.get(path) else {
            continue;
        };
        if !matches!(crate::parse_duplicate_key_free_json(raw.bytes), Some(ref parsed) if parsed == raw.document)
        {
            fail(
                &mut findings,
                "history_only_retirement_facts.raw_receipt_document_bytes",
            );
        }
        let digest = format!("sha256:{:x}", Sha256::digest(raw.bytes));
        if metadata
            .get("candidate_receipt_sha256")
            .and_then(Value::as_str)
            != Some(digest.as_str())
            || metadata.get("artifact_id") != raw.document.get("artifact_id")
            || metadata.get("scope_ref") != raw.document.get("scope_ref")
            || metadata.get("baseline_commit_oid") != raw.document.pointer("/baseline/commit_oid")
            || metadata.get("baseline_tree_oid") != raw.document.pointer("/baseline/tree_oid")
        {
            fail(
                &mut findings,
                "history_only_retirement_facts.raw_receipt_metadata_binding",
            );
        }
        let fact_matches = scm_facts
            .pointer("/retirement_receipt_object_facts")
            .and_then(Value::as_array)
            .map(|rows| {
                rows.iter()
                    .filter(|fact| fact.get("receipt_path").and_then(Value::as_str) == Some(path))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if fact_matches.len() != 1
            || fact_matches[0].get("artifact_id") != metadata.get("artifact_id")
            || fact_matches[0].get("scope_ref") != metadata.get("scope_ref")
            || fact_matches[0].get("receipt_state") != metadata.get("receipt_state")
            || fact_matches[0].get("candidate_receipt_blob_oid")
                != metadata.get("candidate_receipt_blob_oid")
            || fact_matches[0].get("candidate_registry_row_sha256")
                != metadata.get("candidate_receipt_sha256")
            || fact_matches[0].get("baseline_commit_oid") != metadata.get("baseline_commit_oid")
            || fact_matches[0].get("baseline_tree_oid") != metadata.get("baseline_tree_oid")
        {
            fail(
                &mut findings,
                "history_only_retirement_facts.object_fact_metadata_binding",
            );
        }
        findings.extend(evaluate_history_only_retirement_receipt(
            path,
            raw.document,
            scm_facts,
        ));
        documents.push(raw.document.clone());
    }
    findings.extend(evaluate_history_only_retirement_receipt_coverage(
        &documents, scm_facts,
    ));
    validate_adr_0388_e4_bindings(&documents, scm_facts, &mut findings);
    findings
}

/// Result of validating a retirement corpus and, only on success, projecting
/// the verified E4 closure state into the idea-archive transition evaluator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryOnlyRetirementClosureEvaluation {
    /// Fail-closed validation findings. data_class: INTERNAL_ONLY
    pub findings: BTreeSet<Finding>,
    /// Present only when the complete corpus validates. data_class: INTERNAL_ONLY
    pub projection: Option<IdeaArchiveVerifiedClosureProjection>,
}

/// Validate the full retirement corpus before producing any closure projection.
///
/// A valid dormant or preparation corpus yields an empty projection. Only an
/// exact ADR-0388 `closure-new` or `closed-carried` corpus yields the singleton
/// E4 evidence-set projection. Invalid input yields findings and no projection.
#[must_use]
pub fn evaluate_and_project_history_only_retirement_closures(
    corpus: &Value,
) -> HistoryOnlyRetirementClosureEvaluation {
    let findings = evaluate_history_only_retirement_receipts(corpus);
    if !findings.is_empty() {
        return HistoryOnlyRetirementClosureEvaluation {
            findings,
            projection: None,
        };
    }

    let has_verified_adr_0388_closure = corpus
        .pointer("/scm_facts/retirement_receipt_object_facts")
        .and_then(Value::as_array)
        .is_some_and(|facts| {
            facts.iter().any(|fact| {
                fact.get("scope_ref").and_then(Value::as_str) == Some("ADR-0388")
                    && matches!(
                        fact.get("receipt_state").and_then(Value::as_str),
                        Some("closure-new" | "closed-carried")
                    )
            })
        });
    let projection = if has_verified_adr_0388_closure {
        IdeaArchiveVerifiedClosureProjection::verified_adr_0388_history_only_retirement()
    } else {
        IdeaArchiveVerifiedClosureProjection::default()
    };
    HistoryOnlyRetirementClosureEvaluation {
        findings,
        projection: Some(projection),
    }
}

/// Validate the canonical metadata-plus-raw-documents inputs before projection.
#[must_use]
pub fn evaluate_and_project_history_only_retirement_facts(
    facts: &Value,
    raw_receipts: &[RawHistoryOnlyRetirementReceipt<'_>],
) -> HistoryOnlyRetirementClosureEvaluation {
    let findings = evaluate_history_only_retirement_facts(facts, raw_receipts);
    if !findings.is_empty() {
        return HistoryOnlyRetirementClosureEvaluation {
            findings,
            projection: None,
        };
    }
    let has_verified_adr_0388_closure = facts
        .pointer("/scm_facts/retirement_receipt_object_facts")
        .and_then(Value::as_array)
        .is_some_and(|rows| {
            rows.iter().any(|fact| {
                fact.get("scope_ref").and_then(Value::as_str) == Some("ADR-0388")
                    && matches!(
                        fact.get("receipt_state").and_then(Value::as_str),
                        Some("closure-new" | "closed-carried")
                    )
            })
        });
    HistoryOnlyRetirementClosureEvaluation {
        findings,
        projection: Some(if has_verified_adr_0388_closure {
            IdeaArchiveVerifiedClosureProjection::verified_adr_0388_history_only_retirement()
        } else {
            IdeaArchiveVerifiedClosureProjection::default()
        }),
    }
}

fn validate_adr_0388_e4_bindings(
    receipts: &[Value],
    scm_facts: &Value,
    findings: &mut BTreeSet<Finding>,
) {
    let Ok(baseline) = immutable_idea_archive_baseline() else {
        fail(findings, "adr_0388_e4_baseline");
        return;
    };
    let expected = baseline
        .entries
        .iter()
        .map(|entry| {
            (
                format!("{}/{}", baseline.scope_root, entry.path),
                entry.blob_oid.as_str(),
                format!("sha256:{}", entry.sha256),
                entry.byte_length,
            )
        })
        .collect::<Vec<_>>();
    let facts = scm_facts
        .get("retirement_receipt_object_facts")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let adr_receipts = receipts
        .iter()
        .filter(|receipt| receipt.get("scope_ref").and_then(Value::as_str) == Some("ADR-0388"))
        .collect::<Vec<_>>();
    let adr_facts = facts
        .iter()
        .filter(|fact| fact.get("scope_ref").and_then(Value::as_str) == Some("ADR-0388"))
        .collect::<Vec<_>>();
    if adr_receipts.is_empty() && adr_facts.is_empty() {
        return;
    }
    if adr_receipts.len() != 1 || adr_facts.len() != 1 {
        fail(findings, "adr_0388_e4_receipt_cardinality");
        return;
    }
    let receipt = adr_receipts[0];
    let fact = adr_facts[0];
    validate_exact_e4_inputs(receipt.get("retired_inputs"), &expected, false, findings);
    validate_exact_e4_inputs(fact.get("retired_inputs"), &expected, true, findings);

    let expected_paths = expected
        .iter()
        .map(|(path, _, _, _)| path.clone())
        .collect::<BTreeSet<_>>();
    let coverage = child(scm_facts, "retirement_receipt_coverage");
    let scopes = coverage
        .get("scopes")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let adr_scopes = scopes
        .iter()
        .filter(|scope| scope.get("scope_ref").and_then(Value::as_str) == Some("ADR-0388"))
        .collect::<Vec<_>>();
    if adr_scopes.len() != 1
        || path_set(
            adr_scopes
                .first()
                .and_then(|scope| scope.get("required_retired_paths")),
            "adr_0388.required_retired_paths",
            findings,
        ) != expected_paths
    {
        fail(findings, "adr_0388_e4_coverage");
    }
    if let Some(scope) = adr_scopes.first() {
        validate_exact_e4_selectors(scope.get("selectors"), &expected_paths, findings);
    }

    for entries in [
        scm_facts.pointer("/protected_scm_context/control_plane_entries"),
        scm_facts.pointer("/retirement_control_plane_context/control_plane_entries"),
    ] {
        let Some(adr_entry) = entries.and_then(Value::as_array).and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry.get("scope_ref").and_then(Value::as_str) == Some("ADR-0388"))
        }) else {
            fail(findings, "adr_0388_e4_control_plane");
            continue;
        };
        let selectors = path_set(
            adr_entry.get("selectors"),
            "adr_0388.control_plane.selectors",
            findings,
        );
        if selectors != expected_paths {
            fail(findings, "adr_0388_e4_control_plane");
        }
    }
}

fn validate_exact_e4_inputs(
    inputs: Option<&Value>,
    expected: &[(String, &str, String, u64)],
    fact: bool,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(inputs) = inputs.and_then(Value::as_array) else {
        fail(findings, "adr_0388_e4_inputs");
        return;
    };
    if inputs.len() != expected.len() {
        fail(findings, "adr_0388_e4_inputs");
        return;
    }
    for (input, (path, blob, sha256, byte_count)) in inputs.iter().zip(expected) {
        if input.get("path").and_then(Value::as_str) != Some(path.as_str())
            || input.get("predecessor_blob_oid").and_then(Value::as_str) != Some(*blob)
            || input.get("sha256").and_then(Value::as_str) != Some(sha256.as_str())
            || input.get("byte_count").and_then(Value::as_u64) != Some(*byte_count)
        {
            fail(findings, "adr_0388_e4_inputs");
        }
        if fact
            && (input.get("predecessor_sha256").and_then(Value::as_str) != Some(sha256.as_str())
                || input.get("predecessor_byte_count").and_then(Value::as_u64) != Some(*byte_count))
        {
            fail(findings, "adr_0388_e4_object_facts");
        }
    }
}

fn validate_exact_e4_selectors(
    selectors: Option<&Value>,
    expected_paths: &BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(selectors) = selectors.and_then(Value::as_array) else {
        fail(findings, "adr_0388_e4_selectors");
        return;
    };
    if selectors.len() != expected_paths.len() {
        fail(findings, "adr_0388_e4_selectors");
        return;
    }
    let actual = selectors
        .iter()
        .filter_map(|selector| selector.get("selector").and_then(Value::as_str))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if actual != *expected_paths
        || selectors.iter().any(|selector| {
            let path = selector.get("selector").and_then(Value::as_str);
            selector.get("selector_type").and_then(Value::as_str) != Some("exact")
                || path_set(selector.get("protected_paths"), "protected_paths", findings)
                    != path.into_iter().map(str::to_owned).collect()
                || path_set(
                    selector.get("predecessor_paths"),
                    "predecessor_paths",
                    findings,
                ) != path.into_iter().map(str::to_owned).collect()
        })
    {
        fail(findings, "adr_0388_e4_selectors");
    }
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
    let retirement_control_plane_context = child(facts, "retirement_control_plane_context");
    closed_object(
        retirement_control_plane_context,
        RETIREMENT_CONTROL_PLANE_CONTEXT_FIELDS,
        "retirement_control_plane_context",
        &mut findings,
    );
    if facts
        .get("retirement_receipt_object_facts")
        .and_then(Value::as_array)
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| row.get("receipt_state").and_then(Value::as_str) == Some("carried"))
        })
    {
        fail(&mut findings, "retirement_receipt_coverage.legacy_carried");
    }
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
    let evaluated_commit = oid(
        protected_context,
        "evaluated_commit_oid",
        "protected_scm_context.evaluated_commit_oid",
        &mut findings,
    );
    let evaluated_tree = oid(
        protected_context,
        "evaluated_tree_oid",
        "protected_scm_context.evaluated_tree_oid",
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
    let subject_commit = oid(
        protected_context,
        "subject_commit_oid",
        "protected_scm_context.subject_commit_oid",
        &mut findings,
    );
    let subject_tree = oid(
        protected_context,
        "subject_tree_oid",
        "protected_scm_context.subject_tree_oid",
        &mut findings,
    );
    let event = protected_context
        .get("scm_event_name")
        .and_then(Value::as_str);
    let relationship = protected_context
        .get("subject_relationship")
        .and_then(Value::as_str);
    let event_identity_valid = match event {
        Some("pull_request") => {
            relationship == Some("pull-request-head")
                && protected_context.get("subject_is_evaluated_second_parent")
                    == Some(&Value::Bool(true))
                && subject_commit != evaluated_commit
        }
        Some("push") | Some("merge_group") => {
            relationship == Some("evaluated-self")
                && protected_context.get("subject_is_evaluated_second_parent")
                    == Some(&Value::Bool(false))
                && subject_commit == evaluated_commit
                && subject_tree == evaluated_tree
        }
        _ => false,
    };
    if protected_context.get("protected_base_ref") != coverage.get("protected_base_ref")
        || protected_context
            .get("protected_base_is_ancestor_of_evaluated")
            .and_then(Value::as_bool)
            != Some(true)
        || protected_context
            .get("protected_base_is_evaluated_first_parent")
            .and_then(Value::as_bool)
            != Some(true)
        || !event_identity_valid
        || (!prepared.is_empty()
            && (evaluated_commit == protected_commit || evaluated_tree == protected_tree))
    {
        fail(&mut findings, "protected_scm_context.binding");
    }
    if !carried.is_subset(&protected)
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
    let materialized_control_entries = control_plane_entries(
        retirement_control_plane_context.get("control_plane_entries"),
        &mut findings,
    );
    if materialized_control_entries != protected_control_entries {
        fail(
            &mut findings,
            "retirement_control_plane_context.independent_entry_binding",
        );
    }
    validate_retirement_control_plane_context(
        retirement_control_plane_context,
        receipts,
        object_facts,
        &mut findings,
    );
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
    if receipt_scope_keys != declared_scope_keys || receipt_scope_keys.len() != receipts.len() {
        fail(&mut findings, "receipt_scope_key_set");
    }
    let mut scope_states = BTreeMap::new();
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
        let control_digest = fact
            .get("control_plane_entry_sha256")
            .and_then(Value::as_str);
        if !control_digest.is_some_and(|digest| protected_control_entries.contains(digest)) {
            fail(&mut findings, "control_plane_entry.protected_binding");
        }
        let path = fact.get("receipt_path").and_then(Value::as_str);
        let state = fact.get("receipt_state").and_then(Value::as_str);
        let scope = receipt
            .get("scope_ref")
            .and_then(Value::as_str)
            .unwrap_or("");
        if scope_states
            .insert(scope.to_owned(), state.unwrap_or("").to_owned())
            .is_some()
        {
            fail(&mut findings, "receipt_scope_key_set");
        }
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
                if state == Some("closed-carried") {
                    let predecessor = child(fact, "predecessor_context");
                    let path = predecessor.get("receipt_path").and_then(Value::as_str);
                    let blob = predecessor.get("receipt_blob_oid").and_then(Value::as_str);
                    if path
                        .and_then(|path| protected_preparations.get(path))
                        .map(|preparation| preparation.blob_oid.as_str())
                        != blob
                        || path
                            .and_then(|path| protected_preparations.get(path))
                            .is_some_and(|preparation| {
                                receipt
                                    .pointer("/baseline/commit_oid")
                                    .and_then(Value::as_str)
                                    != Some(preparation.baseline_commit_oid.as_str())
                                    || receipt
                                        .pointer("/baseline/tree_oid")
                                        .and_then(Value::as_str)
                                        != Some(preparation.baseline_tree_oid.as_str())
                                    || predecessor.get("commit_oid").and_then(Value::as_str)
                                        != Some(preparation.baseline_commit_oid.as_str())
                                    || predecessor.get("tree_oid").and_then(Value::as_str)
                                        != Some(preparation.baseline_tree_oid.as_str())
                                    || predecessor.get("commit_oid")
                                        == protected_context.get("evaluated_commit_oid")
                                    || predecessor.get("tree_oid")
                                        == protected_context.get("evaluated_tree_oid")
                            })
                    {
                        fail(&mut findings, "closed_carried_predecessor_link");
                    }
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
                    .map(|preparation| preparation.blob_oid.as_str())
                    != preparation_blob
                    || receipt
                        .pointer("/baseline/commit_oid")
                        .and_then(Value::as_str)
                        != preparation_path
                            .and_then(|path| protected_preparations.get(path))
                            .map(|preparation| preparation.baseline_commit_oid.as_str())
                    || receipt
                        .pointer("/baseline/tree_oid")
                        .and_then(Value::as_str)
                        != preparation_path
                            .and_then(|path| protected_preparations.get(path))
                            .map(|preparation| preparation.baseline_tree_oid.as_str())
                    || fact.get("baseline_commit_oid") != receipt.pointer("/baseline/commit_oid")
                    || fact.get("baseline_tree_oid") != receipt.pointer("/baseline/tree_oid")
                    || receipt.pointer("/baseline/commit_oid")
                        == protected_context.get("evaluated_commit_oid")
                    || receipt.pointer("/baseline/tree_oid")
                        == protected_context.get("evaluated_tree_oid")
                    || fact.pointer("/predecessor_context/commit_oid")
                        != receipt.pointer("/baseline/commit_oid")
                    || fact.pointer("/predecessor_context/tree_oid")
                        != receipt.pointer("/baseline/tree_oid")
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
    if !closed_preparation_paths.is_empty()
        && closed_preparation_paths != protected_preparation_paths
    {
        fail(&mut findings, "closure_preparation_ambiguous");
    }
    for (scope, expected) in required {
        let carried_paths = carried_covered.get(&scope).cloned().unwrap_or_default();
        let prepared_paths = prepared_covered.get(&scope).cloned().unwrap_or_default();
        if carried_paths.is_empty() == prepared_paths.is_empty()
            || (!carried_paths.is_empty() && carried_paths != expected.predecessor)
            || (!prepared_paths.is_empty() && prepared_paths != expected.predecessor)
        {
            fail(
                &mut findings,
                &format!("scope_bidirectional_coverage.{scope}"),
            );
        }
        let state = scope_states.get(&scope).map(String::as_str);
        let predecessor_matches_diff = expected.selectors.iter().all(|selector| match state {
            Some("prepared-new") => selector.predecessor == selector.surviving,
            Some("carried" | "closure-new") => selector.predecessor == selector.removed,
            // A closed carried receipt may be verified only against an immutable
            // historical predecessor once both current trees no longer contain it.
            Some("closed-carried") => {
                selector.predecessor == selector.removed
                    || (selector.protected.is_empty() && selector.candidate.is_empty())
            }
            _ => false,
        });
        if !predecessor_matches_diff {
            fail(
                &mut findings,
                &format!("scope_diff_predecessor_binding.{scope}"),
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
    for (key, value) in [
        ("repository_effect", "history-only"),
        ("runtime_effect", "none"),
        ("roadmap_effect", "none"),
        ("planning_hold_effect", "HOLD(Planning)"),
    ] {
        if effects.get(key).and_then(Value::as_str) != Some(value) {
            fail(
                findings,
                if state == Some("closure-new") {
                    "closure_claim_ceiling"
                } else {
                    "retirement_no_effect_tuple"
                },
            );
        }
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
        let predecessor_matches = fact_input
            .get("predecessor_path_exists")
            .and_then(Value::as_bool)
            == Some(true)
            && fact_input
                .get("predecessor_path_kind")
                .and_then(Value::as_str)
                == Some("regular")
            && fact_input.get("predecessor_blob_oid") == input.get("predecessor_blob_oid")
            && fact_input.get("predecessor_sha256") == input.get("sha256")
            && fact_input.get("predecessor_byte_count") == input.get("byte_count")
            && fact_input.get("mode").and_then(Value::as_str) == Some("100644")
            && fact_input.get("predecessor_mode").and_then(Value::as_str) == Some("100644");
        if !predecessor_matches {
            fail(
                findings,
                &format!("retired_inputs[{index}].immutable_predecessor_binding"),
            );
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
        let protected_absent = path_snapshot_absent(fact_input, "protected");
        let current_state_matches = match state {
            Some("prepared-new") => protected_matches && candidate_matches,
            Some("closure-new") => protected_matches && candidate_absent,
            Some("closed-carried") => protected_absent && candidate_absent,
            _ => false,
        };
        if !no_equivalent_copy || !current_state_matches {
            fail(
                findings,
                &format!("retired_inputs[{index}].candidate_readable_copy"),
            );
        }
    }
}

fn path_snapshot_absent(input: &Value, prefix: &str) -> bool {
    input
        .get(format!("{prefix}_path_exists"))
        .and_then(Value::as_bool)
        == Some(false)
        && ["path_kind", "blob_oid", "sha256", "byte_count", "mode"]
            .iter()
            .all(|field| {
                input
                    .get(format!("{prefix}_{field}"))
                    .is_some_and(Value::is_null)
            })
}

fn validate_predecessor_context(
    fact: &Value,
    state: Option<&str>,
    receipt: &Value,
    findings: &mut BTreeSet<Finding>,
) {
    let context = child(fact, "predecessor_context");
    closed_object(
        context,
        PREDECESSOR_CONTEXT_FIELDS,
        "predecessor_context",
        findings,
    );
    let source = context.get("source").and_then(Value::as_str);
    let baseline = child(receipt, "baseline");
    let expected = match state {
        Some("prepared-new") => Some("current-protected-base"),
        Some("closure-new") => Some("protected-preparation-receipt"),
        Some("closed-carried") => Some("linked-preparation-history"),
        _ => None,
    };
    let commit = oid(
        context,
        "commit_oid",
        "predecessor_context.commit_oid",
        findings,
    );
    let tree = oid(
        context,
        "tree_oid",
        "predecessor_context.tree_oid",
        findings,
    );
    if source != expected || commit.is_none() || tree.is_none() {
        fail(findings, "predecessor_context");
        return;
    }
    let context_receipt_path = context.get("receipt_path");
    let context_receipt_blob = context.get("receipt_blob_oid");
    let no_link = context_receipt_path.is_some_and(Value::is_null)
        && context_receipt_blob.is_some_and(Value::is_null);
    match state {
        Some("prepared-new") => {
            if !no_link
                || context.get("commit_oid") != baseline.get("commit_oid")
                || context.get("tree_oid") != baseline.get("tree_oid")
            {
                fail(findings, "predecessor_context.binding");
            }
        }
        Some("closure-new" | "closed-carried") => {
            let link_path = fact.get("preparation_receipt_path");
            let link_blob = fact.get("protected_preparation_receipt_blob_oid");
            if context_receipt_path != link_path
                || context_receipt_blob != link_blob
                || !context_receipt_path
                    .and_then(Value::as_str)
                    .is_some_and(repo_path)
                || !context_receipt_blob
                    .and_then(Value::as_str)
                    .is_some_and(|value| {
                        value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
                    })
            {
                fail(findings, "predecessor_context.link");
            }
        }
        _ => (),
    }
}

fn validate_control_plane_entry(
    fact: &Value,
    scope: Option<&str>,
    findings: &mut BTreeSet<Finding>,
) {
    let entry = child(fact, "control_plane_entry");
    closed_object(
        entry,
        CONTROL_PLANE_ENTRY_FIELDS,
        "control_plane_entry",
        findings,
    );
    if entry.get("scope_ref").and_then(Value::as_str) != scope
        || entry.get("scope_type").and_then(Value::as_str) != scope.and_then(scope_type)
        || entry.get("selectors").and_then(Value::as_array).is_none()
        || !sha256(
            fact.get("control_plane_entry_sha256")
                .and_then(Value::as_str),
        )
        || control_plane_entry_digest(entry).as_deref()
            != fact
                .get("control_plane_entry_sha256")
                .and_then(Value::as_str)
    {
        fail(findings, "control_plane_entry.binding");
    }
    validate_fixed_control_plane_mapping(entry, findings);
}

fn validate_fixed_control_plane_mapping(entry: &Value, findings: &mut BTreeSet<Finding>) {
    let is_adr_0388 = entry.get("scope_ref").and_then(Value::as_str) == Some("ADR-0388");
    let expected = if is_adr_0388 {
        [
            ("evidence_set_id", Some(ADR_0388_EVIDENCE_SET_ID)),
            (
                "preparation_artifact_id",
                Some(ADR_0388_PREPARATION_ARTIFACT_ID),
            ),
            ("preparation_receipt_path", Some(ADR_0388_PREPARATION_PATH)),
            ("closure_artifact_id", Some(ADR_0388_CLOSURE_ARTIFACT_ID)),
            ("closure_receipt_path", Some(ADR_0388_CLOSURE_PATH)),
        ]
    } else {
        [
            ("evidence_set_id", None),
            ("preparation_artifact_id", None),
            ("preparation_receipt_path", None),
            ("closure_artifact_id", None),
            ("closure_receipt_path", None),
        ]
    };
    for (key, value) in expected {
        let matches = value.map_or_else(
            || entry.get(key).is_some_and(Value::is_null),
            |value| entry.get(key).and_then(Value::as_str) == Some(value),
        );
        if !matches {
            fail(findings, "control_plane_entry.fixed_mapping");
        }
    }
    if is_adr_0388 {
        let expected_paths = immutable_idea_archive_baseline()
            .ok()
            .map(|baseline| {
                baseline
                    .entries
                    .iter()
                    .map(|row| format!("{}/{}", baseline.scope_root, row.path))
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        if expected_paths.is_empty()
            || path_set(
                entry.get("selectors"),
                "control_plane_entry.selectors",
                findings,
            ) != expected_paths
        {
            fail(findings, "control_plane_entry.fixed_mapping");
        }
    }
}

fn validate_fixed_receipt_identity(
    receipt_path: &str,
    receipt: &Value,
    fact: &Value,
    state: Option<&str>,
    findings: &mut BTreeSet<Finding>,
) {
    if receipt.get("scope_ref").and_then(Value::as_str) != Some("ADR-0388") {
        return;
    }
    let (artifact_id, expected_path, requires_preparation_link) = match state {
        Some("prepared-new") => (
            ADR_0388_PREPARATION_ARTIFACT_ID,
            ADR_0388_PREPARATION_PATH,
            false,
        ),
        Some("closure-new" | "closed-carried") => {
            (ADR_0388_CLOSURE_ARTIFACT_ID, ADR_0388_CLOSURE_PATH, true)
        }
        _ => return,
    };
    if receipt_path != expected_path
        || receipt.get("artifact_id").and_then(Value::as_str) != Some(artifact_id)
        || fact.get("receipt_path").and_then(Value::as_str) != Some(expected_path)
        || (requires_preparation_link
            && fact.get("preparation_receipt_path").and_then(Value::as_str)
                != Some(ADR_0388_PREPARATION_PATH))
    {
        fail(findings, "adr_0388_fixed_receipt_identity");
    }
}

fn control_plane_entry_digest(entry: &Value) -> Option<String> {
    crate::canonical_json(entry)
        .map(|canonical| format!("sha256:{:x}", Sha256::digest(canonical.as_bytes())))
}

struct ScopeDiff {
    predecessor: BTreeSet<String>,
    selectors: Vec<SelectorDiff>,
}

struct SelectorDiff {
    predecessor: BTreeSet<String>,
    protected: BTreeSet<String>,
    candidate: BTreeSet<String>,
    removed: BTreeSet<String>,
    surviving: BTreeSet<String>,
}

fn validate_scopes(
    coverage: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, ScopeDiff> {
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
        let mut predecessor = BTreeSet::new();
        let selectors = scope
            .get("selectors")
            .and_then(Value::as_array)
            .map_or(&[][..], Vec::as_slice);
        let mut selector_diffs = Vec::with_capacity(selectors.len());
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
            let predecessor_paths = path_set(
                selector.get("predecessor_paths"),
                "predecessor_paths",
                findings,
            );
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
                    || !predecessor_paths.is_empty()
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
            selector_diffs.push(SelectorDiff {
                predecessor: predecessor_paths.clone(),
                protected: p,
                candidate: c,
                removed: r.clone(),
                surviving: s,
            });
            predecessor.extend(predecessor_paths);
        }
        let declared = path_set(
            scope.get("required_retired_paths"),
            "required_retired_paths",
            findings,
        );
        if declared != predecessor {
            fail(findings, &format!("scopes[{i}].required_retired_paths"));
        }
        if !union.is_disjoint(&declared) {
            fail(findings, "scope_retired_path_overlap");
        }
        union.extend(declared);
        result.insert(
            reference.to_owned(),
            ScopeDiff {
                predecessor,
                selectors: selector_diffs,
            },
        );
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
        && path
            .split('/')
            .all(|component| !component.is_empty() && component != "." && component != "..")
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
#[derive(Debug, Clone)]
struct ProtectedPreparationReceipt {
    blob_oid: String,
    baseline_commit_oid: String,
    baseline_tree_oid: String,
}

fn protected_preparation_receipts(
    value: Option<&Value>,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, ProtectedPreparationReceipt> {
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
        let commit = oid(
            receipt,
            "baseline_commit_oid",
            &format!("protected_preparation_receipts[{index}].baseline_commit_oid"),
            findings,
        );
        let tree = oid(
            receipt,
            "baseline_tree_oid",
            &format!("protected_preparation_receipts[{index}].baseline_tree_oid"),
            findings,
        );
        if let (Some(path), Some(blob_oid), Some(baseline_commit_oid), Some(baseline_tree_oid)) =
            (path, blob, commit, tree)
        {
            if result
                .insert(
                    path.to_owned(),
                    ProtectedPreparationReceipt {
                        blob_oid,
                        baseline_commit_oid,
                        baseline_tree_oid,
                    },
                )
                .is_some()
            {
                fail(findings, "protected_preparation_receipts");
            }
        } else {
            fail(findings, "protected_preparation_receipts");
        }
    }
    result
}

fn control_plane_entries(
    value: Option<&Value>,
    findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    let Some(entries) = value.and_then(Value::as_array) else {
        fail(findings, "control_plane_entries");
        return BTreeSet::new();
    };
    let mut digests = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        closed_object(
            entry,
            CONTROL_PLANE_ENTRY_FIELDS,
            &format!("control_plane_entries[{index}]"),
            findings,
        );
        let scope = entry.get("scope_ref").and_then(Value::as_str);
        if scope.is_none()
            || entry.get("scope_type").and_then(Value::as_str) != scope.and_then(scope_type)
            || entry.get("selectors").and_then(Value::as_array).is_none()
        {
            fail(findings, "control_plane_entries.binding");
            continue;
        }
        validate_fixed_control_plane_mapping(entry, findings);
        let Some(digest) = control_plane_entry_digest(entry) else {
            fail(findings, "control_plane_entries.digest");
            continue;
        };
        if !digests.insert(digest) {
            fail(findings, "control_plane_entries.duplicate");
        }
    }
    digests
}

fn validate_retirement_control_plane_context(
    context: &Value,
    receipts: &[Value],
    object_facts: &[Value],
    findings: &mut BTreeSet<Finding>,
) {
    if context.get("control_plane_path").and_then(Value::as_str)
        != Some(RETIREMENT_CONTROL_PLANE_PATH)
    {
        fail(
            findings,
            "retirement_control_plane_context.control_plane_path",
        );
    }
    let protected_entries = context
        .get("control_plane_entries")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let scopes = protected_entries
        .iter()
        .filter_map(|entry| entry.get("scope_ref").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    let expected = BTreeSet::from(["ADR-0363", "ADR-0388", "artifact:masterplan"]);
    if scopes != expected || protected_entries.len() != expected.len() {
        fail(
            findings,
            "retirement_control_plane_context.fixed_three_scope_rows",
        );
    }
    let protected_oid = context.get("protected_control_plane_blob_oid");
    let protected_sha256 = context.get("protected_control_plane_sha256");
    let protected_bytes = context.get("protected_control_plane_byte_count");
    let candidate_oid = context.get("candidate_control_plane_blob_oid");
    let candidate_sha256 = context.get("candidate_control_plane_sha256");
    let candidate_bytes = context.get("candidate_control_plane_byte_count");
    let empty = receipts.is_empty() && object_facts.is_empty();
    let null_binding = |oid: Option<&Value>, sha256: Option<&Value>, bytes: Option<&Value>| {
        oid.is_some_and(Value::is_null)
            && sha256.is_some_and(Value::is_null)
            && bytes.is_some_and(Value::is_null)
    };
    let candidate_binding = || {
        oid_value(candidate_oid)
            && sha256(candidate_sha256.and_then(Value::as_str))
            && candidate_bytes.and_then(Value::as_u64).is_some()
    };
    let immutable_binding = || {
        oid_value(protected_oid)
            && protected_oid == candidate_oid
            && sha256(protected_sha256.and_then(Value::as_str))
            && protected_sha256 == candidate_sha256
            && protected_bytes.and_then(Value::as_u64).is_some()
            && candidate_bytes.and_then(Value::as_u64) == protected_bytes.and_then(Value::as_u64)
    };
    if empty {
        if context.get("bootstrap").and_then(Value::as_bool) == Some(true) {
            if !null_binding(protected_oid, protected_sha256, protected_bytes)
                || !candidate_binding()
            {
                fail(findings, "retirement_control_plane_context.dormant_empty");
            }
        } else if !immutable_binding() {
            fail(findings, "retirement_control_plane_context.dormant_empty");
        }
    } else if !immutable_binding() {
        fail(
            findings,
            "retirement_control_plane_context.immutable_blob_binding",
        );
    }
}

fn oid_value(value: Option<&Value>) -> bool {
    value.and_then(Value::as_str).is_some_and(|value| {
        value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
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
        json!({"$schema":"https://json-schema.org/draft/2020-12/schema","artifact_id":id,"artifact_type":"migration-closure-receipt","status":"history-only-retired-nonauthoritative","recorded_at":"2026-07-22","scope_ref":scope,"authority":{"decisions":[authority],"planning_state":"HOLD(Planning)","dispatch_authorized":false,"completion_claims_promoted":0},"baseline":{"commit_oid":commit,"tree_oid":tree},"retired_inputs":[{"path":path,"mode":"100644","predecessor_blob_oid":BLOB,"sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","byte_count":1,"successor_refs":[authority],"disposition":"retired-git-history-only"}],"provenance":{"content_store":"authorized Git object history only","readable_tracked_copy_retained":false,"readable_archive_directory_retained":false,"tombstone_content_retained":false,"receipt_reproduces_retired_content":false},"verification_contract":{"expected_absent_paths":[path],"expected_tracked_readable_archive_directory_count":0,"required_gates":REQUIRED_GATES},"effects":{"repository_effect":"history-only","runtime_effect":"none","roadmap_effect":"none","planning_hold_effect":"HOLD(Planning)"}})
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
        let current_present = matches!(state, "prepared-new" | "closure-new");
        let source = match state {
            "prepared-new" => "control-plane-predecessor",
            "closure-new" => "protected-preparation-receipt",
            "carried" => "receipt-baseline",
            "closed-carried" => "linked-preparation-history",
            _ => "invalid",
        };
        let entry = test_control_entry(scope);
        let mut fact = json!({"artifact_id":id,"receipt_path":receipt_path,"protected_base_ref":"origin/dev","receipt_state":state,"scope_ref":scope,"scope_type":scope_type(scope).unwrap(),"baseline_commit_oid":commit,"baseline_tree_oid":tree,"protected_receipt_blob_oid":if prepared { Value::Null } else { json!(BLOB) },"candidate_receipt_blob_oid":BLOB,"protected_registry_row_sha256":if prepared { Value::Null } else { json!("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa") },"candidate_registry_row_sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","retired_inputs":[{"path":path,"mode":"100644","predecessor_blob_oid":BLOB,"sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","byte_count":1,"predecessor_path_exists":true,"predecessor_path_kind":"regular","predecessor_sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","predecessor_byte_count":1,"predecessor_mode":"100644","protected_path_exists":current_present,"protected_path_kind":if current_present { json!("regular") } else { Value::Null },"protected_blob_oid":if current_present { json!(BLOB) } else { Value::Null },"protected_sha256":if current_present { json!("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa") } else { Value::Null },"protected_byte_count":if current_present { json!(1) } else { Value::Null },"protected_mode":if current_present { json!("100644") } else { Value::Null },"candidate_path_exists":prepared,"candidate_path_kind":if prepared { json!("regular") } else { Value::Null },"candidate_blob_oid":if prepared { json!(BLOB) } else { Value::Null },"candidate_sha256":if prepared { json!("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa") } else { Value::Null },"candidate_byte_count":if prepared { json!(1) } else { Value::Null },"candidate_mode":if prepared { json!("100644") } else { Value::Null },"candidate_new_equivalent_paths":[],"candidate_equivalent_paths":[]} ],"predecessor_context":{"source":source,"commit_oid":commit,"tree_oid":tree,"receipt_path":Value::Null,"receipt_blob_oid":Value::Null},"control_plane_entry":entry});
        let digest = control_plane_entry_digest(&fact["control_plane_entry"]).expect("test entry");
        fact["control_plane_entry_sha256"] = json!(digest);
        fact
    }
    fn protected_scm_context(prepared_receipt_paths: &[&str]) -> Value {
        let controls = ["ADR-0363", "ADR-0388", "artifact:masterplan"]
            .into_iter()
            .map(test_control_entry)
            .collect::<Vec<_>>();
        json!({"protected_base_ref":"origin/dev","protected_base_commit_oid":OLD,"protected_base_tree_oid":OLD_TREE,"evaluated_commit_oid":NEW,"evaluated_tree_oid":NEW_TREE,"subject_commit_oid":NEW,"subject_tree_oid":NEW_TREE,"scm_event_name":"push","subject_relationship":"evaluated-self","protected_base_is_ancestor_of_evaluated":true,"protected_base_is_evaluated_first_parent":true,"subject_is_evaluated_second_parent":false,"predecessor_commit_oid":OLD,"predecessor_tree_oid":OLD_TREE,"predecessor_commit_exists":true,"predecessor_tree_exists":true,"predecessor_commit_tree_bound":true,"predecessor_is_ancestor_of_protected_base":true,"prepared_receipt_paths":prepared_receipt_paths,"protected_preparation_receipts":[],"control_plane_entries":controls})
    }
    fn retirement_control_plane_context(active: bool) -> Value {
        let controls = ["ADR-0363", "ADR-0388", "artifact:masterplan"]
            .into_iter()
            .map(test_control_entry)
            .collect::<Vec<_>>();
        let hashes = controls.iter().map(|entry| json!({"scope_ref":entry["scope_ref"],"sha256":control_plane_entry_digest(entry).expect("test control digest")})).collect::<Vec<_>>();
        json!({"control_plane_path":"registry/history-only-retirement/control-plane.json","receipt_root":"evidence/history-only-retirement","bootstrap":!active,"lifecycle_state":if active { "prepared-new" } else { "dormant" },"protected_control_plane_blob_oid":if active { json!(BLOB) } else { Value::Null },"protected_control_plane_sha256":if active { json!("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa") } else { Value::Null },"protected_control_plane_byte_count":if active { json!(1) } else { Value::Null },"candidate_control_plane_blob_oid":json!(BLOB),"candidate_control_plane_sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","candidate_control_plane_byte_count":1,"control_plane_entries":controls,"control_plane_entry_hashes":hashes,"protected_receipt_root_paths":[],"candidate_receipt_root_paths":[],"unexpected_protected_receipt_paths":[],"unexpected_candidate_receipt_paths":[]})
    }
    fn test_control_entry(scope: &str) -> Value {
        let adr_0388 = scope == "ADR-0388";
        let selectors = if adr_0388 {
            crate::immutable_idea_archive_baseline()
                .expect("E4 baseline")
                .entries
                .iter()
                .map(|row| format!("docs/ideas/archive/{}", row.path))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        json!({"evidence_set_id":if adr_0388 { json!(ADR_0388_EVIDENCE_SET_ID) } else { Value::Null },"scope_ref":scope,"scope_type":scope_type(scope).unwrap(),"selectors":selectors,"preparation_artifact_id":if adr_0388 { json!(ADR_0388_PREPARATION_ARTIFACT_ID) } else { Value::Null },"preparation_receipt_path":if adr_0388 { json!(ADR_0388_PREPARATION_PATH) } else { Value::Null },"closure_artifact_id":if adr_0388 { json!(ADR_0388_CLOSURE_ARTIFACT_ID) } else { Value::Null },"closure_receipt_path":if adr_0388 { json!(ADR_0388_CLOSURE_PATH) } else { Value::Null }})
    }
    fn closure_receipt(
        id: &str,
        scope: &str,
        path: &str,
        preparation_receipt_path: &str,
        preparation_receipt_blob_oid: &str,
    ) -> Value {
        let mut receipt = receipt(id, scope, OLD, OLD_TREE, path);
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
        let mut fact = fact(id, receipt_path, scope, "closure-new", OLD, OLD_TREE, path);
        fact["protected_receipt_blob_oid"] = Value::Null;
        fact["protected_registry_row_sha256"] = Value::Null;
        fact["preparation_receipt_path"] = json!(preparation_receipt_path);
        fact["protected_preparation_receipt_blob_oid"] = json!(preparation_receipt_blob_oid);
        fact["predecessor_context"]["receipt_path"] = json!(preparation_receipt_path);
        fact["predecessor_context"]["receipt_blob_oid"] = json!(preparation_receipt_blob_oid);
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
            "baseline_commit_oid": OLD,
            "baseline_tree_oid": OLD_TREE,
        }]);
        context
    }

    fn exact_adr_0388_corpus(state: &str) -> Value {
        let baseline = crate::immutable_idea_archive_baseline().expect("E4 baseline");
        let paths = baseline
            .entries
            .iter()
            .map(|entry| format!("{}/{}", baseline.scope_root, entry.path))
            .collect::<Vec<_>>();
        let receipt_path = if state == "prepared-new" {
            ADR_0388_PREPARATION_PATH
        } else {
            ADR_0388_CLOSURE_PATH
        };
        let artifact_id = if state == "prepared-new" {
            ADR_0388_PREPARATION_ARTIFACT_ID
        } else {
            ADR_0388_CLOSURE_ARTIFACT_ID
        };
        let mut receipt = if state == "prepared-new" {
            prepared_receipt(artifact_id, "ADR-0388", OLD, OLD_TREE, &paths[0])
        } else if state == "closure-new" {
            closure_receipt(
                artifact_id,
                "ADR-0388",
                &paths[0],
                ADR_0388_PREPARATION_PATH,
                BLOB,
            )
        } else {
            receipt(artifact_id, "ADR-0388", OLD, OLD_TREE, &paths[0])
        };
        receipt["retired_inputs"] = Value::Array(
            baseline
                .entries
                .iter()
                .zip(&paths)
                .map(|(entry, path)| {
                    json!({
                        "path": path,
                        "predecessor_blob_oid": entry.blob_oid,
                        "sha256": format!("sha256:{}", entry.sha256),
                        "byte_count": entry.byte_length,
                        "successor_refs": ["ADR-0388"],
                        "disposition": if state == "prepared-new" {
                            "prepared-for-history-only-retirement"
                        } else {
                            "retired-git-history-only"
                        }
                    })
                })
                .collect(),
        );
        if state != "prepared-new" {
            receipt["verification_contract"]["expected_absent_paths"] = json!(paths);
        }

        let mut fact = if state == "closure-new" {
            closure_fact(
                artifact_id,
                receipt_path,
                "ADR-0388",
                &paths[0],
                ADR_0388_PREPARATION_PATH,
                BLOB,
            )
        } else {
            fact(
                artifact_id,
                receipt_path,
                "ADR-0388",
                state,
                OLD,
                OLD_TREE,
                &paths[0],
            )
        };
        if state == "closed-carried" {
            fact["preparation_receipt_path"] = json!(ADR_0388_PREPARATION_PATH);
            fact["protected_preparation_receipt_blob_oid"] = json!(BLOB);
            fact["predecessor_context"]["receipt_path"] = json!(ADR_0388_PREPARATION_PATH);
            fact["predecessor_context"]["receipt_blob_oid"] = json!(BLOB);
        }
        fact["predecessor_context"]["commit_exists"] = json!(true);
        fact["predecessor_context"]["tree_exists"] = json!(true);
        fact["predecessor_context"]["commit_tree_oid"] = json!(OLD_TREE);
        fact["predecessor_context"]["is_ancestor_of_candidate"] = json!(true);
        fact["retired_inputs"] = Value::Array(
            baseline
                .entries
                .iter()
                .zip(&paths)
                .map(|(entry, path)| {
                    let protected_present = matches!(state, "prepared-new" | "closure-new");
                    let candidate_present = state == "prepared-new";
                    json!({
                        "path": path,
                        "predecessor_blob_oid": entry.blob_oid,
                        "sha256": format!("sha256:{}", entry.sha256),
                        "byte_count": entry.byte_length,
                        "predecessor_path_exists": true,
                        "predecessor_path_kind": "regular",
                        "predecessor_sha256": format!("sha256:{}", entry.sha256),
                        "predecessor_byte_count": entry.byte_length,
                        "predecessor_mode": "100644",
                        "predecessor_tree_oid": OLD_TREE,
                        "protected_path_exists": protected_present,
                        "protected_path_kind": protected_present.then_some("regular"),
                        "protected_blob_oid": protected_present.then_some(entry.blob_oid.as_str()),
                        "protected_sha256": protected_present.then_some(format!("sha256:{}", entry.sha256)),
                        "protected_byte_count": protected_present.then_some(entry.byte_length),
                        "protected_mode": protected_present.then_some("100644"),
                        "candidate_path_exists": candidate_present,
                        "candidate_path_kind": candidate_present.then_some("regular"),
                        "candidate_blob_oid": candidate_present.then_some(entry.blob_oid.as_str()),
                        "candidate_sha256": candidate_present.then_some(format!("sha256:{}", entry.sha256)),
                        "candidate_byte_count": candidate_present.then_some(entry.byte_length),
                        "candidate_mode": candidate_present.then_some("100644"),
                        "candidate_new_equivalent_paths": [],
                        "candidate_equivalent_paths": []
                    })
                })
                .collect(),
        );

        let selectors = paths
            .iter()
            .map(|path| {
                json!({
                    "selector_type": "exact",
                    "selector": path,
                    "protected_paths": [path],
                    "predecessor_paths": [path],
                    "candidate_paths": if state == "prepared-new" { json!([path]) } else { json!([]) },
                    "removed_paths": if state == "prepared-new" { json!([]) } else { json!([path]) },
                    "surviving_paths": if state == "prepared-new" { json!([path]) } else { json!([]) },
                    "candidate_only_paths": [],
                    "external_assertion": "not-applicable"
                })
            })
            .collect::<Vec<_>>();
        let mut context = if state == "prepared-new" {
            protected_scm_context(&[receipt_path])
        } else {
            protected_scm_context_with_preparation(ADR_0388_PREPARATION_PATH, BLOB)
        };
        let control_context = retirement_control_plane_context(true);

        json!({
            "receipts": [{"receipt_path": receipt_path, "receipt": receipt}],
            "scm_facts": {
                "retirement_receipt_coverage": {
                    "protected_base_ref": "origin/dev",
                    "protected_receipt_paths": match state {
                        "prepared-new" => json!([]),
                        "closure-new" => json!([ADR_0388_PREPARATION_PATH]),
                        _ => json!([receipt_path]),
                    },
                    "candidate_receipt_paths": [receipt_path],
                    "carried_receipt_paths": if state == "closed-carried" { json!([receipt_path]) } else { json!([]) },
                    "new_receipt_paths": if state == "closed-carried" { json!([]) } else { json!([receipt_path]) },
                    "scopes": [{
                        "scope_ref": "ADR-0388",
                        "scope_type": "transient-ideas",
                        "selectors": selectors,
                        "required_retired_paths": paths
                    }],
                    "required_retired_paths": paths
                },
                "retirement_receipt_object_facts": [fact],
                "protected_scm_context": context,
                "retirement_control_plane_context": control_context
            }
        })
    }
    #[test]
    fn exact_adr_0388_closure_is_dormant_and_does_not_affect_the_ordinary_gate() {
        let corpus = exact_adr_0388_corpus("closure-new");
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
            json!("invalid");
        assert!(
            evaluate_history_only_retirement_receipts(&rebound)
                .iter()
                .any(|finding| finding.code == RETIREMENT_RECEIPT_CODE)
        );
    }

    #[test]
    fn carried_exact_e4_and_new_masterplan_preparation_are_independently_admitted() {
        let mut corpus = exact_adr_0388_corpus("closed-carried");
        let repository_path = "evidence/masterplan-preparation.json";
        let repository = prepared_receipt(
            "masterplan-retirement-preparation",
            "artifact:masterplan",
            OLD,
            OLD_TREE,
            "specs/masterplan-retired-surface.json",
        );
        let repository_fact = fact(
            "masterplan-retirement-preparation",
            repository_path,
            "artifact:masterplan",
            "prepared-new",
            OLD,
            OLD_TREE,
            "specs/masterplan-retired-surface.json",
        );
        let repository_selector = json!({
            "selector_type": "exact",
            "selector": "specs/masterplan-retired-surface.json",
            "protected_paths": ["specs/masterplan-retired-surface.json"],
            "predecessor_paths": ["specs/masterplan-retired-surface.json"],
            "candidate_paths": ["specs/masterplan-retired-surface.json"],
            "removed_paths": [],
            "surviving_paths": ["specs/masterplan-retired-surface.json"],
            "candidate_only_paths": [],
            "external_assertion": "not-applicable"
        });

        corpus["receipts"]
            .as_array_mut()
            .expect("receipt corpus")
            .push(json!({"receipt_path": repository_path, "receipt": repository}));
        let scm_facts = corpus["scm_facts"].as_object_mut().expect("SCM facts");
        let coverage = scm_facts
            .get_mut("retirement_receipt_coverage")
            .expect("coverage")
            .as_object_mut()
            .expect("coverage object");
        coverage
            .get_mut("candidate_receipt_paths")
            .and_then(Value::as_array_mut)
            .expect("candidate receipt paths")
            .push(json!(repository_path));
        coverage
            .get_mut("new_receipt_paths")
            .and_then(Value::as_array_mut)
            .expect("new receipt paths")
            .push(json!(repository_path));
        coverage
            .get_mut("required_retired_paths")
            .and_then(Value::as_array_mut)
            .expect("required retired paths")
            .push(json!("specs/masterplan-retired-surface.json"));
        coverage
            .get_mut("scopes")
            .and_then(Value::as_array_mut)
            .expect("coverage scopes")
            .push(json!({
                "scope_ref": "artifact:masterplan",
                "scope_type": "masterplan-retired-surfaces",
                "selectors": [repository_selector],
                "required_retired_paths": ["specs/masterplan-retired-surface.json"]
            }));
        scm_facts
            .get_mut("retirement_receipt_object_facts")
            .and_then(Value::as_array_mut)
            .expect("object facts")
            .push(repository_fact);
        scm_facts["protected_scm_context"]["prepared_receipt_paths"] = json!([repository_path]);

        let findings = evaluate_history_only_retirement_receipts(&corpus);
        assert!(findings.is_empty(), "{findings:?}");
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
        // Prepared-new and closed-carried mutations regress over declared facts.
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

        for state in ["closed-carried"] {
            let mut carried = fact(
                "state-receipt",
                "evidence/state-receipt.json",
                "ADR-0363",
                state,
                OLD,
                OLD_TREE,
                "docs/old.md",
            );
            if state == "closed-carried" {
                carried["preparation_receipt_path"] = json!("evidence/protected-closure.json");
                carried["protected_preparation_receipt_blob_oid"] = json!(BLOB);
                carried["predecessor_context"]["receipt_path"] =
                    json!("evidence/protected-closure.json");
                carried["predecessor_context"]["receipt_blob_oid"] = json!(BLOB);
            }
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
    fn legacy_carried_coverage_is_rejected_before_scope_checks() {
        let carry_receipt = receipt("carry-only", "ADR-0363", OLD, OLD_TREE, "docs/old.md");
        let receipt_path = "evidence/carry-only.json";
        let selector = json!({"selector_type":"exact","selector":"docs/old.md","protected_paths":["docs/old.md"],"predecessor_paths":["docs/old.md"],"candidate_paths":[],"removed_paths":["docs/old.md"],"surviving_paths":[],"candidate_only_paths":[],"external_assertion":"not-applicable"});
        let facts = json!({"retirement_receipt_coverage":{"protected_base_ref":"origin/dev","protected_receipt_paths":[receipt_path],"candidate_receipt_paths":[receipt_path],"carried_receipt_paths":[receipt_path],"new_receipt_paths":[],"scopes":[{"scope_ref":"ADR-0363","scope_type":"amended-agentic-vcs-retirement","selectors":[selector.clone()],"required_retired_paths":["docs/old.md"]}],"required_retired_paths":["docs/old.md"]},"retirement_receipt_object_facts":[fact("carry-only",receipt_path,"ADR-0363","carried",OLD,OLD_TREE,"docs/old.md")],"protected_scm_context":protected_scm_context(&[]),"retirement_control_plane_context":retirement_control_plane_context(true)});
        assert!(
            evaluate_history_only_retirement_receipt_coverage(&[carry_receipt], &facts).contains(
                &Finding::new(
                    RETIREMENT_RECEIPT_CODE,
                    "retirement_receipt_coverage.legacy_carried"
                )
            )
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

        let selector = json!({"selector_type":"exact","selector":".omc/legacy-a.md","protected_paths":[".omc/legacy-a.md"],"predecessor_paths":[".omc/legacy-a.md"],"candidate_paths":[".omc/legacy-a.md"],"removed_paths":[],"surviving_paths":[".omc/legacy-a.md"],"candidate_only_paths":[],"external_assertion":"not-applicable"});
        let coverage = json!({"retirement_receipt_coverage":{"protected_base_ref":"origin/dev","protected_receipt_paths":[],"candidate_receipt_paths":["evidence/prepared-authority.json"],"carried_receipt_paths":[],"new_receipt_paths":["evidence/prepared-authority.json"],"scopes":[{"scope_ref":"ADR-0363","scope_type":"amended-agentic-vcs-retirement","selectors":[selector],"required_retired_paths":[".omc/legacy-a.md"]}],"required_retired_paths":[".omc/legacy-a.md"]},"retirement_receipt_object_facts":[prepared_fact],"protected_scm_context":protected_scm_context(&["evidence/prepared-authority.json"]),"retirement_control_plane_context":retirement_control_plane_context(true)});
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
        candidate_rebound["protected_scm_context"]["evaluated_commit_oid"] = json!(OLD);
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
        ancestry_rebound["protected_scm_context"]["protected_base_is_ancestor_of_evaluated"] =
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
        for (field, value) in [
            ("scm_event_name", json!("unknown")),
            ("subject_relationship", json!("pull-request-head")),
            ("subject_is_evaluated_second_parent", json!(true)),
        ] {
            let mut mismatched = coverage.clone();
            mismatched["protected_scm_context"][field] = value;
            assert!(
                evaluate_history_only_retirement_receipt_coverage(
                    std::slice::from_ref(&prepared),
                    &mismatched
                )
                .contains(&Finding::new(
                    RETIREMENT_RECEIPT_CODE,
                    "protected_scm_context.binding"
                )),
                "event identity mismatch for {field} must fail closed"
            );
        }
        let mut candidate_self_proof = coverage.clone();
        candidate_self_proof["retirement_receipt_coverage"]["evaluated_commit_oid"] = json!(OLD);
        candidate_self_proof["retirement_receipt_coverage"]["evaluated_tree_oid"] = json!(OLD_TREE);
        candidate_self_proof["retirement_receipt_object_facts"][0]["evaluated_commit_oid"] =
            json!(OLD);
        candidate_self_proof["retirement_receipt_object_facts"][0]["evaluated_tree_oid"] =
            json!(OLD_TREE);
        candidate_self_proof["retirement_receipt_object_facts"][0]["protected_base_is_ancestor_of_evaluated"] =
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
                "protected_scm_context": protected_scm_context(&[]),
                "retirement_control_plane_context": retirement_control_plane_context(false)
            }
        });
        assert!(
            evaluate_history_only_retirement_receipts(&empty).is_empty(),
            "the dormant no-receipt state must be valid"
        );
        assert!(
            evaluate_history_only_retirement_facts(&empty, &[]).is_empty(),
            "the canonical metadata face admits a dormant state with no raw documents"
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
                            "predecessor_paths": [".omc/legacy-a.md"],
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
                ),
                "retirement_control_plane_context": retirement_control_plane_context(true)
            }
        });
        assert!(
            evaluate_history_only_retirement_receipts(&corpus).is_empty(),
            "a protected preparation must close through a separate linked receipt"
        );

        let mut candidate_predecessor = corpus.clone();
        candidate_predecessor["scm_facts"]["retirement_receipt_object_facts"][0]["predecessor_context"]
            ["commit_oid"] = json!(NEW);
        assert!(
            evaluate_history_only_retirement_receipts(&candidate_predecessor).contains(
                &Finding::new(RETIREMENT_RECEIPT_CODE, "closure_preparation_link")
            )
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
    fn post_e10_closed_carried_uses_immutable_predecessor_not_current_snapshots() {
        const RECEIPT_PATH: &str = "evidence/closed-carried.json";
        const LINK_PATH: &str = "evidence/protected-preparation.json";
        let receipt = receipt(
            "closed-carried",
            "ADR-0363",
            OLD,
            OLD_TREE,
            ".omc/legacy-a.md",
        );
        let mut fact = fact(
            "closed-carried",
            RECEIPT_PATH,
            "ADR-0363",
            "closed-carried",
            OLD,
            OLD_TREE,
            ".omc/legacy-a.md",
        );
        fact["preparation_receipt_path"] = json!(LINK_PATH);
        fact["protected_preparation_receipt_blob_oid"] = json!(BLOB);
        fact["predecessor_context"]["receipt_path"] = json!(LINK_PATH);
        fact["predecessor_context"]["receipt_blob_oid"] = json!(BLOB);
        assert!(
            evaluate_history_only_retirement_receipt(
                RECEIPT_PATH,
                &receipt,
                &json!({"retirement_receipt_object_facts":[fact.clone()]}),
            )
            .is_empty(),
            "post-E10 has both current trees absent while its predecessor remains object-bound"
        );
        let mut effect_claim = receipt.clone();
        effect_claim["effects"]["runtime_effect"] = json!("success");
        assert!(
            evaluate_history_only_retirement_receipt(
                RECEIPT_PATH,
                &effect_claim,
                &json!({"retirement_receipt_object_facts":[fact.clone()]}),
            )
            .contains(&Finding::new(
                RETIREMENT_RECEIPT_CODE,
                "retirement_no_effect_tuple"
            ))
        );

        for (pointer, value) in [
            ("/retired_inputs/0/protected_path_exists", json!(true)),
            ("/retired_inputs/0/predecessor_blob_oid", json!(NEW)),
            (
                "/retired_inputs/0/predecessor_sha256",
                json!("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            ),
            (
                "/predecessor_context/receipt_path",
                json!("evidence/candidate-defined.json"),
            ),
            ("/predecessor_context/receipt_blob_oid", json!(NEW)),
            ("/control_plane_entry/scope_ref", json!("ADR-0388")),
        ] {
            let mut mutated = fact.clone();
            *mutated.pointer_mut(pointer).expect("fixture pointer") = value;
            assert!(
                !evaluate_history_only_retirement_receipt(
                    RECEIPT_PATH,
                    &receipt,
                    &json!({"retirement_receipt_object_facts":[mutated]}),
                )
                .is_empty(),
                "post-E10 anti-lie mutation {pointer} was accepted"
            );
        }

        let selector = json!({"selector_type":"exact","selector":".omc/legacy-a.md","protected_paths":[],"predecessor_paths":[".omc/legacy-a.md"],"candidate_paths":[],"removed_paths":[],"surviving_paths":[],"candidate_only_paths":[],"external_assertion":"not-applicable"});
        let mut context = protected_scm_context(&[]);
        context["protected_preparation_receipts"] = json!([{
            "receipt_path":LINK_PATH,
            "receipt_blob_oid":BLOB,
            "baseline_commit_oid":OLD,
            "baseline_tree_oid":OLD_TREE,
        }]);
        let corpus = json!({"receipts":[{"receipt_path":RECEIPT_PATH,"receipt":receipt}],"scm_facts":{"retirement_receipt_coverage":{"protected_base_ref":"origin/dev","protected_receipt_paths":[RECEIPT_PATH],"candidate_receipt_paths":[RECEIPT_PATH],"carried_receipt_paths":[RECEIPT_PATH],"new_receipt_paths":[],"scopes":[{"scope_ref":"ADR-0363","scope_type":"amended-agentic-vcs-retirement","selectors":[selector],"required_retired_paths":[".omc/legacy-a.md"]}],"required_retired_paths":[".omc/legacy-a.md"]},"retirement_receipt_object_facts":[fact],"protected_scm_context":context,"retirement_control_plane_context":retirement_control_plane_context(true)}});
        let corpus_findings = evaluate_history_only_retirement_receipts(&corpus);
        assert!(corpus_findings.is_empty(), "{corpus_findings:?}");
        let mut candidate_predecessor = corpus.clone();
        candidate_predecessor["scm_facts"]["retirement_receipt_object_facts"][0]["predecessor_context"]
            ["commit_oid"] = json!(NEW);
        assert!(
            evaluate_history_only_retirement_receipts(&candidate_predecessor).contains(
                &Finding::new(RETIREMENT_RECEIPT_CODE, "closed_carried_predecessor_link")
            )
        );
        let mut missing_object_context = corpus;
        missing_object_context["scm_facts"]["protected_scm_context"]["protected_preparation_receipts"] =
            json!([]);
        assert!(
            evaluate_history_only_retirement_receipts(&missing_object_context).contains(
                &Finding::new(RETIREMENT_RECEIPT_CODE, "closed_carried_predecessor_link")
            )
        );
    }

    #[test]
    fn adr_0388_predeclared_receipt_identity_rejects_aliases() {
        let receipt = prepared_receipt(
            ADR_0388_PREPARATION_ARTIFACT_ID,
            "ADR-0388",
            OLD,
            OLD_TREE,
            "docs/idea-old.md",
        );
        let fact = fact(
            ADR_0388_PREPARATION_ARTIFACT_ID,
            ADR_0388_PREPARATION_PATH,
            "ADR-0388",
            "prepared-new",
            OLD,
            OLD_TREE,
            "docs/idea-old.md",
        );
        assert!(
            evaluate_history_only_retirement_receipt(
                ADR_0388_PREPARATION_PATH,
                &receipt,
                &json!({"retirement_receipt_object_facts":[fact.clone()]}),
            )
            .is_empty()
        );
        let alias = "evidence/history-only-retirement/adr-0388-alias.json";
        let mut alias_fact = fact;
        alias_fact["receipt_path"] = json!(alias);
        assert!(
            evaluate_history_only_retirement_receipt(
                alias,
                &receipt,
                &json!({"retirement_receipt_object_facts":[alias_fact]}),
            )
            .contains(&Finding::new(
                RETIREMENT_RECEIPT_CODE,
                "adr_0388_fixed_receipt_identity"
            ))
        );
    }

    #[test]
    fn coverage_binds_prepared_predecessors_to_surviving_diff_paths() {
        let receipt_path = "evidence/prepared-stale-path.json";
        let receipt = prepared_receipt(
            "prepared-stale-path",
            "ADR-0363",
            OLD,
            OLD_TREE,
            "docs/stale.md",
        );
        let fact = fact(
            "prepared-stale-path",
            receipt_path,
            "ADR-0363",
            "prepared-new",
            OLD,
            OLD_TREE,
            "docs/stale.md",
        );
        let selector = json!({
            "selector_type": "glob",
            "selector": "docs/**",
            "protected_paths": ["docs/live.md"],
            "predecessor_paths": ["docs/stale.md"],
            "candidate_paths": ["docs/live.md"],
            "removed_paths": [],
            "surviving_paths": ["docs/live.md"],
            "candidate_only_paths": [],
            "external_assertion": "not-applicable"
        });
        let facts = json!({
            "retirement_receipt_coverage": {
                "protected_base_ref": "origin/dev",
                "protected_receipt_paths": [],
                "candidate_receipt_paths": [receipt_path],
                "carried_receipt_paths": [],
                "new_receipt_paths": [receipt_path],
                "scopes": [{
                    "scope_ref": "ADR-0363",
                    "scope_type": "amended-agentic-vcs-retirement",
                    "selectors": [selector],
                    "required_retired_paths": ["docs/stale.md"]
                }],
                "required_retired_paths": ["docs/stale.md"]
            },
            "retirement_receipt_object_facts": [fact],
            "protected_scm_context": protected_scm_context(&[receipt_path]),
            "retirement_control_plane_context": retirement_control_plane_context(true)
        });
        assert!(
            evaluate_history_only_retirement_receipt_coverage(&[receipt], &facts).contains(
                &Finding::new(
                    RETIREMENT_RECEIPT_CODE,
                    "scope_diff_predecessor_binding.ADR-0363"
                )
            )
        );
    }

    #[test]
    fn git_path_aliases_are_not_canonical_receipt_paths() {
        for alias in [
            ".",
            "./docs/old.md",
            "docs/./old.md",
            "docs/old.md/",
            "docs/.",
        ] {
            assert!(
                !repo_path(alias),
                "accepted noncanonical Git path alias: {alias}"
            );
        }
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

    #[test]
    fn adr_0388_binds_every_selector_receipt_and_snapshot_to_the_exact_e4_rows() {
        let corpus = exact_adr_0388_corpus("closure-new");
        let evaluation = evaluate_and_project_history_only_retirement_closures(&corpus);
        assert!(evaluation.findings.is_empty(), "{:?}", evaluation.findings);

        for pointer in [
            "/scm_facts/retirement_receipt_coverage/scopes/0/selectors/0/selector",
            "/receipts/0/receipt/retired_inputs/0/path",
            "/receipts/0/receipt/retired_inputs/0/predecessor_blob_oid",
            "/receipts/0/receipt/retired_inputs/0/sha256",
            "/receipts/0/receipt/retired_inputs/0/byte_count",
            "/scm_facts/retirement_receipt_object_facts/0/retired_inputs/0/path",
            "/scm_facts/retirement_receipt_object_facts/0/retired_inputs/0/predecessor_blob_oid",
            "/scm_facts/retirement_receipt_object_facts/0/retired_inputs/0/predecessor_sha256",
            "/scm_facts/retirement_receipt_object_facts/0/retired_inputs/0/predecessor_byte_count",
        ] {
            let mut drifted = corpus.clone();
            *drifted.pointer_mut(pointer).expect("fixture pointer") = match pointer {
                value if value.ends_with("byte_count") => json!(1),
                value if value.ends_with("path") || value.ends_with("selector") => {
                    json!("docs/idea-old.md")
                }
                value if value.ends_with("sha256") => {
                    json!("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                }
                _ => json!(BLOB),
            };
            assert!(
                evaluate_and_project_history_only_retirement_closures(&drifted)
                    .projection
                    .is_none(),
                "drift at {pointer} must not project"
            );
        }

        let mut omitted = corpus.clone();
        omitted["receipts"][0]["receipt"]["retired_inputs"]
            .as_array_mut()
            .expect("inputs")
            .pop();
        assert!(
            evaluate_and_project_history_only_retirement_closures(&omitted)
                .projection
                .is_none()
        );
    }

    #[test]
    fn predecessor_objects_ancestry_links_and_tree_snapshots_are_proven() {
        let corpus = exact_adr_0388_corpus("closure-new");
        for (pointer, value) in [
            (
                "/scm_facts/retirement_receipt_object_facts/0/predecessor_context/commit_exists",
                json!(false),
            ),
            (
                "/scm_facts/retirement_receipt_object_facts/0/predecessor_context/tree_exists",
                json!(false),
            ),
            (
                "/scm_facts/retirement_receipt_object_facts/0/predecessor_context/is_ancestor_of_candidate",
                json!(false),
            ),
            (
                "/scm_facts/retirement_receipt_object_facts/0/predecessor_context/commit_tree_oid",
                json!(NEW_TREE),
            ),
            (
                "/scm_facts/retirement_receipt_object_facts/0/retired_inputs/0/predecessor_tree_oid",
                json!(NEW_TREE),
            ),
            (
                "/scm_facts/protected_scm_context/protected_preparation_receipts/0/baseline_commit_oid",
                json!(NEW),
            ),
            (
                "/scm_facts/retirement_receipt_object_facts/0/predecessor_context/commit_oid",
                json!(NEW),
            ),
        ] {
            let mut drifted = corpus.clone();
            *drifted.pointer_mut(pointer).expect("fixture pointer") = value;
            let evaluation = evaluate_and_project_history_only_retirement_closures(&drifted);
            assert!(!evaluation.findings.is_empty(), "{pointer} must fail");
            assert!(
                evaluation.projection.is_none(),
                "{pointer} must not project"
            );
        }
    }

    #[test]
    fn protected_and_candidate_control_planes_are_independent_complete_and_identical() {
        let corpus = exact_adr_0388_corpus("closure-new");
        for pointer in [
            "/scm_facts/retirement_control_plane_context/protected_control_plane_blob_oid",
            "/scm_facts/retirement_control_plane_context/candidate_control_plane_blob_oid",
            "/scm_facts/retirement_control_plane_context/protected_control_plane_entries",
            "/scm_facts/retirement_control_plane_context/candidate_control_plane_entries",
        ] {
            let mut absent = corpus.clone();
            *absent.pointer_mut(pointer).expect("fixture pointer") = Value::Null;
            assert!(
                evaluate_and_project_history_only_retirement_closures(&absent)
                    .projection
                    .is_none(),
                "absence at {pointer} must fail"
            );
        }

        let mut divergent = corpus;
        divergent["scm_facts"]["retirement_control_plane_context"]["candidate_control_plane_entries"]
            [1]["selectors"][0] = json!("docs/idea-old.md");
        assert!(
            evaluate_and_project_history_only_retirement_closures(&divergent)
                .projection
                .is_none()
        );
    }

    #[test]
    fn control_plane_identity_is_path_hash_and_size_bound() {
        let corpus = exact_adr_0388_corpus("closure-new");
        for (pointer, value) in [
            (
                "/scm_facts/retirement_control_plane_context/control_plane_path",
                json!("registry/history-only-retirement/other-control-plane.json"),
            ),
            (
                "/scm_facts/retirement_control_plane_context/protected_control_plane_sha256",
                json!("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            ),
            (
                "/scm_facts/retirement_control_plane_context/candidate_control_plane_sha256",
                json!("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            ),
            (
                "/scm_facts/retirement_control_plane_context/protected_control_plane_byte_count",
                json!(2),
            ),
            (
                "/scm_facts/retirement_control_plane_context/candidate_control_plane_byte_count",
                json!(2),
            ),
        ] {
            let mut drifted = corpus.clone();
            *drifted.pointer_mut(pointer).expect("fixture pointer") = value;
            assert!(
                evaluate_and_project_history_only_retirement_closures(&drifted)
                    .projection
                    .is_none(),
                "{pointer} mutation must fail"
            );
        }

        for pointer in [
            "/scm_facts/retirement_control_plane_context/control_plane_path",
            "/scm_facts/retirement_control_plane_context/protected_control_plane_sha256",
            "/scm_facts/retirement_control_plane_context/candidate_control_plane_sha256",
            "/scm_facts/retirement_control_plane_context/protected_control_plane_byte_count",
            "/scm_facts/retirement_control_plane_context/candidate_control_plane_byte_count",
        ] {
            let mut omitted = corpus.clone();
            *omitted.pointer_mut(pointer).expect("fixture pointer") = Value::Null;
            assert!(
                evaluate_and_project_history_only_retirement_closures(&omitted)
                    .projection
                    .is_none(),
                "{pointer} omission must fail"
            );
        }
    }

    #[test]
    fn unknown_verdict_or_pass_claims_are_rejected() {
        for claim in ["verdict", "pass"] {
            let mut corpus = exact_adr_0388_corpus("closure-new");
            corpus["receipts"][0]["receipt"][claim] = json!(true);
            assert!(
                evaluate_and_project_history_only_retirement_closures(&corpus)
                    .projection
                    .is_none(),
                "unknown {claim} claim must fail"
            );
        }
    }

    #[test]
    fn protected_and_candidate_nonregular_retired_inputs_fail_closed() {
        for field in ["protected_mode", "candidate_mode"] {
            for mode in ["100755", "120000", "160000"] {
                let mut corpus = exact_adr_0388_corpus("prepared-new");
                corpus["scm_facts"]["retirement_receipt_object_facts"][0]["retired_inputs"][0]
                    [field] = json!(mode);
                assert!(
                    evaluate_and_project_history_only_retirement_closures(&corpus)
                        .projection
                        .is_none(),
                    "{field}={mode} must fail"
                );
            }
        }
    }

    #[test]
    fn retirement_evaluation_is_deterministic_for_each_valid_lifecycle() {
        for state in ["dormant", "prepared-new", "closure-new", "closed-carried"] {
            let corpus = if state == "dormant" {
                json!({
                    "receipts": [],
                    "scm_facts": {
                        "retirement_receipt_coverage": {"protected_base_ref":"origin/dev","protected_receipt_paths":[],"candidate_receipt_paths":[],"carried_receipt_paths":[],"new_receipt_paths":[],"scopes":[],"required_retired_paths":[]},
                        "retirement_receipt_object_facts": [],
                        "protected_scm_context": protected_scm_context(&[]),
                        "retirement_control_plane_context": retirement_control_plane_context(false)
                    }
                })
            } else {
                exact_adr_0388_corpus(state)
            };
            assert_eq!(
                evaluate_and_project_history_only_retirement_closures(&corpus),
                evaluate_and_project_history_only_retirement_closures(&corpus),
                "{state} evaluation must be deterministic"
            );
        }
    }

    #[test]
    fn executable_and_symlink_retired_inputs_fail_closed() {
        let corpus = exact_adr_0388_corpus("closure-new");
        for (field, value) in [
            ("predecessor_mode", json!("100755")),
            ("predecessor_mode", json!("120000")),
            ("predecessor_path_kind", json!("symlink")),
        ] {
            let mut drifted = corpus.clone();
            drifted["scm_facts"]["retirement_receipt_object_facts"][0]["retired_inputs"][0]
                [field] = value;
            assert!(
                evaluate_and_project_history_only_retirement_closures(&drifted)
                    .projection
                    .is_none()
            );
        }
    }

    #[test]
    fn projection_is_gated_by_a_fully_validated_closure_state() {
        let dormant = json!({
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
                "protected_scm_context": protected_scm_context(&[]),
                "retirement_control_plane_context": retirement_control_plane_context(false)
            }
        });
        let dormant = evaluate_and_project_history_only_retirement_closures(&dormant);
        assert!(dormant.findings.is_empty(), "{:?}", dormant.findings);
        assert!(
            dormant
                .projection
                .expect("valid bootstrap")
                .evidence_set_ids()
                .is_empty()
        );

        let prepared = evaluate_and_project_history_only_retirement_closures(
            &exact_adr_0388_corpus("prepared-new"),
        );
        assert!(prepared.findings.is_empty(), "{:?}", prepared.findings);
        assert!(
            prepared
                .projection
                .expect("valid preparation")
                .evidence_set_ids()
                .is_empty()
        );

        for state in ["closure-new", "closed-carried"] {
            let evaluation = evaluate_and_project_history_only_retirement_closures(
                &exact_adr_0388_corpus(state),
            );
            assert!(
                evaluation.findings.is_empty(),
                "{state}: {:?}",
                evaluation.findings
            );
            assert_eq!(
                evaluation
                    .projection
                    .expect("validated closure")
                    .evidence_set_ids(),
                &BTreeSet::from([ADR_0388_EVIDENCE_SET_ID.to_owned()])
            );
        }

        let mut invalid = exact_adr_0388_corpus("closure-new");
        invalid["receipts"][0]["receipt"]["authority"]["dispatch_authorized"] = json!(true);
        let invalid = evaluate_and_project_history_only_retirement_closures(&invalid);
        assert!(!invalid.findings.is_empty());
        assert!(invalid.projection.is_none());
    }

    #[test]
    fn installed_dormant_is_nonclaiming_and_projects_nothing() {
        let mut corpus = json!({
            "receipts": [],
            "scm_facts": {
                "retirement_receipt_coverage": {"protected_base_ref":"origin/dev","protected_receipt_paths":[],"candidate_receipt_paths":[],"carried_receipt_paths":[],"new_receipt_paths":[],"scopes":[],"required_retired_paths":[]},
                "retirement_receipt_object_facts": [],
                "protected_scm_context": protected_scm_context(&[]),
                "retirement_control_plane_context": retirement_control_plane_context(false)
            }
        });
        let context = &mut corpus["scm_facts"]["retirement_control_plane_context"];
        context["bootstrap"] = json!(false);
        context["protected_control_plane_blob_oid"] = json!(BLOB);
        context["protected_control_plane_sha256"] =
            json!("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        context["protected_control_plane_byte_count"] = json!(1);
        let evaluation = evaluate_and_project_history_only_retirement_closures(&corpus);
        assert!(evaluation.findings.is_empty(), "{:?}", evaluation.findings);
        assert!(
            evaluation
                .projection
                .expect("installed dormant")
                .evidence_set_ids()
                .is_empty()
        );
    }

    #[test]
    fn predeclared_control_row_cannot_forge_strict_mode_projection() {
        let mut prepared = exact_adr_0388_corpus("prepared-new");
        prepared["scm_facts"]["retirement_receipt_object_facts"][0]["receipt_state"] =
            json!("prepared-new");
        let evaluation = evaluate_and_project_history_only_retirement_closures(&prepared);
        assert!(evaluation.findings.is_empty(), "{:?}", evaluation.findings);
        let projection = evaluation.projection.expect("valid preparation");
        assert!(projection.evidence_set_ids().is_empty());

        let closure = evaluate_and_project_history_only_retirement_closures(
            &exact_adr_0388_corpus("closure-new"),
        );
        let projection = closure.projection.expect("validated closure");
        let baseline = crate::immutable_idea_archive_baseline().expect("E4 baseline");
        let first_baseline_sha256 = baseline.entries[0].sha256.clone();
        let policy = crate::parse_idea_archive_policy(&json!({
            "retention_rules": {"idea_archive": {
                "policy_version": 1,
                "mode": "git-history-only",
                "transition": {
                    "state": "closed",
                    "baseline_id": baseline.baseline_id,
                    "manifest_path": "ci/facade/cross-artifact-agreement/src/idea-archive-transition-baseline.json",
                    "sha256": baseline.manifest_sha256,
                    "closure_evidence_set_id": ADR_0388_EVIDENCE_SET_ID
                }
            }}
        }))
        .expect("strict policy");
        let observation = crate::IdeaArchiveObservation {
            archive_root_kind: crate::IdeaArchivePathKind::Missing,
            nodes: BTreeMap::new(),
            exact_body_locations: BTreeMap::new(),
            verified_closure_projection: projection,
        };
        assert!(crate::evaluate_idea_archive_transition(&policy, &observation).is_ok());

        let mut archive_remains = observation.clone();
        archive_remains.archive_root_kind = crate::IdeaArchivePathKind::Directory;
        assert!(crate::evaluate_idea_archive_transition(&policy, &archive_remains).is_err());

        let mut body_remains = observation;
        body_remains.exact_body_locations.insert(
            first_baseline_sha256,
            BTreeSet::from(["docs/ideas/copied-live-body.md".to_owned()]),
        );
        assert!(crate::evaluate_idea_archive_transition(&policy, &body_remains).is_err());
    }

    #[test]
    fn required_gates_name_only_implemented_validator_identities() {
        assert_eq!(
            REQUIRED_GATES,
            &[
                RETIREMENT_RECEIPT_VALIDATOR,
                crate::IDEA_ARCHIVE_TRANSITION_VALIDATOR
            ]
        );
        let retired_staleness_gate =
            ["cloud-ci-staleness-reaper/", "readable_archive_path"].concat();
        let retired_doc_axis_gate = ["oya-check-doc-axis/", "ReadableArchiveDirectory"].concat();
        assert!(!include_str!("retirement_receipt.rs").contains(&retired_staleness_gate));
        assert!(!include_str!("retirement_receipt.rs").contains(&retired_doc_axis_gate));
    }

    fn canonical_facts_and_raw(state: &str) -> (Value, Vec<u8>, Value) {
        let mut corpus = exact_adr_0388_corpus(state);
        let receipt = corpus["receipts"][0]["receipt"].take();
        let bytes = serde_json::to_vec(&receipt).expect("serialize raw receipt");
        let fact = &corpus["scm_facts"]["retirement_receipt_object_facts"][0];
        corpus["receipts"] = json!([{
            "receipt_path": fact["receipt_path"],
            "artifact_id": fact["artifact_id"],
            "scope_ref": fact["scope_ref"],
            "receipt_state": fact["receipt_state"],
            "candidate_receipt_blob_oid": fact["candidate_receipt_blob_oid"],
            "candidate_receipt_sha256": format!("sha256:{:x}", Sha256::digest(&bytes)),
            "baseline_commit_oid": fact["baseline_commit_oid"],
            "baseline_tree_oid": fact["baseline_tree_oid"]
        }]);
        (corpus, bytes, receipt)
    }

    #[test]
    fn facts_and_raw_adapter_requires_an_exact_path_bound_raw_set() {
        let (facts, bytes, document) = canonical_facts_and_raw("closure-new");
        let raw = RawHistoryOnlyRetirementReceipt {
            receipt_path: ADR_0388_CLOSURE_PATH,
            bytes: &bytes,
            document: &document,
        };
        for state in ["closure-new", "closed-carried"] {
            let (facts, bytes, document) = canonical_facts_and_raw(state);
            let raw = RawHistoryOnlyRetirementReceipt {
                receipt_path: ADR_0388_CLOSURE_PATH,
                bytes: &bytes,
                document: &document,
            };
            let valid = evaluate_and_project_history_only_retirement_facts(&facts, &[raw]);
            assert!(valid.findings.is_empty(), "{state}: {:?}", valid.findings);
            assert_eq!(
                valid
                    .projection
                    .expect("validated closure projection")
                    .evidence_set_ids(),
                &BTreeSet::from([ADR_0388_EVIDENCE_SET_ID.to_owned()])
            );
        }

        let missing = evaluate_and_project_history_only_retirement_facts(&facts, &[]);
        assert!(!missing.findings.is_empty());
        assert!(missing.projection.is_none());

        let extra = RawHistoryOnlyRetirementReceipt {
            receipt_path: "evidence/history-only-retirement/extra.json",
            ..raw
        };
        let extra = evaluate_and_project_history_only_retirement_facts(&facts, &[raw, extra]);
        assert!(!extra.findings.is_empty());
        assert!(extra.projection.is_none());
    }

    #[test]
    fn facts_and_raw_adapter_binds_bytes_document_metadata_and_object_facts() {
        let (facts, bytes, document) = canonical_facts_and_raw("closure-new");
        let raw = RawHistoryOnlyRetirementReceipt {
            receipt_path: ADR_0388_CLOSURE_PATH,
            bytes: &bytes,
            document: &document,
        };
        for pointer in [
            "/receipts/0/candidate_receipt_sha256",
            "/receipts/0/artifact_id",
            "/receipts/0/scope_ref",
            "/receipts/0/receipt_state",
            "/receipts/0/baseline_commit_oid",
            "/receipts/0/baseline_tree_oid",
        ] {
            let mut drifted = facts.clone();
            let replacement = if pointer.ends_with("sha256") {
                json!("sha256:0000000000000000000000000000000000000000000000000000000000000000")
            } else if pointer.ends_with("artifact_id") {
                json!("wrong-artifact")
            } else if pointer.ends_with("scope_ref") {
                json!("ADR-0363")
            } else if pointer.ends_with("receipt_state") {
                json!("prepared-new")
            } else {
                json!(NEW)
            };
            *drifted.pointer_mut(pointer).expect("fixture pointer") = replacement;
            let evaluation =
                evaluate_and_project_history_only_retirement_facts(&drifted, &[raw.clone()]);
            assert!(
                !evaluation.findings.is_empty(),
                "{pointer} drift was accepted"
            );
            assert!(evaluation.projection.is_none());
        }

        let mut drifted_fact_digest = facts.clone();
        drifted_fact_digest["scm_facts"]["retirement_receipt_object_facts"][0]["candidate_registry_row_sha256"] =
            json!("sha256:0000000000000000000000000000000000000000000000000000000000000000");
        let evaluation = evaluate_and_project_history_only_retirement_facts(
            &drifted_fact_digest,
            &[raw.clone()],
        );
        assert!(
            !evaluation.findings.is_empty(),
            "object-fact candidate registry digest drift was accepted"
        );
        assert!(evaluation.projection.is_none());

        let mut unknown_key = facts.clone();
        unknown_key["receipts"][0]["unexpected"] = json!(true);
        assert!(!evaluate_history_only_retirement_facts(&unknown_key, &[raw]).is_empty());

        let mut tampered_bytes_facts = facts;
        let tampered_bytes = br#"{}"#;
        tampered_bytes_facts["receipts"][0]["candidate_receipt_sha256"] =
            json!(format!("sha256:{:x}", Sha256::digest(tampered_bytes)));
        let tampered = RawHistoryOnlyRetirementReceipt {
            receipt_path: ADR_0388_CLOSURE_PATH,
            bytes: tampered_bytes,
            document: &document,
        };
        assert!(
            !evaluate_history_only_retirement_facts(&tampered_bytes_facts, &[tampered]).is_empty(),
            "raw bytes that do not encode the supplied receipt document were accepted"
        );
    }

    #[test]
    fn facts_and_raw_adapter_rejects_duplicate_json_keys() {
        let (mut facts, bytes, document) = canonical_facts_and_raw("closure-new");
        let artifact_id = document["artifact_id"]
            .as_str()
            .expect("fixture artifact id");
        let original = format!(r#""artifact_id":"{artifact_id}""#);
        let duplicate = format!(r#"{original},"artifact_id":"{artifact_id}""#);
        let duplicate_bytes = String::from_utf8(bytes)
            .expect("fixture JSON")
            .replacen(&original, &duplicate, 1)
            .into_bytes();
        let digest = format!("sha256:{:x}", Sha256::digest(&duplicate_bytes));
        facts["receipts"][0]["candidate_receipt_sha256"] = json!(digest);
        facts["scm_facts"]["retirement_receipt_object_facts"][0]["candidate_registry_row_sha256"] =
            facts["receipts"][0]["candidate_receipt_sha256"].clone();
        let raw = RawHistoryOnlyRetirementReceipt {
            receipt_path: ADR_0388_CLOSURE_PATH,
            bytes: &duplicate_bytes,
            document: &document,
        };
        let evaluation = evaluate_and_project_history_only_retirement_facts(&facts, &[raw]);
        assert!(
            !evaluation.findings.is_empty(),
            "duplicate-key bytes were accepted as the supplied receipt document"
        );
        assert!(evaluation.projection.is_none());
    }

    #[test]
    fn facts_and_raw_adapter_rejects_legacy_carried_and_keeps_dormant_nonclaiming() {
        let bootstrap = json!({
            "receipts": [],
            "scm_facts": {
                "retirement_receipt_coverage": {"protected_base_ref":"origin/dev","protected_receipt_paths":[],"candidate_receipt_paths":[],"carried_receipt_paths":[],"new_receipt_paths":[],"scopes":[],"required_retired_paths":[]},
                "retirement_receipt_object_facts": [],
                "protected_scm_context": protected_scm_context(&[]),
                "retirement_control_plane_context": retirement_control_plane_context(false)
            }
        });
        for installed in [false, true] {
            let mut facts = bootstrap.clone();
            if installed {
                let context = &mut facts["scm_facts"]["retirement_control_plane_context"];
                context["bootstrap"] = json!(false);
                context["protected_control_plane_blob_oid"] = json!(BLOB);
                context["protected_control_plane_sha256"] = json!(
                    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                );
                context["protected_control_plane_byte_count"] = json!(1);
            }
            let evaluation = evaluate_and_project_history_only_retirement_facts(&facts, &[]);
            assert!(evaluation.findings.is_empty(), "{:?}", evaluation.findings);
            assert!(
                evaluation
                    .projection
                    .expect("dormant projection")
                    .evidence_set_ids()
                    .is_empty()
            );
        }

        let (mut legacy, bytes, document) = canonical_facts_and_raw("closed-carried");
        legacy["receipts"][0]["receipt_state"] = json!("carried");
        legacy["scm_facts"]["retirement_receipt_object_facts"][0]["receipt_state"] =
            json!("carried");
        let raw = RawHistoryOnlyRetirementReceipt {
            receipt_path: ADR_0388_CLOSURE_PATH,
            bytes: &bytes,
            document: &document,
        };
        let evaluation = evaluate_and_project_history_only_retirement_facts(&legacy, &[raw]);
        assert!(!evaluation.findings.is_empty());
        assert!(evaluation.projection.is_none());
    }
}
