//! Pure verification of history-only retirement receipts.
//!
//! Git access belongs to the out-of-graph SCM-facts emitter. This module
//! consumes only the emitter's immutable object facts, the candidate tracked
//! path universe, and the capability registry.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::Finding;

pub const RETIREMENT_RECEIPT_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/history-only-retirement-receipt";
pub const RETIREMENT_RECEIPT_CODE: &str = "history_only_retirement_receipt_invalid";
const RECEIPT_SCHEMA: &str = "https://json-schema.org/draft/2020-12/schema";
const VERIFICATION_COMMAND: &str = "buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root . && buck2 test //ci/facade/cross-artifact-agreement:ci-cross-artifact-agreement-gate";
const MAX_SUCCESSOR_REFS: usize = 4;
const MAX_SUCCESSOR_REF_BYTES: usize = 128;
const MAX_RETIRED_INPUTS: usize = 256;
const REQUIRED_GATES: &[&str] = &[
    RETIREMENT_RECEIPT_VALIDATOR,
    "cloud-ci-cross-artifact-agreement/masterplan-v2-history-only-retirement",
    "cloud-ci-cross-artifact-agreement/masterplan-v2-read-surface-resurrection",
    "cloud-ci-staleness-reaper/readable_archive_path",
    "cloud-ci-staleness-reaper/retired_row_still_tracked",
    "oya-check-doc-axis/ReadableArchiveDirectory",
];
const EMBEDDED_RETIRED_CONTENT_FIELDS: &[&str] = &[
    "content",
    "contents",
    "retired_content",
    "retired_contents",
    "source_content",
];
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
const RETIRED_INPUT_FIELDS: &[&str] = &[
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
const VERIFICATION_CONTRACT_FIELDS: &[&str] = &[
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
const OBJECT_FACT_FIELDS: &[&str] = &[
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
const OBJECT_FACT_INPUT_FIELDS: &[&str] = &[
    "path",
    "predecessor_blob_oid",
    "sha256",
    "byte_count",
    "candidate_path_exists",
    "candidate_new_equivalent_paths",
];
const COVERAGE_FACT_FIELDS: &[&str] = &[
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
const SCOPE_FACT_FIELDS: &[&str] = &[
    "scope_ref",
    "scope_type",
    "selectors",
    "required_retired_paths",
];
const SELECTOR_FACT_FIELDS: &[&str] = &[
    "selector_type",
    "selector",
    "protected_paths",
    "candidate_paths",
    "removed_paths",
    "surviving_paths",
    "candidate_only_paths",
    "external_assertion",
];
const MASTERPLAN_SCOPE_REF: &str = "artifact:masterplan";
const MASTERPLAN_SCOPE_TYPE: &str = "masterplan-retired-surfaces";
const TRANSIENT_IDEAS_SCOPE_REF: &str = "ADR-0388";
const TRANSIENT_IDEAS_SCOPE_TYPE: &str = "transient-ideas";

pub fn evaluate_history_only_retirement_receipt(
    receipt_path: &str,
    receipt: &Value,
    scm_facts: &Value,
    volatile_facts: &Value,
    registry: &Value,
) -> BTreeSet<Finding> {
    evaluate_history_only_retirement_receipt_with_decisions(
        receipt_path,
        receipt,
        scm_facts,
        volatile_facts,
        registry,
        &Value::Null,
    )
}

pub fn evaluate_history_only_retirement_receipt_with_decisions(
    receipt_path: &str,
    receipt: &Value,
    scm_facts: &Value,
    volatile_facts: &Value,
    registry: &Value,
    decisions: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let Some(receipt_object) = receipt.as_object() else {
        findings.insert(invalid("<receipt-object>"));
        return findings;
    };
    reject_unknown_fields(receipt, RECEIPT_FIELDS, "receipt.", &mut findings);
    require_exact_string(receipt, "$schema", RECEIPT_SCHEMA, "$schema", &mut findings);

    let artifact_id = required_string(receipt, "artifact_id", "artifact_id", &mut findings);
    if artifact_id.is_some_and(|value| !is_bounded_identifier(value)) {
        findings.insert(invalid("artifact_id"));
    }
    require_exact_string(
        receipt,
        "artifact_type",
        "migration-closure-receipt",
        "artifact_type",
        &mut findings,
    );
    require_exact_string(
        receipt,
        "status",
        "candidate-recorded-nonauthoritative",
        "status",
        &mut findings,
    );
    if receipt
        .get("recorded_at")
        .and_then(Value::as_str)
        .is_none_or(|value| !is_iso_date(value))
    {
        findings.insert(invalid("recorded_at"));
    }
    let scope_ref = required_string(receipt, "scope_ref", "scope_ref", &mut findings);
    let scope_type = scope_ref.and_then(expected_scope_type);
    if scope_type.is_none()
        || scope_ref.is_some_and(|value| !scope_authority_is_resolvable(value, registry, decisions))
    {
        findings.insert(invalid("scope_ref.authority"));
    }
    let authority = receipt.get("authority").unwrap_or(&Value::Null);
    reject_unknown_fields(authority, AUTHORITY_FIELDS, "authority.", &mut findings);
    let authority_decisions = string_set(
        authority.get("decisions"),
        "authority.decisions",
        &mut findings,
    );
    if authority_decisions.is_empty() {
        findings.insert(invalid("authority.decisions.empty"));
    }
    for decision in authority_decisions {
        if !is_adr_id(&decision) {
            findings.insert(invalid(&format!("authority.decisions.invalid.{decision}")));
        }
    }
    require_exact_string(
        authority,
        "planning_state",
        "HOLD(Planning)",
        "authority.planning_state",
        &mut findings,
    );
    if receipt
        .pointer("/authority/dispatch_authorized")
        .and_then(Value::as_bool)
        != Some(false)
    {
        findings.insert(invalid("authority.dispatch_authorized"));
    }
    if receipt
        .pointer("/authority/completion_claims_promoted")
        .and_then(Value::as_u64)
        != Some(0)
    {
        findings.insert(invalid("authority.completion_claims_promoted"));
    }

    let baseline = receipt.get("baseline").unwrap_or(&Value::Null);
    reject_unknown_fields(baseline, BASELINE_FIELDS, "baseline.", &mut findings);
    let baseline_commit =
        required_oid(baseline, "commit_oid", "baseline.commit_oid", &mut findings);
    let baseline_tree = required_oid(baseline, "tree_oid", "baseline.tree_oid", &mut findings);

    let provenance = receipt.get("provenance").unwrap_or(&Value::Null);
    reject_unknown_fields(provenance, PROVENANCE_FIELDS, "provenance.", &mut findings);
    for (field, key) in [
        (
            "readable_tracked_copy_retained",
            "provenance.readable_tracked_copy_retained",
        ),
        (
            "readable_archive_directory_retained",
            "provenance.readable_archive_directory_retained",
        ),
        (
            "tombstone_content_retained",
            "provenance.tombstone_content_retained",
        ),
        (
            "receipt_reproduces_retired_content",
            "provenance.receipt_reproduces_retired_content",
        ),
    ] {
        if provenance.get(field).and_then(Value::as_bool) != Some(false) {
            findings.insert(invalid(key));
        }
    }
    require_exact_string(
        provenance,
        "content_store",
        "authorized Git object history only",
        "provenance.content_store",
        &mut findings,
    );

    let tracked_paths = string_set(
        scm_facts.get("tracked_paths"),
        "scm_facts.tracked_paths",
        &mut findings,
    );
    let expected_absent = string_set(
        receipt.pointer("/verification_contract/expected_absent_paths"),
        "verification_contract.expected_absent_paths",
        &mut findings,
    );
    let verification_contract = receipt.get("verification_contract").unwrap_or(&Value::Null);
    reject_unknown_fields(
        verification_contract,
        VERIFICATION_CONTRACT_FIELDS,
        "verification_contract.",
        &mut findings,
    );
    if verification_contract
        .get("expected_tracked_readable_archive_directory_count")
        .and_then(Value::as_u64)
        != Some(0)
    {
        findings.insert(invalid(
            "verification_contract.expected_tracked_readable_archive_directory_count",
        ));
    }
    let required_gates = string_set(
        verification_contract.get("required_gates"),
        "verification_contract.required_gates",
        &mut findings,
    );
    let exact_required_gates = REQUIRED_GATES
        .iter()
        .map(|gate| (*gate).to_owned())
        .collect::<BTreeSet<_>>();
    if required_gates != exact_required_gates {
        findings.insert(invalid("verification_contract.required_gates.exact_set"));
    }
    for (index, gate) in required_gates.iter().enumerate() {
        if !is_bounded_reference(gate) {
            findings.insert(invalid(&format!(
                "verification_contract.required_gates[{index}]"
            )));
        }
    }

    let effects = receipt.get("effects").unwrap_or(&Value::Null);
    reject_unknown_fields(effects, EFFECT_FIELDS, "effects.", &mut findings);
    require_exact_string(
        effects,
        "repository_effect",
        "pending-protected-pr-admission",
        "effects.repository_effect",
        &mut findings,
    );
    if !matches!(
        effects.get("runtime_effect").and_then(Value::as_str),
        Some("none" | "retired-read-surfaces-removed")
    ) {
        findings.insert(invalid("effects.runtime_effect"));
    }
    require_exact_string(
        effects,
        "roadmap_effect",
        "none",
        "effects.roadmap_effect",
        &mut findings,
    );
    require_exact_string(
        effects,
        "planning_hold_effect",
        "preserved",
        "effects.planning_hold_effect",
        &mut findings,
    );

    let mut retired_inputs = BTreeMap::new();
    let Some(inputs) = receipt_object
        .get("retired_inputs")
        .and_then(Value::as_array)
    else {
        findings.insert(invalid("retired_inputs"));
        validate_registry(receipt_path, artifact_id, registry, &mut findings);
        return findings;
    };
    if inputs.is_empty() {
        findings.insert(invalid("retired_inputs.empty"));
    }
    if inputs.len() > MAX_RETIRED_INPUTS {
        findings.insert(invalid("retired_inputs.too_many"));
    }
    for (index, input) in inputs.iter().enumerate() {
        let input_key = format!("retired_inputs[{index}]");
        let Some(path) = input
            .get("path")
            .and_then(Value::as_str)
            .filter(|path| is_canonical_repo_path(path))
        else {
            findings.insert(invalid(&format!("{input_key}.path")));
            continue;
        };
        if retired_inputs.insert(path.to_owned(), input).is_some() {
            findings.insert(invalid(&format!("{path}.duplicate_retired_input")));
        }
        reject_unknown_fields(
            input,
            RETIRED_INPUT_FIELDS,
            &format!("{path}."),
            &mut findings,
        );
        reject_embedded_retired_content(input, path, "", &mut findings);
        required_oid(
            input,
            "predecessor_blob_oid",
            &format!("{path}.predecessor_blob_oid"),
            &mut findings,
        );
        required_sha256(input, "sha256", &format!("{path}.sha256"), &mut findings);
        if input.get("byte_count").and_then(Value::as_u64).is_none() {
            findings.insert(invalid(&format!("{path}.byte_count")));
        }
        let successor_refs = string_set(
            input.get("successor_refs"),
            &format!("{path}.successor_refs"),
            &mut findings,
        );
        if successor_refs.is_empty()
            || successor_refs.len() > MAX_SUCCESSOR_REFS
            || successor_refs.iter().map(String::len).sum::<usize>() > MAX_SUCCESSOR_REF_BYTES
        {
            findings.insert(invalid(&format!("{path}.successor_refs")));
        }
        for (index, successor_ref) in successor_refs.iter().enumerate() {
            if !is_resolvable_successor_ref(successor_ref, registry, decisions) {
                findings.insert(invalid(&format!(
                    "{path}.successor_refs[{index}].reference"
                )));
            }
        }
        if input.get("successor_ref").is_some() {
            findings.insert(invalid(&format!("{path}.successor_ref.forbidden_singular")));
        }
        let disposition = input.get("disposition").and_then(Value::as_str);
        if !matches!(
            disposition,
            Some("retired-git-history-only" | "promoted-source-copy-deleted-from-head")
        ) {
            findings.insert(invalid(&format!("{path}.disposition")));
        }
        if tracked_paths.contains(path) {
            findings.insert(invalid(&format!("{path}.candidate_tracked_path_present")));
        }
        if !expected_absent.contains(path) {
            findings.insert(invalid(&format!("{path}.expected_absent_paths.missing")));
        }
    }
    for path in expected_absent.difference(&retired_inputs.keys().cloned().collect()) {
        findings.insert(invalid(&format!(
            "{path}.expected_absent_paths.unregistered"
        )));
    }

    let object_expectations = ReceiptObjectExpectations {
        receipt_path,
        artifact_id,
        scope_ref,
        scope_type,
        baseline_commit,
        baseline_tree,
    };
    validate_object_facts(
        &object_expectations,
        &retired_inputs,
        volatile_facts,
        &mut findings,
    );
    validate_registry(receipt_path, artifact_id, registry, &mut findings);
    findings
}

pub fn evaluate_history_only_retirement_receipt_coverage(
    receipts: &[Value],
    volatile_facts: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let coverage = volatile_facts
        .get("retirement_receipt_coverage")
        .unwrap_or(&Value::Null);
    reject_unknown_fields(
        coverage,
        COVERAGE_FACT_FIELDS,
        "retirement_receipt_coverage.",
        &mut findings,
    );
    if coverage.get("protected_base_ref").and_then(Value::as_str) != Some("origin/dev") {
        findings.insert(invalid("retirement_receipt_coverage.protected_base_ref"));
    }
    let current_epoch_commit = required_oid(
        coverage,
        "current_epoch_commit_oid",
        "retirement_receipt_coverage.current_epoch_commit_oid",
        &mut findings,
    );
    let current_epoch_tree = required_oid(
        coverage,
        "current_epoch_tree_oid",
        "retirement_receipt_coverage.current_epoch_tree_oid",
        &mut findings,
    );
    let protected_receipt_paths = canonical_path_set(
        coverage.get("protected_receipt_paths"),
        "retirement_receipt_coverage.protected_receipt_paths",
        &mut findings,
    );
    let candidate_receipt_paths = canonical_path_set(
        coverage.get("candidate_receipt_paths"),
        "retirement_receipt_coverage.candidate_receipt_paths",
        &mut findings,
    );
    let carried_receipt_paths = canonical_path_set(
        coverage.get("carried_receipt_paths"),
        "retirement_receipt_coverage.carried_receipt_paths",
        &mut findings,
    );
    let new_receipt_paths = canonical_path_set(
        coverage.get("new_receipt_paths"),
        "retirement_receipt_coverage.new_receipt_paths",
        &mut findings,
    );
    if carried_receipt_paths != protected_receipt_paths
        || !carried_receipt_paths.is_disjoint(&new_receipt_paths)
        || candidate_receipt_paths
            != carried_receipt_paths
                .union(&new_receipt_paths)
                .cloned()
                .collect()
    {
        findings.insert(invalid(
            "retirement_receipt_coverage.receipt_epoch_partition",
        ));
    }

    let required_by_scope = validate_coverage_scopes(coverage, &mut findings);
    let facts_by_artifact = receipt_epoch_facts_by_artifact(volatile_facts, &mut findings);
    let fact_paths = facts_by_artifact
        .values()
        .filter_map(|fact| fact.get("receipt_path").and_then(Value::as_str))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if fact_paths != candidate_receipt_paths {
        findings.insert(invalid("retirement_receipt_coverage.object_fact_path_set"));
    }

    let mut covered_by_scope: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut seen_artifacts = BTreeSet::new();
    for (receipt_index, receipt) in receipts.iter().enumerate() {
        let Some(artifact_id) = receipt.get("artifact_id").and_then(Value::as_str) else {
            findings.insert(invalid(&format!(
                "retirement_receipts[{receipt_index}].artifact_id"
            )));
            continue;
        };
        if !seen_artifacts.insert(artifact_id.to_owned()) {
            findings.insert(invalid(&format!(
                "retirement_receipts[{receipt_index}].artifact_id.duplicate"
            )));
        }
        let Some(fact) = facts_by_artifact.get(artifact_id) else {
            findings.insert(invalid(&format!(
                "retirement_receipts[{receipt_index}].object_fact_missing"
            )));
            continue;
        };
        let Some(receipt_path) = fact.get("receipt_path").and_then(Value::as_str) else {
            findings.insert(invalid(&format!(
                "retirement_receipts[{receipt_index}].object_fact_receipt_path"
            )));
            continue;
        };
        let epoch = fact.get("receipt_epoch").and_then(Value::as_str);
        let in_expected_epoch_set = match epoch {
            Some("carried") => carried_receipt_paths.contains(receipt_path),
            Some("new") => new_receipt_paths.contains(receipt_path),
            _ => false,
        };
        if !in_expected_epoch_set {
            findings.insert(invalid(&format!(
                "retirement_receipt_coverage.unclassified_receipt.{receipt_path}"
            )));
        }
        if epoch != Some("new") {
            continue;
        }
        if receipt
            .pointer("/baseline/commit_oid")
            .and_then(Value::as_str)
            != current_epoch_commit
        {
            findings.insert(invalid(&format!(
                "retirement_receipts[{receipt_index}].baseline.commit_oid.current_epoch_mismatch"
            )));
        }
        if receipt
            .pointer("/baseline/tree_oid")
            .and_then(Value::as_str)
            != current_epoch_tree
        {
            findings.insert(invalid(&format!(
                "retirement_receipts[{receipt_index}].baseline.tree_oid.current_epoch_mismatch"
            )));
        }
        let Some(scope_ref) = receipt.get("scope_ref").and_then(Value::as_str) else {
            findings.insert(invalid(&format!(
                "retirement_receipts[{receipt_index}].scope_ref"
            )));
            continue;
        };
        let Some(inputs) = receipt.get("retired_inputs").and_then(Value::as_array) else {
            findings.insert(invalid(&format!(
                "retirement_receipts[{receipt_index}].retired_inputs"
            )));
            continue;
        };
        let scope_paths = covered_by_scope.entry(scope_ref.to_owned()).or_default();
        for (input_index, input) in inputs.iter().enumerate() {
            let Some(path) = input
                .get("path")
                .and_then(Value::as_str)
                .filter(|path| is_canonical_repo_path(path))
            else {
                findings.insert(invalid(&format!(
                    "retirement_receipts[{receipt_index}].retired_inputs[{input_index}].path"
                )));
                continue;
            };
            if !scope_paths.insert(path.to_owned()) {
                findings.insert(invalid(&format!(
                    "retirement_receipt_coverage.duplicate_path.{path}"
                )));
            }
        }
    }

    for (scope_ref, required_paths) in &required_by_scope {
        let covered_paths = covered_by_scope.get(scope_ref).cloned().unwrap_or_default();
        for path in required_paths.difference(&covered_paths) {
            findings.insert(invalid(&format!(
                "retirement_receipt_coverage.scope.{scope_ref}.missing.{path}"
            )));
        }
        for path in covered_paths.difference(required_paths) {
            findings.insert(invalid(&format!(
                "retirement_receipt_coverage.scope.{scope_ref}.unexpected.{path}"
            )));
        }
    }
    for (scope_ref, covered_paths) in &covered_by_scope {
        if !required_by_scope.contains_key(scope_ref) {
            for path in covered_paths {
                findings.insert(invalid(&format!(
                    "retirement_receipt_coverage.scope.{scope_ref}.unexpected.{path}"
                )));
            }
        }
    }
    findings
}

fn receipt_epoch_facts_by_artifact<'a>(
    volatile_facts: &'a Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, &'a Value> {
    let Some(facts) = volatile_facts
        .get("retirement_receipt_object_facts")
        .and_then(Value::as_array)
    else {
        findings.insert(invalid("volatile_facts.retirement_receipt_object_facts"));
        return BTreeMap::new();
    };
    let mut by_artifact = BTreeMap::new();
    for (index, fact) in facts.iter().enumerate() {
        let Some(artifact_id) = fact.get("artifact_id").and_then(Value::as_str) else {
            findings.insert(invalid(&format!(
                "retirement_receipt_object_facts[{index}].artifact_id"
            )));
            continue;
        };
        if by_artifact.insert(artifact_id.to_owned(), fact).is_some() {
            findings.insert(invalid(&format!(
                "retirement_receipt_object_facts[{index}].artifact_id.duplicate"
            )));
        }
    }
    by_artifact
}

fn validate_coverage_scopes(
    coverage: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, BTreeSet<String>> {
    let Some(scopes) = coverage.get("scopes").and_then(Value::as_array) else {
        findings.insert(invalid("retirement_receipt_coverage.scopes"));
        return BTreeMap::new();
    };
    let mut required_by_scope = BTreeMap::new();
    let mut required_union = BTreeSet::new();
    for (scope_index, scope) in scopes.iter().enumerate() {
        reject_unknown_fields(
            scope,
            SCOPE_FACT_FIELDS,
            &format!("retirement_receipt_coverage.scopes[{scope_index}]."),
            findings,
        );
        let Some(scope_ref) = scope.get("scope_ref").and_then(Value::as_str) else {
            findings.insert(invalid(&format!(
                "retirement_receipt_coverage.scopes[{scope_index}].scope_ref"
            )));
            continue;
        };
        let expected_type = expected_scope_type(scope_ref);
        if scope.get("scope_type").and_then(Value::as_str) != expected_type {
            findings.insert(invalid(&format!(
                "retirement_receipt_coverage.scopes[{scope_index}].scope_ref_type_pair"
            )));
        }
        let mut removed_union = BTreeSet::new();
        let Some(selectors) = scope.get("selectors").and_then(Value::as_array) else {
            findings.insert(invalid(&format!(
                "retirement_receipt_coverage.scopes[{scope_index}].selectors"
            )));
            continue;
        };
        let mut selector_identities = BTreeSet::new();
        for (selector_index, selector) in selectors.iter().enumerate() {
            reject_unknown_fields(
                selector,
                SELECTOR_FACT_FIELDS,
                &format!(
                    "retirement_receipt_coverage.scopes[{scope_index}].selectors[{selector_index}]."
                ),
                findings,
            );
            let selector_type = selector.get("selector_type").and_then(Value::as_str);
            let selector_value = selector.get("selector").and_then(Value::as_str);
            if !matches!(selector_type, Some("exact" | "glob" | "external"))
                || selector_value.is_none()
                || !selector_value
                    .is_some_and(|value| selector_shape_is_valid(selector_type, value))
            {
                findings.insert(invalid(&format!(
                    "retirement_receipt_coverage.scopes[{scope_index}].selectors[{selector_index}].selector"
                )));
            }
            if let (Some(kind), Some(value)) = (selector_type, selector_value)
                && !selector_identities.insert((kind.to_owned(), value.to_owned()))
            {
                findings.insert(invalid(&format!(
                    "retirement_receipt_coverage.scopes[{scope_index}].selectors[{selector_index}].duplicate"
                )));
            }
            let key_prefix = format!(
                "retirement_receipt_coverage.scopes[{scope_index}].selectors[{selector_index}]"
            );
            let protected = canonical_path_set(
                selector.get("protected_paths"),
                &format!("{key_prefix}.protected_paths"),
                findings,
            );
            let candidate = canonical_path_set(
                selector.get("candidate_paths"),
                &format!("{key_prefix}.candidate_paths"),
                findings,
            );
            let removed = canonical_path_set(
                selector.get("removed_paths"),
                &format!("{key_prefix}.removed_paths"),
                findings,
            );
            let surviving = canonical_path_set(
                selector.get("surviving_paths"),
                &format!("{key_prefix}.surviving_paths"),
                findings,
            );
            let candidate_only = canonical_path_set(
                selector.get("candidate_only_paths"),
                &format!("{key_prefix}.candidate_only_paths"),
                findings,
            );
            if selector_type == Some("external") {
                if selector.get("external_assertion").and_then(Value::as_str)
                    != Some("outside-repository-authority-not-inspected")
                {
                    findings.insert(invalid(&format!("{key_prefix}.external_assertion")));
                }
                if [
                    &protected,
                    &candidate,
                    &removed,
                    &surviving,
                    &candidate_only,
                ]
                .into_iter()
                .any(|paths| !paths.is_empty())
                {
                    findings.insert(invalid(&format!("{key_prefix}.external_path_claim")));
                }
            } else {
                if selector.get("external_assertion").and_then(Value::as_str)
                    != Some("not-applicable")
                {
                    findings.insert(invalid(&format!("{key_prefix}.external_assertion")));
                }
                if let Some(selector_value) = selector_value {
                    for path in protected.union(&candidate) {
                        if !selector_matches_path(selector_type, selector_value, path) {
                            findings.insert(invalid(&format!(
                                "{key_prefix}.selector_path_mismatch.{path}"
                            )));
                        }
                    }
                }
                let expected_removed = protected
                    .difference(&candidate)
                    .cloned()
                    .collect::<BTreeSet<_>>();
                let expected_surviving = protected
                    .intersection(&candidate)
                    .cloned()
                    .collect::<BTreeSet<_>>();
                let expected_candidate_only = candidate
                    .difference(&protected)
                    .cloned()
                    .collect::<BTreeSet<_>>();
                if removed != expected_removed {
                    findings.insert(invalid(&format!("{key_prefix}.removed_paths.relation")));
                }
                if surviving != expected_surviving {
                    findings.insert(invalid(&format!("{key_prefix}.surviving_paths.relation")));
                }
                if candidate_only != expected_candidate_only {
                    findings.insert(invalid(&format!(
                        "{key_prefix}.candidate_only_paths.relation"
                    )));
                }
                for path in &surviving {
                    findings.insert(invalid(&format!(
                        "retirement_receipt_coverage.survivor.{scope_ref}.{path}"
                    )));
                }
                for path in &candidate_only {
                    findings.insert(invalid(&format!(
                        "retirement_receipt_coverage.candidate_only.{scope_ref}.{path}"
                    )));
                }
                removed_union.extend(removed);
            }
        }
        let declared_required = canonical_path_set(
            scope.get("required_retired_paths"),
            &format!("retirement_receipt_coverage.scopes[{scope_index}].required_retired_paths"),
            findings,
        );
        if declared_required != removed_union {
            findings.insert(invalid(&format!(
                "retirement_receipt_coverage.scopes[{scope_index}].required_union_mismatch"
            )));
        }
        for path in &declared_required {
            if !required_union.insert(path.clone()) {
                findings.insert(invalid(&format!(
                    "retirement_receipt_coverage.cross_scope_duplicate.{path}"
                )));
            }
        }
        if required_by_scope
            .insert(scope_ref.to_owned(), declared_required)
            .is_some()
        {
            findings.insert(invalid(&format!(
                "retirement_receipt_coverage.scopes[{scope_index}].scope_ref.duplicate"
            )));
        }
    }
    for (scope_ref, _) in [
        (MASTERPLAN_SCOPE_REF, MASTERPLAN_SCOPE_TYPE),
        (TRANSIENT_IDEAS_SCOPE_REF, TRANSIENT_IDEAS_SCOPE_TYPE),
    ] {
        if !required_by_scope.contains_key(scope_ref) {
            findings.insert(invalid(&format!(
                "retirement_receipt_coverage.scope_missing.{scope_ref}"
            )));
        }
    }
    let declared_union = canonical_path_set(
        coverage.get("required_retired_paths"),
        "retirement_receipt_coverage.required_retired_paths",
        findings,
    );
    if declared_union != required_union {
        findings.insert(invalid(
            "retirement_receipt_coverage.required_union_mismatch",
        ));
    }
    required_by_scope
}

fn selector_shape_is_valid(selector_type: Option<&str>, selector: &str) -> bool {
    match selector_type {
        Some("exact") => is_canonical_repo_path(selector),
        Some("glob") => selector
            .strip_suffix("/**")
            .is_some_and(is_canonical_repo_path),
        Some("external") => {
            selector.starts_with("~/")
                && selector
                    .strip_prefix("~/")
                    .and_then(|value| value.strip_suffix("/**"))
                    .is_some_and(is_canonical_repo_path)
        }
        _ => false,
    }
}

fn selector_matches_path(selector_type: Option<&str>, selector: &str, path: &str) -> bool {
    match selector_type {
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

fn invalid(key: &str) -> Finding {
    Finding::new(RETIREMENT_RECEIPT_CODE, key)
}

fn required_string<'a>(
    value: &'a Value,
    field: &str,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<&'a str> {
    let result = value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty());
    if result.is_none() {
        findings.insert(invalid(key));
    }
    result
}

fn require_exact_string(
    value: &Value,
    field: &str,
    expected: &str,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) {
    if value.get(field).and_then(Value::as_str) != Some(expected) {
        findings.insert(invalid(key));
    }
}

fn is_lower_hex(value: &str, width: usize) -> bool {
    value.len() == width
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_bounded_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn is_canonical_repo_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1024
        && !value.starts_with('/')
        && !value.starts_with('~')
        && !value.contains('\\')
        && !value.contains('*')
        && !value.contains('#')
        && value
            .split('/')
            .all(|segment| !segment.is_empty() && !matches!(segment, "." | ".."))
}

fn is_bounded_reference(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"/_:.-".contains(&byte))
}

fn is_resolvable_successor_ref(value: &str, registry: &Value, decisions: &Value) -> bool {
    if is_adr_id(value) {
        return decisions
            .get("decisions")
            .and_then(Value::as_array)
            .is_some_and(|rows| {
                rows.iter()
                    .filter(|row| row.get("adr").and_then(Value::as_str) == Some(value))
                    .count()
                    == 1
            });
    }

    let Some(artifact_id) = value.strip_prefix("artifact:") else {
        return false;
    };
    is_bounded_identifier(artifact_id)
        && registry
            .get("rows")
            .and_then(Value::as_array)
            .is_some_and(|rows| {
                rows.iter()
                    .filter(|row| {
                        row.get("artifact_id").and_then(Value::as_str) == Some(artifact_id)
                    })
                    .count()
                    == 1
            })
}

fn expected_scope_type(scope_ref: &str) -> Option<&'static str> {
    match scope_ref {
        MASTERPLAN_SCOPE_REF => Some(MASTERPLAN_SCOPE_TYPE),
        TRANSIENT_IDEAS_SCOPE_REF => Some(TRANSIENT_IDEAS_SCOPE_TYPE),
        _ => None,
    }
}

fn scope_authority_is_resolvable(scope_ref: &str, registry: &Value, decisions: &Value) -> bool {
    match scope_ref {
        MASTERPLAN_SCOPE_REF => {
            registry
                .get("rows")
                .and_then(Value::as_array)
                .is_some_and(|rows| {
                    rows.iter()
                        .filter(|row| {
                            row.get("artifact_id").and_then(Value::as_str) == Some("masterplan")
                                && row.get("artifact_path").and_then(Value::as_str)
                                    == Some("/specs/masterplan.json")
                        })
                        .count()
                        == 1
                })
        }
        TRANSIENT_IDEAS_SCOPE_REF => decisions
            .get("decisions")
            .and_then(Value::as_array)
            .is_some_and(|rows| {
                rows.iter()
                    .filter(|row| {
                        row.get("adr").and_then(Value::as_str) == Some(TRANSIENT_IDEAS_SCOPE_REF)
                            && row.get("status").and_then(Value::as_str) == Some("Accepted")
                    })
                    .count()
                    == 1
            }),
        _ => false,
    }
}

fn is_adr_id(value: &str) -> bool {
    value
        .strip_prefix("ADR-")
        .is_some_and(|suffix| suffix.len() == 4 && suffix.bytes().all(|byte| byte.is_ascii_digit()))
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if bytes
        .iter()
        .enumerate()
        .any(|(index, byte)| !matches!(index, 4 | 7) && !byte.is_ascii_digit())
    {
        return false;
    }
    let month = (bytes[5] - b'0') * 10 + (bytes[6] - b'0');
    let day = (bytes[8] - b'0') * 10 + (bytes[9] - b'0');
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

fn required_oid<'a>(
    value: &'a Value,
    field: &str,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<&'a str> {
    let result = value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| is_lower_hex(value, 40));
    if result.is_none() {
        findings.insert(invalid(key));
    }
    result
}

fn required_sha256<'a>(
    value: &'a Value,
    field: &str,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<&'a str> {
    let result = value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| is_lower_hex(value, 64));
    if result.is_none() {
        findings.insert(invalid(key));
    }
    result
}

fn string_set(
    value: Option<&Value>,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    let Some(values) = value.and_then(Value::as_array) else {
        findings.insert(invalid(key));
        return BTreeSet::new();
    };
    let mut result = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        let Some(value) = value
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            findings.insert(invalid(&format!("{key}[{index}]")));
            continue;
        };
        if !result.insert(value.to_owned()) {
            findings.insert(invalid(&format!("{key}[{index}].duplicate")));
        }
    }
    result
}

fn canonical_path_set(
    value: Option<&Value>,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    let raw = string_set(value, key, findings);
    raw.into_iter()
        .filter_map(|path| {
            if is_canonical_repo_path(&path) {
                Some(path)
            } else {
                findings.insert(invalid(&format!("{key}.path")));
                None
            }
        })
        .collect()
}

fn reject_embedded_retired_content(
    value: &Value,
    path: &str,
    key_suffix: &str,
    findings: &mut BTreeSet<Finding>,
) {
    if EMBEDDED_RETIRED_CONTENT_FIELDS
        .iter()
        .any(|field| value.get(*field).is_some())
    {
        findings.insert(invalid(&format!(
            "{path}.{key_suffix}embedded_retired_content"
        )));
    }
}

fn reject_unknown_fields(
    value: &Value,
    allowed: &[&str],
    key_prefix: &str,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(object) = value.as_object() else {
        return;
    };
    for field in object.keys() {
        if !allowed.contains(&field.as_str()) {
            findings.insert(invalid(&format!("{key_prefix}unknown_field.{field}")));
        }
    }
}

struct ReceiptObjectExpectations<'a> {
    receipt_path: &'a str,
    artifact_id: Option<&'a str>,
    scope_ref: Option<&'a str>,
    scope_type: Option<&'a str>,
    baseline_commit: Option<&'a str>,
    baseline_tree: Option<&'a str>,
}

fn validate_object_facts(
    expected: &ReceiptObjectExpectations<'_>,
    retired_inputs: &BTreeMap<String, &Value>,
    volatile_facts: &Value,
    findings: &mut BTreeSet<Finding>,
) {
    let receipt_path = expected.receipt_path;
    let Some(facts) = volatile_facts
        .get("retirement_receipt_object_facts")
        .and_then(Value::as_array)
    else {
        findings.insert(invalid("volatile_facts.retirement_receipt_object_facts"));
        return;
    };
    let matches: Vec<&Value> = facts
        .iter()
        .filter(|fact| fact.get("receipt_path").and_then(Value::as_str) == Some(receipt_path))
        .collect();
    if matches.len() != 1 {
        findings.insert(invalid(&format!("{receipt_path}.object_facts_coverage")));
        return;
    }
    let facts = matches[0];
    reject_unknown_fields(
        facts,
        OBJECT_FACT_FIELDS,
        &format!("{receipt_path}.object_facts."),
        findings,
    );
    if facts.get("protected_base_ref").and_then(Value::as_str) != Some("origin/dev") {
        findings.insert(invalid(&format!("{receipt_path}.protected_base_ref")));
    }
    if facts.get("artifact_id").and_then(Value::as_str) != expected.artifact_id {
        findings.insert(invalid("artifact_id.object_fact_mismatch"));
    }
    if facts.get("scope_ref").and_then(Value::as_str) != expected.scope_ref {
        findings.insert(invalid("scope_ref.object_fact_mismatch"));
    }
    if facts.get("scope_type").and_then(Value::as_str) != expected.scope_type {
        findings.insert(invalid("scope_type.object_fact_mismatch"));
    }
    if facts.get("baseline_commit_oid").and_then(Value::as_str) != expected.baseline_commit {
        findings.insert(invalid("baseline.commit_oid.object_fact_mismatch"));
    }
    if facts.get("baseline_tree_oid").and_then(Value::as_str) != expected.baseline_tree {
        findings.insert(invalid("baseline.tree_oid.object_fact_mismatch"));
    }
    if facts
        .get("baseline_is_ancestor_of_current_epoch")
        .and_then(Value::as_bool)
        != Some(true)
    {
        findings.insert(invalid("baseline.commit_oid.not_ancestor_of_current_epoch"));
    }

    match facts.get("receipt_epoch").and_then(Value::as_str) {
        Some("new") => {
            if !facts
                .get("protected_receipt_blob_oid")
                .is_some_and(Value::is_null)
            {
                findings.insert(invalid("protected_receipt_blob_oid.new_receipt"));
            }
            if !facts
                .get("protected_registry_binding_preserved")
                .is_some_and(Value::is_null)
            {
                findings.insert(invalid("protected_registry_binding_preserved.new_receipt"));
            }
        }
        Some("carried") => {
            let protected_blob = facts
                .get("protected_receipt_blob_oid")
                .and_then(Value::as_str)
                .filter(|value| is_lower_hex(value, 40));
            let candidate_blob = facts
                .get("candidate_receipt_blob_oid")
                .and_then(Value::as_str)
                .filter(|value| is_lower_hex(value, 40));
            if protected_blob.is_none() || protected_blob != candidate_blob {
                findings.insert(invalid("protected_receipt_blob_oid.immutable"));
            }
            if facts
                .get("protected_registry_binding_preserved")
                .and_then(Value::as_bool)
                != Some(true)
            {
                findings.insert(invalid("protected_registry_binding_preserved"));
            }
        }
        _ => {
            findings.insert(invalid("receipt_epoch"));
        }
    }
    required_oid(
        facts,
        "candidate_receipt_blob_oid",
        "candidate_receipt_blob_oid",
        findings,
    );

    let mut facts_by_path = BTreeMap::new();
    let Some(object_inputs) = facts.get("retired_inputs").and_then(Value::as_array) else {
        findings.insert(invalid(&format!(
            "{receipt_path}.object_facts.retired_inputs"
        )));
        return;
    };
    for (index, fact) in object_inputs.iter().enumerate() {
        let Some(path) = fact.get("path").and_then(Value::as_str) else {
            findings.insert(invalid(&format!(
                "{receipt_path}.object_facts.retired_inputs[{index}].path"
            )));
            continue;
        };
        if facts_by_path.insert(path.to_owned(), fact).is_some() {
            findings.insert(invalid(&format!("{path}.duplicate_object_fact")));
        }
        reject_unknown_fields(
            fact,
            OBJECT_FACT_INPUT_FIELDS,
            &format!("{path}.object_fact_"),
            findings,
        );
        reject_embedded_retired_content(fact, path, "object_fact_", findings);
    }

    for (path, input) in retired_inputs {
        let Some(fact) = facts_by_path.get(path) else {
            findings.insert(invalid(&format!("{path}.object_fact_missing")));
            continue;
        };
        for field in ["predecessor_blob_oid", "sha256", "byte_count"] {
            if input.get(field) != fact.get(field) {
                findings.insert(invalid(&format!("{path}.{field}.object_fact_mismatch")));
            }
        }
        if fact.get("candidate_path_exists").and_then(Value::as_bool) != Some(false) {
            findings.insert(invalid(&format!("{path}.candidate_path_exists")));
        }
        let equivalent_paths = canonical_path_set(
            fact.get("candidate_new_equivalent_paths"),
            &format!("{path}.candidate_new_equivalent_paths"),
            findings,
        );
        if !equivalent_paths.is_empty() {
            findings.insert(invalid(&format!("{path}.candidate_equivalent_copy")));
        }
    }
    for path in facts_by_path.keys() {
        if !retired_inputs.contains_key(path) {
            findings.insert(invalid(&format!("{path}.unexpected_object_fact")));
        }
    }
}

fn validate_registry(
    receipt_path: &str,
    artifact_id: Option<&str>,
    registry: &Value,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(artifact_id) = artifact_id else {
        return;
    };
    let Some(rows) = registry.get("rows").and_then(Value::as_array) else {
        findings.insert(invalid("registry.rows"));
        return;
    };
    let expected_path = format!("/{receipt_path}");
    let matches: Vec<&Value> = rows
        .iter()
        .filter(|row| row.get("artifact_id").and_then(Value::as_str) == Some(artifact_id))
        .collect();
    if matches.len() != 1 {
        findings.insert(invalid(&format!("{artifact_id}.registry_coverage")));
        return;
    }
    let row = matches[0];
    if row.get("artifact_path").and_then(Value::as_str) != Some(expected_path.as_str()) {
        findings.insert(invalid(&format!("{artifact_id}.registry_artifact_path")));
    }
    if row
        .pointer("/capability_overrides/validation/validator")
        .and_then(Value::as_str)
        != Some(RETIREMENT_RECEIPT_VALIDATOR)
    {
        findings.insert(invalid(&format!("{artifact_id}.registry_validator")));
    }
    for (pointer, suffix) in [
        (
            "/capability_overrides/verification/command",
            "registry_verification_command",
        ),
        (
            "/evidence_contract/verification/command",
            "registry_evidence_command",
        ),
    ] {
        if row.pointer(pointer).and_then(Value::as_str) != Some(VERIFICATION_COMMAND) {
            findings.insert(invalid(&format!("{artifact_id}.{suffix}")));
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const RECEIPT_PATH: &str =
        "evidence/goals/repository-authority-history-only-retirement-closure-20260721.json";
    const COMMIT: &str = "1111111111111111111111111111111111111111";
    const TREE: &str = "2222222222222222222222222222222222222222";
    const BLOB: &str = "3333333333333333333333333333333333333333";
    const RECEIPT_BLOB: &str = "6666666666666666666666666666666666666666";
    const SHA256: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn receipt() -> Value {
        json!({
            "$schema": RECEIPT_SCHEMA,
            "artifact_id": "repository-authority-history-only-retirement-closure-20260721",
            "artifact_type": "migration-closure-receipt",
            "status": "candidate-recorded-nonauthoritative",
            "recorded_at": "2026-07-21",
            "scope_ref": MASTERPLAN_SCOPE_REF,
            "authority": {
                "decisions": ["ADR-0515", "ADR-0619"],
                "planning_state": "HOLD(Planning)",
                "dispatch_authorized": false,
                "completion_claims_promoted": 0
            },
            "baseline": {"commit_oid": COMMIT, "tree_oid": TREE},
            "retired_inputs": [{
                "path": "docs/ROADMAP.md",
                "predecessor_blob_oid": BLOB,
                "sha256": SHA256,
                "byte_count": 12,
                "successor_refs": ["artifact:masterplan"],
                "disposition": "retired-git-history-only"
            }],
            "provenance": {
                "content_store": "authorized Git object history only",
                "readable_tracked_copy_retained": false,
                "readable_archive_directory_retained": false,
                "tombstone_content_retained": false,
                "receipt_reproduces_retired_content": false
            },
            "verification_contract": {
                "expected_absent_paths": ["docs/ROADMAP.md"],
                "expected_tracked_readable_archive_directory_count": 0,
                "required_gates": REQUIRED_GATES
            },
            "effects": {
                "repository_effect": "pending-protected-pr-admission",
                "runtime_effect": "retired-read-surfaces-removed",
                "roadmap_effect": "none",
                "planning_hold_effect": "preserved"
            }
        })
    }

    fn scm_facts() -> Value {
        json!({
            "schema": "oya-ci/scm-facts/v2",
            "tracked_paths": ["specs/masterplan.json"]
        })
    }

    fn volatile_facts() -> Value {
        json!({
            "schema": "oya-ci/scm-volatile-facts/v1",
            "retirement_receipt_object_facts": [{
                "artifact_id": "repository-authority-history-only-retirement-closure-20260721",
                "receipt_path": RECEIPT_PATH,
                "protected_base_ref": "origin/dev",
                "receipt_epoch": "new",
                "scope_ref": MASTERPLAN_SCOPE_REF,
                "scope_type": MASTERPLAN_SCOPE_TYPE,
                "baseline_commit_oid": COMMIT,
                "baseline_tree_oid": TREE,
                "baseline_is_ancestor_of_current_epoch": true,
                "protected_receipt_blob_oid": null,
                "candidate_receipt_blob_oid": RECEIPT_BLOB,
                "protected_registry_binding_preserved": null,
                "retired_inputs": [{
                    "path": "docs/ROADMAP.md",
                    "predecessor_blob_oid": BLOB,
                    "sha256": SHA256,
                    "byte_count": 12,
                    "candidate_path_exists": false,
                    "candidate_new_equivalent_paths": []
                }]
            }]
        })
    }

    fn registry() -> Value {
        json!({
            "rows": [
                {
                    "artifact_id": "masterplan",
                    "artifact_path": "/specs/masterplan.json"
                },
                {
                    "artifact_id": "repository-authority-history-only-retirement-closure-20260721",
                    "artifact_path": format!("/{RECEIPT_PATH}"),
                    "capability_overrides": {
                        "verification": {
                            "command": VERIFICATION_COMMAND
                        },
                        "validation": {"validator": RETIREMENT_RECEIPT_VALIDATOR}
                    },
                    "evidence_contract": {
                        "verification": {
                            "command": VERIFICATION_COMMAND
                        }
                    }
                }
            ]
        })
    }

    #[test]
    fn matching_protected_object_receipt_is_green() {
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn protected_parent_coverage_requires_every_retired_path() {
        let facts = json!({
            "retirement_receipt_coverage": {
                "protected_base_ref": "origin/dev",
                "current_epoch_commit_oid": COMMIT,
                "current_epoch_tree_oid": TREE,
                "protected_receipt_paths": [],
                "candidate_receipt_paths": [RECEIPT_PATH],
                "carried_receipt_paths": [],
                "new_receipt_paths": [RECEIPT_PATH],
                "scopes": [{
                    "scope_ref": MASTERPLAN_SCOPE_REF,
                    "scope_type": MASTERPLAN_SCOPE_TYPE,
                    "selectors": [{
                        "selector_type": "exact",
                        "selector": "docs/ROADMAP.md",
                        "protected_paths": ["docs/ROADMAP.md"],
                        "candidate_paths": [],
                        "removed_paths": ["docs/ROADMAP.md"],
                        "surviving_paths": [],
                        "candidate_only_paths": [],
                        "external_assertion": "not-applicable"
                    }],
                    "required_retired_paths": ["docs/ROADMAP.md"]
                }, {
                    "scope_ref": TRANSIENT_IDEAS_SCOPE_REF,
                    "scope_type": TRANSIENT_IDEAS_SCOPE_TYPE,
                    "selectors": [],
                    "required_retired_paths": []
                }],
                "required_retired_paths": ["docs/ROADMAP.md"]
            },
            "retirement_receipt_object_facts": volatile_facts()["retirement_receipt_object_facts"]
        });
        assert!(evaluate_history_only_retirement_receipt_coverage(&[receipt()], &facts).is_empty());

        let findings = evaluate_history_only_retirement_receipt_coverage(&[], &facts);
        assert!(
            findings.contains(&invalid(
                "retirement_receipt_coverage.scope.artifact:masterplan.missing.docs/ROADMAP.md"
            )),
            "{findings:?}"
        );
    }

    #[test]
    fn first_post_merge_pr_carries_receipts_without_rebinding_their_baseline() {
        let mut carried = receipt();
        carried["scope_ref"] = json!("artifact:masterplan");
        let facts = json!({
            "retirement_receipt_coverage": {
                "protected_base_ref": "origin/dev",
                "current_epoch_commit_oid": "4444444444444444444444444444444444444444",
                "current_epoch_tree_oid": "5555555555555555555555555555555555555555",
                "protected_receipt_paths": [RECEIPT_PATH],
                "candidate_receipt_paths": [RECEIPT_PATH],
                "carried_receipt_paths": [RECEIPT_PATH],
                "new_receipt_paths": [],
                "scopes": [{
                    "scope_ref": "artifact:masterplan",
                    "scope_type": "masterplan-retired-surfaces",
                    "selectors": [],
                    "required_retired_paths": []
                }, {
                    "scope_ref": "ADR-0388",
                    "scope_type": "transient-ideas",
                    "selectors": [],
                    "required_retired_paths": []
                }],
                "required_retired_paths": []
            },
            "retirement_receipt_object_facts": [{
                "artifact_id": "repository-authority-history-only-retirement-closure-20260721",
                "receipt_path": RECEIPT_PATH,
                "protected_base_ref": "origin/dev",
                "receipt_epoch": "carried",
                "scope_ref": "artifact:masterplan",
                "scope_type": "masterplan-retired-surfaces",
                "baseline_commit_oid": COMMIT,
                "baseline_tree_oid": TREE,
                "baseline_is_ancestor_of_current_epoch": true,
                "protected_receipt_blob_oid": "6666666666666666666666666666666666666666",
                "candidate_receipt_blob_oid": "6666666666666666666666666666666666666666",
                "protected_registry_binding_preserved": true,
                "retired_inputs": []
            }]
        });

        let findings = evaluate_history_only_retirement_receipt_coverage(&[carried], &facts);
        assert!(
            findings.is_empty(),
            "a protected receipt keeps its original reachable baseline after promotion: {findings:?}"
        );
    }

    #[test]
    fn receipt_scope_cannot_launder_inputs_across_authority_scopes() {
        let mut scoped_receipt = receipt();
        scoped_receipt["scope_ref"] = json!("artifact:masterplan");
        let mut facts = volatile_facts();
        facts["retirement_receipt_object_facts"][0]["scope_ref"] = json!("ADR-0388");
        facts["retirement_receipt_object_facts"][0]["scope_type"] = json!("transient-ideas");

        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &scoped_receipt,
            &scm_facts(),
            &facts,
            &registry(),
        );

        assert!(
            findings.contains(&invalid("scope_ref.object_fact_mismatch")),
            "candidate-authored scope labels must not permit cross-scope input swapping: {findings:?}"
        );
    }

    #[test]
    fn scope_authority_requires_exact_masterplan_binding_or_an_accepted_adr() {
        let mut missing_masterplan = registry();
        missing_masterplan["rows"]
            .as_array_mut()
            .unwrap()
            .retain(|row| row["artifact_id"] != "masterplan");
        let masterplan_findings = evaluate_history_only_retirement_receipt_with_decisions(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &volatile_facts(),
            &missing_masterplan,
            &Value::Null,
        );
        assert!(
            masterplan_findings.contains(&invalid("scope_ref.authority")),
            "masterplan scope must resolve through the exact registry binding: {masterplan_findings:?}"
        );

        let mut idea_receipt = receipt();
        idea_receipt["scope_ref"] = json!(TRANSIENT_IDEAS_SCOPE_REF);
        let mut idea_facts = volatile_facts();
        idea_facts["retirement_receipt_object_facts"][0]["scope_ref"] =
            json!(TRANSIENT_IDEAS_SCOPE_REF);
        idea_facts["retirement_receipt_object_facts"][0]["scope_type"] =
            json!(TRANSIENT_IDEAS_SCOPE_TYPE);
        let proposed = json!({"decisions": [{
            "adr": TRANSIENT_IDEAS_SCOPE_REF,
            "status": "Proposed"
        }]});
        let proposed_findings = evaluate_history_only_retirement_receipt_with_decisions(
            RECEIPT_PATH,
            &idea_receipt,
            &scm_facts(),
            &idea_facts,
            &registry(),
            &proposed,
        );
        assert!(
            proposed_findings.contains(&invalid("scope_ref.authority")),
            "a non-accepted ADR cannot own a retirement scope: {proposed_findings:?}"
        );

        let accepted = json!({"decisions": [{
            "adr": TRANSIENT_IDEAS_SCOPE_REF,
            "status": "Accepted"
        }]});
        let accepted_findings = evaluate_history_only_retirement_receipt_with_decisions(
            RECEIPT_PATH,
            &idea_receipt,
            &scm_facts(),
            &idea_facts,
            &registry(),
            &accepted,
        );
        assert!(
            !accepted_findings.contains(&invalid("scope_ref.authority")),
            "the unique accepted ADR is the idea-scope authority: {accepted_findings:?}"
        );
    }

    #[test]
    fn protected_receipt_bytes_and_registry_binding_are_immutable() {
        let mut scoped_receipt = receipt();
        scoped_receipt["scope_ref"] = json!("artifact:masterplan");
        let mut facts = volatile_facts();
        let object_fact = &mut facts["retirement_receipt_object_facts"][0];
        object_fact["receipt_epoch"] = json!("carried");
        object_fact["scope_ref"] = json!("artifact:masterplan");
        object_fact["scope_type"] = json!("masterplan-retired-surfaces");
        object_fact["baseline_is_ancestor_of_current_epoch"] = json!(true);
        object_fact["protected_receipt_blob_oid"] =
            json!("6666666666666666666666666666666666666666");
        object_fact["candidate_receipt_blob_oid"] =
            json!("7777777777777777777777777777777777777777");
        object_fact["protected_registry_binding_preserved"] = json!(false);

        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &scoped_receipt,
            &scm_facts(),
            &facts,
            &registry(),
        );

        for key in [
            "protected_receipt_blob_oid.immutable",
            "protected_registry_binding_preserved",
        ] {
            assert!(
                findings.contains(&invalid(key)),
                "missing protected carry-forward guard {key}: {findings:?}"
            );
        }
    }

    #[test]
    fn selector_survivors_and_candidate_only_resurrections_fail_closed() {
        let mut scoped_receipt = receipt();
        scoped_receipt["scope_ref"] = json!("artifact:masterplan");
        let facts = json!({
            "retirement_receipt_coverage": {
                "protected_base_ref": "origin/dev",
                "current_epoch_commit_oid": COMMIT,
                "current_epoch_tree_oid": TREE,
                "protected_receipt_paths": [],
                "candidate_receipt_paths": [RECEIPT_PATH],
                "carried_receipt_paths": [],
                "new_receipt_paths": [RECEIPT_PATH],
                "scopes": [{
                    "scope_ref": "artifact:masterplan",
                    "scope_type": "masterplan-retired-surfaces",
                    "selectors": [{
                        "selector_type": "glob",
                        "selector": ".omc/**",
                        "protected_paths": [".omc/old.txt"],
                        "candidate_paths": [".omc/old.txt", ".omc/new.txt"],
                        "removed_paths": [],
                        "surviving_paths": [".omc/old.txt"],
                        "candidate_only_paths": [".omc/new.txt"],
                        "external_assertion": "not-applicable"
                    }, {
                        "selector_type": "external",
                        "selector": "~/.omx/**",
                        "protected_paths": [".omx/claimed.txt"],
                        "candidate_paths": [],
                        "removed_paths": [],
                        "surviving_paths": [],
                        "candidate_only_paths": [],
                        "external_assertion": "outside-repository-authority-not-inspected"
                    }],
                    "required_retired_paths": []
                }]
            }
        });

        let findings = evaluate_history_only_retirement_receipt_coverage(&[scoped_receipt], &facts);
        for key in [
            "retirement_receipt_coverage.survivor.artifact:masterplan..omc/old.txt",
            "retirement_receipt_coverage.candidate_only.artifact:masterplan..omc/new.txt",
            "retirement_receipt_coverage.scopes[0].selectors[1].external_path_claim",
        ] {
            assert!(
                findings.contains(&invalid(key)),
                "missing selector-state guard {key}: {findings:?}"
            );
        }
    }

    #[test]
    fn tampered_digest_fails_closed() {
        let mut receipt = receipt();
        receipt["retired_inputs"][0]["sha256"] =
            json!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );
        assert!(findings.contains(&Finding::new(
            RETIREMENT_RECEIPT_CODE,
            "docs/ROADMAP.md.sha256.object_fact_mismatch"
        )));
    }

    #[test]
    fn singular_successor_and_candidate_resurrection_fail_closed() {
        let mut receipt = receipt();
        receipt["retired_inputs"][0]
            .as_object_mut()
            .expect("retired input")
            .insert("successor_ref".to_owned(), json!("/specs/masterplan.json"));
        receipt["retired_inputs"][0]
            .as_object_mut()
            .expect("retired input")
            .remove("successor_refs");
        let mut scm = scm_facts();
        scm["tracked_paths"] = json!(["docs/ROADMAP.md", "specs/masterplan.json"]);
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm,
            &volatile_facts(),
            &registry(),
        );
        for key in [
            "docs/ROADMAP.md.successor_refs",
            "docs/ROADMAP.md.candidate_tracked_path_present",
        ] {
            assert!(
                findings.contains(&Finding::new(RETIREMENT_RECEIPT_CODE, key)),
                "missing {key}: {findings:?}"
            );
        }
    }

    #[test]
    fn missing_baseline_commit_oid_fails_closed() {
        let mut receipt = receipt();
        receipt["baseline"]
            .as_object_mut()
            .expect("baseline")
            .remove("commit_oid");
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );
        assert!(
            findings.contains(&invalid("baseline.commit_oid")),
            "{findings:?}"
        );
    }

    #[test]
    fn malformed_predecessor_blob_oid_fails_closed() {
        let mut receipt = receipt();
        receipt["retired_inputs"][0]["predecessor_blob_oid"] = json!("NOT-AN-OID");
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );
        assert!(
            findings.contains(&invalid("docs/ROADMAP.md.predecessor_blob_oid")),
            "{findings:?}"
        );
    }

    #[test]
    fn wrong_baseline_tree_object_fact_fails_closed() {
        let mut facts = volatile_facts();
        facts["retirement_receipt_object_facts"][0]["baseline_tree_oid"] = json!(COMMIT);
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &facts,
            &registry(),
        );
        assert!(
            findings.contains(&invalid("baseline.tree_oid.object_fact_mismatch")),
            "{findings:?}"
        );
    }

    #[test]
    fn wrong_baseline_commit_object_fact_fails_closed() {
        let mut facts = volatile_facts();
        facts["retirement_receipt_object_facts"][0]["baseline_commit_oid"] = json!(TREE);
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &facts,
            &registry(),
        );
        assert!(
            findings.contains(&invalid("baseline.commit_oid.object_fact_mismatch")),
            "{findings:?}"
        );
    }

    #[test]
    fn mismatched_byte_count_fails_closed() {
        let mut facts = volatile_facts();
        facts["retirement_receipt_object_facts"][0]["retired_inputs"][0]["byte_count"] = json!(13);
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &facts,
            &registry(),
        );
        assert!(
            findings.contains(&invalid("docs/ROADMAP.md.byte_count.object_fact_mismatch")),
            "{findings:?}"
        );
    }

    #[test]
    fn missing_scm_object_fact_fails_closed() {
        let mut facts = volatile_facts();
        facts["retirement_receipt_object_facts"][0]["retired_inputs"] = json!([]);
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &facts,
            &registry(),
        );
        assert!(
            findings.contains(&invalid("docs/ROADMAP.md.object_fact_missing")),
            "{findings:?}"
        );
    }

    #[test]
    fn duplicate_scm_object_fact_fails_closed() {
        let mut facts = volatile_facts();
        let duplicate = facts["retirement_receipt_object_facts"][0]["retired_inputs"][0].clone();
        facts["retirement_receipt_object_facts"][0]["retired_inputs"]
            .as_array_mut()
            .expect("object inputs")
            .push(duplicate);
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &facts,
            &registry(),
        );
        assert!(
            findings.contains(&invalid("docs/ROADMAP.md.duplicate_object_fact")),
            "{findings:?}"
        );
    }

    #[test]
    fn extra_scm_object_fact_fails_closed() {
        let mut facts = volatile_facts();
        facts["retirement_receipt_object_facts"][0]["retired_inputs"]
            .as_array_mut()
            .expect("object inputs")
            .push(json!({"path": "docs/RETIRED.md"}));
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &facts,
            &registry(),
        );
        assert!(
            findings.contains(&invalid("docs/RETIRED.md.unexpected_object_fact")),
            "{findings:?}"
        );
    }

    #[test]
    fn python_only_registry_verification_fails_closed() {
        let mut registry = registry();
        registry["rows"][1]["capability_overrides"]["verification"]["command"] =
            json!("python3 verify_receipt.py");
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &volatile_facts(),
            &registry,
        );
        assert!(
            findings.contains(&invalid(
                "repository-authority-history-only-retirement-closure-20260721.registry_verification_command"
            )),
            "{findings:?}"
        );
    }

    #[test]
    fn unregistered_expected_absent_path_fails_closed() {
        let mut receipt = receipt();
        receipt["verification_contract"]["expected_absent_paths"] =
            json!(["docs/ROADMAP.md", "docs/RETIRED.md"]);
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );
        assert!(
            findings.contains(&invalid(
                "docs/RETIRED.md.expected_absent_paths.unregistered"
            )),
            "{findings:?}"
        );
    }

    #[test]
    fn retired_input_missing_from_expected_absent_paths_fails_closed() {
        let mut receipt = receipt();
        receipt["verification_contract"]["expected_absent_paths"] = json!([]);
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );
        assert!(
            findings.contains(&invalid("docs/ROADMAP.md.expected_absent_paths.missing")),
            "{findings:?}"
        );
    }

    #[test]
    fn candidate_path_exists_object_fact_fails_closed() {
        let mut facts = volatile_facts();
        facts["retirement_receipt_object_facts"][0]["retired_inputs"][0]["candidate_path_exists"] =
            json!(true);
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &facts,
            &registry(),
        );
        assert!(
            findings.contains(&invalid("docs/ROADMAP.md.candidate_path_exists")),
            "{findings:?}"
        );
    }

    #[test]
    fn renamed_exact_copy_of_retired_content_fails_closed() {
        let mut facts = volatile_facts();
        facts["retirement_receipt_object_facts"][0]["retired_inputs"][0]["candidate_new_equivalent_paths"] =
            json!(["docs/not-an-archive/roadmap-copy.md"]);
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &facts,
            &registry(),
        );
        assert!(
            findings.contains(&invalid("docs/ROADMAP.md.candidate_equivalent_copy")),
            "{findings:?}"
        );
    }

    #[test]
    fn duplicate_receipt_object_identity_fails_closed() {
        let mut facts = volatile_facts();
        let duplicate = facts["retirement_receipt_object_facts"][0].clone();
        facts["retirement_receipt_object_facts"]
            .as_array_mut()
            .expect("receipt facts")
            .push(duplicate);
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &facts,
            &registry(),
        );
        assert!(
            findings.contains(&invalid(&format!("{RECEIPT_PATH}.object_facts_coverage"))),
            "{findings:?}"
        );
    }

    #[test]
    fn duplicate_retired_input_identity_fails_closed() {
        let mut receipt = receipt();
        let duplicate = receipt["retired_inputs"][0].clone();
        receipt["retired_inputs"]
            .as_array_mut()
            .expect("retired inputs")
            .push(duplicate);
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );
        assert!(
            findings.contains(&invalid("docs/ROADMAP.md.duplicate_retired_input")),
            "{findings:?}"
        );
    }

    #[test]
    fn embedded_retired_content_fails_closed() {
        let mut receipt = receipt();
        receipt["retired_inputs"][0]["content"] = json!("retired source text");
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );
        assert!(
            findings.contains(&invalid("docs/ROADMAP.md.embedded_retired_content")),
            "{findings:?}"
        );
    }

    #[test]
    fn embedded_retired_content_in_scm_object_fact_fails_closed() {
        let mut facts = volatile_facts();
        facts["retirement_receipt_object_facts"][0]["retired_inputs"][0]["source_content"] =
            json!("retired source text");
        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt(),
            &scm_facts(),
            &facts,
            &registry(),
        );
        assert!(
            findings.contains(&invalid(
                "docs/ROADMAP.md.object_fact_embedded_retired_content"
            )),
            "{findings:?}"
        );
    }

    #[test]
    fn unknown_receipt_fields_cannot_smuggle_retired_content() {
        let mut receipt = receipt();
        receipt["payload"] = json!("retired source text");
        receipt["retired_inputs"][0]["body"] = json!("retired source text");
        let mut facts = volatile_facts();
        facts["retirement_receipt_object_facts"][0]["retired_inputs"][0]["data"] =
            json!("retired source text");

        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &facts,
            &registry(),
        );

        for key in [
            "receipt.unknown_field.payload",
            "docs/ROADMAP.md.unknown_field.body",
            "docs/ROADMAP.md.object_fact_unknown_field.data",
        ] {
            assert!(
                findings.contains(&invalid(key)),
                "missing {key}: {findings:?}"
            );
        }
    }

    #[test]
    fn free_form_receipt_narrative_is_rejected() {
        let mut receipt = receipt();
        receipt["retired_inputs"][0]
            .as_object_mut()
            .expect("retired input")
            .insert(
                "successor_coverage".to_owned(),
                json!("a readable narrative copy of retired authority"),
            );
        receipt["effects"]["runtime_effect"] =
            json!("a readable narrative copy of retired authority");
        receipt["retired_inputs"][0]["successor_refs"] =
            json!(["a readable narrative copy of retired authority"]);

        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );

        for key in [
            "docs/ROADMAP.md.unknown_field.successor_coverage",
            "effects.runtime_effect",
            "docs/ROADMAP.md.successor_refs[0].reference",
        ] {
            assert!(
                findings.contains(&invalid(key)),
                "missing {key}: {findings:?}"
            );
        }
    }

    #[test]
    fn encoded_payload_cannot_masquerade_as_successor_reference() {
        let mut receipt = receipt();
        receipt["retired_inputs"][0]["successor_refs"] = json!(["A".repeat(512)]);

        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );

        for key in [
            "docs/ROADMAP.md.successor_refs",
            "docs/ROADMAP.md.successor_refs[0].reference",
        ] {
            assert!(
                findings.contains(&invalid(key)),
                "missing {key}: {findings:?}"
            );
        }
    }

    #[test]
    fn unknown_adr_successor_reference_fails_closed() {
        let mut receipt = receipt();
        receipt["retired_inputs"][0]["successor_refs"] = json!(["ADR-9999"]);

        let findings = evaluate_history_only_retirement_receipt_with_decisions(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
            &json!({"decisions": [{"adr": "ADR-0515"}]}),
        );

        assert!(
            findings.contains(&invalid("docs/ROADMAP.md.successor_refs[0].reference")),
            "{findings:?}"
        );
    }

    #[test]
    fn archive_count_and_exact_gate_set_are_mandatory() {
        let mut receipt = receipt();
        receipt["verification_contract"]
            .as_object_mut()
            .expect("verification contract")
            .remove("expected_tracked_readable_archive_directory_count");
        receipt["verification_contract"]["required_gates"] = json!([RETIREMENT_RECEIPT_VALIDATOR]);

        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );

        for key in [
            "verification_contract.expected_tracked_readable_archive_directory_count",
            "verification_contract.required_gates.exact_set",
        ] {
            assert!(
                findings.contains(&invalid(key)),
                "missing {key}: {findings:?}"
            );
        }
    }

    #[test]
    fn non_ascii_recorded_at_fails_closed_without_panicking() {
        let mut receipt = receipt();
        receipt["recorded_at"] = json!("2026-0é-21");

        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );

        assert!(findings.contains(&invalid("recorded_at")), "{findings:?}");
    }

    #[test]
    fn aliased_retired_path_fails_closed() {
        let mut receipt = receipt();
        receipt["retired_inputs"][0]["path"] = json!("./docs/ROADMAP.md");

        let findings = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );

        assert!(
            findings.contains(&invalid("retired_inputs[0].path")),
            "{findings:?}"
        );
    }

    #[test]
    fn violation_output_is_deterministic_for_reordered_inputs() {
        let mut receipt = receipt();
        receipt["retired_inputs"][0]["byte_count"] = json!("twelve");
        receipt["retired_inputs"][0]["sha256"] = json!("invalid");
        let first = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );
        let second = evaluate_history_only_retirement_receipt(
            RECEIPT_PATH,
            &receipt,
            &scm_facts(),
            &volatile_facts(),
            &registry(),
        );
        assert_eq!(first, second);
        assert_eq!(
            first.iter().map(|finding| &finding.key).collect::<Vec<_>>(),
            [
                "docs/ROADMAP.md.byte_count",
                "docs/ROADMAP.md.byte_count.object_fact_mismatch",
                "docs/ROADMAP.md.sha256",
                "docs/ROADMAP.md.sha256.object_fact_mismatch",
            ]
        );
    }
}
