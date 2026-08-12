//! ci-reorg-target-debt — the no-new-reorg-target-debt gate (Global Binding Rule 1;
//! North-Star Completion Programme bootstrap step T3b).
//!
//! The engine is shape-neutral: every reorg-target form (name prefixes, path prefixes),
//! the baseline location, the scan parameters, the manifest/masterplan paths, the anchor
//! field names, and the reduction-claim schema live in `reorg-target-debt-policy.json`
//! as DATA. This kernel fixes only the evaluation algorithm, so another repository
//! adopts the gate by editing the policy, never by forking logic.
//!
//! Four blocking arms plus one audit mode, all pure functions over injected values:
//! - Arm A ([`evaluate_tree`]): new tracked files under target path prefixes, against a
//!   shrink-only committed baseline (stale rows force regeneration).
//! - Arm B ([`evaluate_workspace_manifest`]): new `[workspace.dependencies]` entries
//!   whose path points into a target prefix or whose name carries a target name prefix.
//! - Arm C ([`evaluate_masterplan`]): work items whose evidence anchors point under a
//!   target path prefix.
//! - Arm D ([`evaluate_reduction_claims`]): net target-surface-reduction claims lacking
//!   the census-bound before/after measurement.
//! - Audit mode ([`audit_interval`]): replays Arms A–C over a captured commit-set for an
//!   explicit range; fails closed on missing/empty/incomplete input and on any finding
//!   without a remediation record. Git I/O stays at the caller boundary.
//!
//! Every report carries the liveness signal (`evaluated_path_count`, `evaluated_arms`):
//! a missing scheduled run is a gap, never a pass.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

pub const GATE_ID: &str = "ci-reorg-target-debt";
/// Repo-relative policy location. `ci/facade/` is the gate fleet's own home (an allowed
/// literal shape for gate crates); every reorg-target-specific string lives in the policy.
pub const POLICY_PATH: &str = "ci/facade/reorg-target-debt/reorg-target-debt-policy.json";
pub const AUDIT_INPUT_SCHEMA: &str = "ci-reorg-target-debt-interval-audit-input.v1";

pub const ARM_A: &str = "arm-a-target-path-files";
pub const ARM_B: &str = "arm-b-workspace-target-deps";
pub const ARM_C: &str = "arm-c-work-item-target-anchors";
pub const ARM_D: &str = "arm-d-unproven-reduction-claims";
pub const ARM_AUDIT: &str = "interval-audit";

pub const CODE_NEW_TARGET_PATH: &str = "RTD_NEW_TARGET_PATH";
pub const CODE_STALE_BASELINE_PATH: &str = "RTD_STALE_BASELINE_PATH";
pub const CODE_NEW_TARGET_PATH_DEP: &str = "RTD_NEW_TARGET_PATH_DEP";
pub const CODE_NEW_TARGET_DEP_NAME: &str = "RTD_NEW_TARGET_DEP_NAME";
pub const CODE_NEW_TARGET_ANCHOR: &str = "RTD_NEW_TARGET_ANCHOR";
pub const CODE_UNPROVEN_REDUCTION_CLAIM: &str = "RTD_UNPROVEN_REDUCTION_CLAIM";
pub const CODE_AUDIT_INPUT_INVALID: &str = "RTD_AUDIT_INPUT_INVALID";
pub const CODE_AUDIT_TARGET_DEBT_COMMIT: &str = "RTD_AUDIT_TARGET_DEBT_COMMIT";
pub const CODE_POLICY_INVALID: &str = "RTD_POLICY_INVALID";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub arm: String,
    pub subject: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, arm: &str, subject: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            arm: arm.to_owned(),
            subject: subject.into(),
            detail: detail.into(),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "code": self.code,
            "arm": self.arm,
            "subject": self.subject,
            "detail": self.detail,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Green => write!(f, "green"),
            Self::Red => write!(f, "red"),
        }
    }
}

/// A gate evaluation result carrying the mandatory liveness signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub evaluated_arms: Vec<String>,
    pub evaluated_path_count: usize,
    pub findings: Vec<Finding>,
}

impl Report {
    pub fn verdict(&self) -> Verdict {
        if self.findings.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "gate_id": GATE_ID,
            "verdict": self.verdict().to_string(),
            "evaluated_arms": self.evaluated_arms,
            "evaluated_path_count": self.evaluated_path_count,
            "findings": self.findings.iter().map(Finding::to_json).collect::<Vec<_>>(),
        })
    }

    /// Merge arm reports into one blocking verdict, summing the liveness signal.
    pub fn merge(reports: Vec<Report>) -> Report {
        let mut evaluated_arms = Vec::new();
        let mut evaluated_path_count = 0;
        let mut findings = Vec::new();
        for report in reports {
            evaluated_arms.extend(report.evaluated_arms);
            evaluated_path_count += report.evaluated_path_count;
            findings.extend(report.findings);
        }
        findings.sort();
        Report {
            evaluated_arms,
            evaluated_path_count,
            findings,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    Io(String),
    Policy(String),
    Input(String),
}

impl fmt::Display for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(f, "io: {message}"),
            Self::Policy(message) => write!(f, "policy: {message}"),
            Self::Input(message) => write!(f, "input: {message}"),
        }
    }
}

impl std::error::Error for GateError {}

/// The parsed, validated policy. Every repo-specific string enters through here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub name_prefixes: Vec<String>,
    pub path_prefixes: Vec<String>,
    pub baseline_file: String,
    pub regeneration_command: String,
    pub exempt_path_prefixes: Vec<String>,
    pub skip_dir_names: BTreeSet<String>,
    pub workspace_manifest_path: String,
    pub masterplan_path: String,
    pub anchor_field_names: BTreeSet<String>,
    pub claim_field: String,
    pub claim_values: BTreeSet<String>,
    pub required_claim_fields: Vec<String>,
}

fn required_str_array(value: &Value, section: &str, key: &str) -> Result<Vec<String>, GateError> {
    let items = value
        .get(section)
        .and_then(|s| s.get(key))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            GateError::Policy(format!("{CODE_POLICY_INVALID}: missing array {section}.{key}"))
        })?;
    items
        .iter()
        .map(|item| {
            item.as_str().map(str::to_owned).ok_or_else(|| {
                GateError::Policy(format!(
                    "{CODE_POLICY_INVALID}: non-string entry in {section}.{key}"
                ))
            })
        })
        .collect()
}

fn required_str(value: &Value, section: &str, key: &str) -> Result<String, GateError> {
    value
        .get(section)
        .and_then(|s| s.get(key))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            GateError::Policy(format!("{CODE_POLICY_INVALID}: missing string {section}.{key}"))
        })
}

impl Policy {
    /// Fail-closed parse: a policy missing any load-bearing field refuses to evaluate.
    pub fn from_value(policy: &Value) -> Result<Self, GateError> {
        if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
            return Err(GateError::Policy(format!(
                "{CODE_POLICY_INVALID}: gate_id must be {GATE_ID}"
            )));
        }
        let name_prefixes = required_str_array(policy, "target_forms", "name_prefixes")?;
        let path_prefixes = required_str_array(policy, "target_forms", "path_prefixes")?;
        if name_prefixes.is_empty() || path_prefixes.is_empty() {
            return Err(GateError::Policy(format!(
                "{CODE_POLICY_INVALID}: target_forms prefixes must be non-empty"
            )));
        }
        for prefix in &path_prefixes {
            if !prefix.ends_with('/') {
                return Err(GateError::Policy(format!(
                    "{CODE_POLICY_INVALID}: path prefix {prefix:?} must end with '/'"
                )));
            }
        }
        Ok(Self {
            name_prefixes,
            path_prefixes,
            baseline_file: required_str(policy, "baseline", "file")?,
            regeneration_command: required_str(policy, "baseline", "regeneration_command")?,
            exempt_path_prefixes: required_str_array(policy, "exemptions", "path_prefixes")?,
            skip_dir_names: required_str_array(policy, "scan", "skip_dir_names")?
                .into_iter()
                .collect(),
            workspace_manifest_path: required_str(policy, "workspace_manifest", "path")?,
            masterplan_path: required_str(policy, "masterplan", "path")?,
            anchor_field_names: required_str_array(policy, "masterplan", "anchor_field_names")?
                .into_iter()
                .collect(),
            claim_field: required_str(policy, "reduction_claims", "claim_field")?,
            claim_values: required_str_array(policy, "reduction_claims", "claim_values")?
                .into_iter()
                .collect(),
            required_claim_fields: required_str_array(policy, "reduction_claims", "required_fields")?,
        })
    }

    pub fn under_target_path_prefix(&self, path: &str) -> bool {
        self.path_prefixes.iter().any(|p| path.starts_with(p))
    }

    pub fn carries_target_name_prefix(&self, name: &str) -> bool {
        self.name_prefixes.iter().any(|p| name.starts_with(p))
    }

    pub fn exempt(&self, path: &str) -> bool {
        self.exempt_path_prefixes.iter().any(|p| path.starts_with(p))
    }
}

/// The committed shrink-only baseline: the migration-inventory estate the gate covers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Baseline {
    pub paths: BTreeSet<String>,
    pub workspace_path_deps: BTreeSet<String>,
    pub dep_names: BTreeSet<String>,
    pub anchors: BTreeSet<String>,
}

fn baseline_set(value: &Value, key: &str) -> Result<BTreeSet<String>, GateError> {
    let items = value.get(key).and_then(Value::as_array).ok_or_else(|| {
        GateError::Policy(format!("{CODE_POLICY_INVALID}: baseline missing array {key}"))
    })?;
    items
        .iter()
        .map(|item| {
            item.as_str().map(str::to_owned).ok_or_else(|| {
                GateError::Policy(format!(
                    "{CODE_POLICY_INVALID}: non-string entry in baseline {key}"
                ))
            })
        })
        .collect()
}

impl Baseline {
    pub fn from_value(value: &Value) -> Result<Self, GateError> {
        Ok(Self {
            paths: baseline_set(value, "arm_a_paths")?,
            workspace_path_deps: baseline_set(value, "arm_b_workspace_path_deps")?,
            dep_names: baseline_set(value, "arm_b_dep_names")?,
            anchors: baseline_set(value, "arm_c_anchors")?,
        })
    }

    pub fn to_json(&self) -> Value {
        json!({
            "_comment": format!(
                "Committed shrink-only baseline for {GATE_ID} (strategy: committed-sorted-path-list). \
                 The target-prefix estate at freeze time is migration inventory; anything NEW fails \
                 closed. Regenerate ONLY alongside an admissible shrink, with the policy-declared \
                 regeneration command."
            ),
            "gate_id": GATE_ID,
            "arm_a_paths": self.paths,
            "arm_b_workspace_path_deps": self.workspace_path_deps,
            "arm_b_dep_names": self.dep_names,
            "arm_c_anchors": self.anchors,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Arm A — new tracked files under target path prefixes (shrink-only baseline).
// ─────────────────────────────────────────────────────────────────────────────

/// Pure Arm A evaluator. `paths` is the full candidate set under test (the caller owns
/// collection); entries outside the target path prefixes are counted for liveness and
/// otherwise ignored. A target-prefix path absent from the baseline is NEW debt; a
/// baseline row absent from `paths` is a stale row that forces regeneration (shrink-only).
pub fn evaluate_tree(policy: &Policy, baseline: &Baseline, paths: &BTreeSet<String>) -> Report {
    let mut findings = Vec::new();
    for path in paths {
        if !policy.under_target_path_prefix(path) || policy.exempt(path) {
            continue;
        }
        if !baseline.paths.contains(path) {
            findings.push(Finding::new(
                CODE_NEW_TARGET_PATH,
                ARM_A,
                path.clone(),
                "new tracked file under a reorg-target path prefix; Global Binding Rule 1 \
                 (no-new-reorg-target-debt) refuses new target-prefix files — home it outside \
                 the target prefixes",
            ));
        }
    }
    for row in &baseline.paths {
        if !paths.contains(row) {
            findings.push(Finding::new(
                CODE_STALE_BASELINE_PATH,
                ARM_A,
                row.clone(),
                format!(
                    "baseline row no longer exists in the tree; the removal itself is always \
                     admissible (shrink-only), but it requires baseline regeneration in the same \
                     change so burned-down debt cannot regain headroom. Regenerate with: {}",
                    policy.regeneration_command
                ),
            ));
        }
    }
    findings.sort();
    Report {
        evaluated_arms: vec![ARM_A.to_owned()],
        evaluated_path_count: paths.len(),
        findings,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Arm B — new workspace dependencies into / named under target forms.
// ─────────────────────────────────────────────────────────────────────────────

/// One `[workspace.dependencies]` entry: the dependency name and, when declared, its
/// `path` value (empty string when the entry has no path key).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkspaceDep {
    pub name: String,
    pub path: String,
}

fn toml_str_value(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_end_matches(',').trim();
    trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .map(str::to_owned)
}

fn inline_table_path(value: &str) -> Option<String> {
    let (_, tail) = value.split_once("path")?;
    let tail = tail.trim_start();
    let tail = tail.strip_prefix('=')?;
    let tail = tail.trim_start();
    let tail = tail.strip_prefix('"')?;
    tail.split('"').next().map(str::to_owned)
}

/// Deliberately a line parser, not a TOML dependency: the workspace table is a flat
/// declaration surface and this gate must not acquire a parser of its own to drift.
/// Handles both `name = { path = "…" }` inline entries and
/// `[workspace.dependencies.name]` subsections with a `path = "…"` line.
pub fn parse_workspace_dependencies(manifest: &str, section: &str) -> Vec<WorkspaceDep> {
    let mut deps: Vec<WorkspaceDep> = Vec::new();
    let section_header = format!("[{section}]");
    let subsection_prefix = format!("[{section}.");
    let mut in_table = false;
    let mut current_subsection: Option<usize> = None;

    for line in manifest.lines() {
        let bare = line.split('#').next().unwrap_or("").trim();
        if bare.is_empty() {
            continue;
        }
        if bare.starts_with('[') {
            if bare == section_header {
                in_table = true;
                current_subsection = None;
            } else if let Some(rest) = bare.strip_prefix(subsection_prefix.as_str()) {
                let name = rest.trim_end_matches(']').trim().to_owned();
                deps.push(WorkspaceDep {
                    name,
                    path: String::new(),
                });
                in_table = false;
                current_subsection = Some(deps.len() - 1);
            } else {
                in_table = false;
                current_subsection = None;
            }
            continue;
        }
        if let Some(index) = current_subsection {
            if let Some(value) = bare.strip_prefix("path") {
                let value = value.trim_start();
                if let Some(value) = value.strip_prefix('=')
                    && let Some(path) = toml_str_value(value)
                {
                    deps[index].path = path;
                }
            }
            continue;
        }
        if !in_table {
            continue;
        }
        let Some((name, value)) = bare.split_once('=') else {
            continue;
        };
        let name = name.trim().trim_matches('"').to_owned();
        if name.is_empty() {
            continue;
        }
        let path = inline_table_path(value).unwrap_or_default();
        deps.push(WorkspaceDep { name, path });
    }
    deps
}

/// Pure Arm B evaluator over the parsed workspace-dependency entries.
pub fn evaluate_workspace_deps(
    policy: &Policy,
    baseline: &Baseline,
    deps: &[WorkspaceDep],
) -> Report {
    let mut findings = Vec::new();
    for dep in deps {
        if !dep.path.is_empty()
            && policy.under_target_path_prefix(&dep.path)
            && !baseline.workspace_path_deps.contains(&dep.path)
        {
            findings.push(Finding::new(
                CODE_NEW_TARGET_PATH_DEP,
                ARM_B,
                format!("{} -> {}", dep.name, dep.path),
                "new workspace path dependency into a reorg-target path prefix; Global Binding \
                 Rule 1 refuses new dependency edges into the target estate",
            ));
        }
        if policy.carries_target_name_prefix(&dep.name) && !baseline.dep_names.contains(&dep.name) {
            findings.push(Finding::new(
                CODE_NEW_TARGET_DEP_NAME,
                ARM_B,
                dep.name.clone(),
                "new workspace dependency named under a reorg-target name prefix; Global Binding \
                 Rule 1 refuses minting new target-form names",
            ));
        }
    }
    findings.sort();
    Report {
        evaluated_arms: vec![ARM_B.to_owned()],
        evaluated_path_count: deps.len(),
        findings,
    }
}

/// Convenience Arm B entry point over raw manifest text.
pub fn evaluate_workspace_manifest(
    policy: &Policy,
    baseline: &Baseline,
    manifest: &str,
    section: &str,
) -> Report {
    evaluate_workspace_deps(policy, baseline, &parse_workspace_dependencies(manifest, section))
}

// ─────────────────────────────────────────────────────────────────────────────
// Arm C — work items with target-path evidence anchors.
// ─────────────────────────────────────────────────────────────────────────────

fn collect_anchor_strings(
    value: &Value,
    field_names: &BTreeSet<String>,
    location: &str,
    out: &mut Vec<(String, String)>,
) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_location = format!("{location}/{key}");
                if field_names.contains(key) {
                    match child {
                        Value::String(anchor) => {
                            out.push((child_location.clone(), anchor.clone()));
                        }
                        Value::Array(items) => {
                            for (index, item) in items.iter().enumerate() {
                                if let Value::String(anchor) = item {
                                    out.push((
                                        format!("{child_location}/{index}"),
                                        anchor.clone(),
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                collect_anchor_strings(child, field_names, &child_location, out);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_anchor_strings(item, field_names, &format!("{location}/{index}"), out);
            }
        }
        _ => {}
    }
}

/// Pure Arm C evaluator over a parsed planning value. Any string found under a
/// policy-declared anchor field that begins with a target path prefix and is not carried
/// by the shrink-only baseline is NEW target-anchored evidence and fails closed.
pub fn evaluate_masterplan(policy: &Policy, baseline: &Baseline, plan: &Value) -> Report {
    let mut anchors = Vec::new();
    collect_anchor_strings(plan, &policy.anchor_field_names, "", &mut anchors);
    let mut findings = Vec::new();
    for (location, anchor) in &anchors {
        if policy.under_target_path_prefix(anchor)
            && !policy.exempt(anchor)
            && !baseline.anchors.contains(anchor)
        {
            findings.push(Finding::new(
                CODE_NEW_TARGET_ANCHOR,
                ARM_C,
                format!("{anchor} (at {location})"),
                "work-item evidence anchor points under a reorg-target path prefix; Global \
                 Binding Rule 1 refuses new target-anchored work items — anchor evidence at the \
                 artifact's post-migration home",
            ));
        }
    }
    findings.sort();
    Report {
        evaluated_arms: vec![ARM_C.to_owned()],
        evaluated_path_count: anchors.len(),
        findings,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Arm D — unproven net-debt-reduction claims.
// ─────────────────────────────────────────────────────────────────────────────

fn collect_claim_objects<'v>(
    value: &'v Value,
    claim_field: &str,
    claim_values: &BTreeSet<String>,
    location: &str,
    out: &mut Vec<(String, &'v Map<String, Value>)>,
) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(claim)) = map.get(claim_field)
                && claim_values.contains(claim)
            {
                out.push((location.to_owned(), map));
            }
            for (key, child) in map {
                collect_claim_objects(child, claim_field, claim_values, &format!("{location}/{key}"), out);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_claim_objects(item, claim_field, claim_values, &format!("{location}/{index}"), out);
            }
        }
        _ => {}
    }
}

fn claim_field_proven(map: &Map<String, Value>, field: &str) -> bool {
    match map.get(field) {
        Some(Value::Number(_)) => true,
        Some(Value::String(text)) => !text.trim().is_empty(),
        _ => false,
    }
}

/// Pure Arm D evaluator. A net target-surface-reduction claim is refused unless every
/// policy-declared companion field is present and non-empty: the counts are DATA carried
/// by the artifact under test, bound to a census snapshot ref — the gate compares nothing
/// numeric and mints no threshold.
pub fn evaluate_reduction_claims(policy: &Policy, artifact: &Value) -> Report {
    let mut claims = Vec::new();
    collect_claim_objects(artifact, &policy.claim_field, &policy.claim_values, "", &mut claims);
    let mut findings = Vec::new();
    for (location, map) in &claims {
        let missing: Vec<&String> = policy
            .required_claim_fields
            .iter()
            .filter(|field| !claim_field_proven(map, field))
            .collect();
        if !missing.is_empty() {
            findings.push(Finding::new(
                CODE_UNPROVEN_REDUCTION_CLAIM,
                ARM_D,
                format!("claim at {location}"),
                format!(
                    "net target-surface-reduction claim without its bound measurement; missing \
                     or empty required field(s): {missing:?}. A reduction claim must carry the \
                     before/after counts and the census snapshot ref they were measured against"
                ),
            ));
        }
    }
    findings.sort();
    Report {
        evaluated_arms: vec![ARM_D.to_owned()],
        evaluated_path_count: claims.len(),
        findings,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Audit mode — deterministic interval audit over a captured commit-set.
// ─────────────────────────────────────────────────────────────────────────────

/// A machine-readable interval-audit report, suitable for committing as a durable
/// evidence record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditReport {
    pub range_from: String,
    pub range_to: String,
    pub evaluated_commit_count: usize,
    pub evaluated_path_count: usize,
    pub findings: Vec<Finding>,
    pub remediated_commits: BTreeSet<String>,
    pub unremediated_finding_count: usize,
}

impl AuditReport {
    pub fn verdict(&self) -> Verdict {
        if self.unremediated_finding_count == 0 {
            Verdict::Green
        } else {
            Verdict::Red
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "gate_id": GATE_ID,
            "mode": ARM_AUDIT,
            "range": { "from": self.range_from, "to": self.range_to },
            "verdict": self.verdict().to_string(),
            "evaluated_arms": [ARM_A, ARM_B, ARM_C],
            "evaluated_commit_count": self.evaluated_commit_count,
            "evaluated_path_count": self.evaluated_path_count,
            "findings": self.findings.iter().map(Finding::to_json).collect::<Vec<_>>(),
            "remediated_commits": self.remediated_commits,
            "unremediated_finding_count": self.unremediated_finding_count,
        })
    }
}

fn audit_invalid(detail: impl Into<String>) -> GateError {
    GateError::Input(format!("{CODE_AUDIT_INPUT_INVALID}: {}", detail.into()))
}

fn commit_str_array(commit: &Value, sha: &str, key: &str) -> Result<Vec<String>, GateError> {
    let items = commit.get(key).and_then(Value::as_array).ok_or_else(|| {
        audit_invalid(format!(
            "commit {sha} is missing fact array {key:?}; an incomplete capture never audits green"
        ))
    })?;
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| audit_invalid(format!("commit {sha}: non-string entry in {key:?}")))
        })
        .collect()
}

/// Deterministic interval audit over a captured commit-set (the caller owns git I/O).
///
/// FAIL-CLOSED contract: a wrong schema, missing/empty range, `complete != true`, an
/// empty commit list, or a commit missing any of the four fact arrays is an error the
/// binary surfaces as a non-zero `RTD_AUDIT_INPUT_INVALID` verdict — never a pass. Every
/// commit introducing a file/dep/anchor under a target form relative to the range base is
/// reported; the report stays red until each finding's commit carries a remediation
/// record with a non-empty resolution.
pub fn audit_interval(policy: &Policy, input: &Value) -> Result<AuditReport, GateError> {
    if input.get("schema").and_then(Value::as_str) != Some(AUDIT_INPUT_SCHEMA) {
        return Err(audit_invalid(format!(
            "input schema must be {AUDIT_INPUT_SCHEMA:?}"
        )));
    }
    let range_from = input
        .pointer("/range/from")
        .and_then(Value::as_str)
        .filter(|sha| !sha.trim().is_empty())
        .ok_or_else(|| audit_invalid("range.from must be a non-empty commit ref"))?
        .to_owned();
    let range_to = input
        .pointer("/range/to")
        .and_then(Value::as_str)
        .filter(|sha| !sha.trim().is_empty())
        .ok_or_else(|| audit_invalid("range.to must be a non-empty commit ref"))?
        .to_owned();
    if input.get("complete").and_then(Value::as_bool) != Some(true) {
        return Err(audit_invalid(
            "complete must be true; a partial or unresolved range never audits green",
        ));
    }
    let commits = input
        .get("commits")
        .and_then(Value::as_array)
        .ok_or_else(|| audit_invalid("commits must be an array"))?;
    if commits.is_empty() {
        return Err(audit_invalid(
            "commits is empty; an empty or unresolvable range is a finding, not a pass",
        ));
    }

    let mut remediated_commits = BTreeSet::new();
    let no_records = Vec::new();
    for record in input
        .get("remediation_records")
        .and_then(Value::as_array)
        .unwrap_or(&no_records)
    {
        let sha = record.get("commit").and_then(Value::as_str).unwrap_or("");
        let resolution = record.get("resolution").and_then(Value::as_str).unwrap_or("");
        if !sha.is_empty() && !resolution.trim().is_empty() {
            remediated_commits.insert(sha.to_owned());
        }
    }

    let mut findings = Vec::new();
    let mut evaluated_path_count = 0;
    for commit in commits {
        let sha = commit
            .get("sha")
            .and_then(Value::as_str)
            .filter(|sha| !sha.trim().is_empty())
            .ok_or_else(|| audit_invalid("every commit entry must carry a non-empty sha"))?;
        let added_paths = commit_str_array(commit, sha, "added_paths")?;
        let added_dep_names = commit_str_array(commit, sha, "added_dep_names")?;
        let added_anchors = commit_str_array(commit, sha, "added_evidence_anchors")?;
        let added_path_deps = commit
            .get("added_workspace_path_deps")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                audit_invalid(format!(
                    "commit {sha} is missing fact array \"added_workspace_path_deps\"; an \
                     incomplete capture never audits green"
                ))
            })?;

        evaluated_path_count += added_paths.len() + added_anchors.len();
        for path in &added_paths {
            if policy.under_target_path_prefix(path) && !policy.exempt(path) {
                findings.push(Finding::new(
                    CODE_AUDIT_TARGET_DEBT_COMMIT,
                    ARM_A,
                    format!("{sha}: {path}"),
                    "commit introduced a tracked file under a reorg-target path prefix within \
                     the audited range",
                ));
            }
        }
        for dep in added_path_deps {
            let name = dep.get("name").and_then(Value::as_str).unwrap_or("");
            let path = dep.get("path").and_then(Value::as_str).unwrap_or("");
            if !path.is_empty() && policy.under_target_path_prefix(path) {
                findings.push(Finding::new(
                    CODE_AUDIT_TARGET_DEBT_COMMIT,
                    ARM_B,
                    format!("{sha}: {name} -> {path}"),
                    "commit introduced a workspace path dependency into a reorg-target prefix \
                     within the audited range",
                ));
            }
        }
        for name in &added_dep_names {
            if policy.carries_target_name_prefix(name) {
                findings.push(Finding::new(
                    CODE_AUDIT_TARGET_DEBT_COMMIT,
                    ARM_B,
                    format!("{sha}: {name}"),
                    "commit introduced a workspace dependency named under a reorg-target name \
                     prefix within the audited range",
                ));
            }
        }
        for anchor in &added_anchors {
            if policy.under_target_path_prefix(anchor) && !policy.exempt(anchor) {
                findings.push(Finding::new(
                    CODE_AUDIT_TARGET_DEBT_COMMIT,
                    ARM_C,
                    format!("{sha}: {anchor}"),
                    "commit introduced a work-item evidence anchor under a reorg-target path \
                     prefix within the audited range",
                ));
            }
        }
    }
    findings.sort();
    let unremediated_finding_count = findings
        .iter()
        .filter(|finding| {
            let sha = finding.subject.split(':').next().unwrap_or("");
            !remediated_commits.contains(sha)
        })
        .count();
    Ok(AuditReport {
        range_from,
        range_to,
        evaluated_commit_count: commits.len(),
        evaluated_path_count,
        findings,
        remediated_commits,
        unremediated_finding_count,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Read-only collectors (I/O at the edge; the evaluators above stay pure).
// ─────────────────────────────────────────────────────────────────────────────

pub fn load_json(path: &Path) -> Result<Value, GateError> {
    let text = fs::read_to_string(path)
        .map_err(|error| GateError::Io(format!("read {}: {error}", path.display())))?;
    serde_json::from_str(&text)
        .map_err(|error| GateError::Io(format!("parse {}: {error}", path.display())))
}

pub fn load_policy(repo_root: &Path, policy_rel: &str) -> Result<(Policy, Value), GateError> {
    let value = load_json(&repo_root.join(policy_rel))?;
    Ok((Policy::from_value(&value)?, value))
}

pub fn load_baseline(repo_root: &Path, policy: &Policy) -> Result<Baseline, GateError> {
    Baseline::from_value(&load_json(&repo_root.join(&policy.baseline_file))?)
}

fn walk_files(
    root: &Path,
    repo_root: &Path,
    skip: &BTreeSet<String>,
    out: &mut BTreeSet<String>,
) -> Result<(), GateError> {
    let entries = fs::read_dir(root)
        .map_err(|error| GateError::Io(format!("read_dir {}: {error}", root.display())))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| GateError::Io(format!("dir entry {}: {error}", root.display())))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if !skip.contains(&name) {
                walk_files(&path, repo_root, skip, out)?;
            }
        } else {
            let rel = path
                .strip_prefix(repo_root)
                .map_err(|error| GateError::Io(format!("relativize {}: {error}", path.display())))?;
            let mut rel_text = String::new();
            for component in rel.components() {
                if !rel_text.is_empty() {
                    rel_text.push('/');
                }
                rel_text.push_str(&component.as_os_str().to_string_lossy());
            }
            out.insert(rel_text);
        }
    }
    Ok(())
}

/// Collect every file under the policy's target path prefixes via a deterministic
/// read-only walk (skip dirs are policy DATA). On a clean checkout this equals the
/// tracked set; untracked residue under a target prefix fails closed by design.
pub fn collect_target_prefix_paths(
    repo_root: &Path,
    policy: &Policy,
) -> Result<BTreeSet<String>, GateError> {
    let mut out = BTreeSet::new();
    for prefix in &policy.path_prefixes {
        let root: PathBuf = repo_root.join(prefix.trim_end_matches('/'));
        if root.is_dir() {
            walk_files(&root, repo_root, &policy.skip_dir_names, &mut out)?;
        }
    }
    Ok(out)
}

/// Snapshot the current tree into a fresh baseline value (the `--regen-baseline` surface).
pub fn regenerate_baseline(repo_root: &Path, policy: &Policy) -> Result<Baseline, GateError> {
    let paths = collect_target_prefix_paths(repo_root, policy)?;
    let manifest = fs::read_to_string(repo_root.join(&policy.workspace_manifest_path))
        .map_err(|error| {
            GateError::Io(format!("read {}: {error}", policy.workspace_manifest_path))
        })?;
    let deps = parse_workspace_dependencies(&manifest, "workspace.dependencies");
    let mut workspace_path_deps = BTreeSet::new();
    let mut dep_names = BTreeSet::new();
    for dep in deps {
        if !dep.path.is_empty() && policy.under_target_path_prefix(&dep.path) {
            workspace_path_deps.insert(dep.path);
        }
        if policy.carries_target_name_prefix(&dep.name) {
            dep_names.insert(dep.name);
        }
    }
    let plan = load_json(&repo_root.join(&policy.masterplan_path))?;
    let mut anchor_hits = Vec::new();
    collect_anchor_strings(&plan, &policy.anchor_field_names, "", &mut anchor_hits);
    let anchors = anchor_hits
        .into_iter()
        .filter(|(_, anchor)| policy.under_target_path_prefix(anchor) && !policy.exempt(anchor))
        .map(|(_, anchor)| anchor)
        .collect();
    Ok(Baseline {
        paths,
        workspace_path_deps,
        dep_names,
        anchors,
    })
}

/// Run all four blocking arms over the live tree and merge into one verdict.
pub fn check_live_tree(repo_root: &Path, policy: &Policy, baseline: &Baseline) -> Result<Report, GateError> {
    let paths = collect_target_prefix_paths(repo_root, policy)?;
    let manifest = fs::read_to_string(repo_root.join(&policy.workspace_manifest_path))
        .map_err(|error| {
            GateError::Io(format!("read {}: {error}", policy.workspace_manifest_path))
        })?;
    let plan = load_json(&repo_root.join(&policy.masterplan_path))?;
    Ok(Report::merge(vec![
        evaluate_tree(policy, baseline, &paths),
        evaluate_workspace_manifest(policy, baseline, &manifest, "workspace.dependencies"),
        evaluate_masterplan(policy, baseline, &plan),
        evaluate_reduction_claims(policy, &plan),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_policy_value() -> Value {
        json!({
            "gate_id": GATE_ID,
            "target_forms": {
                "name_prefixes": ["cloud-", "cloud_", "oya-", "oya_"],
                "path_prefixes": ["oya/", "cloud/"]
            },
            "baseline": {
                "file": "ci/facade/reorg-target-debt/reorg-target-debt-baseline.json",
                "regeneration_command": "buck2 run //ci/facade/reorg-target-debt:ci-reorg-target-debt-bin -- --regen-baseline"
            },
            "exemptions": { "path_prefixes": [] },
            "scan": { "skip_dir_names": [".git", "buck-out", "target"] },
            "workspace_manifest": { "path": "Cargo.toml", "section": "workspace.dependencies" },
            "masterplan": {
                "path": "specs/masterplan.json",
                "anchor_field_names": ["source_anchors", "evidence_anchor", "anchor"]
            },
            "reduction_claims": {
                "claim_field": "claim",
                "claim_values": ["net_target_surface_reduction"],
                "required_fields": ["before_count", "after_count", "census_snapshot_ref"]
            }
        })
    }

    fn test_policy() -> Policy {
        Policy::from_value(&test_policy_value()).unwrap()
    }

    fn baseline_with_paths(paths: &[&str]) -> Baseline {
        Baseline {
            paths: paths.iter().map(|p| (*p).to_owned()).collect(),
            ..Baseline::default()
        }
    }

    fn codes(report: &Report) -> Vec<&str> {
        report.findings.iter().map(|f| f.code.as_str()).collect()
    }

    #[test]
    fn policy_missing_field_fails_closed() {
        let mut value = test_policy_value();
        value["target_forms"]["path_prefixes"] = json!([]);
        assert!(Policy::from_value(&value).is_err());
        let mut value = test_policy_value();
        value.as_object_mut().unwrap().remove("reduction_claims");
        assert!(Policy::from_value(&value).is_err());
        let mut value = test_policy_value();
        value["target_forms"]["path_prefixes"] = json!(["oya"]);
        assert!(
            Policy::from_value(&value).is_err(),
            "a path prefix without a trailing slash would match sibling names"
        );
    }

    #[test]
    fn arm_a_new_target_path_is_refused_and_baseline_rows_pass() {
        let policy = test_policy();
        let baseline = baseline_with_paths(&["cloud/legacy/kernel.rs"]);
        let paths: BTreeSet<String> = ["cloud/legacy/kernel.rs", "docs/fine.md"]
            .iter()
            .map(|p| (*p).to_owned())
            .collect();
        let report = evaluate_tree(&policy, &baseline, &paths);
        assert_eq!(report.verdict(), Verdict::Green);
        assert_eq!(report.evaluated_path_count, 2);
        assert_eq!(report.evaluated_arms, vec![ARM_A.to_owned()]);

        let mut with_new = paths.clone();
        with_new.insert("oya/newly-minted/mod.rs".to_owned());
        let report = evaluate_tree(&policy, &baseline, &with_new);
        assert_eq!(codes(&report), vec![CODE_NEW_TARGET_PATH]);
        assert!(report.findings[0].subject.contains("newly-minted"));
    }

    #[test]
    fn arm_a_stale_baseline_row_forces_regeneration_with_exact_command() {
        let policy = test_policy();
        let baseline = baseline_with_paths(&["cloud/legacy/kernel.rs", "cloud/legacy/gone.rs"]);
        let paths: BTreeSet<String> = ["cloud/legacy/kernel.rs"]
            .iter()
            .map(|p| (*p).to_owned())
            .collect();
        let report = evaluate_tree(&policy, &baseline, &paths);
        assert_eq!(codes(&report), vec![CODE_STALE_BASELINE_PATH]);
        assert!(
            report.findings[0].detail.contains("--regen-baseline"),
            "the failure message must emit the exact regeneration command"
        );
    }

    #[test]
    fn arm_b_parses_inline_and_subsection_forms_and_refuses_new_target_deps() {
        let policy = test_policy();
        let manifest = concat!(
            "[workspace]\nmembers = []\n\n",
            "[workspace.dependencies]\n",
            "serde = { version = \"1\" }\n",
            "storage-legacy = { path = \"cloud/legacy-storage\" }\n",
            "oya-shiny-new-kernel = { version = \"1\" }\n\n",
            "[workspace.dependencies.subsection-dep]\n",
            "path = \"oya/legacy-subsection\"\n\n",
            "[profile.release]\nlto = true\n",
        );
        let deps = parse_workspace_dependencies(manifest, "workspace.dependencies");
        assert_eq!(deps.len(), 4);
        let report = evaluate_workspace_deps(&policy, &Baseline::default(), &deps);
        assert_eq!(
            codes(&report),
            vec![
                CODE_NEW_TARGET_DEP_NAME,
                CODE_NEW_TARGET_PATH_DEP,
                CODE_NEW_TARGET_PATH_DEP,
            ]
        );
        assert_eq!(report.evaluated_path_count, 4);

        let baseline = Baseline {
            workspace_path_deps: ["cloud/legacy-storage", "oya/legacy-subsection"]
                .iter()
                .map(|p| (*p).to_owned())
                .collect(),
            dep_names: ["oya-shiny-new-kernel"].iter().map(|p| (*p).to_owned()).collect(),
            ..Baseline::default()
        };
        let report = evaluate_workspace_deps(&policy, &baseline, &deps);
        assert_eq!(report.verdict(), Verdict::Green, "baselined entries are shrink-only inventory");
    }

    #[test]
    fn arm_c_new_target_anchor_is_refused_and_baselined_anchor_passes() {
        let policy = test_policy();
        let plan = json!({
            "work_items": [
                { "id": "SYN-RTD-001", "source_anchors": ["oya/synthetic-legacy-module"] },
                { "id": "SYN-RTD-002", "evidence_anchor": "specs/fine.json" }
            ]
        });
        let report = evaluate_masterplan(&policy, &Baseline::default(), &plan);
        assert_eq!(codes(&report), vec![CODE_NEW_TARGET_ANCHOR]);
        assert_eq!(report.evaluated_path_count, 2);

        let baseline = Baseline {
            anchors: ["oya/synthetic-legacy-module"].iter().map(|p| (*p).to_owned()).collect(),
            ..Baseline::default()
        };
        let report = evaluate_masterplan(&policy, &baseline, &plan);
        assert_eq!(report.verdict(), Verdict::Green);
    }

    #[test]
    fn arm_d_unproven_claim_refused_and_fully_bound_claim_passes() {
        let policy = test_policy();
        let unproven = json!({
            "claims": [{ "claim": "net_target_surface_reduction", "note": "trust me" }]
        });
        let report = evaluate_reduction_claims(&policy, &unproven);
        assert_eq!(codes(&report), vec![CODE_UNPROVEN_REDUCTION_CLAIM]);

        let proven = json!({
            "claims": [{
                "claim": "net_target_surface_reduction",
                "before_count": 4981,
                "after_count": 4600,
                "census_snapshot_ref": "reorg-census@deadbeef"
            }]
        });
        let report = evaluate_reduction_claims(&policy, &proven);
        assert_eq!(report.verdict(), Verdict::Green);
        assert_eq!(report.evaluated_path_count, 1);

        let empty_ref = json!({
            "claims": [{
                "claim": "net_target_surface_reduction",
                "before_count": 4981,
                "after_count": 4600,
                "census_snapshot_ref": "  "
            }]
        });
        let report = evaluate_reduction_claims(&policy, &empty_ref);
        assert_eq!(codes(&report), vec![CODE_UNPROVEN_REDUCTION_CLAIM]);
    }

    fn audit_input(commits: Value, remediation: Value) -> Value {
        json!({
            "schema": AUDIT_INPUT_SCHEMA,
            "range": { "from": "a1a1a1", "to": "b2b2b2" },
            "complete": true,
            "commits": commits,
            "remediation_records": remediation,
        })
    }

    fn clean_commit(sha: &str) -> Value {
        json!({
            "sha": sha,
            "added_paths": ["docs/fine.md"],
            "added_workspace_path_deps": [],
            "added_dep_names": [],
            "added_evidence_anchors": [],
        })
    }

    #[test]
    fn audit_reports_exactly_the_planted_commit_and_stays_red_until_remediated() {
        let policy = test_policy();
        let planted = json!({
            "sha": "b2b2b2",
            "added_paths": ["cloud/planted-debt/src/lib.rs"],
            "added_workspace_path_deps": [],
            "added_dep_names": [],
            "added_evidence_anchors": [],
        });
        let input = audit_input(json!([clean_commit("a1a1a1"), planted]), json!([]));
        let report = audit_interval(&policy, &input).unwrap();
        assert_eq!(report.verdict(), Verdict::Red);
        assert_eq!(report.findings.len(), 1);
        assert!(report.findings[0].subject.starts_with("b2b2b2:"));
        assert_eq!(report.evaluated_commit_count, 2);

        let remediated = audit_input(
            input["commits"].clone(),
            json!([{ "commit": "b2b2b2", "resolution": "reverted in c3c3c3; census re-measured" }]),
        );
        let report = audit_interval(&policy, &remediated).unwrap();
        assert_eq!(report.verdict(), Verdict::Green, "a remediation record resolves the finding");
        assert_eq!(report.findings.len(), 1, "the finding stays reported as evidence");
    }

    #[test]
    fn audit_fails_closed_on_empty_incomplete_or_malformed_input() {
        let policy = test_policy();
        // Empty commit list.
        let empty = audit_input(json!([]), json!([]));
        assert!(audit_interval(&policy, &empty).is_err());
        // complete != true.
        let mut incomplete = audit_input(json!([clean_commit("a1a1a1")]), json!([]));
        incomplete["complete"] = json!(false);
        assert!(audit_interval(&policy, &incomplete).is_err());
        // Missing fact array on a commit.
        let mut missing = audit_input(json!([clean_commit("a1a1a1")]), json!([]));
        missing["commits"][0].as_object_mut().unwrap().remove("added_paths");
        assert!(audit_interval(&policy, &missing).is_err());
        // Wrong schema.
        let mut wrong = audit_input(json!([clean_commit("a1a1a1")]), json!([]));
        wrong["schema"] = json!("something-else.v1");
        assert!(audit_interval(&policy, &wrong).is_err());
        // Missing range.
        let mut rangeless = audit_input(json!([clean_commit("a1a1a1")]), json!([]));
        rangeless["range"]["to"] = json!("");
        assert!(audit_interval(&policy, &rangeless).is_err());
        // An empty-resolution remediation record does not resolve anything.
        let planted = json!({
            "sha": "b2b2b2",
            "added_paths": ["oya/planted"],
            "added_workspace_path_deps": [],
            "added_dep_names": [],
            "added_evidence_anchors": [],
        });
        let hollow = audit_input(
            json!([planted]),
            json!([{ "commit": "b2b2b2", "resolution": "  " }]),
        );
        let report = audit_interval(&policy, &hollow).unwrap();
        assert_eq!(report.verdict(), Verdict::Red);
    }

    #[test]
    fn merged_report_carries_the_liveness_signal() {
        let policy = test_policy();
        let paths: BTreeSet<String> = ["docs/fine.md"].iter().map(|p| (*p).to_owned()).collect();
        let merged = Report::merge(vec![
            evaluate_tree(&policy, &Baseline::default(), &paths),
            evaluate_reduction_claims(&policy, &json!({})),
        ]);
        let rendered = merged.to_json();
        assert_eq!(rendered["evaluated_path_count"], json!(1));
        assert_eq!(rendered["evaluated_arms"], json!([ARM_A, ARM_D]));
        assert_eq!(rendered["verdict"], json!("green"));
    }
}
