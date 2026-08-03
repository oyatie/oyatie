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

const MATERIALIZATION_MODES: [&str; 5] = [
    "source-authored",
    "branch-committed-regenerated-until-controller-materialization",
    "merge-candidate-regenerated",
    "main-branch-materialized",
    "ci-artifact-only",
];

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

impl DeclaredArtifact {
    fn is_untracked_ci_artifact(&self) -> bool {
        self.materialization_mode == "ci-artifact-only" && self.merge_policy == "not-tracked-in-git"
    }

    fn requires_tracked_path(&self) -> bool {
        !self.is_untracked_ci_artifact()
    }
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
    manifest: &Value,
    _findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    let mut ignored_artifact_findings = BTreeSet::new();
    parse_declared_artifacts(manifest, &mut ignored_artifact_findings)
        .into_iter()
        .filter(|artifact| {
            matches!(
                (
                    artifact.materialization_mode.as_str(),
                    artifact.merge_policy.as_str()
                ),
                (
                    "merge-candidate-regenerated",
                    "never-manual-merge-regenerate-from-source-tree"
                ) | (
                    "main-branch-materialized",
                    "controller-owned-main-materialization"
                )
            )
        })
        .map(|artifact| artifact.path)
        .collect()
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

/// Productized predicate: every tracked generated output matched by the repo-declared
/// `generated_path_rules` must be declared, and every declared generated artifact must carry
/// enough policy for a controller/merge queue to know who owns it, how it materializes, and why
/// broad merge drivers are not authority.
pub fn evaluate_keyed(manifest: &Value, scm_facts: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let generated_path_rules = parse_generated_path_rules(manifest, &mut findings);
    let declared = parse_declared_artifacts(manifest, &mut findings);
    let declared_paths: BTreeSet<String> = declared
        .iter()
        .map(|artifact| artifact.path.clone())
        .collect();
    let tracked_required_paths: BTreeSet<String> = declared
        .iter()
        .filter(|artifact| artifact.requires_tracked_path())
        .map(|artifact| artifact.path.clone())
        .collect();
    let tracked_generated = tracked_generated_artifact_paths(scm_facts, &generated_path_rules);

    for path in tracked_generated.difference(&declared_paths) {
        findings.insert(Finding::new(
            "generated_artifact_tracked_generated_output_not_declared",
            path,
        ));
    }
    for path in tracked_required_paths.difference(&tracked_generated) {
        findings.insert(Finding::new(
            "generated_artifact_declared_path_not_tracked",
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
        if artifact.materialization_mode == "ci-artifact-only"
            && artifact.merge_policy != "not-tracked-in-git"
        {
            findings.insert(Finding::new(
                "generated_artifact_ci_artifact_only_requires_not_tracked_policy",
                &artifact.artifact_id,
            ));
        }
        if artifact.merge_policy == "not-tracked-in-git"
            && artifact.materialization_mode != "ci-artifact-only"
        {
            findings.insert(Finding::new(
                "generated_artifact_not_tracked_policy_requires_ci_artifact_only",
                &artifact.artifact_id,
            ));
        }
        if artifact.is_untracked_ci_artifact() && tracked_generated.contains(&artifact.path) {
            findings.insert(Finding::new(
                "generated_artifact_ci_artifact_only_path_tracked",
                &artifact.path,
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
        if artifact.artifact_class == "scm-facts-boundary-snapshot"
            && artifact.materialization_mode != "main-branch-materialized"
        {
            findings.insert(Finding::new(
                "generated_artifact_scm_facts_not_final_tree_materialized",
                &artifact.artifact_id,
            ));
        }
        if artifact.artifact_class == "scm-facts-boundary-snapshot"
            && artifact.merge_policy != "controller-owned-main-materialization"
        {
            findings.insert(Finding::new(
                "generated_artifact_scm_facts_not_controller_owned",
                &artifact.artifact_id,
            ));
        }
    }

    findings
}

pub fn evaluate(manifest: &Value, scm_facts: &Value) -> Report {
    let findings = evaluate_keyed(manifest, scm_facts);
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
                    "description": "common GraphQL and TypeScript generated output directory"
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
    fn ci_artifact_only_generated_artifacts_may_be_declared_without_tracked_path() {
        let mut face = artifact("example-face", "out/example.generated.json");
        face["materialization_mode"] = json!("ci-artifact-only");
        face["merge_policy"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![face]);
        let scm_facts = scm(&[]);

        assert_eq!(evaluate(&manifest, &scm_facts).verdict, Verdict::Green);
    }

    #[test]
    fn ci_artifact_only_generated_artifacts_must_not_remain_tracked() {
        let mut face = artifact("example-face", "out/example.generated.json");
        face["materialization_mode"] = json!("ci-artifact-only");
        face["merge_policy"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![face]);
        let scm_facts = scm(&["out/example.generated.json"]);

        let findings = evaluate_keyed(&manifest, &scm_facts);

        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_ci_artifact_only_path_tracked"
                && finding.key == "out/example.generated.json"
        }));
    }

    #[test]
    fn ci_artifact_only_mode_and_not_tracked_policy_must_be_paired() {
        let mut ci_without_policy = artifact("ci-without-policy", "out/ci.generated.json");
        ci_without_policy["materialization_mode"] = json!("ci-artifact-only");
        let mut policy_without_ci = artifact("policy-without-ci", "out/policy.generated.json");
        policy_without_ci["merge_policy"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![ci_without_policy, policy_without_ci]);
        let scm_facts = scm(&[]);

        let findings = evaluate_keyed(&manifest, &scm_facts);

        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_ci_artifact_only_requires_not_tracked_policy"
                && finding.key == "ci-without-policy"
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "generated_artifact_not_tracked_policy_requires_ci_artifact_only"
                && finding.key == "policy-without-ci"
        }));
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
        let manifest = manifest(Vec::new());
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
    fn diff_policy_allows_declared_materialized_generated_artifact_edits() {
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
            vec![DiffPolicyViolation {
                status: "M".to_owned(),
                path: "out/untracked.generated.json".to_owned(),
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
