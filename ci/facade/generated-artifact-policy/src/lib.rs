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

const ARTIFACT_CLASSES: [&str; 8] = [
    "authoritative-source",
    "sharded-projection",
    "main-materialized-aggregate",
    "append-only-ledger",
    "ephemeral-build-output",
    "review-artifact",
    "scm-facts-boundary-snapshot",
    // Human-authored ratchet / allowlist baseline: a reviewed, hand-curated shrink-only reference
    // (e.g. a frozen known-debt or known-warning set) with NO machine producer authority. It MUST
    // stay committed and MUST NOT be recomputed over the candidate tree — a candidate recompute
    // would erase a hand-shrunk burn-down or launder new debt. Distinct from the frozen-reference
    // firewall baseline (which the merge-base ratchet materializes via `git show`): this class is
    // the manifest-declared durable identity for hand-curated baselines, and it is the anchor for
    // the `hand_curated_ratchet_artifact_must_stay_committed` guard below.
    HAND_CURATED_RATCHET_CLASS,
];

/// Artifact class marking a human-authored ratchet / allowlist baseline. This is the durable
/// identity a naive de-commit cannot silently shed: flipping such a row to a de-commit
/// `materialization_mode` while the class stays hand-curated fires
/// `hand_curated_ratchet_artifact_must_stay_committed` in [`evaluate_keyed_with_frozen_references`].
const HAND_CURATED_RATCHET_CLASS: &str = "hand-curated-ratchet";

const MATERIALIZATION_MODES: [&str; 7] = [
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
    // Hand-curated-committed class: a human-authored ratchet/allowlist baseline that stays a
    // committed git blob and is NEVER recomputed over the candidate tree. The correct mode for a
    // `hand-curated-ratchet` artifact; the de-commit modes are forbidden for that class.
    "hand-curated-committed",
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

fn clean_repo_relative_path(path: &str) -> bool {
    !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains('\\')
        && path.split('/').all(|component| {
            !component.is_empty()
                && component != "."
                && component != ".."
                && !component.contains(':')
        })
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
    findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    let Some(tracked_paths) = scm_facts.get("tracked_paths") else {
        findings.insert(Finding::new(
            "generated_artifact_scm_facts_tracked_paths_missing",
            "tracked_paths",
        ));
        return BTreeSet::new();
    };
    let Some(tracked_paths) = tracked_paths.as_array() else {
        findings.insert(Finding::new(
            "generated_artifact_scm_facts_tracked_paths_not_array",
            "tracked_paths",
        ));
        return BTreeSet::new();
    };

    let mut parsed = BTreeSet::new();
    for (index, path) in tracked_paths.iter().enumerate() {
        let Some(path) = path.as_str().filter(|path| !path.trim().is_empty()) else {
            findings.insert(Finding::new(
                "generated_artifact_scm_facts_tracked_path_not_string",
                format!("tracked_paths[{index}]"),
            ));
            continue;
        };
        if is_tracked_generated_artifact_path(path, rules) {
            parsed.insert(path.to_owned());
        }
    }
    parsed
}

/// The FULL set of tracked paths from the SCM-facts snapshot (unfiltered by generated_path_rules).
/// Used to verify a hand-curated ratchet baseline — a committed but NOT generated-output artifact —
/// stays tracked in git. Structural findings for a missing/malformed `tracked_paths` are emitted by
/// [`tracked_generated_artifact_paths`], which is always evaluated first, so this reader is pure.
fn scm_tracked_paths(scm_facts: &Value) -> BTreeSet<String> {
    scm_facts
        .get("tracked_paths")
        .and_then(Value::as_array)
        .map(|paths| {
            paths
                .iter()
                .filter_map(|path| path.as_str().filter(|path| !path.trim().is_empty()))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn diff_candidate_paths<'a>(status: &str, paths: &'a [&'a str]) -> Result<Vec<&'a str>, ()> {
    let Some(status_kind) = status.chars().next() else {
        return Err(());
    };
    let single_path = || {
        if status.len() == 1 && paths.len() == 1 && !paths[0].is_empty() {
            Ok(vec![paths[0]])
        } else {
            Err(())
        }
    };

    match status_kind {
        'D' => {
            if status == "D" && paths.len() == 1 && !paths[0].is_empty() {
                Ok(Vec::new())
            } else {
                Err(())
            }
        }
        'R' | 'C' => {
            let score = &status[1..];
            if !score.is_empty()
                && score.chars().all(|character| character.is_ascii_digit())
                && paths.len() == 2
                && !paths[0].is_empty()
                && !paths[1].is_empty()
            {
                Ok(vec![paths[0], paths[1]])
            } else {
                Err(())
            }
        }
        'A' | 'M' | 'T' => single_path(),
        _ => Err(()),
    }
}

/// A rename/copy diff row is a SANCTIONED RELOCATION when it is a BYTE-IDENTICAL move of a
/// control-plane-declared generated artifact whose materialization class remains committed. A
/// capability move (ADR-0562) relocates such an artifact — e.g. the firewall's frozen
/// `gate-baseline.generated.json`, which cannot be
/// de-committed without breaking the merge-base ratchet's `git show <merge_base>:<path>` read
/// (the #828 deadlock the DECOMMIT_MATERIALIZATION_MODES doc names).
///
/// SECURITY — why this cannot launder frozen-baseline bytes: a `main-branch-materialized` face
/// (gate-baseline) is NOT byte-verified committed==regenerated by any gate (registry-drift and
/// freshness only prove its producer is deterministic), so a CONTENT change smuggled through a
/// move would be unverified and would become the trusted frozen reference post-merge. Therefore
/// the exemption requires an EXACTLY byte-identical relocation: `git diff --name-status` reports
/// the similarity score truncated (integer division), so `R100`/`C100` is emitted ONLY when the
/// old and new blobs are identical — a content-modified rename scores strictly below 100 and
/// stays a violation. The contributor may only RELOCATE the existing accepted bytes; the content
/// change a move legitimately needs (path re-keying) is produced by post-merge main-branch
/// materialization, never authored in the PR. Only renames/copies are exempt; a plain add/modify
/// of a declared artifact remains a violation.
fn is_sanctioned_relocation(
    status: &str,
    candidate_paths: &[&str],
    committed_artifact_paths: &BTreeSet<String>,
) -> bool {
    let is_byte_identical_rename =
        matches!(status.chars().next(), Some('R') | Some('C')) && status.get(1..) == Some("100");
    is_byte_identical_rename
        && candidate_paths.len() == 2
        && committed_artifact_paths.contains(candidate_paths[1])
}

/// Productized bridge predicate for presubmit diff surfaces. The caller supplies a
/// `git diff --name-status`-compatible stream, but generated-path classification comes from the
/// manifest's `generated_path_rules`, not from `.gitignore` or runner-local ignore heuristics.
pub fn generated_output_diff_policy_violations(
    manifest: &Value,
    diff_name_status: &str,
) -> (BTreeSet<Finding>, Vec<DiffPolicyViolation>) {
    generated_output_diff_policy_violations_with_ratchet_context(
        manifest,
        diff_name_status,
        &BTreeMap::new(),
        &[],
    )
}

/// Ratchet-context-aware diff policy. Identical to
/// [`generated_output_diff_policy_violations`] except a `normal-source-merge` artifact's plain
/// MODIFY is no longer blanket-allowed: it must pass [`validate_ratchet_diff`] against the
/// supplied merge-base/candidate content pair (`ratchet_contents`: declared `path` ->
/// `(merge_base_content, candidate_content)`) and the active move plan's `(old_path, new_path)`
/// pairs (`move_plan_pairs`). Missing content for an eligible path fails CLOSED (the exemption
/// protects nothing it cannot verify). R100/C100 sanctioned relocations remain exempt only for
/// committed destinations; a de-committed destination cannot appear in a contributor diff.
///
/// This closes the debt-laundering hole a blanket `merge_policy` exemption would open for the 5
/// hand-curated-ratchet baselines that gate merges (friction-accounting, embedded-asset-
/// hermeticity, tier-dependency-acyclicity, port-placement, the glossary-vocabulary allowlist):
/// ONE rule — shrink-only, or a move-plan-backed bijective key substitution with unchanged
/// cardinality/non-key values/ceilings — applies uniformly to all 5, not a special case per
/// artifact. R100/C100 relocation is exempt only when the declared destination is a committed
/// materialization class; a `not-tracked-in-git` destination stays RED because no contributor
/// diff may recreate a controller-only face.
pub fn generated_output_diff_policy_violations_with_ratchet_context(
    manifest: &Value,
    diff_name_status: &str,
    ratchet_contents: &BTreeMap<String, (String, String)>,
    move_plan_pairs: &[(String, String)],
) -> (BTreeSet<Finding>, Vec<DiffPolicyViolation>) {
    let mut findings = BTreeSet::new();
    let generated_path_rules = parse_generated_path_rules(manifest, &mut findings);
    let declared = parse_declared_artifacts(manifest, &mut findings);
    let declared_artifact_paths = declared
        .iter()
        .map(|artifact| artifact.path.clone())
        .collect::<BTreeSet<_>>();
    let committed_artifact_paths = declared
        .iter()
        .filter(|artifact| !is_decommit_materialization_mode(&artifact.materialization_mode))
        .map(|artifact| artifact.path.clone())
        .collect::<BTreeSet<_>>();
    let normal_source_merge_paths = diff_policy_allowed_generated_edit_paths(&declared);
    if !findings.is_empty() {
        return (findings, Vec::new());
    }

    let mut diff_rows = Vec::new();
    for (line_index, line) in diff_name_status.lines().enumerate() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.trim().is_empty() {
            continue;
        }

        let mut fields = line.split('\t');
        let status = fields.next().unwrap_or_default().to_owned();
        let paths = fields.collect::<Vec<_>>();
        match diff_candidate_paths(&status, &paths) {
            Ok(candidate_paths) => {
                // Sanctioned relocation (ADR-0562 capability move): a rename whose DESTINATION is a
                // control-plane-declared generated artifact relocates an already-accepted artifact
                // that MUST stay committed — e.g. the firewall's frozen `gate-baseline.generated.json`,
                // which the ratchet reads from the merge-base git blob, so de-committing it is the
                // #828 deadlock defect. This is a relocation, not contributor-authored bytes: the
                // relocated content is independently bound by the registry-drift / freshness gates
                // (committed==regenerated), so a laundered relocation REDs there, not here.
                if is_sanctioned_relocation(&status, &candidate_paths, &committed_artifact_paths) {
                    continue;
                }
                diff_rows.extend(
                    candidate_paths
                        .into_iter()
                        .map(|path| (status.clone(), path.to_owned())),
                );
            }
            Err(()) => {
                findings.insert(Finding::new(
                    "generated_artifact_diff_name_status_malformed",
                    format!("line {}", line_index + 1),
                ));
            }
        }
    }
    if !findings.is_empty() {
        return (findings, Vec::new());
    }

    let violations = diff_rows
        .into_iter()
        .filter_map(|(status, path)| {
            let is_generated = is_tracked_generated_artifact_path(&path, &generated_path_rules)
                || declared_artifact_paths.contains(&path);
            if !is_generated {
                return None;
            }
            if normal_source_merge_paths.contains(&path) {
                match ratchet_contents.get(&path) {
                    Some((merge_base, candidate)) => {
                        match validate_ratchet_diff(merge_base, candidate, move_plan_pairs) {
                            Ok(()) => None,
                            Err(_reason) => Some(DiffPolicyViolation { status, path }),
                        }
                    }
                    None => Some(DiffPolicyViolation { status, path }),
                }
            } else {
                Some(DiffPolicyViolation { status, path })
            }
        })
        .collect();

    (findings, violations)
}

/// The set of declared-artifact paths whose `merge_policy` is `normal-source-merge` (schema'd +
/// validated by [`parse_declared_artifacts`]) — a HAND-CURATED file (e.g. a `hand-curated-ratchet`
/// baseline like `embedded-asset-hermeticity-baseline.json`) whose manifest entry says a PR
/// author is EXPECTED to edit it directly, as opposed to a machine-only regenerated/materialized
/// face. A path in this set is NOT blanket-exempt — see
/// [`generated_output_diff_policy_violations_with_ratchet_context`], which additionally requires
/// [`validate_ratchet_diff`] to pass. Every OTHER `merge_policy` stays blocked from any edit
/// except a sanctioned R100/C100 relocation ([`is_sanctioned_relocation`]).
fn diff_policy_allowed_generated_edit_paths(declared: &[DeclaredArtifact]) -> BTreeSet<String> {
    declared
        .iter()
        .filter(|artifact| artifact.merge_policy == "normal-source-merge")
        .map(|artifact| artifact.path.clone())
        .collect()
}

/// One normalized row of a hand-curated-ratchet baseline, unified across the known on-disk
/// shapes (JSON `codes: {code: [key, ...]}` maps, JSON arrays-of-objects keyed by `subject` or
/// `member_path`, and TSV `<kind>\t<token>` rows): `group` is the debt CLASS a key belongs to
/// (a `codes` map key, a `violations[].code`, a glossary `kind`; empty when the format has no
/// grouping axis, e.g. port-placement's flat list), `key` is the per-row identity subject to
/// shrink/move-plan-substitution, and `rest` is every OTHER field on the row canonicalized to a
/// deterministic string (must stay byte-identical across a verified substitution).
#[derive(Debug, Clone, PartialEq, Eq)]
struct RatchetRow {
    group: String,
    key: String,
    rest: String,
}

/// Parse a ratchet-baseline file's content into its normalized rows plus its per-group ceiling
/// map (`_provenance.ceilings`, present on 2 of the 5 known formats; absent elsewhere, in which
/// case the ceiling check in [`validate_ratchet_diff`] is a no-op for that content).
fn parse_ratchet_content(
    content: &str,
) -> Result<(Vec<RatchetRow>, BTreeMap<String, u64>), String> {
    match serde_json::from_str::<Value>(content) {
        Ok(value) => parse_ratchet_json(&value),
        Err(_) => Ok((parse_ratchet_tsv(content), BTreeMap::new())),
    }
}

fn parse_ratchet_json(value: &Value) -> Result<(Vec<RatchetRow>, BTreeMap<String, u64>), String> {
    let ceilings: BTreeMap<String, u64> = value
        .get("_provenance")
        .and_then(|p| p.get("ceilings"))
        .and_then(Value::as_object)
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_u64().map(|n| (k.clone(), n)))
                .collect()
        })
        .unwrap_or_default();

    if let Some(codes) = value.get("codes").and_then(Value::as_object) {
        // codes-map shape (friction-accounting, embedded-asset-hermeticity): { code: [key, ...] }.
        let mut rows = Vec::new();
        for (code, keys) in codes {
            let Some(keys) = keys.as_array() else {
                return Err(format!("codes.{code} is not an array"));
            };
            for key in keys {
                let Some(key) = key.as_str() else {
                    return Err(format!("codes.{code} has a non-string key"));
                };
                rows.push(RatchetRow {
                    group: code.clone(),
                    key: key.to_owned(),
                    rest: String::new(),
                });
            }
        }
        return Ok((rows, ceilings));
    }
    for array_field in ["violations", "baseline"] {
        if let Some(array) = value.get(array_field).and_then(Value::as_array) {
            let mut rows = Vec::new();
            for row in array {
                let Some(obj) = row.as_object() else {
                    return Err(format!("{array_field}[] row is not an object"));
                };
                // The KEY field is the first of these present on the row; every other field is
                // `rest` (must stay unchanged across a verified substitution).
                let key_field = ["subject", "member_path"]
                    .into_iter()
                    .find(|f| obj.contains_key(*f))
                    .ok_or_else(|| format!("{array_field}[] row has no recognized key field"))?;
                let key = obj
                    .get(key_field)
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("{array_field}.{key_field} is not a string"))?;
                let group = obj
                    .get("code")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let mut rest = obj.clone();
                rest.remove(key_field);
                rows.push(RatchetRow {
                    group,
                    key: key.to_owned(),
                    rest: serde_json::to_string(&Value::Object(rest)).unwrap_or_default(),
                });
            }
            return Ok((rows, ceilings));
        }
    }
    Err("no recognized ratchet row shape (codes/violations/baseline)".to_owned())
}

/// `<group>\t<key>` TSV rows (the glossary-vocabulary allowlist); `#`-comment and blank lines
/// are skipped. No ceiling concept for this format.
fn parse_ratchet_tsv(content: &str) -> Vec<RatchetRow> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut cols = line.splitn(2, '\t');
            let group = cols.next()?.to_owned();
            let key = cols.next()?.to_owned();
            Some(RatchetRow {
                group,
                key,
                rest: String::new(),
            })
        })
        .collect()
}

/// Boundary-safe path-token replace — mirrors
/// `tools/oya-reorg-codemod-app/src/model.rs::rewrite_path_token` (duplicated rather than
/// cross-crate-shared: this gate crate must not depend on the reorg codemod's local-bridge-tool
/// surface). A match qualifies only when the byte before it (if any) is not a path-continuation
/// byte (alnum/`_`/`-`/`/`/`.`) and the byte after it (if any) is not an identifier-continuation
/// byte (alnum/`_`/`-`) — so `<old>/src/lib.rs:123` rewrites but a longer unrelated name that
/// merely starts with `old`, or `old` nested inside a longer unrelated path, does not.
fn rewrite_path_token_local(text: &str, old: &str, new: &str) -> Option<String> {
    if old.is_empty() {
        return None;
    }
    fn is_ident_continuation(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
    }
    fn is_path_continuation(b: u8) -> bool {
        is_ident_continuation(b) || b == b'/' || b == b'.'
    }
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let mut changed = false;
    let mut i = 0;
    while i < text.len() {
        if text[i..].starts_with(old) {
            let prev_ok = i == 0 || !is_path_continuation(bytes[i - 1]);
            let next = bytes.get(i + old.len()).copied();
            let next_ok = next.map(|b| !is_ident_continuation(b)).unwrap_or(true);
            if prev_ok && next_ok {
                out.push_str(&text[last..i]);
                out.push_str(new);
                last = i + old.len();
                i = last;
                changed = true;
                continue;
            }
        }
        let Some(ch) = text[i..].chars().next() else {
            break;
        };
        i += ch.len_utf8();
    }
    out.push_str(&text[last..]);
    changed.then_some(out)
}

/// Validate a hand-curated-ratchet baseline's content diff (merge-base vs candidate) against
/// the ONE rule applied uniformly to every known format: per debt-group, every key ADDED
/// relative to the merge-base must be explained by an ACTIVE move-plan `(old_path, new_path)`
/// pair rewriting a key that was REMOVED (cardinality-preserving 1:1 substitution, with the
/// row's `rest` unchanged), and every group's declared ceiling (when present in both sides) must
/// not increase. A bare addition, an unmatched substitution, or a ceiling increase is rejected.
fn validate_ratchet_diff(
    merge_base_content: &str,
    candidate_content: &str,
    move_plan_pairs: &[(String, String)],
) -> Result<(), String> {
    let (before_rows, before_ceilings) = parse_ratchet_content(merge_base_content)?;
    let (after_rows, after_ceilings) = parse_ratchet_content(candidate_content)?;

    let mut before_by_group: BTreeMap<&str, Vec<&RatchetRow>> = BTreeMap::new();
    for row in &before_rows {
        before_by_group
            .entry(row.group.as_str())
            .or_default()
            .push(row);
    }
    let mut after_by_group: BTreeMap<&str, Vec<&RatchetRow>> = BTreeMap::new();
    for row in &after_rows {
        after_by_group
            .entry(row.group.as_str())
            .or_default()
            .push(row);
    }

    let mut groups: BTreeSet<&str> = BTreeSet::new();
    groups.extend(before_by_group.keys().copied());
    groups.extend(after_by_group.keys().copied());

    for group in groups {
        let before: &[&RatchetRow] = before_by_group.get(group).map(Vec::as_slice).unwrap_or(&[]);
        let after: &[&RatchetRow] = after_by_group.get(group).map(Vec::as_slice).unwrap_or(&[]);

        let before_keys: BTreeSet<&str> = before.iter().map(|r| r.key.as_str()).collect();
        let after_keys: BTreeSet<&str> = after.iter().map(|r| r.key.as_str()).collect();

        let added: Vec<&&RatchetRow> = after
            .iter()
            .filter(|r| !before_keys.contains(r.key.as_str()))
            .collect();
        let mut removed_pool: Vec<&&RatchetRow> = before
            .iter()
            .filter(|r| !after_keys.contains(r.key.as_str()))
            .collect();

        for add_row in &added {
            let matched_index = removed_pool.iter().position(|rm_row| {
                rm_row.rest == add_row.rest
                    && move_plan_pairs.iter().any(|(old, new)| {
                        rewrite_path_token_local(&rm_row.key, old, new).as_deref()
                            == Some(add_row.key.as_str())
                    })
            });
            match matched_index {
                Some(idx) => {
                    removed_pool.remove(idx);
                }
                None => {
                    return Err(format!(
                        "unexplained new key {:?} in group {group:?} (not a move-plan-backed \
                         substitution of any removed key)",
                        add_row.key
                    ));
                }
            }
        }

        if let (Some(&before_ceiling), Some(&after_ceiling)) =
            (before_ceilings.get(group), after_ceilings.get(group))
            && after_ceiling > before_ceiling
        {
            return Err(format!(
                "ceiling increase in group {group:?}: {before_ceiling} -> {after_ceiling}"
            ));
        }
    }
    Ok(())
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
        if !clean_repo_relative_path(path) {
            findings.insert(Finding::new(
                "generated_artifact_manifest_path_not_clean_repo_relative",
                artifact_id,
            ));
        }
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
        // A hand-curated ratchet baseline is HUMAN-authored, not machine-generated: there is no
        // producer to declare, so a `generator` block is OPTIONAL for this class (and still fully
        // validated when present). Every other class requires one.
        if artifact_class != HAND_CURATED_RATCHET_CLASS || artifact.get("generator").is_some() {
            validate_generator(artifact, artifact_id, artifact_class, findings);
        }
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

/// The ratchet policy's `frozen_reference.source` value marking a frozen reference as REGENERATED
/// from the merge-base source tree (ADR-0616), NOT read from a committed git blob. A frozen
/// reference declaring this source MAY be de-committed (`not-tracked-in-git`): the emitter
/// regenerates it by running the producer over the merge-base worktree, so there is no committed
/// blob to empty and no #828 deadlock. Absent (the default) means the legacy committed-git-blob
/// source (ADR-0596 must-stay-committed).
pub const FROZEN_REFERENCE_SOURCE_REGENERATE: &str = "regenerate-from-merge-base-source";

/// Extract the COMMITTED-GIT-BLOB firewall frozen-reference face paths from one or more parsed
/// `ratchet-policy.json` values. This set is the authoritative, repo-agnostic signal for "this path
/// is materialized from the merge-base git BLOB" (`git show <merge_base>:<face_path>`, ADR-0551), so
/// de-committing it is the #828 deadlock. A frozen reference that DECLARES
/// `frozen_reference.source == FROZEN_REFERENCE_SOURCE_REGENERATE` (ADR-0616) is EXCLUDED: it is
/// regenerated from the merge-base source, so it is not a committed-blob reference and MAY be
/// de-committed. The function tolerates either a single `frozen_reference` object or an array of them
/// so adopters can declare multiple frozen baselines. It carries NO hardcoded oyatie path — both the
/// set and the regenerate-from-source exemption are pure data from the policy files.
pub fn frozen_reference_face_paths_keyed<'a, I>(
    ratchet_policies: I,
) -> (BTreeSet<String>, BTreeSet<Finding>)
where
    I: IntoIterator<Item = &'a Value>,
{
    let mut paths = BTreeSet::new();
    let mut findings = BTreeSet::new();
    for policy in ratchet_policies {
        let Some(frozen) = policy.get("frozen_reference") else {
            continue;
        };
        match frozen {
            Value::Array(entries) => {
                for (index, entry) in entries.iter().enumerate() {
                    collect_frozen_reference_face_path(
                        entry,
                        &format!("frozen_reference[{index}]"),
                        &mut paths,
                        &mut findings,
                    );
                }
            }
            _ => {
                collect_frozen_reference_face_path(
                    frozen,
                    "frozen_reference",
                    &mut paths,
                    &mut findings,
                );
            }
        }
    }
    (paths, findings)
}

fn collect_frozen_reference_face_path(
    frozen_reference: &Value,
    scope: &str,
    paths: &mut BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) {
    if !frozen_reference.is_object() {
        findings.insert(Finding::new(
            "generated_artifact_ratchet_policy_frozen_reference_not_object",
            scope,
        ));
        return;
    }
    let Some(face_path) = required_str(frozen_reference, "face_path") else {
        findings.insert(Finding::new(
            "generated_artifact_ratchet_policy_frozen_reference_face_path_missing",
            format!("{scope}.face_path"),
        ));
        return;
    };
    // ADR-0616 inversion: a frozen reference that declares regenerate-from-merge-base-source is NOT
    // a committed-git-blob reference — it is regenerated from the merge-base source, so it is
    // EXEMPT from the must-stay-committed set (and MAY be de-committed). Data-driven: the exemption
    // is the policy `source` field, no hardcoded path.
    if frozen_reference.get("source").and_then(Value::as_str)
        == Some(FROZEN_REFERENCE_SOURCE_REGENERATE)
    {
        return;
    }
    paths.insert(face_path.to_owned());
}

pub fn frozen_reference_face_paths<'a, I>(ratchet_policies: I) -> BTreeSet<String>
where
    I: IntoIterator<Item = &'a Value>,
{
    let (paths, _) = frozen_reference_face_paths_keyed(ratchet_policies);
    paths
}

/// Make-it-impossible guard for the #828 dev-wide deadlock class (ADR-0596), INVERTED by ADR-0616.
/// A firewall frozen-reference/baseline read from the committed git BLOB at the merge-base
/// (`git show <merge_base>:<face_path>`, ADR-0551) empties the ratchet baseline if de-committed, so
/// every pre-existing repo-wide debt item reads as a NEW regression on every broad-affected-set PR —
/// the #828 dev regression, hotfixed by #830. This rule fires `frozen_reference_artifact_must_stay_
/// committed` when a declared artifact whose `path` is a committed-git-blob frozen reference is
/// declared with a de-commit mode.
///
/// ADR-0616 inversion: a frozen reference MAY be de-committed IFF its ratchet policy declares
/// `frozen_reference.source == FROZEN_REFERENCE_SOURCE_REGENERATE` — the emitter then REGENERATES it
/// from the merge-base source (no committed blob to empty, so #828 stays impossible). Those paths are
/// EXCLUDED from `frozen_reference_paths` by [`frozen_reference_face_paths_keyed`], so this predicate
/// still RED-blocks a de-commit of a committed-git-blob reference while allowing a de-commit of a
/// regenerate-from-source reference. The set is supplied as DATA by the caller (from
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
    // Hand-curated ratchet baselines are committed source-adjacent files, NOT generated outputs:
    // they legitimately do NOT match any generated_path_rule, so they are exempt from the "declared
    // path must be a tracked GENERATED output" difference below. They MUST still be tracked in git;
    // that is checked against the FULL tracked-paths set.
    let hand_curated_paths: BTreeSet<String> = declared
        .iter()
        .filter(|artifact| artifact.artifact_class == HAND_CURATED_RATCHET_CLASS)
        .map(|artifact| artifact.path.clone())
        .collect();
    let all_tracked_paths = scm_tracked_paths(scm_facts);
    let tracked_generated =
        tracked_generated_artifact_paths(scm_facts, &generated_path_rules, &mut findings);

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
        // A hand-curated ratchet baseline need not match a generated_path_rule (it is committed
        // source, not a generated output), but it MUST stay tracked. Passing that full-tracked-set
        // check clears it; an untracked hand-curated path falls through to the finding below, so a
        // silent `git rm` of a hand-curated baseline is still RED.
        if hand_curated_paths.contains(path) && all_tracked_paths.contains(path) {
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
        // Hand-curated ratchet/allowlist baselines (friction accounting, embedded-asset
        // hermeticity, tier-dependency acyclicity, port-placement, glossary warning allowlist) are
        // HUMAN-authored shrink-only references that MUST stay committed and MUST NOT be recomputed
        // over the candidate tree — any other materialization mode erases a hand-shrunk burn-down or
        // launders new debt. Data-driven (the class constant plus the committed-only mode), zero
        // hardcoded paths — the class is the durable identity a naive de-commit cannot silently shed.
        if artifact.artifact_class == HAND_CURATED_RATCHET_CLASS
            && artifact.materialization_mode != "hand-curated-committed"
        {
            findings.insert(Finding::new(
                "hand_curated_ratchet_artifact_must_stay_committed",
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
                "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
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
            "D\tlegacy/generated/hr-api.d.ts\n",
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
    fn diff_policy_rejects_rename_or_copy_from_generated_output() {
        let manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        let diff = concat!(
            "R100\tout/old.generated.json\tsrc/old.rs\n",
            "C100\tout/copied.generated.json\tsrc/copied.rs\n",
        );

        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);

        assert_eq!(findings, BTreeSet::new());
        assert_eq!(
            violations,
            vec![
                DiffPolicyViolation {
                    status: "R100".to_owned(),
                    path: "out/old.generated.json".to_owned(),
                },
                DiffPolicyViolation {
                    status: "C100".to_owned(),
                    path: "out/copied.generated.json".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn diff_policy_rejects_declared_artifact_even_when_path_rules_miss_it() {
        let manifest = manifest(vec![artifact("canonical-face", "out/canonical-face.json")]);
        let diff = "M\tout/canonical-face.json\n";

        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);

        assert_eq!(findings, BTreeSet::new());
        assert_eq!(
            violations,
            vec![DiffPolicyViolation {
                status: "M".to_owned(),
                path: "out/canonical-face.json".to_owned(),
            }]
        );
    }

    #[test]
    fn diff_policy_fails_closed_on_malformed_name_status_lines() {
        let manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        let diff = concat!(
            "M\tout/example.generated.json\n",
            "M out/missing-tab.generated.json\n",
            "R100\tout/old.generated.json\n",
        );

        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);

        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_diff_name_status_malformed"
                    && finding.key == "line 2"
            }),
            "space-delimited WIP diff rows must not be silently ignored: {findings:#?}"
        );
        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_diff_name_status_malformed"
                    && finding.key == "line 3"
            }),
            "rename/copy rows without both paths must fail closed: {findings:#?}"
        );
        assert!(
            violations.is_empty(),
            "malformed diff input should fail before reporting partial policy violations"
        );
    }

    #[test]
    fn diff_policy_exempts_a_byte_identical_relocation_of_a_declared_artifact() {
        // ADR-0562 capability move: a frozen-reference declared artifact (e.g. the firewall's
        // gate-baseline.generated.json) relocates BYTE-IDENTICALLY (R100). It is a relocation of
        // already-accepted bytes, not contributor-authored content; the content re-keying happens
        // in post-merge main-branch materialization.
        let manifest = manifest(vec![artifact(
            "frozen-ref",
            "ci/facade/frozen.generated.json",
        )]);
        let diff = "R100\tcloud/old/frozen.generated.json\tci/facade/frozen.generated.json\n";
        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);
        assert!(findings.is_empty(), "no findings expected: {findings:#?}");
        assert!(
            violations.is_empty(),
            "a BYTE-IDENTICAL (R100) rename to a declared artifact is a sanctioned relocation: {violations:#?}"
        );
    }

    #[test]
    fn diff_policy_rejects_byte_identical_relocation_into_decommitted_artifact() {
        let mut face = artifact("controller-only", "registry/graph/controller-only.json");
        face["materialization_mode"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![face]);
        let diff = concat!(
            "R100\tstaging/old.json\tregistry/graph/controller-only.json\n",
            "C100\tstaging/copy.json\tregistry/graph/controller-only.json\n",
        );

        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);

        assert!(findings.is_empty(), "no findings expected: {findings:#?}");
        assert_eq!(
            violations,
            vec![
                DiffPolicyViolation {
                    status: "R100".to_owned(),
                    path: "registry/graph/controller-only.json".to_owned(),
                },
                DiffPolicyViolation {
                    status: "C100".to_owned(),
                    path: "registry/graph/controller-only.json".to_owned(),
                },
            ],
            "an untracked controller face must reject rename/copy destinations even when the bytes are identical"
        );
    }

    #[test]
    fn diff_policy_rejects_a_content_modified_relocation_of_a_declared_artifact() {
        // The load-bearing security bound: a rename that MODIFIES content (similarity < 100 — git
        // truncates the score, so any content change is < R100) must remain a violation, even to a
        // declared destination. This is what stops a move from laundering frozen-baseline bytes
        // (mode flips / tolerated-key adds) that no downstream gate byte-verifies for a
        // main-branch-materialized face.
        let manifest = manifest(vec![artifact(
            "frozen-ref",
            "ci/facade/frozen.generated.json",
        )]);
        let diff = "R098\tcloud/old/frozen.generated.json\tci/facade/frozen.generated.json\n";
        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);
        assert!(findings.is_empty());
        assert!(
            violations
                .iter()
                .any(|violation| violation.path == "ci/facade/frozen.generated.json"),
            "a content-modified (R<100) relocation must remain a violation: {violations:#?}"
        );
    }

    #[test]
    fn diff_policy_still_rejects_a_plain_modify_of_a_declared_artifact() {
        // The relocation exemption is renames/copies ONLY — a plain modify of a declared generated
        // artifact remains contributor-authored bytes and a violation.
        let manifest = manifest(vec![artifact(
            "frozen-ref",
            "ci/facade/frozen.generated.json",
        )]);
        let diff = "M\tci/facade/frozen.generated.json\n";
        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);
        assert!(findings.is_empty());
        assert!(
            violations
                .iter()
                .any(|violation| violation.path == "ci/facade/frozen.generated.json"),
            "a modify of a declared artifact must remain a violation: {violations:#?}"
        );
    }

    #[test]
    fn diff_policy_allows_a_plain_modify_of_a_normal_source_merge_artifact_with_valid_content() {
        // GREEN: the ACTUAL #1335 need — embedded-asset-hermeticity-baseline.json's
        // move-triggered path-key relabel (openapi-domain: `oya/intelligence/crates/
        // oya-intelligence-openapi-domain` -> `intelligence/core/openapi-domain`) is a
        // move-plan-backed bijective substitution (cardinality-preserving, ceiling unchanged),
        // so it passes `validate_ratchet_diff` and the plain modify is allowed. A blanket
        // `merge_policy` exemption is NOT enough on its own (see the RED tests below) — the
        // content must actually be verified.
        let path = "ci/facade/hand-curated-baseline.json";
        let mut face = artifact("hand-curated-ratchet", path);
        face["merge_policy"] = json!("normal-source-merge");
        let manifest = manifest(vec![face]);
        let diff = format!("M\t{path}\n");
        let merge_base = r#"{"_provenance":{"ceilings":{"skip_non_literal_argument":1}},
            "codes":{"skip_non_literal_argument":
                ["oya/intelligence/crates/oya-intelligence-openapi-domain/src/lib.rs:6088"]}}"#;
        let candidate = r#"{"_provenance":{"ceilings":{"skip_non_literal_argument":1}},
            "codes":{"skip_non_literal_argument":
                ["intelligence/core/openapi-domain/src/lib.rs:6088"]}}"#;
        let mut ratchet_contents = BTreeMap::new();
        ratchet_contents.insert(
            path.to_owned(),
            (merge_base.to_owned(), candidate.to_owned()),
        );
        let move_plan_pairs = vec![(
            "oya/intelligence/crates/oya-intelligence-openapi-domain".to_owned(),
            "intelligence/core/openapi-domain".to_owned(),
        )];
        let (findings, violations) = generated_output_diff_policy_violations_with_ratchet_context(
            &manifest,
            &diff,
            &ratchet_contents,
            &move_plan_pairs,
        );
        assert!(
            findings.is_empty(),
            "no manifest findings expected: {findings:#?}"
        );
        assert!(
            violations.is_empty(),
            "the #1335 openapi-domain move-plan-backed relabel must be allowed: {violations:#?}"
        );
    }

    #[test]
    fn diff_policy_rejects_a_normal_source_merge_plain_modify_with_no_ratchet_content_supplied() {
        // RED: the blanket merge_policy exemption alone is NOT sufficient — with no
        // merge-base/candidate content supplied (e.g. via the bare 2-arg wrapper), the
        // exemption fails CLOSED rather than blanket-allowing.
        let path = "ci/facade/hand-curated-baseline.json";
        let mut face = artifact("hand-curated-ratchet", path);
        face["merge_policy"] = json!("normal-source-merge");
        let manifest = manifest(vec![face]);
        let diff = format!("M\t{path}\n");
        let (findings, violations) = generated_output_diff_policy_violations(&manifest, &diff);
        assert!(findings.is_empty());
        assert!(
            violations.iter().any(|v| v.path == path),
            "no ratchet content supplied must fail closed (still a violation): {violations:#?}"
        );
    }

    #[test]
    fn diff_policy_rejects_other_merge_policies_on_plain_modify() {
        // RED preserved: `normal-source-merge` is the ONLY merge_policy this predicate widens.
        // Every other declared policy (regenerate-only here) keeps blocking a plain modify.
        let mut face = artifact(
            "regenerate-only",
            "ci/facade/regenerate-only.generated.json",
        );
        face["merge_policy"] = json!("never-manual-merge-regenerate-from-source-tree");
        let manifest = manifest(vec![face]);
        let diff = "M\tci/facade/regenerate-only.generated.json\n";
        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);
        assert!(findings.is_empty());
        assert!(
            violations
                .iter()
                .any(|violation| violation.path == "ci/facade/regenerate-only.generated.json"),
            "a non-normal-source-merge artifact must remain blocked on plain modify: {violations:#?}"
        );
    }

    // --- validate_ratchet_diff: the ONE shrink-only/move-plan-relabel rule, unit-tested
    // directly across every known ratchet format (codes-map, violations-array,
    // baseline-array, glossary TSV). ---

    #[test]
    fn ratchet_diff_allows_identical_content() {
        let content = r#"{"codes":{"c1":["a","b"]}}"#;
        assert_eq!(validate_ratchet_diff(content, content, &[]), Ok(()));
    }

    #[test]
    fn ratchet_diff_allows_pure_shrink_codes_map() {
        let before = r#"{"codes":{"c1":["a","b"]}}"#;
        let after = r#"{"codes":{"c1":["a"]}}"#;
        assert_eq!(validate_ratchet_diff(before, after, &[]), Ok(()));
    }

    #[test]
    fn ratchet_diff_allows_move_plan_backed_substitution_codes_map() {
        // The #1335 openapi-domain shape: one code, one key, rewritten by an active move.
        let before = r#"{"_provenance":{"ceilings":{"c1":1}},"codes":{"c1":["old/dir/f.rs:1"]}}"#;
        let after = r#"{"_provenance":{"ceilings":{"c1":1}},"codes":{"c1":["new/dir/f.rs:1"]}}"#;
        let moves = vec![("old/dir".to_owned(), "new/dir".to_owned())];
        assert_eq!(validate_ratchet_diff(before, after, &moves), Ok(()));
    }

    #[test]
    fn ratchet_diff_rejects_bare_addition() {
        let before = r#"{"codes":{"c1":["a"]}}"#;
        let after = r#"{"codes":{"c1":["a","b"]}}"#;
        assert!(
            validate_ratchet_diff(before, after, &[]).is_err(),
            "an added key with nothing removed must RED"
        );
    }

    #[test]
    fn ratchet_diff_rejects_unmatched_substitution_not_in_move_plan() {
        // A key removed AND a different key added, but NO active move explains the swap — the
        // debt-laundering hole: substituting real-debt-key-X for an unrelated laundered-key-Y.
        let before = r#"{"codes":{"c1":["a/real-debt.rs:1"]}}"#;
        let after = r#"{"codes":{"c1":["b/laundered.rs:1"]}}"#;
        let moves = vec![("unrelated/old".to_owned(), "unrelated/new".to_owned())];
        assert!(
            validate_ratchet_diff(before, after, &moves).is_err(),
            "a key substitution NOT backed by any active move plan entry must RED"
        );
    }

    #[test]
    fn ratchet_diff_rejects_ceiling_increase() {
        let before = r#"{"_provenance":{"ceilings":{"c1":1}},"codes":{"c1":["a"]}}"#;
        let after = r#"{"_provenance":{"ceilings":{"c1":2}},"codes":{"c1":["a"]}}"#;
        assert!(
            validate_ratchet_diff(before, after, &[]).is_err(),
            "a ceiling bump with no corresponding key growth must still RED"
        );
    }

    #[test]
    fn ratchet_diff_rejects_ceiling_increase_even_alongside_a_valid_substitution() {
        let before = r#"{"_provenance":{"ceilings":{"c1":1}},"codes":{"c1":["old/dir/f.rs:1"]}}"#;
        let after = r#"{"_provenance":{"ceilings":{"c1":2}},"codes":{"c1":["new/dir/f.rs:1"]}}"#;
        let moves = vec![("old/dir".to_owned(), "new/dir".to_owned())];
        assert!(
            validate_ratchet_diff(before, after, &moves).is_err(),
            "a ceiling increase must RED even when the key substitution itself is move-plan-backed"
        );
    }

    #[test]
    fn ratchet_diff_allows_shrink_on_violations_array_shape() {
        // tier-dependency-acyclicity shape: {"violations": [{"code","subject"}, ...]}.
        let before = r#"{"violations":[
            {"code":"TDA-X","subject":"a -> b"},
            {"code":"TDA-X","subject":"c -> d"}
        ]}"#;
        let after = r#"{"violations":[{"code":"TDA-X","subject":"a -> b"}]}"#;
        assert_eq!(validate_ratchet_diff(before, after, &[]), Ok(()));
    }

    #[test]
    fn ratchet_diff_allows_move_plan_substitution_on_baseline_array_shape_with_rest_preserved() {
        // port-placement shape: {"baseline": [{"member_path","trait","reason"}, ...]}. The
        // substitution must preserve the row's OTHER fields (here: trait + reason) exactly.
        let before = r#"{"baseline":[
            {"member_path":"old/dir","trait":"SessionStore","reason":"same reason text"}
        ]}"#;
        let after = r#"{"baseline":[
            {"member_path":"new/dir","trait":"SessionStore","reason":"same reason text"}
        ]}"#;
        let moves = vec![("old/dir".to_owned(), "new/dir".to_owned())];
        assert_eq!(validate_ratchet_diff(before, after, &moves), Ok(()));
    }

    #[test]
    fn ratchet_diff_rejects_baseline_array_substitution_when_rest_changed() {
        // Same move-plan-backed key substitution as above, but the `reason` text ALSO changed —
        // "all non-key values unchanged" must be enforced, not just the key.
        let before = r#"{"baseline":[
            {"member_path":"old/dir","trait":"SessionStore","reason":"original reason"}
        ]}"#;
        let after = r#"{"baseline":[
            {"member_path":"new/dir","trait":"SessionStore","reason":"DIFFERENT reason"}
        ]}"#;
        let moves = vec![("old/dir".to_owned(), "new/dir".to_owned())];
        assert!(
            validate_ratchet_diff(before, after, &moves).is_err(),
            "a substitution that also changes a non-key field must RED"
        );
    }

    #[test]
    fn ratchet_diff_allows_shrink_on_glossary_tsv_shape() {
        let before = "# comment\ncasing-variant\tOya\nuncited-acronym\tAA\n";
        let after = "# comment\ncasing-variant\tOya\n";
        assert_eq!(validate_ratchet_diff(before, after, &[]), Ok(()));
    }

    #[test]
    fn ratchet_diff_rejects_glossary_tsv_addition() {
        let before = "casing-variant\tOya\n";
        let after = "casing-variant\tOya\ncasing-variant\tNewToken\n";
        assert!(validate_ratchet_diff(before, after, &[]).is_err());
    }

    #[test]
    fn diff_policy_relocation_exemption_is_bounded_to_declared_destinations() {
        // Even a BYTE-IDENTICAL (R100) rename whose destination is a generated PATH but NOT a
        // control-plane-declared artifact is not a sanctioned relocation — it must remain a
        // violation (bounds the exemption so a rename cannot introduce an undeclared generated
        // output).
        let manifest = manifest(vec![artifact(
            "frozen-ref",
            "ci/facade/frozen.generated.json",
        )]);
        let diff = "R100\tci/facade/frozen.generated.json\tci/facade/undeclared.generated.json\n";
        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);
        assert!(findings.is_empty());
        assert!(
            violations
                .iter()
                .any(|violation| violation.path == "ci/facade/undeclared.generated.json"),
            "a rename to an UNDECLARED generated path must remain a violation: {violations:#?}"
        );
    }

    #[test]
    fn diff_policy_fails_closed_on_unmerged_unknown_or_broken_pair_status() {
        let manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        let diff = concat!(
            "U\tout/example.generated.json\n",
            "X\tout/example.generated.json\n",
            "B\tout/example.generated.json\n",
        );

        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);

        for line in 1..=3 {
            assert!(
                findings.iter().any(|finding| {
                    finding.code == "generated_artifact_diff_name_status_malformed"
                        && finding.key == format!("line {line}")
                }),
                "U/X/B diff rows must fail closed on line {line}: {findings:#?}"
            );
        }
        assert!(
            violations.is_empty(),
            "malformed U/X/B rows should fail before reporting policy violations"
        );
    }

    #[test]
    fn diff_policy_fails_closed_on_bare_rename_or_copy_status() {
        let manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        let diff = concat!(
            "R\tREADME.md\tREADME-copy.md\n",
            "C\tREADME.md\tREADME-copy.md\n",
        );

        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);

        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_diff_name_status_malformed"
                    && finding.key == "line 1"
            }),
            "bare rename status must fail closed: {findings:#?}"
        );
        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_diff_name_status_malformed"
                    && finding.key == "line 2"
            }),
            "bare copy status must fail closed: {findings:#?}"
        );
        assert!(
            violations.is_empty(),
            "malformed bare rename/copy rows should fail before reporting partial policy violations"
        );
    }

    #[test]
    fn diff_policy_fails_closed_on_conflict_marker_diff_input() {
        let manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        let diff = concat!(
            "<<<<<<< HEAD\n",
            "M\tout/example.generated.json\n",
            "=======\n",
            ">>>>>>> branch\n",
        );

        let (findings, violations) = generated_output_diff_policy_violations(&manifest, diff);

        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_diff_name_status_malformed"
                    && finding.key == "line 1"
            }),
            "preserved conflict-marker WIP must be treated as invalid data: {findings:#?}"
        );
        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_diff_name_status_malformed"
                    && finding.key == "line 3"
            }),
            "conflict separator must be treated as invalid data: {findings:#?}"
        );
        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_diff_name_status_malformed"
                    && finding.key == "line 4"
            }),
            "conflict trailer must be treated as invalid data: {findings:#?}"
        );
        assert!(
            violations.is_empty(),
            "malformed conflict input should fail before reporting partial policy violations"
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
    fn declared_generated_artifact_paths_must_be_clean_repo_relative() {
        for path in [
            "/abs/example.generated.json",
            "../outside.generated.json",
            "out/../outside.generated.json",
            "out//example.generated.json",
            r"out\example.generated.json",
            "C:/abs/example.generated.json",
            "C:relative/example.generated.json",
            "out/C:drive-component.generated.json",
        ] {
            let manifest = manifest(vec![artifact("example-face", path)]);
            let scm_facts = scm(&[path]);
            let findings = evaluate_keyed(&manifest, &scm_facts);
            assert!(
                findings.iter().any(|finding| {
                    finding.code == "generated_artifact_manifest_path_not_clean_repo_relative"
                        && finding.key == "example-face"
                }),
                "unclean path {path} must RED; findings: {findings:#?}"
            );
        }
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

    // A hand-curated ratchet/allowlist baseline: human-authored, stays committed, NO machine
    // producer (generator omitted). Path is intentionally NOT a generated_path_rules match, mirroring
    // the real baselines (`*-baseline.json`, `warning-baseline.tsv`).
    fn hand_curated(id: &str, path: &str) -> Value {
        json!({
            "artifact_id": id,
            "path": path,
            "artifact_class": HAND_CURATED_RATCHET_CLASS,
            "materialization_mode": "hand-curated-committed",
            "merge_policy": "normal-source-merge",
            "owner_team": "cloud-ci-platform",
            "source_inputs": ["human review at gate go-live"],
            "final_tree_validation": "hand-curated shrink-only reference; stays committed, never recomputed over the candidate tree",
            "public_product_contract": "a human-authored ratchet baseline must not be de-committed or laundered"
        })
    }

    #[test]
    fn hand_curated_committed_baseline_is_green_without_a_generator() {
        // GREEN: a declared, committed hand-curated ratchet baseline (no producer to declare) is
        // clean — the generator block is OPTIONAL for this class, and a committed mode requires the
        // path to stay tracked.
        let manifest = manifest(vec![hand_curated(
            "known-debt-baseline",
            "ci/facade/example/known-debt-baseline.json",
        )]);
        let scm_facts = scm(&["ci/facade/example/known-debt-baseline.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(
            !findings
                .iter()
                .any(|finding| finding.code == "generated_artifact_manifest_generator_missing"),
            "a hand-curated baseline needs no generator; findings: {findings:#?}"
        );
        assert_eq!(evaluate(&manifest, &scm_facts).verdict, Verdict::Green);
    }

    #[test]
    fn hand_curated_baseline_git_rmd_without_manifest_change_is_red() {
        // Enforcement: a hand-curated baseline that is `git rm`'d (untracked) while its row stays a
        // committed mode is RED via declared_path_not_tracked — the committed mode requires the file
        // to stay a tracked git blob, closing the "just delete it" de-commit path.
        let manifest = manifest(vec![hand_curated(
            "known-debt-baseline",
            "ci/facade/example/known-debt-baseline.json",
        )]);
        let scm_facts = scm(&[]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_declared_path_not_tracked"
                    && finding.key == "ci/facade/example/known-debt-baseline.json"
            }),
            "an untracked hand-curated baseline must RED; findings: {findings:#?}"
        );
    }

    #[test]
    fn hand_curated_baseline_declared_de_commit_is_red() {
        // RED make-it-impossible guard: flipping a hand-curated ratchet baseline to a de-commit
        // materialization mode (to launder the git-rm past the declared-path-not-tracked exemption)
        // fires hand_curated_ratchet_artifact_must_stay_committed — the class is the durable identity.
        let mut face = hand_curated(
            "known-debt-baseline",
            "ci/facade/example/known-debt-baseline.json",
        );
        face["materialization_mode"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![face]);
        let scm_facts = scm(&[]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(
            findings.iter().any(|finding| {
                finding.code == "hand_curated_ratchet_artifact_must_stay_committed"
                    && finding.key == "known-debt-baseline"
            }),
            "de-committing a hand-curated baseline must RED; findings: {findings:#?}"
        );
        assert_eq!(evaluate(&manifest, &scm_facts).verdict, Verdict::Red);
    }

    #[test]
    fn hand_curated_baseline_declared_non_committed_mode_is_red() {
        // Not enough to forbid explicit de-commit modes: this class is committed-only, or a
        // candidate-regenerated/CI-artifact row can still launder the hand-shrunk baseline.
        let mut face = hand_curated(
            "known-debt-baseline",
            "ci/facade/example/known-debt-baseline.json",
        );
        face["materialization_mode"] = json!("merge-candidate-regenerated");
        face["generator"] = json!({"command": "fake-generator"});
        let manifest = manifest(vec![face]);
        let scm_facts = scm(&["ci/facade/example/known-debt-baseline.json"]);
        let findings = evaluate_keyed(&manifest, &scm_facts);
        assert!(
            findings.iter().any(|finding| {
                finding.code == "hand_curated_ratchet_artifact_must_stay_committed"
                    && finding.key == "known-debt-baseline"
            }),
            "every non-committed hand-curated mode must RED; findings: {findings:#?}"
        );
        assert_eq!(evaluate(&manifest, &scm_facts).verdict, Verdict::Red);
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
    fn malformed_scm_facts_missing_tracked_paths_is_red() {
        let mut face = artifact("decommitted-face", "out/example.generated.json");
        face["materialization_mode"] = json!("not-tracked-in-git");
        face["merge_policy"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![face]);
        let scm_facts = json!({
            "schema": "oya-ci/scm-facts/v1"
        });

        let findings = evaluate_keyed(&manifest, &scm_facts);

        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_scm_facts_tracked_paths_missing"
                    && finding.key == "tracked_paths"
            }),
            "missing tracked_paths must fail closed even when all artifacts are de-commit class: {findings:#?}"
        );
    }

    #[test]
    fn malformed_scm_facts_tracked_paths_shape_is_red() {
        let manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        let scm_facts = json!({
            "schema": "oya-ci/scm-facts/v1",
            "tracked_paths": "out/example.generated.json"
        });

        let findings = evaluate_keyed(&manifest, &scm_facts);

        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_scm_facts_tracked_paths_not_array"
                    && finding.key == "tracked_paths"
            }),
            "non-array tracked_paths must fail closed: {findings:#?}"
        );
    }

    #[test]
    fn malformed_scm_facts_tracked_path_items_are_red() {
        let manifest = manifest(vec![artifact("example-face", "out/example.generated.json")]);
        let scm_facts = json!({
            "schema": "oya-ci/scm-facts/v1",
            "tracked_paths": ["out/example.generated.json", 7, " "]
        });

        let findings = evaluate_keyed(&manifest, &scm_facts);

        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_scm_facts_tracked_path_not_string"
                    && finding.key == "tracked_paths[1]"
            }),
            "non-string tracked path items must fail closed: {findings:#?}"
        );
        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_scm_facts_tracked_path_not_string"
                    && finding.key == "tracked_paths[2]"
            }),
            "blank tracked path items must fail closed: {findings:#?}"
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
    fn malformed_frozen_reference_rows_are_red() {
        let policy = json!({
            "base_ref": "origin/main",
            "frozen_reference": [
                { "out_path": "o/a.json" },
                "not-an-object",
                { "face_path": "b/baseline.generated.json", "out_path": "o/b.json" }
            ]
        });

        let (paths, findings) = frozen_reference_face_paths_keyed(std::iter::once(&policy));

        assert!(paths.contains("b/baseline.generated.json"));
        assert!(
            findings.iter().any(|finding| {
                finding.code
                    == "generated_artifact_ratchet_policy_frozen_reference_face_path_missing"
                    && finding.key == "frozen_reference[0].face_path"
            }),
            "missing frozen_reference face_path must fail closed: {findings:#?}"
        );
        assert!(
            findings.iter().any(|finding| {
                finding.code == "generated_artifact_ratchet_policy_frozen_reference_not_object"
                    && finding.key == "frozen_reference[1]"
            }),
            "non-object frozen_reference rows must fail closed: {findings:#?}"
        );
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
    fn frozen_reference_declaring_regenerate_from_source_may_be_decommitted_adr_0616() {
        // ADR-0616 INVERSION: a frozen reference whose ratchet policy declares
        // `source: regenerate-from-merge-base-source` is REGENERATED from the merge-base source, so
        // it MAY be de-committed (`not-tracked-in-git`) WITHOUT firing the must-stay-committed guard.
        // The emitter never reads a committed blob for it, so #828 stays impossible. Data-driven.
        let mut baseline = artifact(
            "cloud-ci-gate-baseline-ratchet-face",
            "out/gate-baseline.generated.json",
        );
        baseline["materialization_mode"] = json!("not-tracked-in-git");
        baseline["merge_policy"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![baseline]);
        // The frozen reference is de-committed (absent from the tracked tree) — the desired state.
        let scm_facts = scm(&[]);

        // The ratchet policy DECLARES regenerate-from-merge-base-source, so the face_path is EXCLUDED
        // from the committed-git-blob frozen-reference set.
        let policy = json!({
            "base_ref": "origin/dev",
            "frozen_reference": {
                "face_path": "out/gate-baseline.generated.json",
                "out_path": "out/frozen.merge-base.generated.json",
                "source": FROZEN_REFERENCE_SOURCE_REGENERATE
            }
        });
        let frozen = frozen_reference_face_paths(std::iter::once(&policy));
        assert!(
            frozen.is_empty(),
            "a regenerate-from-source frozen reference must be excluded from the committed-blob set"
        );

        let findings = evaluate_keyed_with_frozen_references(&manifest, &scm_facts, &frozen);
        assert!(
            !findings
                .iter()
                .any(|finding| finding.code == "frozen_reference_artifact_must_stay_committed"),
            "de-committing a regenerate-from-source frozen reference must NOT RED (ADR-0616); \
             findings: {findings:#?}"
        );
        assert_eq!(
            evaluate_with_frozen_references(&manifest, &scm_facts, &frozen).verdict,
            Verdict::Green
        );
    }

    #[test]
    fn frozen_reference_without_regenerate_declaration_still_reds_on_decommit() {
        // The INVERSE of the ADR-0616 case: a frozen reference whose policy does NOT declare
        // regenerate-from-source stays a committed-git-blob reference, so de-committing it still
        // RED-blocks (the #828 guard is preserved for un-migrated frozen references).
        let mut baseline = artifact(
            "cloud-ci-gate-baseline-ratchet-face",
            "out/gate-baseline.generated.json",
        );
        baseline["materialization_mode"] = json!("not-tracked-in-git");
        let manifest = manifest(vec![baseline]);
        let scm_facts = scm(&[]);
        // ratchet_policy() declares NO source → committed-git-blob → in the must-stay-committed set.
        let frozen = frozen_set(&["out/gate-baseline.generated.json"]);
        assert!(frozen.contains("out/gate-baseline.generated.json"));

        let findings = evaluate_keyed_with_frozen_references(&manifest, &scm_facts, &frozen);
        assert!(
            findings
                .iter()
                .any(|finding| finding.code == "frozen_reference_artifact_must_stay_committed"),
            "an un-migrated (committed-blob) frozen reference must still RED on de-commit; \
             findings: {findings:#?}"
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
