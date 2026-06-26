//! # cloud-ci-generated-artifact-control-plane
//!
//! Validates a repo's generated-artifact policy manifest against the SCM facts snapshot
//! materialized from the candidate tree used by cloud-ci. This gate exists because generated
//! outputs are product artifacts, not local build trash and not contributor-owned merge surfaces.
//! A public adopter should be able to copy the manifest shape, declare its generated artifacts,
//! and get the same hermetic gate behavior under any runner that can provide the manifest plus
//! SCM-facts JSON.
//!
//! ## Hermetic contract
//! Input A: `registry/generated-artifact-control-plane.json` — the repo-authored policy for each
//! generated artifact family. Input B: the candidate-tree materialized SCM-facts snapshot
//! (`tracked_paths`). The gate calls no VCS, shell, network, or CI provider API. GitHub Actions
//! is only a bridge runner; the gate itself is a Rust predicate over declared data.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// The gate id, matching the workflow matrix identity and public product concept.
pub const GATE_ID: &str = "cloud-ci-generated-artifact-control-plane";

const GENERATED_OUTPUT_CONFLICT_POLICY: &str =
    "reject-generated-output-conflict-hunks-regenerate-from-source-tree";

const ARTIFACT_CLASSES: [&str; 7] = [
    "authoritative-source",
    "sharded-projection",
    "main-materialized-aggregate",
    "append-only-ledger",
    "ephemeral-build-output",
    "review-artifact",
    "scm-facts-boundary-snapshot",
];

const MATERIALIZATION_MODES: [&str; 6] = [
    "source-authored",
    "branch-committed-regenerated-until-controller-materialization",
    "merge-candidate-regenerated",
    "main-branch-materialized",
    "ci-artifact-only",
    // De-commit class (ADR-0595): the artifact is a pure derivation that is intentionally NOT
    // tracked in git. It is derived on demand and materialized out-of-graph for consumers, so it
    // is never a contributor merge surface. A declared path in this mode is EXEMPT from the
    // declared-path-must-be-tracked predicate below.
    "not-tracked-in-git",
];

/// Materialization mode marking a declared generated artifact as intentionally de-committed
/// (derive-on-demand). Paths in this mode must NOT be tracked in git and are therefore exempt
/// from `generated_artifact_declared_path_not_tracked`.
const NOT_TRACKED_IN_GIT_MODE: &str = "not-tracked-in-git";

/// Materialization modes whose SEMANTICS is "the artifact is absent from the committed tree and
/// derived on demand" (the de-commit class). A frozen-reference/baseline artifact declared in any
/// of these modes is the #828 deadlock defect, because the firewall ratchet materializes the
/// frozen reference from the committed git blob at the merge-base (`git show <merge_base>:<path>`)
/// and a de-committed path makes that blob empty. This is DATA: extend the list, not the predicate,
/// when a new derive-on-demand mode is added.
const DECOMMIT_MATERIALIZATION_MODES: [&str; 1] = [NOT_TRACKED_IN_GIT_MODE];

fn is_decommit_materialization_mode(mode: &str) -> bool {
    DECOMMIT_MATERIALIZATION_MODES.contains(&mode)
}

const MERGE_POLICIES: [&str; 5] = [
    "normal-source-merge",
    "append-only-union-with-invariant-check",
    "never-manual-merge-regenerate-from-source-tree",
    "controller-owned-main-materialization",
    "not-tracked-in-git",
];

const LIFECYCLE_LAYER_IDS: [&str; 8] = [
    "project-creation",
    "local-authoring",
    "pre-commit-pre-push",
    "pull-request-presubmit",
    "merge-queue-projected-state",
    "postsubmit-main-materialization",
    "release-and-deployment",
    "operations-drift-repair",
];

const MANIFEST_FIELDS: [&str; 10] = [
    "$schema",
    "schema_version",
    "canonical_name",
    "ci_product_surface",
    "purpose",
    "public_product_contract",
    "final_tree_materialization",
    "generated_path_rules",
    "artifacts",
    "development_lifecycle_enforcement_layers",
];

const ARTIFACT_FIELDS: [&str; 10] = [
    "artifact_id",
    "path",
    "artifact_class",
    "materialization_mode",
    "merge_policy",
    "owner_team",
    "generator",
    "source_inputs",
    "final_tree_validation",
    "public_product_contract",
];

const GENERATOR_FIELDS: [&str; 6] = [
    "runner",
    "generator_target",
    "operation_id",
    "parameters",
    "input_contract",
    "output_mode",
];

const GENERATOR_RUNNERS: [&str; 2] = ["buck2", "oya-ci-native-controller"];

const GENERATOR_OUTPUT_MODES: [&str; 3] = [
    "stdout-json",
    "declared-artifact-path-write",
    "controller-materialized",
];

const GENERATOR_INPUT_CONTRACTS: [&str; 4] = [
    "repo-root",
    "declared-source-inputs",
    "scm-facts-snapshot",
    "full-depth-scm",
];

const LIFECYCLE_LAYER_FIELDS: [&str; 4] = ["layer_id", "name", "automation", "enforcement"];

const GENERATED_PATH_RULE_FIELDS: [&str; 5] = [
    "rule_id",
    "rule_kind",
    "pattern",
    "description",
    "exclude_file_names",
];

const GENERATED_PATH_RULE_KINDS: [&str; 4] =
    ["path_component", "path_prefix", "path_suffix", "file_name"];

const FINAL_TREE_MATERIALIZATION_FIELDS: [&str; 7] = [
    "controller_id",
    "presubmit_authority",
    "postsubmit_authority",
    "protected_branch_trigger",
    "manual_conflict_resolution_policy",
    "drift_repair_policy",
    "portable_runner_contract",
];

const PRESUBMIT_AUTHORITIES: [&str; 2] = ["pull-request-presubmit", "merge-queue-projected-state"];

const DRIFT_REPAIR_POLICIES: [&str; 2] = [
    "fail-closed-and-open-generated-only-repair-pr",
    "fail-closed-manual-generated-only-repair",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
}

impl Finding {
    fn new(code: &str, key: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffPolicyViolation {
    pub status: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_findings(findings: &BTreeSet<Finding>) -> Self {
        let violations = findings
            .iter()
            .map(|finding| finding.code.clone())
            .collect();
        let verdict = if findings.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self {
            verdict,
            violations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeclaredArtifact {
    artifact_id: String,
    path: String,
    artifact_class: String,
    materialization_mode: String,
    merge_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GeneratedPathRule {
    rule_id: String,
    rule_kind: String,
    pattern: String,
    exclude_file_names: BTreeSet<String>,
}

fn allowed(values: &[&str], candidate: &str) -> bool {
    values.contains(&candidate)
}

fn required_str<'a>(object: &'a Value, key: &str) -> Option<&'a str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}

fn required_array<'a>(object: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    object
        .get(key)
        .and_then(Value::as_array)
        .filter(|value| !value.is_empty())
}

fn validate_object_fields(
    value: &Value,
    allowed_fields: &[&str],
    not_object_code: &str,
    unknown_field_code: &str,
    scope: &str,
    findings: &mut BTreeSet<Finding>,
) -> bool {
    let Some(object) = value.as_object() else {
        findings.insert(Finding::new(not_object_code, scope));
        return false;
    };

    for field in object.keys() {
        if !allowed(allowed_fields, field) {
            findings.insert(Finding::new(unknown_field_code, format!("{scope}.{field}")));
        }
    }

    true
}

fn required_string_array(
    object: &Value,
    key: &str,
    owner: &str,
    missing_code: &str,
    item_code: &str,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(items) = required_array(object, key) else {
        findings.insert(Finding::new(missing_code, owner));
        return;
    };

    for (index, item) in items.iter().enumerate() {
        if item.as_str().is_none_or(|value| value.trim().is_empty()) {
            findings.insert(Finding::new(item_code, format!("{owner}.{key}[{index}]")));
        }
    }
}

fn validate_generator(
    artifact: &Value,
    artifact_id: &str,
    artifact_class: &str,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(generator) = artifact.get("generator") else {
        findings.insert(Finding::new(
            "generated_artifact_manifest_generator_missing",
            artifact_id,
        ));
        return;
    };
    if !validate_object_fields(
        generator,
        &GENERATOR_FIELDS,
        "generated_artifact_manifest_generator_not_object",
        "generated_artifact_manifest_generator_unknown_field",
        artifact_id,
        findings,
    ) {
        return;
    }

    let runner = required_str(generator, "runner");
    let generator_target = required_str(generator, "generator_target");
    let operation_id = required_str(generator, "operation_id");
    let output_mode = required_str(generator, "output_mode");

    match runner {
        Some(value) if allowed(&GENERATOR_RUNNERS, value) => {}
        Some(value) => {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generator_runner_unknown",
                format!("{artifact_id}.{value}"),
            ));
        }
        None => {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generator_runner_missing",
                artifact_id,
            ));
        }
    };

    match output_mode {
        Some(value) if allowed(&GENERATOR_OUTPUT_MODES, value) => {}
        Some(value) => {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generator_output_mode_unknown",
                format!("{artifact_id}.{value}"),
            ));
        }
        None => {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generator_output_mode_missing",
                artifact_id,
            ));
        }
    };

    match generator_target {
        Some(value) => {
            if runner == Some("buck2") && !value.starts_with("//") {
                findings.insert(Finding::new(
                    "generated_artifact_manifest_generator_buck2_target_not_canonical",
                    artifact_id,
                ));
            }
            if runner == Some("oya-ci-native-controller")
                && !value.starts_with("oya-ci://generated-artifact-controller/")
            {
                findings.insert(Finding::new(
                    "generated_artifact_manifest_generator_controller_target_not_canonical",
                    artifact_id,
                ));
            }
        }
        None => {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generator_target_missing",
                artifact_id,
            ));
        }
    }
    if operation_id.is_none() {
        findings.insert(Finding::new(
            "generated_artifact_manifest_generator_operation_id_missing",
            artifact_id,
        ));
    }
    let Some(parameters) = generator.get("parameters") else {
        findings.insert(Finding::new(
            "generated_artifact_manifest_generator_parameters_missing",
            artifact_id,
        ));
        return;
    };
    let Some(parameters) = parameters.as_object() else {
        findings.insert(Finding::new(
            "generated_artifact_manifest_generator_parameters_not_object",
            artifact_id,
        ));
        return;
    };
    for (key, value) in parameters {
        if value.as_str().is_none_or(|value| value.trim().is_empty()) {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generator_parameter_not_string",
                format!("{artifact_id}.{key}"),
            ));
        }
    }

    let Some(input_contracts) = required_array(generator, "input_contract") else {
        findings.insert(Finding::new(
            "generated_artifact_manifest_generator_input_contract_missing",
            artifact_id,
        ));
        return;
    };
    let mut input_contract_set = BTreeSet::new();
    for (index, item) in input_contracts.iter().enumerate() {
        let Some(value) = item.as_str().filter(|value| !value.trim().is_empty()) else {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generator_input_contract_item_not_string",
                format!("{artifact_id}.input_contract[{index}]"),
            ));
            continue;
        };
        if !allowed(&GENERATOR_INPUT_CONTRACTS, value) {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generator_input_contract_unknown",
                format!("{artifact_id}.{value}"),
            ));
        }
        input_contract_set.insert(value.to_owned());
    }

    if runner == Some("oya-ci-native-controller") && output_mode != Some("controller-materialized")
    {
        findings.insert(Finding::new(
            "generated_artifact_manifest_generator_controller_output_not_materialized",
            artifact_id,
        ));
    }
    if artifact_class == "scm-facts-boundary-snapshot" && runner != Some("buck2") {
        findings.insert(Finding::new(
            "generated_artifact_scm_facts_generator_not_buck2_boundary",
            artifact_id,
        ));
    }
    if artifact_class == "scm-facts-boundary-snapshot"
        && output_mode != Some("declared-artifact-path-write")
    {
        findings.insert(Finding::new(
            "generated_artifact_scm_facts_generator_not_declared_path_write",
            artifact_id,
        ));
    }
    if artifact_class == "scm-facts-boundary-snapshot"
        && !input_contract_set.contains("full-depth-scm")
    {
        findings.insert(Finding::new(
            "generated_artifact_scm_facts_generator_missing_full_depth_scm_contract",
            artifact_id,
        ));
    }
}

fn generated_path_rule_matches(path: &str, rule: &GeneratedPathRule) -> bool {
    let file_name = path.rsplit('/').next().unwrap_or(path);
    if rule.exclude_file_names.contains(file_name) {
        return false;
    }

    match rule.rule_kind.as_str() {
        "path_component" => path.split('/').any(|component| component == rule.pattern),
        "path_prefix" => path.starts_with(&rule.pattern),
        "path_suffix" => path.ends_with(&rule.pattern),
        "file_name" => file_name == rule.pattern,
        _ => false,
    }
}

fn is_tracked_generated_artifact_path(path: &str, rules: &[GeneratedPathRule]) -> bool {
    rules
        .iter()
        .any(|rule| generated_path_rule_matches(path, rule))
}

fn tracked_generated_artifact_paths(
    scm_facts: &Value,
    rules: &[GeneratedPathRule],
) -> BTreeSet<String> {
    scm_facts
        .get("tracked_paths")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|path| is_tracked_generated_artifact_path(path, rules))
        .map(ToOwned::to_owned)
        .collect()
}

fn diff_candidate_path<'a>(status: &str, paths: &'a [&'a str]) -> Option<&'a str> {
    if status.starts_with('D') {
        return None;
    }
    if status.starts_with('R') || status.starts_with('C') {
        return paths.get(1).copied();
    }
    paths.first().copied()
}

/// Productized bridge predicate for presubmit diff surfaces. The caller supplies a
/// `git diff --name-status`-compatible stream, but generated-path classification comes from the
/// manifest's `generated_path_rules`, not from `.gitignore` or runner-local ignore heuristics.
pub fn generated_output_diff_policy_violations(
    manifest: &Value,
    diff_name_status: &str,
) -> (BTreeSet<Finding>, Vec<DiffPolicyViolation>) {
    let mut findings = BTreeSet::new();
    let generated_path_rules = parse_generated_path_rules(manifest, &mut findings);
    parse_declared_artifacts(manifest, &mut findings);
    let allowed_generated_edit_paths =
        diff_policy_allowed_generated_edit_paths(manifest, &mut findings);
    if !findings.is_empty() {
        return (findings, Vec::new());
    }

    let violations = diff_name_status
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let status = fields.next()?.to_owned();
            let paths = fields.collect::<Vec<_>>();
            let path = diff_candidate_path(&status, &paths)?;
            if is_tracked_generated_artifact_path(path, &generated_path_rules)
                && !allowed_generated_edit_paths.contains(path)
            {
                Some(DiffPolicyViolation {
                    status,
                    path: path.to_owned(),
                })
            } else {
                None
            }
        })
        .collect();

    (findings, violations)
}

fn diff_policy_allowed_generated_edit_paths(
    _manifest: &Value,
    _findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    // Contributor PRs must not own generated-output bytes. Declared artifacts can be regenerated
    // or controller-materialized by cloud-ci, but the merge surface is the source tree.
    BTreeSet::new()
}

fn validate_lifecycle_layers(manifest: &Value, findings: &mut BTreeSet<Finding>) {
    let Some(layers) = required_array(manifest, "development_lifecycle_enforcement_layers") else {
        findings.insert(Finding::new(
            "generated_artifact_manifest_lifecycle_layers_missing",
            "development_lifecycle_enforcement_layers",
        ));
        return;
    };

    let mut seen = BTreeSet::new();
    for (index, layer) in layers.iter().enumerate() {
        let scope = format!("development_lifecycle_enforcement_layers[{index}]");
        if !validate_object_fields(
            layer,
            &LIFECYCLE_LAYER_FIELDS,
            "generated_artifact_manifest_lifecycle_layer_not_object",
            "generated_artifact_manifest_lifecycle_layer_unknown_field",
            &scope,
            findings,
        ) {
            continue;
        }

        let Some(layer_id) = required_str(layer, "layer_id") else {
            findings.insert(Finding::new(
                "generated_artifact_manifest_lifecycle_layer_id_missing",
                format!("{scope}.layer_id"),
            ));
            continue;
        };
        if !allowed(&LIFECYCLE_LAYER_IDS, layer_id) {
            findings.insert(Finding::new(
                "generated_artifact_manifest_lifecycle_layer_unknown",
                layer_id,
            ));
        }
        if !seen.insert(layer_id.to_owned()) {
            findings.insert(Finding::new(
                "generated_artifact_manifest_lifecycle_layer_duplicate",
                layer_id,
            ));
        }
        for field in ["name", "automation", "enforcement"] {
            if required_str(layer, field).is_none() {
                findings.insert(Finding::new(
                    "generated_artifact_manifest_lifecycle_layer_field_missing",
                    format!("{layer_id}.{field}"),
                ));
            }
        }
    }

    for required_layer in LIFECYCLE_LAYER_IDS {
        if !seen.contains(required_layer) {
            findings.insert(Finding::new(
                "generated_artifact_manifest_lifecycle_layer_required_id_missing",
                required_layer,
            ));
        }
    }
}

fn validate_final_tree_materialization(manifest: &Value, findings: &mut BTreeSet<Finding>) {
    let Some(policy) = manifest.get("final_tree_materialization") else {
        findings.insert(Finding::new(
            "generated_artifact_manifest_final_tree_materialization_missing",
            "final_tree_materialization",
        ));
        return;
    };
    if !validate_object_fields(
        policy,
        &FINAL_TREE_MATERIALIZATION_FIELDS,
        "generated_artifact_manifest_final_tree_materialization_not_object",
        "generated_artifact_manifest_final_tree_materialization_unknown_field",
        "final_tree_materialization",
        findings,
    ) {
        return;
    }

    if required_str(policy, "controller_id") != Some("oya-ci-generated-artifact-controller") {
        findings.insert(Finding::new(
            "generated_artifact_manifest_final_tree_controller_invalid",
            "controller_id",
        ));
    }

    let Some(presubmit_authority) = required_str(policy, "presubmit_authority") else {
        findings.insert(Finding::new(
            "generated_artifact_manifest_final_tree_presubmit_authority_missing",
            "presubmit_authority",
        ));
        return;
    };
    if !allowed(&PRESUBMIT_AUTHORITIES, presubmit_authority) {
        findings.insert(Finding::new(
            "generated_artifact_manifest_final_tree_presubmit_authority_unknown",
            presubmit_authority,
        ));
    }

    if required_str(policy, "postsubmit_authority") != Some("postsubmit-main-materialization") {
        findings.insert(Finding::new(
            "generated_artifact_manifest_final_tree_postsubmit_authority_invalid",
            "postsubmit_authority",
        ));
    }
    if required_str(policy, "protected_branch_trigger").is_none() {
        findings.insert(Finding::new(
            "generated_artifact_manifest_final_tree_protected_branch_trigger_missing",
            "protected_branch_trigger",
        ));
    }
    if required_str(policy, "manual_conflict_resolution_policy")
        != Some(GENERATED_OUTPUT_CONFLICT_POLICY)
    {
        findings.insert(Finding::new(
            "generated_artifact_manifest_manual_generated_conflict_policy_invalid",
            "manual_conflict_resolution_policy",
        ));
    }

    let Some(drift_repair_policy) = required_str(policy, "drift_repair_policy") else {
        findings.insert(Finding::new(
            "generated_artifact_manifest_final_tree_drift_repair_policy_missing",
            "drift_repair_policy",
        ));
        return;
    };
    if !allowed(&DRIFT_REPAIR_POLICIES, drift_repair_policy) {
        findings.insert(Finding::new(
            "generated_artifact_manifest_final_tree_drift_repair_policy_unknown",
            drift_repair_policy,
        ));
    }
    if required_str(policy, "portable_runner_contract").is_none() {
        findings.insert(Finding::new(
            "generated_artifact_manifest_final_tree_portable_runner_contract_missing",
            "portable_runner_contract",
        ));
    }
}

fn parse_generated_path_rules(
    manifest: &Value,
    findings: &mut BTreeSet<Finding>,
) -> Vec<GeneratedPathRule> {
    let Some(rules) = manifest
        .get("generated_path_rules")
        .and_then(Value::as_array)
    else {
        findings.insert(Finding::new(
            "generated_artifact_manifest_generated_path_rules_missing",
            "generated_path_rules",
        ));
        return Vec::new();
    };
    if rules.is_empty() {
        findings.insert(Finding::new(
            "generated_artifact_manifest_generated_path_rules_empty",
            "generated_path_rules",
        ));
    }

    let mut parsed = Vec::new();
    let mut seen = BTreeSet::new();

    for (index, rule) in rules.iter().enumerate() {
        let scope = format!("generated_path_rules[{index}]");
        if !validate_object_fields(
            rule,
            &GENERATED_PATH_RULE_FIELDS,
            "generated_artifact_manifest_generated_path_rule_not_object",
            "generated_artifact_manifest_generated_path_rule_unknown_field",
            &scope,
            findings,
        ) {
            continue;
        }

        let Some(rule_id) = required_str(rule, "rule_id") else {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generated_path_rule_id_missing",
                format!("{scope}.rule_id"),
            ));
            continue;
        };
        if !seen.insert(rule_id.to_owned()) {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generated_path_rule_id_duplicate",
                rule_id,
            ));
        }

        let Some(rule_kind) = required_str(rule, "rule_kind") else {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generated_path_rule_kind_missing",
                rule_id,
            ));
            continue;
        };
        if !allowed(&GENERATED_PATH_RULE_KINDS, rule_kind) {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generated_path_rule_kind_unknown",
                format!("{rule_id}.{rule_kind}"),
            ));
        }

        let Some(pattern) = required_str(rule, "pattern") else {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generated_path_rule_pattern_missing",
                rule_id,
            ));
            continue;
        };
        if required_str(rule, "description").is_none() {
            findings.insert(Finding::new(
                "generated_artifact_manifest_generated_path_rule_description_missing",
                rule_id,
            ));
        }

        let mut exclude_file_names = BTreeSet::new();
        if let Some(items) = rule.get("exclude_file_names") {
            let Some(items) = items.as_array() else {
                findings.insert(Finding::new(
                    "generated_artifact_manifest_generated_path_rule_excludes_not_array",
                    rule_id,
                ));
                continue;
            };
            for (item_index, item) in items.iter().enumerate() {
                let Some(value) = item.as_str().filter(|value| !value.trim().is_empty()) else {
                    findings.insert(Finding::new(
                        "generated_artifact_manifest_generated_path_rule_exclude_not_string",
                        format!("{rule_id}.exclude_file_names[{item_index}]"),
                    ));
                    continue;
                };
                exclude_file_names.insert(value.to_owned());
            }
        }

        parsed.push(GeneratedPathRule {
            rule_id: rule_id.to_owned(),
            rule_kind: rule_kind.to_owned(),
            pattern: pattern.to_owned(),
            exclude_file_names,
        });
    }

    parsed
}

fn parse_declared_artifacts(
    manifest: &Value,
    findings: &mut BTreeSet<Finding>,
) -> Vec<DeclaredArtifact> {
    validate_object_fields(
        manifest,
        &MANIFEST_FIELDS,
        "generated_artifact_manifest_not_object",
        "generated_artifact_manifest_unknown_field",
        "manifest",
        findings,
    );

    if manifest.get("schema_version").and_then(Value::as_u64) != Some(1) {
        findings.insert(Finding::new(
            "generated_artifact_manifest_schema_version_not_supported",
            "schema_version",
        ));
    }
    if required_str(manifest, "canonical_name") != Some("generated-artifact-control-plane") {
        findings.insert(Finding::new(
            "generated_artifact_manifest_canonical_name_missing",
            "canonical_name",
        ));
    }
    if required_str(manifest, "ci_product_surface") != Some("oya-ci") {
        findings.insert(Finding::new(
            "generated_artifact_manifest_ci_product_surface_missing",
            "ci_product_surface",
        ));
    }
    if required_str(manifest, "public_product_contract").is_none() {
        findings.insert(Finding::new(
            "generated_artifact_manifest_public_product_contract_missing",
            "public_product_contract",
        ));
    }
    validate_final_tree_materialization(manifest, findings);
    validate_lifecycle_layers(manifest, findings);

    let Some(artifacts) = manifest.get("artifacts").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "generated_artifact_manifest_artifacts_missing",
            "artifacts",
        ));
        return Vec::new();
    };
    if artifacts.is_empty() {
        findings.insert(Finding::new(
            "generated_artifact_manifest_artifacts_empty",
            "artifacts",
        ));
    }

    let mut declared = Vec::new();
    let mut id_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut path_counts: BTreeMap<String, usize> = BTreeMap::new();

    for (index, artifact) in artifacts.iter().enumerate() {
        let scope = format!("artifacts[{index}]");
        if !validate_object_fields(
            artifact,
            &ARTIFACT_FIELDS,
            "generated_artifact_manifest_artifact_not_object",
            "generated_artifact_manifest_artifact_unknown_field",
            &scope,
            findings,
        ) {
            continue;
        }

        let key = |field: &str| format!("artifacts[{index}].{field}");
        let Some(artifact_id) = required_str(artifact, "artifact_id") else {
            findings.insert(Finding::new(
                "generated_artifact_manifest_artifact_id_missing",
                key("artifact_id"),
            ));
            continue;
        };
        let Some(path) = required_str(artifact, "path") else {
            findings.insert(Finding::new(
                "generated_artifact_manifest_path_missing",
                artifact_id,
            ));
            continue;
        };
        let Some(artifact_class) = required_str(artifact, "artifact_class") else {
            findings.insert(Finding::new(
                "generated_artifact_manifest_artifact_class_missing",
                artifact_id,
            ));
            continue;
        };
        if !allowed(&ARTIFACT_CLASSES, artifact_class) {
            findings.insert(Finding::new(
                "generated_artifact_manifest_artifact_class_unknown",
                artifact_id,
            ));
        }
        let Some(materialization_mode) = required_str(artifact, "materialization_mode") else {
            findings.insert(Finding::new(
                "generated_artifact_manifest_materialization_mode_missing",
                artifact_id,
            ));
            continue;
        };
        if !allowed(&MATERIALIZATION_MODES, materialization_mode) {
            findings.insert(Finding::new(
                "generated_artifact_manifest_materialization_mode_unknown",
                artifact_id,
            ));
        }
        let Some(merge_policy) = required_str(artifact, "merge_policy") else {
            findings.insert(Finding::new(
                "generated_artifact_manifest_merge_policy_missing",
                artifact_id,
            ));
            continue;
        };
        if !allowed(&MERGE_POLICIES, merge_policy) {
            findings.insert(Finding::new(
                "generated_artifact_manifest_merge_policy_unknown",
                artifact_id,
            ));
        }
        if required_str(artifact, "owner_team").is_none() {
            findings.insert(Finding::new(
                "generated_artifact_manifest_owner_team_missing",
                artifact_id,
            ));
        }
        validate_generator(artifact, artifact_id, artifact_class, findings);
        required_string_array(
            artifact,
            "source_inputs",
            artifact_id,
            "generated_artifact_manifest_source_inputs_missing",
            "generated_artifact_manifest_source_inputs_item_not_string",
            findings,
        );
        if required_str(artifact, "final_tree_validation").is_none() {
            findings.insert(Finding::new(
                "generated_artifact_manifest_final_tree_validation_missing",
                artifact_id,
            ));
        }
        if required_str(artifact, "public_product_contract").is_none() {
            findings.insert(Finding::new(
                "generated_artifact_manifest_artifact_public_product_contract_missing",
                artifact_id,
            ));
        }

        *id_counts.entry(artifact_id.to_owned()).or_default() += 1;
        *path_counts.entry(path.to_owned()).or_default() += 1;
        declared.push(DeclaredArtifact {
            artifact_id: artifact_id.to_owned(),
            path: path.to_owned(),
            artifact_class: artifact_class.to_owned(),
            materialization_mode: materialization_mode.to_owned(),
            merge_policy: merge_policy.to_owned(),
        });
    }

    for (artifact_id, count) in id_counts {
        if count > 1 {
            findings.insert(Finding::new(
                "generated_artifact_manifest_duplicate_artifact_id",
                artifact_id,
            ));
        }
    }
    for (path, count) in path_counts {
        if count > 1 {
            findings.insert(Finding::new(
                "generated_artifact_manifest_duplicate_path",
                path,
            ));
        }
    }

    declared
}

/// Extract the firewall FROZEN-REFERENCE face paths from one or more parsed `ratchet-policy.json`
/// values. The frozen-reference set is the authoritative, repo-agnostic signal for "this path is
/// materialized from the merge-base git blob": the firewall ratchet (`oya-cloud-ci-firewall-app`)
/// and the scm-facts emitter resolve the baseline via `git show <merge_base>:<frozen_reference.
/// face_path>` (ADR-0551). The function reads `frozen_reference.face_path`, tolerating either a
/// single `frozen_reference` object or an array of them so adopters can declare multiple frozen
/// baselines. It carries NO hardcoded oyatie path — the set is pure data from the policy files.
pub fn frozen_reference_face_paths<'a, I>(ratchet_policies: I) -> BTreeSet<String>
where
    I: IntoIterator<Item = &'a Value>,
{
    let mut paths = BTreeSet::new();
    for policy in ratchet_policies {
        let Some(frozen) = policy.get("frozen_reference") else {
            continue;
        };
        match frozen {
            Value::Array(entries) => {
                for entry in entries {
                    if let Some(face_path) = required_str(entry, "face_path") {
                        paths.insert(face_path.to_owned());
                    }
                }
            }
            _ => {
                if let Some(face_path) = required_str(frozen, "face_path") {
                    paths.insert(face_path.to_owned());
                }
            }
        }
    }
    paths
}

/// Make-it-impossible guard for the #828 dev-wide deadlock class. A firewall FROZEN-REFERENCE /
/// baseline artifact is materialized from the committed git blob at the merge-base (`git show
/// <merge_base>:<face_path>`, ADR-0551). De-committing it (declaring it with a derive-on-demand /
/// de-commit `materialization_mode`) empties the ratchet baseline at the merge-base, so every
/// pre-existing repo-wide debt item reads as a NEW regression on every broad-affected-set PR — the
/// #828 dev regression, hotfixed by #830. This rule fires `frozen_reference_artifact_must_stay_
/// committed` when a declared artifact whose `path` is a frozen reference is declared with a
/// de-commit mode. The frozen-reference set is supplied as DATA by the caller (from
/// `frozen_reference_face_paths` over the repo's `ratchet-policy.json` files), so the predicate has
/// no hardcoded paths and works on any repo with its own ratchet policy + control-plane manifest.
pub fn frozen_reference_decommit_findings(
    manifest: &Value,
    frozen_reference_paths: &BTreeSet<String>,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    if frozen_reference_paths.is_empty() {
        return findings;
    }
    let declared = parse_declared_artifacts(manifest, &mut BTreeSet::new());
    for artifact in &declared {
        if frozen_reference_paths.contains(&artifact.path)
            && is_decommit_materialization_mode(&artifact.materialization_mode)
        {
            findings.insert(Finding::new(
                "frozen_reference_artifact_must_stay_committed",
                &artifact.artifact_id,
            ));
        }
    }
    findings
}

/// Productized predicate: every tracked generated output matched by the repo-declared
/// `generated_path_rules` must be declared, and every declared generated artifact must carry
/// enough policy for a controller/merge queue to know who owns it, how it materializes, and why
/// broad merge drivers are not authority.
///
/// This is the frozen-reference-unaware entry point retained for callers without a ratchet policy
/// (e.g. the diff-policy bridge). Pass the ratchet policies to
/// [`evaluate_keyed_with_frozen_references`] to additionally enforce the #828 make-it-impossible
/// guard.
pub fn evaluate_keyed(manifest: &Value, scm_facts: &Value) -> BTreeSet<Finding> {
    evaluate_keyed_with_frozen_references(manifest, scm_facts, &BTreeSet::new())
}

/// Frozen-reference-aware evaluation. Runs the full control-plane predicate AND the #828
/// make-it-impossible guard ([`frozen_reference_decommit_findings`]) over the supplied
/// frozen-reference path set (derived from the repo's `ratchet-policy.json` data via
/// [`frozen_reference_face_paths`]).
pub fn evaluate_keyed_with_frozen_references(
    manifest: &Value,
    scm_facts: &Value,
    frozen_reference_paths: &BTreeSet<String>,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let generated_path_rules = parse_generated_path_rules(manifest, &mut findings);
    let declared = parse_declared_artifacts(manifest, &mut findings);
    let declared_paths: BTreeSet<String> = declared
        .iter()
        .map(|artifact| artifact.path.clone())
        .collect();
    // De-commit class (ADR-0595): declared paths intentionally NOT tracked in git. These are
    // pure derivations materialized out-of-graph; they are EXEMPT from the
    // `declared_path_not_tracked` predicate and instead FORBIDDEN from being tracked.
    let not_tracked_paths: BTreeSet<String> = declared
        .iter()
        .filter(|artifact| artifact.materialization_mode == NOT_TRACKED_IN_GIT_MODE)
        .map(|artifact| artifact.path.clone())
        .collect();
    let tracked_generated = tracked_generated_artifact_paths(scm_facts, &generated_path_rules);

    for path in tracked_generated.difference(&declared_paths) {
        findings.insert(Finding::new(
            "generated_artifact_tracked_generated_output_not_declared",
            path,
        ));
    }
    for path in declared_paths.difference(&tracked_generated) {
        // A de-commit-class path is SUPPOSED to be absent from the tracked tree, so its absence
        // is the desired state, not a finding. All other declared generated paths must be tracked.
        if not_tracked_paths.contains(path) {
            continue;
        }
        findings.insert(Finding::new(
            "generated_artifact_declared_path_not_tracked",
            path,
        ));
    }
    // One-way door (ADR-0595): once a generated artifact is declared de-commit class it must
    // never be re-tracked in git. Re-adding it to the tree is a hard finding so the de-commit
    // cannot be silently reverted by a future PR.
    for path in not_tracked_paths.intersection(&tracked_generated) {
        findings.insert(Finding::new(
            "generated_artifact_not_tracked_path_is_tracked",
            path,
        ));
    }

    for artifact in &declared {
        if is_tracked_generated_artifact_path(&artifact.path, &generated_path_rules)
            && artifact.merge_policy == "normal-source-merge"
        {
            findings.insert(Finding::new(
                "generated_artifact_generated_output_uses_normal_source_merge",
                &artifact.artifact_id,
            ));
        }
        if artifact.artifact_class == "main-materialized-aggregate"
            && artifact.materialization_mode == "source-authored"
        {
            findings.insert(Finding::new(
                "generated_artifact_main_aggregate_marked_source_authored",
                &artifact.artifact_id,
            ));
        }
        // The SCM-facts boundary snapshot must be controller-materialized, never a hand-merged
        // contributor surface. Two controller-materialized shapes are valid (ADR-0604):
        //   - `main-branch-materialized` (legacy committed shape): the controller writes the face
        //     on the protected branch and it is tracked in git.
        //   - `not-tracked-in-git` (de-commit class, ADR-0604): the controller materializes it on
        //     demand and it is NEVER tracked — killing the faces-serialization cascade, because a
        //     committed snapshot lists itself in tracked_paths and so mutates on every PR.
        // Both are controller-owned and non-source-authored; only these two pass. Any other mode
        // (source-authored, branch-committed, merge-candidate-regenerated, ci-artifact-only) would
        // let the boundary snapshot become a contributor merge surface and is RED.
        if artifact.artifact_class == "scm-facts-boundary-snapshot"
            && artifact.materialization_mode != "main-branch-materialized"
            && artifact.materialization_mode != NOT_TRACKED_IN_GIT_MODE
        {
            findings.insert(Finding::new(
                "generated_artifact_scm_facts_not_final_tree_materialized",
                &artifact.artifact_id,
            ));
        }
        // Merge policy must keep the snapshot a controller-owned, regenerate-not-hand-merge surface.
        // `main-branch-materialized` pairs with `controller-owned-main-materialization`; the
        // de-commit class pairs with `never-manual-merge-regenerate-from-source-tree` (the same
        // policy every ADR-0595 de-committed face carries — derive-on-demand, never hand-merged).
        let scm_facts_merge_policy_ok = match artifact.materialization_mode.as_str() {
            NOT_TRACKED_IN_GIT_MODE => {
                artifact.merge_policy == "never-manual-merge-regenerate-from-source-tree"
            }
            _ => artifact.merge_policy == "controller-owned-main-materialization",
        };
        if artifact.artifact_class == "scm-facts-boundary-snapshot" && !scm_facts_merge_policy_ok {
            findings.insert(Finding::new(
                "generated_artifact_scm_facts_not_controller_owned",
                &artifact.artifact_id,
            ));
        }
    }

    // #828 make-it-impossible guard: a frozen-reference/baseline artifact materialized from the
    // merge-base git blob must NEVER be declared with a de-commit materialization mode.
    findings.extend(frozen_reference_decommit_findings(
        manifest,
        frozen_reference_paths,
    ));

    findings
}

/// Frozen-reference-unaware verdict, retained for callers without a ratchet policy.
pub fn evaluate(manifest: &Value, scm_facts: &Value) -> Report {
    let findings = evaluate_keyed(manifest, scm_facts);
    Report::from_findings(&findings)
}

/// Frozen-reference-aware verdict. Folds the #828 make-it-impossible guard into the report so the
/// gate fails RED when a firewall frozen reference is declared de-commit class.
pub fn evaluate_with_frozen_references(
    manifest: &Value,
    scm_facts: &Value,
    frozen_reference_paths: &BTreeSet<String>,
) -> Report {
    let findings =
        evaluate_keyed_with_frozen_references(manifest, scm_facts, frozen_reference_paths);
    Report::from_findings(&findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn scm(paths: &[&str]) -> Value {
        json!({
            "schema": "oya-ci/scm-facts/v1",
            "tracked_paths": paths,
        })
    }

    fn artifact(id: &str, path: &str) -> Value {
        json!({
            "artifact_id": id,
            "path": path,
            "artifact_class": "main-materialized-aggregate",
            "materialization_mode": "branch-committed-regenerated-until-controller-materialization",
            "merge_policy": "never-manual-merge-regenerate-from-source-tree",
            "owner_team": "cloud-ci-platform",
            "generator": {
                "runner": "buck2",
                "generator_target": "//cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app:oya-cloud-ci-accounting-registry-app-bin",
                "operation_id": "emit-accounting-face",
                "parameters": {"face": "registry"},
                "input_contract": ["repo-root", "declared-source-inputs", "scm-facts-snapshot"],
                "output_mode": "stdout-json"
            },
            "source_inputs": ["src/**"],
            "final_tree_validation": "regenerate from final candidate tree and compare bytes",
            "public_product_contract": "portable hermetic Rust cloud-ci generated artifact policy"
        })
    }

    fn manifest(artifacts: Vec<Value>) -> Value {
        json!({
            "schema_version": 1,
            "canonical_name": "generated-artifact-control-plane",
            "ci_product_surface": "oya-ci",
            "public_product_contract": "A public, hermetic Rust CI product can adopt this manifest shape in any repo.",
            "final_tree_materialization": {
                "controller_id": "oya-ci-generated-artifact-controller",
                "presubmit_authority": "merge-queue-projected-state",
                "postsubmit_authority": "postsubmit-main-materialization",
                "protected_branch_trigger": "push:dev",
                "manual_conflict_resolution_policy": GENERATED_OUTPUT_CONFLICT_POLICY,
                "drift_repair_policy": "fail-closed-and-open-generated-only-repair-pr",
                "portable_runner_contract": "regenerate declared generated artifacts from projected and protected final trees"
            },
            "generated_path_rules": [
                {
                    "rule_id": "generated-json-files",
                    "rule_kind": "path_suffix",
                    "pattern": ".generated.json",
                    "description": "canonical oya-ci generated JSON face suffix"
                },
                {
                    "rule_id": "generated-directory-component",
                    "rule_kind": "path_component",
                    "pattern": "generated",
                    "description": "default generated output directory component",
                    "exclude_file_names": [".gitkeep"]
                },
                {
                    "rule_id": "double-underscore-generated-component",
                    "rule_kind": "path_component",
                    "pattern": "__generated__",
                    "description": "common generated-client output directory (e.g. TypeScript clients)"
                },
                {
                    "rule_id": "generated-sources-component",
                    "rule_kind": "path_component",
                    "pattern": "generated-sources",
                    "description": "common JVM generated output directory"
                },
                {
                    "rule_id": "protobuf-generated-file-suffix",
                    "rule_kind": "path_suffix",
                    "pattern": ".pb.go",
                    "description": "common Go protobuf generated output suffix"
                },
                {
                    "rule_id": "typescript-generated-file-suffix",
                    "rule_kind": "path_suffix",
                    "pattern": ".generated.ts",
                    "description": "common TypeScript generated output suffix"
                }
            ],
            "artifacts": artifacts,
            "development_lifecycle_enforcement_layers": LIFECYCLE_LAYER_IDS
                .iter()
                .map(|layer_id| json!({
                    "layer_id": layer_id,
                    "name": layer_id.replace('-', " "),
                    "automation": "automate generated artifact policy",
                    "enforcement": "enforce generated artifact policy"
                }))
                .collect::<Vec<_>>(),
        })
    }

    #[test]
    fn declared_generated_artifacts_are_green() {
        let manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        let scm_facts = scm(&["out/example.generated.json"]);
        assert_eq!(evaluate(&manifest, &scm_facts).verdict, Verdict::Green);
    }

    #[test]
    fn undeclared_tracked_generated_json_is_red() {
        let manifest = manifest(Vec::new());
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_tracked_generated_output_not_declared"
                && finding.key == "out/example.generated.json"
        }));
    }

    #[test]
    fn undeclared_tracked_generated_directory_output_is_red() {
        let manifest = manifest(Vec::new());
        let scm_facts = scm(&["app/generated/client.d.ts", "app/generated/.gitkeep"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_tracked_generated_output_not_declared"
                && finding.key == "app/generated/client.d.ts"
        }));
        assert!(!findings.iter().any(|finding| {
            finding.code == "generated_artifact_tracked_generated_output_not_declared"
                && finding.key == "app/generated/.gitkeep"
        }));
    }

    #[test]
    fn public_generated_output_conventions_are_manifest_rule_driven() {
        let manifest = manifest(Vec::new());
        let scm_facts = scm(&[
            "src/__generated__/types.ts",
            "proto/foo.pb.go",
            "openapi/client.generated.ts",
            "target/generated-sources/foo.java",
        ]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        for path in [
            "src/__generated__/types.ts",
            "proto/foo.pb.go",
            "openapi/client.generated.ts",
            "target/generated-sources/foo.java",
        ] {
            assert!(
                findings.iter().any(|finding| {
                    finding.code == "generated_artifact_tracked_generated_output_not_declared"
                        && finding.key == path
                }),
                "{path} should be classified as generated output"
            );
        }
    }

    #[test]
    fn diff_policy_is_manifest_derived_and_allows_deletions() {
        let manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        let diff = concat!(
            "M\tsrc/__generated__/types.ts\n",
            "A\topenapi/client.generated.ts\n",
            "D\toya/app-shell-frontend/generated/hr-api.d.ts\n",
            "M\tapp/generated/.gitkeep\n",
            "M\tsrc/source.rs\n",
        );
        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);
        assert_eq!(findings, BTreeSet::new());
        assert_eq!(
            violations,
            vec![
                DiffPolicyViolation {
                    status: "M".to_owned(),
                    path: "src/__generated__/types.ts".to_owned(),
                },
                DiffPolicyViolation {
                    status: "A".to_owned(),
                    path: "openapi/client.generated.ts".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn diff_policy_rejects_declared_generated_artifact_edits() {
        let mut face = artifact(
            "cloud-ci-accounting-registry-face",
            "out/example.generated.json",
        );
        face["materialization_mode"] = json!("merge-candidate-regenerated");
        let mut scm_facts = artifact("cloud-ci-scm-facts", "out/scm-facts.generated.json");
        scm_facts["artifact_class"] = json!("scm-facts-boundary-snapshot");
        scm_facts["materialization_mode"] = json!("main-branch-materialized");
        scm_facts["merge_policy"] = json!("controller-owned-main-materialization");
        scm_facts["generator"]["operation_id"] = json!("emit-scm-facts-boundary-snapshot");
        scm_facts["generator"]["output_mode"] = json!("declared-artifact-path-write");
        scm_facts["generator"]["input_contract"] = json!(["repo-root", "full-depth-scm"]);
        let manifest = manifest(vec![face, scm_facts]);
        let diff = concat!(
            "M\tout/example.generated.json\n",
            "M\tout/scm-facts.generated.json\n",
            "M\tout/untracked.generated.json\n",
        );

        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);

        assert_eq!(findings, BTreeSet::new());
        assert_eq!(
            violations,
            vec![
                DiffPolicyViolation {
                    status: "M".to_owned(),
                    path: "out/example.generated.json".to_owned(),
                },
                DiffPolicyViolation {
                    status: "M".to_owned(),
                    path: "out/scm-facts.generated.json".to_owned(),
                },
                DiffPolicyViolation {
                    status: "M".to_owned(),
                    path: "out/untracked.generated.json".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn product_graph_dashboard_rule_is_gate_accepted_and_pr_edit_rejected() {
        let mut product_graph = artifact(
            "architecture-product-graph-dashboard",
            "docs/architecture/product-graph.html",
        );
        product_graph["materialization_mode"] = json!("main-branch-materialized");
        product_graph["merge_policy"] = json!("controller-owned-main-materialization");
        product_graph["generator"]["generator_target"] =
            json!("//tools/oya-architecture-graph-generator-app:oya-architecture-graph-generator");
        product_graph["generator"]["operation_id"] =
            json!("emit-architecture-product-graph-dashboard");
        product_graph["generator"]["parameters"] = json!({"mode": "write"});
        product_graph["generator"]["input_contract"] =
            json!(["repo-root", "declared-source-inputs"]);
        product_graph["generator"]["output_mode"] = json!("declared-artifact-path-write");

        let mut manifest = manifest(vec![product_graph]);
        manifest["generated_path_rules"]
            .as_array_mut()
            .expect("generated path rules")
            .push(json!({
                "rule_id": "architecture-product-graph-dashboard",
                "rule_kind": "path_suffix",
                "pattern": "docs/architecture/product-graph.html",
                "description": "controller-owned generated architecture dashboard"
            }));
        let scm_facts = scm(&["docs/architecture/product-graph.html"]);

        assert_eq!(evaluate(&manifest, &scm_facts).verdict, Verdict::Green);

        let (findings, violations) = generated_output_diff_policy_violations(
            &manifest,
            "M\tdocs/architecture/product-graph.html\n",
        );

        assert_eq!(findings, BTreeSet::new());
        assert_eq!(
            violations,
            vec![DiffPolicyViolation {
                status: "M".to_owned(),
                path: "docs/architecture/product-graph.html".to_owned(),
            }]
        );
    }

    #[test]
    fn invalid_generated_path_rules_are_red() {
        let mut manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        manifest["generated_path_rules"][0]["rule_kind"] = json!("regex");
        manifest["generated_path_rules"][0]["pattern"] = json!("");
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_generated_path_rule_kind_unknown"
                && finding.key == "generated-json-files.regex"
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_generated_path_rule_pattern_missing"
                && finding.key == "generated-json-files"
        }));
    }

    #[test]
    fn declared_missing_path_is_red() {
        let manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        let scm_facts = scm(&[]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_declared_path_not_tracked"
                && finding.key == "out/example.generated.json"
        }));
    }

    #[test]
    fn not_tracked_in_git_declared_path_is_exempt_from_not_tracked_finding() {
        // ADR-0595: a de-commit-class declared path is SUPPOSED to be absent from the tracked
        // tree, so its absence must NOT fire declared_path_not_tracked.
        let mut face = artifact("decommitted-face", "out/example.generated.json");
        face["materialization_mode"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![face]);
        let scm_facts = scm(&[]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(
            !findings.iter().any(|finding| {
                finding.code == "generated_artifact_declared_path_not_tracked"
                    && finding.key == "out/example.generated.json"
            }),
            "de-commit-class path must be exempt; findings: {findings:#?}"
        );
        assert_eq!(evaluate(&manifest, &scm_facts).verdict, Verdict::Green);
    }

    #[test]
    fn not_tracked_in_git_path_that_is_still_tracked_is_red_one_way_door() {
        // ADR-0595 one-way door: once a face is de-commit class it must never be re-tracked.
        let mut face = artifact("decommitted-face", "out/example.generated.json");
        face["materialization_mode"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![face]);
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_not_tracked_path_is_tracked"
                    && finding.key == "out/example.generated.json"
            }),
            "re-tracking a de-commit-class face must RED; findings: {findings:#?}"
        );
    }

    #[test]
    fn generated_json_cannot_use_normal_source_merge() {
        let mut artifact = artifact("example-face", "out/example.generated.json");
        artifact["merge_policy"] = json!("normal-source-merge");
        let manifest = manifest(vec![artifact]);
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_generated_output_uses_normal_source_merge"
                && finding.key == "example-face"
        }));
    }

    #[test]
    fn generated_directory_output_cannot_use_normal_source_merge() {
        let mut artifact = artifact("client-types", "app/generated/client.d.ts");
        artifact["merge_policy"] = json!("normal-source-merge");
        let manifest = manifest(vec![artifact]);
        let scm_facts = scm(&["app/generated/client.d.ts"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_generated_output_uses_normal_source_merge"
                && finding.key == "client-types"
        }));
    }

    #[test]
    fn duplicate_paths_are_red() {
        let manifest = manifest(vec![
            artifact("one", "out/example.generated.json"),
            artifact("two", "out/example.generated.json"),
        ]);
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_duplicate_path"
                && finding.key == "out/example.generated.json"
        }));
    }

    #[test]
    fn missing_public_product_contract_is_red() {
        let mut manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        manifest
            .as_object_mut()
            .unwrap()
            .remove("public_product_contract");
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_public_product_contract_missing"
        }));
    }

    #[test]
    fn missing_final_tree_materialization_is_red() {
        let mut manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        manifest
            .as_object_mut()
            .unwrap()
            .remove("final_tree_materialization");
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_final_tree_materialization_missing"
        }));
    }

    #[test]
    fn diff_policy_fails_closed_on_invalid_control_plane_manifest() {
        let mut manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        manifest
            .as_object_mut()
            .unwrap()
            .remove("final_tree_materialization");

        let (findings, violations) = generated_output_diff_policy_violations(&manifest, "");

        assert!(violations.is_empty());
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_final_tree_materialization_missing"
        }));
    }

    #[test]
    fn unsafe_manual_generated_conflict_policy_is_red() {
        let mut manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        manifest["final_tree_materialization"]["manual_conflict_resolution_policy"] =
            json!("merge-driver-takes-theirs");
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_manual_generated_conflict_policy_invalid"
        }));
    }

    #[test]
    fn scm_facts_boundary_requires_final_tree_controller_materialization() {
        // The default artifact() helper uses branch-committed mode + never-manual-merge policy:
        // neither of the two controller-materialized shapes (main-branch-materialized /
        // not-tracked-in-git with controller-owned / regenerate policy), so BOTH rules fire.
        let mut scm_artifact = artifact("scm-facts", "out/scm-facts.generated.json");
        scm_artifact["artifact_class"] = json!("scm-facts-boundary-snapshot");
        let manifest = manifest(vec![scm_artifact]);
        let scm_facts = scm(&["out/scm-facts.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_scm_facts_not_final_tree_materialized"
                && finding.key == "scm-facts"
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_scm_facts_not_controller_owned"
                && finding.key == "scm-facts"
        }));
    }

    /// Build a controller-materialized scm-facts boundary-snapshot artifact in the legacy committed
    /// shape (main-branch-materialized + controller-owned-main-materialization), with the boundary
    /// generator contract (buck2 + declared-artifact-path-write + full-depth-scm) satisfied.
    fn scm_facts_boundary(path: &str) -> Value {
        let mut a = artifact("cloud-ci-scm-facts-boundary-snapshot", path);
        a["artifact_class"] = json!("scm-facts-boundary-snapshot");
        a["materialization_mode"] = json!("main-branch-materialized");
        a["merge_policy"] = json!("controller-owned-main-materialization");
        a["generator"]["operation_id"] = json!("emit-scm-facts-boundary-snapshot");
        a["generator"]["output_mode"] = json!("declared-artifact-path-write");
        a["generator"]["input_contract"] = json!(["repo-root", "full-depth-scm"]);
        a
    }

    #[test]
    fn scm_facts_boundary_main_branch_materialized_is_green() {
        // GREEN baseline: the legacy committed controller shape stays valid (no regression).
        let scm_artifact = scm_facts_boundary("out/scm-facts.generated.json");
        let manifest = manifest(vec![scm_artifact]);
        let scm_facts = scm(&["out/scm-facts.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(
            !findings.iter().any(|f| {
                f.code == "generated_artifact_scm_facts_not_final_tree_materialized"
                    || f.code == "generated_artifact_scm_facts_not_controller_owned"
            }),
            "main-branch-materialized boundary snapshot must be green; findings: {findings:#?}"
        );
    }

    #[test]
    fn scm_facts_boundary_not_tracked_in_git_is_green() {
        // ADR-0604 keystone GREEN: the de-commit class is a valid controller-materialized shape for
        // the SCM-facts boundary snapshot. A not-tracked snapshot paired with the regenerate-from-
        // source-tree merge policy must NOT fire either boundary rule, and (being absent from the
        // tracked tree) is exempt from declared_path_not_tracked.
        let mut scm_artifact = scm_facts_boundary("out/scm-facts.generated.json");
        scm_artifact["materialization_mode"] = json!("not-tracked-in-git");
        scm_artifact["merge_policy"] = json!("never-manual-merge-regenerate-from-source-tree");
        let manifest = manifest(vec![scm_artifact]);
        // De-commit class: the snapshot is intentionally NOT in tracked_paths.
        let scm_facts = scm(&[]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(
            !findings.iter().any(|f| {
                f.code == "generated_artifact_scm_facts_not_final_tree_materialized"
                    || f.code == "generated_artifact_scm_facts_not_controller_owned"
                    || f.code == "generated_artifact_declared_path_not_tracked"
            }),
            "not-tracked-in-git boundary snapshot must be green; findings: {findings:#?}"
        );
        assert_eq!(evaluate(&manifest, &scm_facts).verdict, Verdict::Green);
    }

    #[test]
    fn scm_facts_boundary_not_tracked_with_wrong_merge_policy_is_red() {
        // ADR-0604 RED guard: the de-commit class must still carry the regenerate-from-source-tree
        // merge policy. A not-tracked snapshot with controller-owned-main-materialization (the
        // committed-shape policy) is a mismatch and must RED — the two valid shapes do not
        // cross-pollinate their merge policies.
        let mut scm_artifact = scm_facts_boundary("out/scm-facts.generated.json");
        scm_artifact["materialization_mode"] = json!("not-tracked-in-git");
        // merge_policy left as controller-owned-main-materialization (wrong for the de-commit class).
        let manifest = manifest(vec![scm_artifact]);
        let scm_facts = scm(&[]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(
            findings.iter().any(|f| {
                f.code == "generated_artifact_scm_facts_not_controller_owned"
                    && f.key == "cloud-ci-scm-facts-boundary-snapshot"
            }),
            "not-tracked boundary snapshot with wrong merge policy must RED; findings: {findings:#?}"
        );
    }

    #[test]
    fn scm_facts_boundary_committed_with_wrong_merge_policy_is_red() {
        // Symmetric RED guard: the committed shape must keep controller-owned-main-materialization.
        let mut scm_artifact = scm_facts_boundary("out/scm-facts.generated.json");
        scm_artifact["merge_policy"] = json!("never-manual-merge-regenerate-from-source-tree");
        let manifest = manifest(vec![scm_artifact]);
        let scm_facts = scm(&["out/scm-facts.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(
            findings.iter().any(|f| {
                f.code == "generated_artifact_scm_facts_not_controller_owned"
                    && f.key == "cloud-ci-scm-facts-boundary-snapshot"
            }),
            "main-branch-materialized boundary snapshot with wrong merge policy must RED; findings: {findings:#?}"
        );
    }

    #[test]
    fn missing_lifecycle_layers_is_red() {
        let mut manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        manifest
            .as_object_mut()
            .unwrap()
            .remove("development_lifecycle_enforcement_layers");
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_lifecycle_layers_missing"
        }));
    }

    #[test]
    fn source_inputs_must_be_non_empty_strings() {
        let mut artifact = artifact("example-face", "out/example.generated.json");
        artifact["source_inputs"] = json!(["src/**", 7, " "]);
        let manifest = manifest(vec![artifact]);
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_source_inputs_item_not_string"
                && finding.key == "example-face.source_inputs[1]"
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_source_inputs_item_not_string"
                && finding.key == "example-face.source_inputs[2]"
        }));
    }

    #[test]
    fn shell_shaped_generator_command_is_rejected() {
        let mut artifact = artifact("example-face", "out/example.generated.json");
        artifact["generator_command"] =
            json!("cargo run --quiet -p example && curl https://example.invalid");
        artifact.as_object_mut().unwrap().remove("generator");
        let manifest = manifest(vec![artifact]);
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_artifact_unknown_field"
                && finding.key == "artifacts[0].generator_command"
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_generator_missing"
                && finding.key == "example-face"
        }));
    }

    #[test]
    fn generator_requires_canonical_target_and_output_contract() {
        let mut artifact = artifact("example-face", "out/example.generated.json");
        artifact["generator"]["generator_target"] = json!("cargo run -p example");
        artifact["generator"]["output_mode"] = json!("shell-script");
        artifact["generator"]["input_contract"] = json!(["repo-root", "host-shell"]);
        let manifest = manifest(vec![artifact]);
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_generator_buck2_target_not_canonical"
                && finding.key == "example-face"
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_generator_output_mode_unknown"
                && finding.key == "example-face.shell-script"
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_generator_input_contract_unknown"
                && finding.key == "example-face.host-shell"
        }));
    }

    #[test]
    fn unknown_manifest_fields_are_red() {
        let mut manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        manifest["extra"] = json!(true);
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_unknown_field"
                && finding.key == "manifest.extra"
        }));
    }

    #[test]
    fn unknown_artifact_fields_are_red() {
        let mut artifact = artifact("example-face", "out/example.generated.json");
        artifact["extra"] = json!(true);
        let manifest = manifest(vec![artifact]);
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_artifact_unknown_field"
                && finding.key == "artifacts[0].extra"
        }));
    }

    fn ratchet_policy(face_path: &str) -> Value {
        json!({
            "base_ref": "origin/dev",
            "frozen_reference": {
                "face_path": face_path,
                "out_path": "out/frozen.merge-base.generated.json"
            }
        })
    }

    fn frozen_set(face_paths: &[&str]) -> BTreeSet<String> {
        let policies: Vec<Value> = face_paths.iter().map(|p| ratchet_policy(p)).collect();
        frozen_reference_face_paths(&policies)
    }

    #[test]
    fn frozen_reference_face_paths_extracts_single_object() {
        let policy = ratchet_policy("out/gate-baseline.generated.json");
        let paths = frozen_reference_face_paths(std::iter::once(&policy));
        assert!(paths.contains("out/gate-baseline.generated.json"));
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn frozen_reference_face_paths_extracts_array_of_frozen_references() {
        // Adopters may declare multiple frozen baselines; the set is pure data, not hardcoded.
        let policy = json!({
            "base_ref": "origin/main",
            "frozen_reference": [
                { "face_path": "a/baseline.generated.json", "out_path": "o/a.json" },
                { "face_path": "b/baseline.generated.json", "out_path": "o/b.json" }
            ]
        });
        let paths = frozen_reference_face_paths(std::iter::once(&policy));
        assert!(paths.contains("a/baseline.generated.json"));
        assert!(paths.contains("b/baseline.generated.json"));
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn frozen_reference_decommitted_is_red_the_828_class() {
        // RED fixture (the #828 incident shape): the frozen-reference baseline face is declared
        // de-commit class (not-tracked-in-git). De-committing it empties the merge-base ratchet
        // baseline and deadlocks the merge queue. This MUST fail.
        let mut baseline = artifact(
            "cloud-ci-gate-baseline-ratchet-face",
            "out/gate-baseline.generated.json",
        );
        baseline["materialization_mode"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![baseline]);
        let scm_facts = scm(&[]);
        let frozen = frozen_set(&["out/gate-baseline.generated.json"]);

        let findings = evaluate_keyed_with_frozen_references(&manifest, &scm_facts, &frozen);
        assert!(
            findings.iter().any(|finding| {
                finding.code == "frozen_reference_artifact_must_stay_committed"
                    && finding.key == "cloud-ci-gate-baseline-ratchet-face"
            }),
            "de-committing a frozen reference must RED; findings: {findings:#?}"
        );
        assert_eq!(
            evaluate_with_frozen_references(&manifest, &scm_facts, &frozen).verdict,
            Verdict::Red
        );
    }

    #[test]
    fn frozen_reference_committed_with_decommitted_pure_views_is_green() {
        // GREEN fixture (current dev state): the frozen-reference baseline stays committed
        // (a non-de-commit materialization mode), while a sibling pure-view face is de-committed.
        // The de-commit class is legitimate for pure-view faces and must NOT fire on them.
        let mut baseline = artifact(
            "cloud-ci-gate-baseline-ratchet-face",
            "out/gate-baseline.generated.json",
        );
        baseline["materialization_mode"] = json!("main-branch-materialized");
        baseline["merge_policy"] = json!("controller-owned-main-materialization");
        let mut pure_view = artifact("pure-view-face", "out/accounting-registry.generated.json");
        pure_view["materialization_mode"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![baseline, pure_view]);
        // Frozen baseline tracked (committed); the de-committed pure view is absent from scm.
        let scm_facts = scm(&["out/gate-baseline.generated.json"]);
        let frozen = frozen_set(&["out/gate-baseline.generated.json"]);

        let findings = evaluate_keyed_with_frozen_references(&manifest, &scm_facts, &frozen);
        assert!(
            !findings
                .iter()
                .any(|finding| finding.code == "frozen_reference_artifact_must_stay_committed"),
            "committed frozen reference + de-committed pure view must not RED; findings: {findings:#?}"
        );
        assert_eq!(
            evaluate_with_frozen_references(&manifest, &scm_facts, &frozen).verdict,
            Verdict::Green
        );
    }

    #[test]
    fn decommitted_non_frozen_pure_view_is_not_a_frozen_reference_violation() {
        // A de-committed face that is NOT a frozen reference is legitimate and must not fire the
        // frozen-reference guard — the guard is scoped strictly to the frozen-reference set.
        let mut pure_view = artifact("pure-view-face", "out/ttl-policy.generated.json");
        pure_view["materialization_mode"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![pure_view]);
        let scm_facts = scm(&[]);
        let frozen = frozen_set(&["out/gate-baseline.generated.json"]);

        let findings = evaluate_keyed_with_frozen_references(&manifest, &scm_facts, &frozen);
        assert!(
            !findings
                .iter()
                .any(|finding| finding.code == "frozen_reference_artifact_must_stay_committed"),
            "a de-committed non-frozen pure view must not fire the frozen-reference guard; findings: {findings:#?}"
        );
    }

    #[test]
    fn frozen_reference_guard_is_inert_without_a_ratchet_policy() {
        // Backward compatibility: the legacy frozen-reference-unaware entry point (empty frozen
        // set) never fires the guard, so existing callers (e.g. the diff-policy bridge) are unchanged.
        let mut baseline = artifact(
            "cloud-ci-gate-baseline-ratchet-face",
            "out/gate-baseline.generated.json",
        );
        baseline["materialization_mode"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![baseline]);
        let scm_facts = scm(&[]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(
            !findings
                .iter()
                .any(|finding| finding.code == "frozen_reference_artifact_must_stay_committed"),
            "without a ratchet policy the guard must be inert; findings: {findings:#?}"
        );
    }

    #[test]
    fn malformed_lifecycle_rows_are_red() {
        let mut manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        let layers = manifest
            .get_mut("development_lifecycle_enforcement_layers")
            .and_then(Value::as_array_mut)
            .expect("lifecycle layers");
        layers[0]["extra"] = json!(true);
        layers[0].as_object_mut().unwrap().remove("automation");
        let scm_facts = scm(&["out/example.generated.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_lifecycle_layer_unknown_field"
                && finding.key == "development_lifecycle_enforcement_layers[0].extra"
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_manifest_lifecycle_layer_field_missing"
                && finding.key == "project-creation.automation"
        }));
    }
}
