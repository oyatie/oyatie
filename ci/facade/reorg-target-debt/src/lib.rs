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
//!   shrink-only committed baseline of per-path sha256 digests (stale digests force
//!   regeneration). The baseline commits DIGESTS, never literal path strings, so the
//!   committed file carries none of the migration-inventory vocabulary the brand-residue
//!   ratchet refuses in new files; exact set semantics are preserved and NEW debt is
//!   still reported by its literal live path.
//! - Arm B ([`evaluate_workspace_manifest`], [`evaluate_name_surface`]): new dependency
//!   edges into a target prefix and new target-form Cargo package/dependency, Buck target,
//!   or Rust module declarations, keyed by declaration identity.
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
use sha2::{Digest, Sha256};
use toml::Value as TomlValue;

pub const GATE_ID: &str = "ci-reorg-target-debt";
/// Repo-relative policy location. `ci/facade/` is the gate fleet's own home (an allowed
/// literal shape for gate crates); every reorg-target-specific string lives in the policy.
pub const POLICY_PATH: &str = "ci/facade/reorg-target-debt/reorg-target-debt-policy.json";
const PROTECTED_BASE_REF: &str = "origin/dev";
const FROZEN_REFERENCE_PATH: &str =
    "ci/facade/reorg-target-debt/reorg-target-debt-merge-base.generated.json";
/// The T2 protected-base commit from which this gate is first adopted. The policy and
/// baseline do not exist at this exact merge base, so this one bootstrap transition freezes
/// the live estate. Every later change has a protected merge-base baseline and is subject to
/// the immutable ceiling.
const INITIAL_ADOPTION_BASE_SHA: &str = "fecc126ebe7ded4949c8ac26b59b8a1e6bcb371c";
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
pub const CODE_NEW_TARGET_NAME: &str = "RTD_NEW_TARGET_NAME";
pub const CODE_DEP_PATH_UNPARSEABLE: &str = "RTD_DEP_PATH_UNPARSEABLE";
pub const CODE_NAME_SURFACE_UNPARSEABLE: &str = "RTD_NAME_SURFACE_UNPARSEABLE";
pub const CODE_BASELINE_EXPANSION: &str = "RTD_BASELINE_EXPANSION";
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

    /// Merge arm reports into one blocking verdict, summing the liveness signal. Arm
    /// ids are deduplicated: several inputs may contribute to the same arm (e.g. the
    /// root workspace table and the member-manifest scan both feed Arm B).
    pub fn merge(reports: Vec<Report>) -> Report {
        let mut evaluated_arms: Vec<String> = Vec::new();
        let mut evaluated_path_count = 0;
        let mut findings = Vec::new();
        for report in reports {
            for arm in report.evaluated_arms {
                if !evaluated_arms.contains(&arm) {
                    evaluated_arms.push(arm);
                }
            }
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
    pub workspace_section: String,
    pub cargo_manifest_file_name: String,
    pub buck_file_names: BTreeSet<String>,
    pub member_dependency_sections: Vec<String>,
    pub masterplan_path: String,
    pub anchor_field_names: BTreeSet<String>,
    pub claim_field: String,
    pub claim_values: BTreeSet<String>,
    pub required_claim_fields: Vec<String>,
    pub before_count_field: String,
    pub after_count_field: String,
    pub census_snapshot_ref_field: String,
    pub census_snapshot_schema: String,
    pub census_snapshot_claim_field: String,
    pub claim_scan_roots: Vec<String>,
}

fn required_str_array(value: &Value, section: &str, key: &str) -> Result<Vec<String>, GateError> {
    let items = value
        .get(section)
        .and_then(|s| s.get(key))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            GateError::Policy(format!(
                "{CODE_POLICY_INVALID}: missing array {section}.{key}"
            ))
        })?;
    items
        .iter()
        .map(|item| {
            item.as_str()
                .filter(|entry| !entry.trim().is_empty())
                .map(str::to_owned)
                .ok_or_else(|| {
                    GateError::Policy(format!(
                        "{CODE_POLICY_INVALID}: non-string or empty entry in {section}.{key}"
                    ))
                })
        })
        .collect()
}

/// Load-bearing collections must not be empty: an empty list would silently disable an
/// arm while the report still lists it as evaluated (a false-green required check).
fn non_empty(list: Vec<String>, section: &str, key: &str) -> Result<Vec<String>, GateError> {
    if list.is_empty() {
        return Err(GateError::Policy(format!(
            "{CODE_POLICY_INVALID}: {section}.{key} must be a non-empty array — an empty \
             load-bearing collection silently disables an arm"
        )));
    }
    Ok(list)
}

fn required_str(value: &Value, section: &str, key: &str) -> Result<String, GateError> {
    value
        .get(section)
        .and_then(|s| s.get(key))
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            GateError::Policy(format!(
                "{CODE_POLICY_INVALID}: missing or empty string {section}.{key}"
            ))
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
        let required_claim_fields = non_empty(
            required_str_array(policy, "reduction_claims", "required_fields")?,
            "reduction_claims",
            "required_fields",
        )?;
        let before_count_field = required_str(policy, "reduction_claims", "before_count_field")?;
        let after_count_field = required_str(policy, "reduction_claims", "after_count_field")?;
        let census_snapshot_ref_field =
            required_str(policy, "reduction_claims", "census_snapshot_ref_field")?;
        let census_snapshot_schema =
            required_str(policy, "reduction_claims", "census_snapshot_schema")?;
        let census_snapshot_claim_field =
            required_str(policy, "reduction_claims", "census_snapshot_claim_field")?;
        for field in [
            &before_count_field,
            &after_count_field,
            &census_snapshot_ref_field,
        ] {
            if !required_claim_fields.contains(field) {
                return Err(GateError::Policy(format!(
                    "{CODE_POLICY_INVALID}: reduction_claims proof field {field:?} must also \
                     appear in reduction_claims.required_fields"
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
            workspace_section: required_str(policy, "workspace_manifest", "section")?,
            cargo_manifest_file_name: required_str(
                policy,
                "name_surface",
                "cargo_manifest_file_name",
            )?,
            buck_file_names: required_str_array(policy, "name_surface", "buck_file_names")?
                .into_iter()
                .collect(),
            member_dependency_sections: non_empty(
                required_str_array(policy, "name_surface", "dependency_sections")?,
                "name_surface",
                "dependency_sections",
            )?,
            masterplan_path: required_str(policy, "masterplan", "path")?,
            anchor_field_names: non_empty(
                required_str_array(policy, "masterplan", "anchor_field_names")?,
                "masterplan",
                "anchor_field_names",
            )?
            .into_iter()
            .collect(),
            claim_field: required_str(policy, "reduction_claims", "claim_field")?,
            claim_values: non_empty(
                required_str_array(policy, "reduction_claims", "claim_values")?,
                "reduction_claims",
                "claim_values",
            )?
            .into_iter()
            .collect(),
            required_claim_fields,
            before_count_field,
            after_count_field,
            census_snapshot_ref_field,
            census_snapshot_schema,
            census_snapshot_claim_field,
            claim_scan_roots: required_str_array(policy, "reduction_claims", "scan_roots")?,
        })
    }

    pub fn under_target_path_prefix(&self, path: &str) -> bool {
        self.path_prefixes.iter().any(|p| path.starts_with(p))
    }

    pub fn carries_target_name_prefix(&self, name: &str) -> bool {
        self.name_prefixes.iter().any(|p| name.starts_with(p))
    }

    pub fn exempt(&self, path: &str) -> bool {
        self.exempt_path_prefixes
            .iter()
            .any(|p| path.starts_with(p))
    }
}

/// The canonical per-entry digest for the hashed baseline sets: lowercase sha256 hex of
/// the exact frozen string. The baseline commits these digests instead of literal
/// strings so the committed file never carries the retired vocabulary embedded in
/// migration-inventory path names (brand-residue ratchet), while membership stays an
/// exact set comparison: hash the live string, look it up.
///
/// Arm A hashes the repo-relative path. Arm B name hashes hash the crate/dep name.
/// Arm B path-dep hashes hash the edge-identity tuple
/// `"<declaring-manifest>\0<dependency-name>\0<normalized-destination>"` via
/// [`workspace_path_dep_digest`] so a new edge to an already-baselined destination
/// from a different manifest cannot hide behind destination-only membership.
pub fn entry_digest(entry: &str) -> String {
    format!("{:x}", Sha256::digest(entry.as_bytes()))
}

/// Canonical Arm B path-dep baseline key: declaring manifest path (empty for the
/// root workspace table), dependency name, and the lexically normalized destination.
pub fn workspace_path_dep_key(origin: &str, name: &str, normalized_dest: &str) -> String {
    format!("{origin}\0{name}\0{normalized_dest}")
}

/// Digest of [`workspace_path_dep_key`] — the membership token stored in
/// `arm_b_workspace_path_dep_hashes`.
pub fn workspace_path_dep_digest(origin: &str, name: &str, normalized_dest: &str) -> String {
    entry_digest(&workspace_path_dep_key(origin, name, normalized_dest))
}

/// Canonical Arm B name-declaration baseline key. Cargo package/dependency names and
/// Buck/Rust module names are scoped by the file that declares them; name-only hashing
/// would let a new declaration reuse an unrelated package's baselined spelling.
pub fn name_decl_key(origin: &str, name: &str) -> String {
    format!("{origin}\0{name}")
}

pub fn name_decl_digest(origin: &str, name: &str) -> String {
    entry_digest(&name_decl_key(origin, name))
}

/// Directory of a repo-relative manifest path (`Cargo.toml` at the root → empty).
fn manifest_base_dir(origin: &str) -> &str {
    origin.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
}

/// True when a path spelling is rooted, Windows-prefixed, or backslash-ambiguous.
fn path_spelling_rejected(raw: &str) -> bool {
    if raw.contains('\\') {
        return true;
    }
    if raw.starts_with('/') || raw.starts_with("//") {
        return true;
    }
    let bytes = raw.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// The committed shrink-only baseline: the migration-inventory estate the gate covers.
/// Arm A and Arm B sets hold per-entry sha256 digests ([`entry_digest`] /
/// [`workspace_path_dep_digest`]); Arm C anchors stay literal (verified free of
/// scanner-relevant tokens at authoring time).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Baseline {
    pub path_hashes: BTreeSet<String>,
    pub workspace_path_dep_hashes: BTreeSet<String>,
    pub dep_name_hashes: BTreeSet<String>,
    pub anchors: BTreeSet<String>,
}

fn baseline_set(value: &Value, key: &str) -> Result<BTreeSet<String>, GateError> {
    let items = value.get(key).and_then(Value::as_array).ok_or_else(|| {
        GateError::Policy(format!(
            "{CODE_POLICY_INVALID}: baseline missing array {key}"
        ))
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

fn baseline_digest_set(value: &Value, key: &str) -> Result<BTreeSet<String>, GateError> {
    let set = baseline_set(value, key)?;
    for entry in &set {
        let well_formed = entry.len() == 64
            && entry
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        if !well_formed {
            return Err(GateError::Policy(format!(
                "{CODE_POLICY_INVALID}: baseline {key} entry {entry:?} is not a lowercase \
                 sha256 hex digest; the baseline commits per-entry digests, never literal \
                 path strings"
            )));
        }
    }
    Ok(set)
}

impl Baseline {
    pub fn from_value(value: &Value) -> Result<Self, GateError> {
        Ok(Self {
            path_hashes: baseline_digest_set(value, "arm_a_path_hashes")?,
            workspace_path_dep_hashes: baseline_digest_set(
                value,
                "arm_b_workspace_path_dep_hashes",
            )?,
            dep_name_hashes: baseline_digest_set(value, "arm_b_dep_name_hashes")?,
            anchors: baseline_set(value, "arm_c_anchors")?,
        })
    }

    pub fn to_json(&self) -> Value {
        json!({
            "_comment": format!(
                "Committed shrink-only baseline for {GATE_ID} (strategy: committed-sorted-path-digest-list). \
                 Arm A entries are lowercase sha256 hex digests of the exact path strings; Arm B path-dep \
                 entries are digests of the (declaring-manifest, dependency-name, normalized-destination) \
                 tuple; Arm B name entries are digests of the (declaration-origin, name) tuple, including \
                 Cargo package/dependency, Buck target, and Rust module declarations — the \
                 migration-inventory estate at freeze time, committed WITHOUT its literal names so this \
                 file stays free of brand-residue vocabulary. Anything NEW fails closed and is reported by \
                 its literal live path. Regenerate ONLY alongside an admissible shrink, with the \
                 policy-declared regeneration command."
            ),
            "gate_id": GATE_ID,
            "entry_digest": "sha256-hex-of-exact-entry-string",
            "arm_a_path_hashes": self.path_hashes,
            "arm_b_workspace_path_dep_hashes": self.workspace_path_dep_hashes,
            "arm_b_dep_name_hashes": self.dep_name_hashes,
            "arm_c_anchors": self.anchors,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Arm A — new tracked files under target path prefixes (shrink-only baseline).
// ─────────────────────────────────────────────────────────────────────────────

/// Pure Arm A evaluator. `paths` is the full candidate set under test (the caller owns
/// collection); entries outside the target path prefixes are counted for liveness and
/// otherwise ignored. A target-prefix path whose digest is absent from the baseline is
/// NEW debt, reported by its LITERAL live path (the digests hide nothing from the
/// violator's report). A baseline digest matching no live path is a stale row that forces
/// regeneration (shrink-only); stale rows are reported as a count plus a digest-prefix
/// bucket — the literal removed paths are unrecoverable from digests by design.
pub fn evaluate_tree(policy: &Policy, baseline: &Baseline, paths: &BTreeSet<String>) -> Report {
    let mut findings = Vec::new();
    let mut live_hashes = BTreeSet::new();
    for path in paths {
        if !policy.under_target_path_prefix(path) || policy.exempt(path) {
            continue;
        }
        let digest = entry_digest(path);
        if !baseline.path_hashes.contains(&digest) {
            findings.push(Finding::new(
                CODE_NEW_TARGET_PATH,
                ARM_A,
                path.clone(),
                "new tracked file under a reorg-target path prefix; Global Binding Rule 1 \
                 (no-new-reorg-target-debt) refuses new target-prefix files — home it outside \
                 the target prefixes",
            ));
        }
        live_hashes.insert(digest);
    }
    let stale: Vec<&String> = baseline.path_hashes.difference(&live_hashes).collect();
    if !stale.is_empty() {
        let bucket: Vec<String> = stale.iter().take(8).map(|d| d[..12].to_owned()).collect();
        findings.push(Finding::new(
            CODE_STALE_BASELINE_PATH,
            ARM_A,
            format!("{} stale digest(s) in arm_a_path_hashes", stale.len()),
            format!(
                "baseline digest(s) match no live target-prefix path (digest prefix bucket: \
                 {bucket:?}); the removal itself is always admissible (shrink-only), but it \
                 requires baseline regeneration in the same change so burned-down debt cannot \
                 regain headroom. Regenerate with: {}",
                policy.regeneration_command
            ),
        ));
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

/// One dependency-table entry: the dependency name and, when declared, its `path` value
/// (empty string when the entry has no path key). `path_unparseable` is the fail-closed
/// marker: a `path` key whose value is not a TOML string, or a destination that cannot
/// be normalized, must surface as a finding, never as a silently empty path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkspaceDep {
    pub name: String,
    pub path: String,
    pub path_unparseable: bool,
}

/// Parse a TOML basic or literal string value (double OR single quotes — both are valid
/// Cargo path spellings). Used only for Buck `name =` literals, not Cargo manifests.
fn toml_str_value(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_end_matches(',').trim();
    for quote in ['"', '\''] {
        if let Some(rest) = trimmed.strip_prefix(quote) {
            return rest
                .strip_suffix(quote)
                .filter(|inner| !inner.contains(quote))
                .map(str::to_owned);
        }
    }
    None
}

/// Lexically normalize a repo-relative path: resolve `.` and `..` components against
/// `base_dir` (the declaring manifest's repo-relative directory; empty for the root) so
/// equivalent spellings such as `./cloud/x` or `../../cloud/x` cannot evade the prefix
/// check. Absolute, Windows-prefixed, backslash-ambiguous, or repo-escaping spellings
/// return `None` — callers fail closed instead of prefix-testing the raw string.
pub fn try_normalize_rel_path(base_dir: &str, raw: &str) -> Option<String> {
    if path_spelling_rejected(raw) || path_spelling_rejected(base_dir) {
        return None;
    }
    let mut stack: Vec<&str> = Vec::new();
    for part in base_dir.split('/').chain(raw.split('/')) {
        match part {
            "" | "." => {}
            ".." => {
                if stack.pop().is_none() {
                    return None;
                }
            }
            other => stack.push(other),
        }
    }
    Some(stack.join("/"))
}
fn repo_relative_dep_dest(base_dir: &str, raw: &str) -> Option<String> {
    match try_normalize_rel_path("", raw) {
        Some(path) if path == raw => Some(path),
        _ => try_normalize_rel_path(base_dir, raw),
    }
}

fn toml_parse_error(detail: impl fmt::Display) -> GateError {
    GateError::Input(format!("{CODE_DEP_PATH_UNPARSEABLE}: {detail}"))
}

fn parse_toml_document(manifest: &str) -> Result<TomlValue, GateError> {
    manifest
        .parse::<TomlValue>()
        .map_err(|error| toml_parse_error(format!("cargo manifest is not valid TOML: {error}")))
}

fn toml_table<'a>(
    value: &'a TomlValue,
    context: &str,
) -> Result<&'a toml::map::Map<String, TomlValue>, GateError> {
    value.as_table().ok_or_else(|| {
        toml_parse_error(format!(
            "{context} must be a TOML table; unsupported Cargo shape fails closed"
        ))
    })
}

fn walk_dotted_table<'a>(
    root: &'a TomlValue,
    dotted: &str,
) -> Result<Option<&'a toml::map::Map<String, TomlValue>>, GateError> {
    let mut current = root;
    for part in dotted.split('.') {
        let table = match current.as_table() {
            Some(table) => table,
            None => {
                return Err(toml_parse_error(format!(
                    "dotted table {dotted:?} walks through a non-table at {part:?}"
                )));
            }
        };
        match table.get(part) {
            None => return Ok(None),
            Some(next) => current = next,
        }
    }
    Ok(Some(toml_table(
        current,
        &format!("dotted table {dotted:?}"),
    )?))
}

fn dep_from_spec(name: &str, spec: &TomlValue) -> Result<WorkspaceDep, GateError> {
    if name.is_empty() {
        return Err(toml_parse_error(
            "dependency table contains an empty key; unsupported Cargo shape fails closed",
        ));
    }
    match spec {
        TomlValue::String(_) | TomlValue::Integer(_) | TomlValue::Float(_) => Ok(WorkspaceDep {
            name: name.to_owned(),
            path: String::new(),
            path_unparseable: false,
        }),
        TomlValue::Table(table) => match table.get("path") {
            None => Ok(WorkspaceDep {
                name: name.to_owned(),
                path: String::new(),
                path_unparseable: false,
            }),
            Some(TomlValue::String(path)) => Ok(WorkspaceDep {
                name: name.to_owned(),
                path: path.clone(),
                path_unparseable: false,
            }),
            Some(_) => Ok(WorkspaceDep {
                name: name.to_owned(),
                path: String::new(),
                path_unparseable: true,
            }),
        },
        TomlValue::Boolean(_) | TomlValue::Datetime(_) | TomlValue::Array(_) => {
            Err(toml_parse_error(format!(
                "dependency {name:?} uses an unsupported Cargo value shape"
            )))
        }
    }
}

fn collect_dep_table(
    value: &TomlValue,
    context: &str,
    out: &mut Vec<WorkspaceDep>,
) -> Result<(), GateError> {
    for (name, spec) in toml_table(value, context)? {
        out.push(dep_from_spec(name, spec)?);
    }
    Ok(())
}

fn collect_named_dep_sections(
    table: &toml::map::Map<String, TomlValue>,
    sections: &[String],
    context: &str,
    out: &mut Vec<WorkspaceDep>,
) -> Result<(), GateError> {
    for section in sections {
        if let Some(value) = table.get(section) {
            collect_dep_table(value, &format!("{context}.{section}"), out)?;
        }
    }
    Ok(())
}

/// Parse `[workspace.dependencies]` (or any policy-declared dotted table) with a
/// semantic TOML walk. Inline tables, dotted subsections, quoted keys, and
/// string-decoded path values are collected; parse errors and unsupported shapes
/// fail closed.
pub fn parse_workspace_dependencies(
    manifest: &str,
    section: &str,
) -> Result<Vec<WorkspaceDep>, GateError> {
    let document = parse_toml_document(manifest)?;
    match walk_dotted_table(&document, section)? {
        Some(table) => {
            let mut deps = Vec::new();
            for (name, spec) in table {
                deps.push(dep_from_spec(name, spec)?);
            }
            Ok(deps)
        }
        None => Ok(Vec::new()),
    }
}

/// Pure Arm B evaluator over the parsed workspace-dependency entries of the root
/// workspace table (`origin` is empty so the edge key is `"\0<name>\0<dest>"`).
pub fn evaluate_workspace_deps(
    policy: &Policy,
    baseline: &Baseline,
    deps: &[WorkspaceDep],
) -> Report {
    let mut findings = Vec::new();
    for dep in deps {
        push_dep_findings(policy, baseline, dep, "", "", &mut findings);
    }
    findings.sort();
    Report {
        evaluated_arms: vec![ARM_B.to_owned()],
        evaluated_path_count: deps.len(),
        findings,
    }
}

/// Shared Arm B refusal logic for one dependency entry. Destinations must already be
/// canonical repo-relative paths (the I/O collector resolves them) or else fail closed
/// as unparseable — raw-on-escape is never prefix-tested.
fn push_dep_findings(
    policy: &Policy,
    baseline: &Baseline,
    dep: &WorkspaceDep,
    origin: &str,
    base_dir: &str,
    findings: &mut Vec<Finding>,
) {
    let origin_note = if origin.is_empty() {
        String::new()
    } else {
        format!(" (declared in {origin})")
    };
    if dep.path_unparseable {
        findings.push(Finding::new(
            CODE_DEP_PATH_UNPARSEABLE,
            ARM_B,
            format!("{}{origin_note}", dep.name),
            "dependency declares a `path` whose value this gate cannot read as a quoted \
             string or cannot canonicalize inside the repo; fail-closed — spell the path \
             as a plainly quoted TOML string that stays repo-relative",
        ));
    }
    if !dep.path.is_empty() {
        let path = match repo_relative_dep_dest(base_dir, &dep.path) {
            Some(path) => path,
            None => {
                findings.push(Finding::new(
                    CODE_DEP_PATH_UNPARSEABLE,
                    ARM_B,
                    format!("{}{origin_note}", dep.name),
                    format!(
                        "dependency path {:?} (origin {origin:?}) is not a canonical \
                         repo-relative destination; fail-closed rather than prefix-testing \
                         the raw spelling",
                        dep.path
                    ),
                ));
                String::new()
            }
        };
        if !path.is_empty()
            && policy.under_target_path_prefix(&path)
            && !baseline
                .workspace_path_dep_hashes
                .contains(&workspace_path_dep_digest(origin, &dep.name, &path))
        {
            findings.push(Finding::new(
                CODE_NEW_TARGET_PATH_DEP,
                ARM_B,
                format!("{} -> {path}{origin_note}", dep.name),
                "new path dependency into a reorg-target path prefix; Global Binding \
                 Rule 1 refuses new dependency edges into the target estate",
            ));
        }
    }
    if policy.carries_target_name_prefix(&dep.name)
        && !baseline
            .dep_name_hashes
            .contains(&name_decl_digest(origin, &dep.name))
    {
        findings.push(Finding::new(
            CODE_NEW_TARGET_DEP_NAME,
            ARM_B,
            format!("{}{origin_note}", dep.name),
            "new dependency named under a reorg-target name prefix; Global Binding \
             Rule 1 refuses minting new target-form names",
        ));
    }
}

/// Convenience Arm B entry point over raw manifest text.
pub fn evaluate_workspace_manifest(
    policy: &Policy,
    baseline: &Baseline,
    manifest: &str,
    section: &str,
) -> Result<Report, GateError> {
    Ok(evaluate_workspace_deps(
        policy,
        baseline,
        &parse_workspace_dependencies(manifest, section)?,
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// Arm B (name surface) — crate/binary names and member-manifest path dependencies.
// The policy binds target NAME prefixes to every NEW crate, module, and binary name,
// and Cargo members may declare direct path dependencies without inheriting from the
// root [workspace.dependencies] table — so Arm B must cover member manifests and
// declared names, not only the root dependency table.
// ─────────────────────────────────────────────────────────────────────────────

/// A declared name (crate/package, `[[bin]]`, or build-graph target) plus the
/// repo-relative file that declares it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NameDecl {
    pub name: String,
    pub origin: String,
}

/// The facts one Cargo manifest contributes to Arm B.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManifestFacts {
    pub package_name: Option<String>,
    pub bin_names: Vec<String>,
    pub path_deps: Vec<WorkspaceDep>,
}

fn toml_string_field(
    table: &toml::map::Map<String, TomlValue>,
    key: &str,
    context: &str,
) -> Result<Option<String>, GateError> {
    match table.get(key) {
        None => Ok(None),
        Some(TomlValue::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(toml_parse_error(format!(
            "{context}.{key} must be a string"
        ))),
    }
}

/// Semantically parse one Cargo manifest for its `[package]` name, `[[bin]]` names,
/// and every entry in the policy-declared dependency sections, including
/// `[target.<…>.<section>]` tables. Parse errors and unsupported shapes fail closed.
pub fn parse_manifest_facts(
    manifest: &str,
    dependency_sections: &[String],
) -> Result<ManifestFacts, GateError> {
    let document = parse_toml_document(manifest)?;
    let root = toml_table(&document, "cargo manifest")?;
    let mut facts = ManifestFacts::default();
    if let Some(package) = root.get("package") {
        let package = toml_table(package, "[package]")?;
        facts.package_name = toml_string_field(package, "name", "[package]")?;
    }
    if let Some(bins) = root.get("bin") {
        let bins = bins
            .as_array()
            .ok_or_else(|| toml_parse_error("[[bin]] must be an array of tables"))?;
        for (index, bin) in bins.iter().enumerate() {
            let bin = toml_table(bin, &format!("[[bin]][{index}]"))?;
            if let Some(name) = toml_string_field(bin, "name", &format!("[[bin]][{index}]"))? {
                facts.bin_names.push(name);
            }
        }
    }
    collect_named_dep_sections(root, dependency_sections, "", &mut facts.path_deps)?;
    if let Some(targets) = root.get("target") {
        let targets = toml_table(targets, "[target]")?;
        for (cfg, target) in targets {
            let target = toml_table(target, &format!("[target.{cfg}]"))?;
            collect_named_dep_sections(
                target,
                dependency_sections,
                &format!("target.{cfg}"),
                &mut facts.path_deps,
            )?;
        }
    }
    Ok(facts)
}

/// Line-parse a Buck build file for `name = "…"` target declarations (either quote
/// style). Names whose values are not plain quoted strings are skipped — a target-form
/// name must be a literal to exist, and literals are what this scan binds.
pub fn parse_buck_target_names(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in text.lines() {
        let bare = line.split('#').next().unwrap_or("").trim();
        if let Some(rest) = bare.strip_prefix("name")
            && rest.starts_with(|c: char| c == '=' || c.is_whitespace())
            && let Some(rest) = rest.trim_start().strip_prefix('=')
            && let Some(name) = toml_str_value(rest)
        {
            names.push(name);
        }
    }
    names
}

fn collect_rust_module_declarations(
    items: &[syn::Item],
    parent_scope: &str,
    out: &mut Vec<(String, String)>,
) {
    for item in items {
        let syn::Item::Mod(module) = item else {
            continue;
        };
        let rendered = module.ident.to_string();
        let name = rendered.strip_prefix("r#").unwrap_or(&rendered).to_owned();
        out.push((name.clone(), parent_scope.to_owned()));
        if let Some((_, nested)) = &module.content {
            let nested_scope = if parent_scope.is_empty() {
                name
            } else {
                format!("{parent_scope}::{name}")
            };
            collect_rust_module_declarations(nested, &nested_scope, out);
        }
    }
}

/// Semantically parse actual Rust `mod` items. Comments, strings, macro input, and
/// similarly-spelled identifiers are excluded by the Rust AST parser. Each result carries
/// the leaf name and its inline parent scope so two valid declarations with the same leaf
/// in one file retain distinct baseline identities.
pub fn parse_rust_module_declarations(text: &str) -> Result<Vec<(String, String)>, GateError> {
    let file = syn::parse_file(text).map_err(|error| {
        GateError::Input(format!(
            "{CODE_NAME_SURFACE_UNPARSEABLE}: Rust source is not parseable: {error}"
        ))
    })?;
    let mut modules = Vec::new();
    collect_rust_module_declarations(&file.items, "", &mut modules);
    Ok(modules)
}

/// Everything the repo-wide name-surface scan feeds Arm B beyond the root workspace
/// table: declared names plus member-manifest path dependencies, each with the
/// repo-relative origin manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NameSurface {
    pub names: Vec<NameDecl>,
    pub member_path_deps: Vec<(String, WorkspaceDep)>,
}

/// Pure Arm B evaluator over the repo-wide name surface.
pub fn evaluate_name_surface(
    policy: &Policy,
    baseline: &Baseline,
    surface: &NameSurface,
) -> Report {
    let mut findings = Vec::new();
    for decl in &surface.names {
        if policy.carries_target_name_prefix(&decl.name)
            && !baseline
                .dep_name_hashes
                .contains(&name_decl_digest(&decl.origin, &decl.name))
        {
            findings.push(Finding::new(
                CODE_NEW_TARGET_NAME,
                ARM_B,
                format!("{} (declared in {})", decl.name, decl.origin),
                "new crate/binary/build-target name minted under a reorg-target name prefix; \
                 Global Binding Rule 1 refuses minting new target-form names",
            ));
        }
    }
    for (origin, dep) in &surface.member_path_deps {
        let base_dir = manifest_base_dir(origin);
        push_dep_findings(policy, baseline, dep, origin, base_dir, &mut findings);
    }
    findings.sort();
    Report {
        evaluated_arms: vec![ARM_B.to_owned()],
        evaluated_path_count: surface.names.len() + surface.member_path_deps.len(),
        findings,
    }
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
                                    out.push((format!("{child_location}/{index}"), anchor.clone()));
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
                collect_claim_objects(
                    child,
                    claim_field,
                    claim_values,
                    &format!("{location}/{key}"),
                    out,
                );
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_claim_objects(
                    item,
                    claim_field,
                    claim_values,
                    &format!("{location}/{index}"),
                    out,
                );
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

/// Every reason a reduction claim fails: missing/empty companion fields, non-numeric
/// counts, or counts that do not actually reduce (`after >= before`). The gate still
/// mints no threshold — the counts are DATA of the artifact under test; the only
/// comparison is the direction the claim itself asserts.
fn claim_problems(policy: &Policy, map: &Map<String, Value>) -> Vec<String> {
    let mut problems = Vec::new();
    for field in &policy.required_claim_fields {
        if field == &policy.before_count_field || field == &policy.after_count_field {
            if !matches!(map.get(field.as_str()), Some(Value::Number(_))) {
                problems.push(format!("{field} must be a number"));
            }
        } else if field == &policy.census_snapshot_ref_field {
            if !matches!(map.get(field.as_str()), Some(Value::String(text)) if !text.trim().is_empty())
            {
                problems.push(format!("{field} must be a non-empty string ref"));
            }
        } else if !claim_field_proven(map, field) {
            problems.push(format!("missing or empty {field}"));
        }
    }
    if problems.is_empty() {
        let before = map.get(&policy.before_count_field).and_then(Value::as_f64);
        let after = map.get(&policy.after_count_field).and_then(Value::as_f64);
        match (before, after) {
            (Some(before), Some(after)) if after < before => {}
            (Some(before), Some(after)) => problems.push(format!(
                "claimed reduction does not reduce: {}={after} is not below {}={before}",
                policy.after_count_field, policy.before_count_field
            )),
            _ => problems.push("counts are not finite numbers".to_owned()),
        }
    }
    problems
}

/// Pure Arm D evaluator. A net target-surface-reduction claim is refused unless every
/// policy-declared companion field is present, the counts are numeric, the claimed
/// direction actually reduces, and the census snapshot ref is bound.
pub fn evaluate_reduction_claims(policy: &Policy, artifact: &Value) -> Report {
    evaluate_reduction_claims_at(policy, "", artifact)
}

/// Arm D over one artifact, with `origin` (the artifact's repo-relative path, empty for
/// the planning SSOT) prefixed onto every reported claim location.
pub fn evaluate_reduction_claims_at(policy: &Policy, origin: &str, artifact: &Value) -> Report {
    let mut claims = Vec::new();
    collect_claim_objects(
        artifact,
        &policy.claim_field,
        &policy.claim_values,
        origin,
        &mut claims,
    );
    let mut findings = Vec::new();
    for (location, map) in &claims {
        let problems = claim_problems(policy, map);
        if !problems.is_empty() {
            findings.push(Finding::new(
                CODE_UNPROVEN_REDUCTION_CLAIM,
                ARM_D,
                format!("claim at {location}"),
                format!(
                    "net target-surface-reduction claim without its proven measurement: \
                     {problems:?}. A reduction claim must carry numeric before/after counts \
                     that actually reduce and the census snapshot ref they were measured \
                     against"
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

fn claim_snapshot_problem(
    repo_root: &Path,
    policy: &Policy,
    artifact_origin: &str,
    claim: &Map<String, Value>,
) -> Option<String> {
    let raw_ref = match claim
        .get(&policy.census_snapshot_ref_field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => value,
        None => return Some("snapshot ref must be a non-empty string".into()),
    };
    let lexical = match try_normalize_rel_path("", raw_ref) {
        Some(path) => path,
        None => {
            return Some(
                "snapshot ref escapes the repo or uses an unsupported path spelling".into(),
            );
        }
    };
    let canonical = match canonicalize_repo_rel(repo_root, &lexical) {
        Ok(path) => path,
        Err(error) => {
            return Some(format!(
                "snapshot ref does not resolve inside the repo: {error}"
            ));
        }
    };
    let allowed = policy.claim_scan_roots.iter().any(|root| {
        canonical == *root || canonical.starts_with(&format!("{}/", root.trim_end_matches('/')))
    });
    if !allowed {
        return Some(format!(
            "snapshot {canonical:?} is outside the policy claim scan roots {:?}",
            policy.claim_scan_roots
        ));
    }
    if canonical == artifact_origin {
        return Some(
            "a reduction claim cannot cite its own artifact as its census snapshot".into(),
        );
    }
    let snapshot = match load_json(&repo_root.join(&canonical)) {
        Ok(snapshot) => snapshot,
        Err(error) => return Some(format!("snapshot {canonical:?} is unreadable: {error}")),
    };
    let Some(snapshot) = snapshot.as_object() else {
        return Some(format!("snapshot {canonical:?} must be a JSON object"));
    };
    if snapshot.get("schema").and_then(Value::as_str)
        != Some(policy.census_snapshot_schema.as_str())
    {
        return Some(format!(
            "snapshot {canonical:?} must declare schema {:?}",
            policy.census_snapshot_schema
        ));
    }
    for (snapshot_field, claim_field) in [
        (
            policy.census_snapshot_claim_field.as_str(),
            policy.claim_field.as_str(),
        ),
        (
            policy.before_count_field.as_str(),
            policy.before_count_field.as_str(),
        ),
        (
            policy.after_count_field.as_str(),
            policy.after_count_field.as_str(),
        ),
    ] {
        if snapshot.get(snapshot_field) != claim.get(claim_field) {
            return Some(format!(
                "snapshot {canonical:?} field {snapshot_field:?} does not bind claim field \
                 {claim_field:?}"
            ));
        }
    }
    None
}

/// I/O-bound Arm D proof: every syntactically valid reduction claim must resolve its
/// census snapshot to a parseable repository artifact under a policy scan root. The
/// snapshot schema and claim/before/after facts must match exactly.
pub fn evaluate_census_snapshot_refs_at(
    repo_root: &Path,
    policy: &Policy,
    artifact_origin: &str,
    artifact: &Value,
) -> Report {
    let mut claims = Vec::new();
    collect_claim_objects(
        artifact,
        &policy.claim_field,
        &policy.claim_values,
        artifact_origin,
        &mut claims,
    );
    let mut findings = Vec::new();
    for (location, claim) in &claims {
        if !claim_problems(policy, claim).is_empty() {
            continue;
        }
        if let Some(problem) = claim_snapshot_problem(repo_root, policy, artifact_origin, claim) {
            findings.push(Finding::new(
                CODE_UNPROVEN_REDUCTION_CLAIM,
                ARM_D,
                format!("claim at {location}"),
                format!(
                    "net target-surface-reduction census snapshot is not valid evidence: \
                     {problem}"
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
fn audit_manifest_origin(origin: &str) -> Option<String> {
    if path_spelling_rejected(origin) {
        return None;
    }
    if origin.contains("/./")
        || origin.contains("/../")
        || origin.starts_with("./")
        || origin.starts_with("../")
    {
        return None;
    }
    let normalized = try_normalize_rel_path("", origin)?;
    if normalized != origin.trim_start_matches("./") && normalized != origin {
        return None;
    }
    if normalized.is_empty() {
        return None;
    }
    let file_name = normalized
        .rsplit_once('/')
        .map(|(_, name)| name)
        .unwrap_or(&normalized);
    if file_name != "Cargo.toml" {
        return None;
    }
    Some(normalized)
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
        let resolution = record
            .get("resolution")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !sha.is_empty() && !resolution.trim().is_empty() {
            remediated_commits.insert(sha.to_owned());
        }
    }

    // Bind the capture to the declared range: the materialization recipe produces
    // `git rev-list --reverse <from>..<to>`, so the LAST captured commit must be the
    // declared range head. A truncated or unrelated capture cannot simply assert
    // `complete: true` and audit green over commits that never reach the range head.
    let last_sha = commits
        .last()
        .and_then(|commit| commit.get("sha"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if last_sha != range_to {
        return Err(audit_invalid(format!(
            "capture is not bound to the declared range: the last captured commit \
             {last_sha:?} is not range.to {range_to:?}"
        )));
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
            // FAIL-CLOSED: a malformed dependency fact (non-object, or missing typed
            // name/path) must never silently degrade to an empty dependency that audits
            // green — that would hide a target-prefix edge from durable audit evidence.
            let object = dep.as_object().ok_or_else(|| {
                audit_invalid(format!(
                    "commit {sha}: added_workspace_path_deps entry must be an object with \
                     string name and path fields"
                ))
            })?;
            let name = object
                .get("name")
                .and_then(Value::as_str)
                .filter(|name| !name.trim().is_empty())
                .ok_or_else(|| {
                    audit_invalid(format!(
                        "commit {sha}: added_workspace_path_deps entry is missing a \
                         non-empty string name"
                    ))
                })?;
            let path = object.get("path").and_then(Value::as_str).ok_or_else(|| {
                audit_invalid(format!(
                    "commit {sha}: added_workspace_path_deps entry {name:?} is missing a \
                     string path (name-only entries carry an empty string path)"
                ))
            })?;
            if policy.carries_target_name_prefix(name) {
                findings.push(Finding::new(
                    CODE_AUDIT_TARGET_DEBT_COMMIT,
                    ARM_B,
                    format!("{sha}: {name}"),
                    "commit introduced a workspace dependency named under a reorg-target name \
                     prefix within the audited range",
                ));
            }
            if !path.is_empty() {
                // Relative spellings such as `../../cloud/x` are valid Cargo path
                // values; the live Arm B evaluator normalizes them against the
                // declaring manifest. Audit mode must do the same, so the capture
                // must carry a canonical repo-relative Cargo.toml origin. A missing
                // origin, a non-manifest origin, or a path that cannot be
                // normalized is RTD_AUDIT_INPUT_INVALID — never a silent green.
                let origin = object
                    .get("origin")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        audit_invalid(format!(
                            "commit {sha}: added_workspace_path_deps entry {name:?} with a \
                             non-empty path must carry a string origin (the declaring \
                             manifest) so the path can be normalized"
                        ))
                    })?;
                let origin = audit_manifest_origin(origin).ok_or_else(|| {
                    audit_invalid(format!(
                        "commit {sha}: added_workspace_path_deps entry {name:?} origin \
                         {origin:?} is not a normalized repo-relative Cargo.toml identity"
                    ))
                })?;
                if path_spelling_rejected(path) {
                    return Err(audit_invalid(format!(
                        "commit {sha}: added_workspace_path_deps entry {name:?} path \
                         {path:?} is absolute, Windows-prefixed, or backslash-ambiguous"
                    )));
                }
                let normalized = try_normalize_rel_path(manifest_base_dir(&origin), path)
                    .ok_or_else(|| {
                        audit_invalid(format!(
                            "commit {sha}: added_workspace_path_deps entry {name:?} path \
                             {path:?} (origin {origin:?}) is not a normalizable repo-relative \
                             path"
                        ))
                    })?;
                if policy.under_target_path_prefix(&normalized) {
                    findings.push(Finding::new(
                        CODE_AUDIT_TARGET_DEBT_COMMIT,
                        ARM_B,
                        format!("{sha}: {name} -> {normalized}"),
                        "commit introduced a workspace path dependency into a reorg-target prefix \
                         within the audited range",
                    ));
                }
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

/// Read the allowance source from the materialized immutable protected merge-base snapshot.
/// The generated-artifact materializer owns the single sanctioned Git/worktree boundary;
/// this gate consumes only the declared plain JSON input, so Buck/RBE execution never depends
/// on ambient `.git`.
///
/// `None` is the single exact initial-adoption state: the gate did not exist at the
/// protected T2 anchor. Once the first gate PR lands, every subsequent merge base must
/// provide a valid policy and baseline or the gate fails closed.
#[derive(Debug)]
struct FrozenReference {
    policy: Policy,
    baseline: Baseline,
}

fn load_frozen_baseline_from_merge_base(
    repo_root: &Path,
) -> Result<Option<FrozenReference>, GateError> {
    let snapshot = load_json(&repo_root.join(FROZEN_REFERENCE_PATH)).map_err(|error| {
        GateError::Io(format!(
            "load immutable reorg-target-debt snapshot {FROZEN_REFERENCE_PATH}: {error}; \
             materialize it with \
             `buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .`"
        ))
    })?;
    if snapshot.get("schema").and_then(Value::as_str)
        != Some("ci-reorg-target-debt-merge-base-snapshot.v1")
        || snapshot.get("base_ref").and_then(Value::as_str) != Some(PROTECTED_BASE_REF)
    {
        return Err(GateError::Policy(format!(
            "{CODE_POLICY_INVALID}: frozen reorg-target-debt snapshot has a foreign schema or \
             base_ref"
        )));
    }
    let merge_base = snapshot
        .get("merge_base")
        .and_then(Value::as_str)
        .unwrap_or("");
    if merge_base.len() != 40 || !merge_base.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(GateError::Policy(format!(
            "{CODE_POLICY_INVALID}: frozen snapshot merge_base is not a full Git object id"
        )));
    }
    let missing_at_merge_base = snapshot
        .get("missing_at_merge_base")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            GateError::Policy(format!(
                "{CODE_POLICY_INVALID}: frozen snapshot missing boolean missing_at_merge_base"
            ))
        })?;
    if missing_at_merge_base {
        if merge_base != INITIAL_ADOPTION_BASE_SHA
            || !snapshot.get("policy").is_some_and(Value::is_null)
            || !snapshot.get("baseline").is_some_and(Value::is_null)
        {
            return Err(GateError::Policy(format!(
                "{CODE_POLICY_INVALID}: missing-at-merge-base is lawful only for exact initial \
                 adoption anchor {INITIAL_ADOPTION_BASE_SHA} with null policy and baseline"
            )));
        }
        return Ok(None);
    }
    let frozen_policy_value = snapshot.get("policy").ok_or_else(|| {
        GateError::Policy(format!(
            "{CODE_POLICY_INVALID}: frozen snapshot missing policy"
        ))
    })?;
    let frozen_policy = Policy::from_value(frozen_policy_value)?;
    let baseline_value = snapshot.get("baseline").ok_or_else(|| {
        GateError::Policy(format!(
            "{CODE_POLICY_INVALID}: frozen snapshot missing baseline"
        ))
    })?;
    let baseline = Baseline::from_value(baseline_value)?;
    Ok(Some(FrozenReference {
        policy: frozen_policy,
        baseline,
    }))
}

fn validate_candidate_policy_against_frozen(
    frozen: &Policy,
    candidate: &Policy,
) -> Result<(), GateError> {
    let exact_fields = [
        (
            "baseline.file",
            frozen.baseline_file.as_str(),
            candidate.baseline_file.as_str(),
        ),
        (
            "workspace_manifest.path",
            frozen.workspace_manifest_path.as_str(),
            candidate.workspace_manifest_path.as_str(),
        ),
        (
            "workspace_manifest.section",
            frozen.workspace_section.as_str(),
            candidate.workspace_section.as_str(),
        ),
        (
            "name_surface.cargo_manifest_file_name",
            frozen.cargo_manifest_file_name.as_str(),
            candidate.cargo_manifest_file_name.as_str(),
        ),
        (
            "masterplan.path",
            frozen.masterplan_path.as_str(),
            candidate.masterplan_path.as_str(),
        ),
        (
            "reduction_claims.claim_field",
            frozen.claim_field.as_str(),
            candidate.claim_field.as_str(),
        ),
        (
            "reduction_claims.before_count_field",
            frozen.before_count_field.as_str(),
            candidate.before_count_field.as_str(),
        ),
        (
            "reduction_claims.after_count_field",
            frozen.after_count_field.as_str(),
            candidate.after_count_field.as_str(),
        ),
        (
            "reduction_claims.census_snapshot_ref_field",
            frozen.census_snapshot_ref_field.as_str(),
            candidate.census_snapshot_ref_field.as_str(),
        ),
        (
            "reduction_claims.census_snapshot_schema",
            frozen.census_snapshot_schema.as_str(),
            candidate.census_snapshot_schema.as_str(),
        ),
        (
            "reduction_claims.census_snapshot_claim_field",
            frozen.census_snapshot_claim_field.as_str(),
            candidate.census_snapshot_claim_field.as_str(),
        ),
    ];
    for (field, frozen_value, candidate_value) in exact_fields {
        if candidate_value != frozen_value {
            return Err(GateError::Policy(format!(
                "{CODE_POLICY_INVALID}: candidate {field} {candidate_value:?} differs from \
                 immutable merge-base value {frozen_value:?}"
            )));
        }
    }

    fn as_set(values: &[String]) -> BTreeSet<&str> {
        values.iter().map(String::as_str).collect()
    }
    let must_preserve = [
        (
            "target_forms.name_prefixes",
            as_set(&frozen.name_prefixes),
            as_set(&candidate.name_prefixes),
        ),
        (
            "target_forms.path_prefixes",
            as_set(&frozen.path_prefixes),
            as_set(&candidate.path_prefixes),
        ),
        (
            "name_surface.dependency_sections",
            as_set(&frozen.member_dependency_sections),
            as_set(&candidate.member_dependency_sections),
        ),
        (
            "reduction_claims.required_fields",
            as_set(&frozen.required_claim_fields),
            as_set(&candidate.required_claim_fields),
        ),
        (
            "reduction_claims.scan_roots",
            as_set(&frozen.claim_scan_roots),
            as_set(&candidate.claim_scan_roots),
        ),
    ];
    for (field, frozen_values, candidate_values) in must_preserve {
        if !frozen_values.is_subset(&candidate_values) {
            return Err(GateError::Policy(format!(
                "{CODE_POLICY_INVALID}: candidate {field} narrows immutable merge-base coverage"
            )));
        }
    }
    let set_must_preserve = [
        (
            "name_surface.buck_file_names",
            &frozen.buck_file_names,
            &candidate.buck_file_names,
        ),
        (
            "masterplan.anchor_field_names",
            &frozen.anchor_field_names,
            &candidate.anchor_field_names,
        ),
        (
            "reduction_claims.claim_values",
            &frozen.claim_values,
            &candidate.claim_values,
        ),
    ];
    for (field, frozen_values, candidate_values) in set_must_preserve {
        if !frozen_values.is_subset(candidate_values) {
            return Err(GateError::Policy(format!(
                "{CODE_POLICY_INVALID}: candidate {field} narrows immutable merge-base coverage"
            )));
        }
    }
    if !candidate
        .exempt_path_prefixes
        .iter()
        .all(|entry| frozen.exempt_path_prefixes.contains(entry))
    {
        return Err(GateError::Policy(format!(
            "{CODE_POLICY_INVALID}: candidate exemptions.path_prefixes adds an exemption beyond \
             the immutable merge-base policy"
        )));
    }
    if !candidate
        .skip_dir_names
        .iter()
        .all(|entry| frozen.skip_dir_names.contains(entry))
    {
        return Err(GateError::Policy(format!(
            "{CODE_POLICY_INVALID}: candidate scan.skip_dir_names adds a skipped directory beyond \
             the immutable merge-base policy"
        )));
    }
    Ok(())
}

fn rel_string(path: &Path, repo_root: &Path) -> Result<String, GateError> {
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
    Ok(rel_text)
}
fn canonicalize_repo_rel(repo_root: &Path, rel: &str) -> Result<String, GateError> {
    if path_spelling_rejected(rel) {
        return Err(GateError::Input(format!(
            "{CODE_DEP_PATH_UNPARSEABLE}: destination {rel:?} is absolute, Windows-prefixed, \
             or backslash-ambiguous"
        )));
    }
    let joined = if rel.is_empty() {
        repo_root.to_path_buf()
    } else {
        repo_root.join(rel)
    };
    let canonical = fs::canonicalize(&joined).map_err(|error| {
        GateError::Input(format!(
            "{CODE_DEP_PATH_UNPARSEABLE}: cannot canonicalize {rel:?}: {error}"
        ))
    })?;
    let root_canonical = fs::canonicalize(repo_root).map_err(|error| {
        GateError::Io(format!(
            "canonicalize repo root {}: {error}",
            repo_root.display()
        ))
    })?;
    rel_string(&canonical, &root_canonical).map_err(|_| {
        GateError::Input(format!(
            "{CODE_DEP_PATH_UNPARSEABLE}: destination {rel:?} escapes the repo root after \
             symlink resolution"
        ))
    })
}

fn canonicalize_declared_dep_path(
    repo_root: &Path,
    origin: &str,
    raw: &str,
) -> Result<String, GateError> {
    if path_spelling_rejected(raw) {
        return Err(GateError::Input(format!(
            "{CODE_DEP_PATH_UNPARSEABLE}: dependency path {raw:?} (origin {origin:?}) is \
             absolute, Windows-prefixed, or backslash-ambiguous"
        )));
    }
    let lexical = try_normalize_rel_path(manifest_base_dir(origin), raw).ok_or_else(|| {
        GateError::Input(format!(
            "{CODE_DEP_PATH_UNPARSEABLE}: dependency path {raw:?} (origin {origin:?}) escapes \
             the repo root"
        ))
    })?;
    canonicalize_repo_rel(repo_root, &lexical)
}

fn canonicalize_workspace_dep(
    repo_root: &Path,
    origin: &str,
    dep: WorkspaceDep,
) -> Result<WorkspaceDep, GateError> {
    if dep.path_unparseable || dep.path.is_empty() {
        return Ok(dep);
    }
    match canonicalize_declared_dep_path(repo_root, origin, &dep.path) {
        Ok(path) => Ok(WorkspaceDep {
            name: dep.name,
            path,
            path_unparseable: false,
        }),
        Err(_) => Ok(WorkspaceDep {
            name: dep.name,
            path: dep.path,
            path_unparseable: true,
        }),
    }
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
        let entry = entry
            .map_err(|error| GateError::Io(format!("dir entry {}: {error}", root.display())))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        // Symlink-safe: DirEntry::file_type never follows symlinks, so a directory
        // symlink is recorded as a LEAF entry instead of being traversed — a link such
        // as `<target>/loop -> ..` can neither hide debt nor hang the gate in a cycle.
        let file_type = entry
            .file_type()
            .map_err(|error| GateError::Io(format!("file_type {}: {error}", path.display())))?;
        if file_type.is_dir() {
            if !skip.contains(&name) {
                walk_files(&path, repo_root, skip, out)?;
            }
        } else {
            out.insert(rel_string(&path, repo_root)?);
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

/// Collect the repo-wide name surface: every Cargo manifest's package/bin names and
/// dependency-section path entries, every Buck file's target names, and every Rust module
/// declaration. File names are policy DATA where applicable; the walk is symlink-safe and
/// skips the policy's skip-dir names. Dependency destinations are canonicalized at this
/// I/O boundary.
pub fn collect_name_surface(repo_root: &Path, policy: &Policy) -> Result<NameSurface, GateError> {
    let mut wanted: BTreeSet<String> = policy.buck_file_names.clone();
    wanted.insert(policy.cargo_manifest_file_name.clone());
    let mut files = BTreeSet::new();
    walk_files(repo_root, repo_root, &policy.skip_dir_names, &mut files)?;
    let mut surface = NameSurface::default();
    for rel in files {
        let file_name = rel.rsplit_once('/').map(|(_, name)| name).unwrap_or(&rel);
        let is_rust = rel.ends_with(".rs");
        if !wanted.contains(file_name) && !is_rust {
            continue;
        }
        let text = fs::read_to_string(repo_root.join(&rel))
            .map_err(|error| GateError::Io(format!("read {rel}: {error}")))?;
        if file_name == policy.cargo_manifest_file_name {
            let facts = parse_manifest_facts(&text, &policy.member_dependency_sections)?;
            if let Some(name) = facts.package_name {
                surface.names.push(NameDecl {
                    name,
                    origin: rel.clone(),
                });
            }
            for name in facts.bin_names {
                surface.names.push(NameDecl {
                    name,
                    origin: rel.clone(),
                });
            }
            for dep in facts.path_deps {
                surface.member_path_deps.push((
                    rel.clone(),
                    canonicalize_workspace_dep(repo_root, &rel, dep)?,
                ));
            }
        } else if is_rust {
            for (name, parent_scope) in parse_rust_module_declarations(&text)? {
                let origin = if parent_scope.is_empty() {
                    rel.clone()
                } else {
                    format!("{rel}::{parent_scope}")
                };
                surface.names.push(NameDecl { name, origin });
            }
        } else {
            for name in parse_buck_target_names(&text) {
                surface.names.push(NameDecl {
                    name,
                    origin: rel.clone(),
                });
            }
        }
    }
    Ok(surface)
}

/// Collect every JSON artifact under the policy's reduction-claim scan roots (an absent
/// root contributes nothing; an unparseable artifact under a scan root fails closed).
pub fn collect_claim_artifacts(
    repo_root: &Path,
    policy: &Policy,
) -> Result<Vec<(String, Value)>, GateError> {
    let mut out = Vec::new();
    for root_rel in &policy.claim_scan_roots {
        let root = repo_root.join(root_rel);
        if !root.is_dir() {
            continue;
        }
        let mut files = BTreeSet::new();
        walk_files(&root, repo_root, &policy.skip_dir_names, &mut files)?;
        for rel in files {
            if rel.ends_with(".json") {
                out.push((rel.clone(), load_json(&repo_root.join(&rel))?));
            }
        }
    }
    Ok(out)
}

/// The literal (pre-digest) snapshot a regeneration would freeze. Kept literal so the
/// shrink-only enforcement can NAME exactly what a refused expansion tried to add.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BaselineCandidate {
    pub paths: BTreeSet<String>,
    pub workspace_path_deps: BTreeSet<String>,
    pub dep_names: BTreeSet<String>,
    pub anchors: BTreeSet<String>,
}

impl BaselineCandidate {
    pub fn to_baseline(&self) -> Baseline {
        Baseline {
            path_hashes: self.paths.iter().map(|entry| entry_digest(entry)).collect(),
            workspace_path_dep_hashes: self
                .workspace_path_deps
                .iter()
                .map(|entry| entry_digest(entry))
                .collect(),
            dep_name_hashes: self
                .dep_names
                .iter()
                .map(|entry| entry_digest(entry))
                .collect(),
            anchors: self.anchors.clone(),
        }
    }
}

fn record_baseline_dep(
    candidate: &mut BaselineCandidate,
    policy: &Policy,
    dep: &WorkspaceDep,
    origin: &str,
) -> Result<(), GateError> {
    if dep.path_unparseable {
        return Err(GateError::Input(format!(
            "{CODE_DEP_PATH_UNPARSEABLE}: dependency {:?} in {origin} declares a path \
             this gate cannot read; refusing to regenerate over an unevaluable surface",
            dep.name
        )));
    }
    if !dep.path.is_empty() && policy.under_target_path_prefix(&dep.path) {
        candidate
            .workspace_path_deps
            .insert(workspace_path_dep_key(origin, &dep.name, &dep.path));
    }
    if policy.carries_target_name_prefix(&dep.name) {
        candidate.dep_names.insert(name_decl_key(origin, &dep.name));
    }
    Ok(())
}

/// Snapshot the current tree into the literal baseline candidate the regeneration path
/// freezes. Fails closed on any unparseable dependency path.
pub fn collect_baseline_candidate(
    repo_root: &Path,
    policy: &Policy,
) -> Result<BaselineCandidate, GateError> {
    let mut candidate = BaselineCandidate {
        paths: collect_target_prefix_paths(repo_root, policy)?,
        ..BaselineCandidate::default()
    };
    let manifest =
        fs::read_to_string(repo_root.join(&policy.workspace_manifest_path)).map_err(|error| {
            GateError::Io(format!("read {}: {error}", policy.workspace_manifest_path))
        })?;
    for dep in parse_workspace_dependencies(&manifest, &policy.workspace_section)? {
        let dep = canonicalize_workspace_dep(repo_root, "", dep)?;
        record_baseline_dep(&mut candidate, policy, &dep, "")?;
    }
    let surface = collect_name_surface(repo_root, policy)?;
    for (origin, dep) in &surface.member_path_deps {
        record_baseline_dep(&mut candidate, policy, dep, origin)?;
    }
    for decl in surface.names {
        if policy.carries_target_name_prefix(&decl.name) {
            candidate
                .dep_names
                .insert(name_decl_key(&decl.origin, &decl.name));
        }
    }
    let plan = load_json(&repo_root.join(&policy.masterplan_path))?;
    let mut anchor_hits = Vec::new();
    collect_anchor_strings(&plan, &policy.anchor_field_names, "", &mut anchor_hits);
    candidate.anchors = anchor_hits
        .into_iter()
        .filter(|(_, anchor)| policy.under_target_path_prefix(anchor) && !policy.exempt(anchor))
        .map(|(_, anchor)| anchor)
        .collect();
    Ok(candidate)
}

/// Exact live-vs-committed Arm B set comparison. Extra or stale tuple/name hashes
/// fail closed; membership-only checks would leave preauthorized headroom.
pub fn evaluate_arm_b_baseline_exactness(
    policy: &Policy,
    baseline: &Baseline,
    candidate: &BaselineCandidate,
) -> Report {
    let live = candidate.to_baseline();
    let mut findings = Vec::new();
    let extra_tuples: Vec<&String> = baseline
        .workspace_path_dep_hashes
        .difference(&live.workspace_path_dep_hashes)
        .collect();
    let extra_names: Vec<&String> = baseline
        .dep_name_hashes
        .difference(&live.dep_name_hashes)
        .collect();
    if !extra_tuples.is_empty() {
        let bucket: Vec<String> = extra_tuples
            .iter()
            .take(8)
            .map(|digest| digest[..12].to_owned())
            .collect();
        findings.push(Finding::new(
            CODE_STALE_BASELINE_PATH,
            ARM_B,
            format!(
                "{} extra/stale digest(s) in arm_b_workspace_path_dep_hashes",
                extra_tuples.len()
            ),
            format!(
                "committed Arm B tuple hashes are not an exact match of the live collected \
                 set (digest prefix bucket: {bucket:?}); extra or preauthorized hashes are \
                 refused. Regenerate with: {}",
                policy.regeneration_command
            ),
        ));
    }
    if !extra_names.is_empty() {
        let bucket: Vec<String> = extra_names
            .iter()
            .take(8)
            .map(|digest| digest[..12].to_owned())
            .collect();
        findings.push(Finding::new(
            CODE_STALE_BASELINE_PATH,
            ARM_B,
            format!(
                "{} extra/stale digest(s) in arm_b_dep_name_hashes",
                extra_names.len()
            ),
            format!(
                "committed Arm B name hashes are not an exact match of the live collected \
                 set (digest prefix bucket: {bucket:?}); extra or preauthorized hashes are \
                 refused. Regenerate with: {}",
                policy.regeneration_command
            ),
        ));
    }
    findings.sort();
    Report {
        evaluated_arms: vec![ARM_B.to_owned()],
        evaluated_path_count: candidate.workspace_path_deps.len() + candidate.dep_names.len(),
        findings,
    }
}

/// Enforce the protected merge-base baseline as an immutable anti-expansion ceiling.
/// Candidate baseline rows may shrink, but cannot grow to waive debt introduced in the
/// same change. This check is independent of live-vs-candidate exactness: both predicates
/// are required to prevent either stale headroom or same-change laundering.
pub fn evaluate_baseline_ceiling(frozen: &Baseline, candidate: &Baseline) -> Report {
    let dimensions = [
        (
            ARM_A,
            "arm_a_path_hashes",
            &candidate.path_hashes,
            &frozen.path_hashes,
        ),
        (
            ARM_B,
            "arm_b_workspace_path_dep_hashes",
            &candidate.workspace_path_dep_hashes,
            &frozen.workspace_path_dep_hashes,
        ),
        (
            ARM_B,
            "arm_b_dep_name_hashes",
            &candidate.dep_name_hashes,
            &frozen.dep_name_hashes,
        ),
        (ARM_C, "arm_c_anchors", &candidate.anchors, &frozen.anchors),
    ];
    let mut findings = Vec::new();
    for (arm, field, candidate_values, frozen_values) in dimensions {
        let growth: Vec<&String> = candidate_values.difference(frozen_values).collect();
        if growth.is_empty() {
            continue;
        }
        let bucket: Vec<String> = growth
            .iter()
            .take(8)
            .map(|entry| {
                if entry.len() > 12 {
                    entry[..12].to_owned()
                } else {
                    (*entry).clone()
                }
            })
            .collect();
        findings.push(Finding::new(
            CODE_BASELINE_EXPANSION,
            arm,
            format!("{} new protected-baseline row(s) in {field}", growth.len()),
            format!(
                "candidate baseline grows beyond the immutable {PROTECTED_BASE_REF} merge-base \
                 ceiling (prefix bucket: {bucket:?}); remove the new target-form debt instead \
                 of adding its hash to the baseline"
            ),
        ));
    }
    findings.sort();
    Report {
        evaluated_arms: vec![ARM_A.to_owned(), ARM_B.to_owned(), ARM_C.to_owned()],
        evaluated_path_count: candidate.path_hashes.len()
            + candidate.workspace_path_dep_hashes.len()
            + candidate.dep_name_hashes.len()
            + candidate.anchors.len(),
        findings,
    }
}

/// SHRINK-ONLY regeneration guard: refuse any candidate that would ADD an entry to any
/// baseline set. Without this, the documented `--regen-baseline` workflow could launder
/// new debt into the baseline (add file + regenerate + commit both), bypassing the
/// zero-new-debt ratchet through its own tooling. Removals remain always admissible.
pub fn enforce_shrink_only(
    prior: &Baseline,
    candidate: &BaselineCandidate,
) -> Result<(), GateError> {
    let mut added: Vec<String> = Vec::new();
    for (kind, entries, prior_hashes) in [
        ("path", &candidate.paths, &prior.path_hashes),
        (
            "workspace-path-dep",
            &candidate.workspace_path_deps,
            &prior.workspace_path_dep_hashes,
        ),
        ("name", &candidate.dep_names, &prior.dep_name_hashes),
    ] {
        for entry in entries {
            if !prior_hashes.contains(&entry_digest(entry)) {
                added.push(format!("{kind}: {entry}"));
            }
        }
    }
    for anchor in &candidate.anchors {
        if !prior.anchors.contains(anchor) {
            added.push(format!("anchor: {anchor}"));
        }
    }
    if added.is_empty() {
        return Ok(());
    }
    let shown: Vec<&String> = added.iter().take(20).collect();
    Err(GateError::Input(format!(
        "{CODE_BASELINE_EXPANSION}: regeneration would ADD {} entr(y/ies) to the \
         shrink-only baseline (first {}: {shown:?}); refusing. The baseline only \
         shrinks — remove the new target-form debt instead of baselining it. \
         (Initial adoption freezes a baseline only when no committed baseline file \
         exists yet; expanding coverage is a reviewed governance act on the committed \
         file, never a regeneration side effect.)",
        added.len(),
        shown.len(),
    )))
}

/// Snapshot the current tree into a fresh baseline value (the `--regen-baseline`
/// surface). When a committed baseline already exists, the regeneration is
/// SHRINK-ONLY: any entry the candidate would add is a hard refusal
/// ([`enforce_shrink_only`]). A missing baseline file is the initial-adoption freeze.
pub fn regenerate_baseline(repo_root: &Path, policy: &Policy) -> Result<Baseline, GateError> {
    let candidate = collect_baseline_candidate(repo_root, policy)?;
    let prior_path = repo_root.join(&policy.baseline_file);
    if prior_path.is_file() {
        let prior = Baseline::from_value(&load_json(&prior_path)?)?;
        enforce_shrink_only(&prior, &candidate)?;
    }
    Ok(candidate.to_baseline())
}

/// Run all four blocking arms over the live tree and merge into one verdict.
pub fn check_live_tree(
    repo_root: &Path,
    policy: &Policy,
    baseline: &Baseline,
) -> Result<Report, GateError> {
    let frozen_reference = load_frozen_baseline_from_merge_base(repo_root)?;
    if let Some(frozen) = &frozen_reference {
        validate_candidate_policy_against_frozen(&frozen.policy, policy)?;
    }
    let paths = collect_target_prefix_paths(repo_root, policy)?;
    let manifest =
        fs::read_to_string(repo_root.join(&policy.workspace_manifest_path)).map_err(|error| {
            GateError::Io(format!("read {}: {error}", policy.workspace_manifest_path))
        })?;
    let workspace_deps = parse_workspace_dependencies(&manifest, &policy.workspace_section)?
        .into_iter()
        .map(|dep| canonicalize_workspace_dep(repo_root, "", dep))
        .collect::<Result<Vec<_>, _>>()?;
    let surface = collect_name_surface(repo_root, policy)?;
    let candidate = collect_baseline_candidate(repo_root, policy)?;
    let plan = load_json(&repo_root.join(&policy.masterplan_path))?;
    let mut reports = vec![
        evaluate_tree(policy, baseline, &paths),
        evaluate_workspace_deps(policy, baseline, &workspace_deps),
        evaluate_name_surface(policy, baseline, &surface),
        evaluate_arm_b_baseline_exactness(policy, baseline, &candidate),
        evaluate_masterplan(policy, baseline, &plan),
        evaluate_reduction_claims(policy, &plan),
        evaluate_census_snapshot_refs_at(repo_root, policy, &policy.masterplan_path, &plan),
    ];
    if let Some(frozen) = frozen_reference {
        reports.push(evaluate_baseline_ceiling(&frozen.baseline, baseline));
    }
    for (origin, artifact) in collect_claim_artifacts(repo_root, policy)? {
        reports.push(evaluate_reduction_claims_at(policy, &origin, &artifact));
        reports.push(evaluate_census_snapshot_refs_at(
            repo_root, policy, &origin, &artifact,
        ));
    }
    Ok(Report::merge(reports))
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
            "name_surface": {
                "cargo_manifest_file_name": "Cargo.toml",
                "buck_file_names": ["BUCK"],
                "dependency_sections": ["dependencies", "dev-dependencies", "build-dependencies"]
            },
            "masterplan": {
                "path": "specs/masterplan.json",
                "anchor_field_names": ["source_anchors", "evidence_anchor", "anchor"]
            },
            "reduction_claims": {
                "claim_field": "claim",
                "claim_values": ["net_target_surface_reduction"],
                "required_fields": ["before_count", "after_count", "census_snapshot_ref"],
                "before_count_field": "before_count",
                "after_count_field": "after_count",
                "census_snapshot_ref_field": "census_snapshot_ref",
                "census_snapshot_schema": "ci-reorg-target-debt-census-snapshot.v1",
                "census_snapshot_claim_field": "measurement_claim",
                "scan_roots": ["evidence", "registry"]
            }
        })
    }

    fn test_policy() -> Policy {
        Policy::from_value(&test_policy_value()).unwrap()
    }

    fn baseline_with_paths(paths: &[&str]) -> Baseline {
        Baseline {
            path_hashes: paths.iter().map(|p| entry_digest(p)).collect(),
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
    fn policy_rejects_empty_load_bearing_collections_and_empty_entries() {
        // An empty anchor/claim/required-field collection silently disables an arm
        // while the report still lists it as evaluated — refuse at load time.
        for (section, key) in [
            ("masterplan", "anchor_field_names"),
            ("reduction_claims", "claim_values"),
            ("reduction_claims", "required_fields"),
            ("name_surface", "dependency_sections"),
        ] {
            let mut value = test_policy_value();
            value[section][key] = json!([]);
            assert!(
                Policy::from_value(&value).is_err(),
                "empty {section}.{key} must fail closed"
            );
        }
        // Empty-string entries are refused everywhere; an empty exemption prefix would
        // exempt EVERYTHING.
        let mut value = test_policy_value();
        value["exemptions"]["path_prefixes"] = json!([""]);
        assert!(Policy::from_value(&value).is_err());
        // The count fields must be covered by required_fields, or the numeric proof
        // could be skipped entirely.
        let mut value = test_policy_value();
        value["reduction_claims"]["required_fields"] = json!(["census_snapshot_ref"]);
        assert!(Policy::from_value(&value).is_err());
    }

    #[test]
    fn candidate_policy_cannot_narrow_or_repoint_frozen_detection() {
        let frozen = test_policy();

        let mut narrowed = test_policy_value();
        narrowed["target_forms"]["name_prefixes"] = json!(["cloud-", "oya-", "oya_"]);
        let narrowed = Policy::from_value(&narrowed).unwrap();
        assert!(validate_candidate_policy_against_frozen(&frozen, &narrowed).is_err());

        let mut exempted = test_policy_value();
        exempted["exemptions"]["path_prefixes"] = json!(["cloud/newly-exempt/"]);
        let exempted = Policy::from_value(&exempted).unwrap();
        assert!(validate_candidate_policy_against_frozen(&frozen, &exempted).is_err());

        let mut repointed = test_policy_value();
        repointed["baseline"]["file"] = json!("candidate-controlled-baseline.json");
        let repointed = Policy::from_value(&repointed).unwrap();
        assert!(validate_candidate_policy_against_frozen(&frozen, &repointed).is_err());
    }

    #[test]
    fn frozen_snapshot_bootstrap_is_only_the_exact_t2_anchor() {
        let root = std::env::temp_dir().join(format!(
            "rtd-bootstrap-snapshot-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("ci/facade/reorg-target-debt")).unwrap();
        let write_snapshot = |merge_base: &str| {
            fs::write(
                root.join(FROZEN_REFERENCE_PATH),
                serde_json::to_vec_pretty(&json!({
                    "schema": "ci-reorg-target-debt-merge-base-snapshot.v1",
                    "base_ref": PROTECTED_BASE_REF,
                    "merge_base": merge_base,
                    "missing_at_merge_base": true,
                    "policy": null,
                    "baseline": null,
                }))
                .unwrap(),
            )
            .unwrap();
        };
        write_snapshot(INITIAL_ADOPTION_BASE_SHA);
        assert!(
            load_frozen_baseline_from_merge_base(&root)
                .unwrap()
                .is_none()
        );
        write_snapshot("1111111111111111111111111111111111111111");
        assert!(load_frozen_baseline_from_merge_base(&root).is_err());
        fs::remove_dir_all(root).unwrap();
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
        assert!(
            report.findings[0].subject.contains("1 stale digest(s)"),
            "stale rows are reported as a count over digests, never literal removed paths: {}",
            report.findings[0].subject
        );
        assert!(
            !report.findings[0].subject.contains("gone.rs")
                && !report.findings[0].detail.contains("gone.rs"),
            "the stale-row report must not resurrect the literal removed path"
        );
    }

    #[test]
    fn baseline_refuses_non_digest_entries() {
        let value = json!({
            "arm_a_path_hashes": ["cloud/legacy/kernel.rs"],
            "arm_b_workspace_path_dep_hashes": [],
            "arm_b_dep_name_hashes": [],
            "arm_c_anchors": [],
        });
        assert!(
            Baseline::from_value(&value).is_err(),
            "a literal path string in a digest set fails closed — the committed baseline \
             must never carry literal target-prefix names"
        );
        let digest = entry_digest("cloud/legacy/kernel.rs");
        let value = json!({
            "arm_a_path_hashes": [digest],
            "arm_b_workspace_path_dep_hashes": [],
            "arm_b_dep_name_hashes": [],
            "arm_c_anchors": [],
        });
        let baseline = Baseline::from_value(&value).unwrap();
        assert!(
            baseline
                .path_hashes
                .contains(&entry_digest("cloud/legacy/kernel.rs"))
        );
    }

    #[test]
    fn entry_digest_is_deterministic_lowercase_sha256_hex() {
        let digest = entry_digest("docs/example.md");
        assert_eq!(digest.len(), 64);
        assert!(
            digest
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        );
        assert_eq!(digest, entry_digest("docs/example.md"));
        assert_ne!(digest, entry_digest("docs/example.md "));
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
        let deps = parse_workspace_dependencies(manifest, "workspace.dependencies").unwrap();
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
            workspace_path_dep_hashes: [
                ("", "storage-legacy", "cloud/legacy-storage"),
                ("", "subsection-dep", "oya/legacy-subsection"),
            ]
            .iter()
            .map(|(origin, name, dest)| workspace_path_dep_digest(origin, name, dest))
            .collect(),
            dep_name_hashes: ["oya-shiny-new-kernel"]
                .iter()
                .map(|name| name_decl_digest("", name))
                .collect(),
            ..Baseline::default()
        };
        let report = evaluate_workspace_deps(&policy, &baseline, &deps);
        assert_eq!(
            report.verdict(),
            Verdict::Green,
            "baselined entries are shrink-only inventory"
        );
    }

    #[test]
    fn arm_b_parses_single_quoted_relative_and_unparseable_paths_fail_closed() {
        let policy = test_policy();
        let manifest = concat!(
            "[workspace.dependencies]\n",
            "legacy-single = { path = 'cloud/legacy-single-quoted' }\n",
            "legacy-relative = { path = \"./cloud/legacy-relative\" }\n",
            "legacy-mystery = { path = unquoted_nonsense }\n",
        );
        let error = parse_workspace_dependencies(manifest, "workspace.dependencies").unwrap_err();
        assert!(
            error.to_string().contains(CODE_DEP_PATH_UNPARSEABLE),
            "an unreadable path value must fail closed, not parse: {error}"
        );

        let readable = concat!(
            "[workspace.dependencies]\n",
            "legacy-single = { path = 'cloud/legacy-single-quoted' }\n",
            "legacy-relative = { path = \"./cloud/legacy-relative\" }\n",
        );
        let deps = parse_workspace_dependencies(readable, "workspace.dependencies").unwrap();
        assert_eq!(deps.len(), 2);
        let single = deps
            .iter()
            .find(|dep| dep.name == "legacy-single")
            .expect("legacy-single dependency");
        let relative = deps
            .iter()
            .find(|dep| dep.name == "legacy-relative")
            .expect("legacy-relative dependency");
        assert_eq!(single.path, "cloud/legacy-single-quoted");
        assert_eq!(relative.path, "./cloud/legacy-relative");

        let report = evaluate_workspace_deps(&policy, &Baseline::default(), &deps);
        assert_eq!(
            codes(&report),
            vec![CODE_NEW_TARGET_PATH_DEP, CODE_NEW_TARGET_PATH_DEP]
        );
        // `./cloud/...` is normalized before the prefix check and baselined under its
        // normalized spelling.
        let baseline = Baseline {
            workspace_path_dep_hashes: [
                ("", "legacy-single", "cloud/legacy-single-quoted"),
                ("", "legacy-relative", "cloud/legacy-relative"),
            ]
            .iter()
            .map(|(origin, name, dest)| workspace_path_dep_digest(origin, name, dest))
            .collect(),
            ..Baseline::default()
        };
        let report = evaluate_workspace_deps(&policy, &baseline, &deps);
        assert_eq!(report.verdict(), Verdict::Green);
    }

    #[test]
    fn try_normalize_rel_path_resolves_dot_and_parent_components() {
        assert_eq!(
            try_normalize_rel_path("", "./cloud/x").as_deref(),
            Some("cloud/x")
        );
        assert_eq!(
            try_normalize_rel_path("libs/foo", "../../cloud/x").as_deref(),
            Some("cloud/x")
        );
        assert_eq!(
            try_normalize_rel_path("libs/foo", "./bar").as_deref(),
            Some("libs/foo/bar")
        );
        assert_eq!(
            try_normalize_rel_path("", "../outside"),
            None,
            "a path escaping the repo root fails closed"
        );
        assert_eq!(
            try_normalize_rel_path("", "/checkout/cloud/y"),
            None,
            "unix-absolute destinations fail closed"
        );
        assert_eq!(
            try_normalize_rel_path("", "C:/checkout/cloud/y"),
            None,
            "windows-absolute destinations fail closed"
        );
        assert_eq!(
            try_normalize_rel_path("", "cloud\\y"),
            None,
            "backslash-ambiguous destinations fail closed"
        );
    }

    #[test]
    fn name_surface_refuses_new_target_names_and_member_path_deps() {
        let policy = test_policy();
        let manifest = concat!(
            "[package]\n",
            "name = \"cloud-synthetic-new-kernel\"\n\n",
            "[[bin]]\n",
            "name = \"oya_synthetic_new_bin\"\n\n",
            "[dependencies]\n",
            "legacy-estate = { path = \"../../cloud/legacy-estate-crate\" }\n\n",
            "[dev-dependencies.legacy-dev]\n",
            "path = \"../../oya/legacy-dev-crate\"\n",
        );
        let facts = parse_manifest_facts(
            manifest,
            &["dependencies".to_owned(), "dev-dependencies".to_owned()],
        )
        .unwrap();
        assert_eq!(
            facts.package_name.as_deref(),
            Some("cloud-synthetic-new-kernel")
        );
        assert_eq!(facts.bin_names, vec!["oya_synthetic_new_bin"]);
        assert_eq!(facts.path_deps.len(), 2);

        let origin = "libs/synthetic/Cargo.toml".to_owned();
        let mut surface = NameSurface::default();
        if let Some(name) = facts.package_name.clone() {
            surface.names.push(NameDecl {
                name,
                origin: origin.clone(),
            });
        }
        for name in facts.bin_names.clone() {
            surface.names.push(NameDecl {
                name,
                origin: origin.clone(),
            });
        }
        for dep in facts.path_deps.clone() {
            surface.member_path_deps.push((origin.clone(), dep));
        }
        let report = evaluate_name_surface(&policy, &Baseline::default(), &surface);
        assert_eq!(
            codes(&report),
            vec![
                CODE_NEW_TARGET_NAME,
                CODE_NEW_TARGET_NAME,
                CODE_NEW_TARGET_PATH_DEP,
                CODE_NEW_TARGET_PATH_DEP,
            ],
            "findings: {:#?}",
            report.findings
        );
        // Baselined names and member dep paths (normalized against the declaring
        // manifest's directory) are shrink-only inventory.
        let baseline = Baseline {
            dep_name_hashes: ["cloud-synthetic-new-kernel", "oya_synthetic_new_bin"]
                .iter()
                .map(|name| name_decl_digest(&origin, name))
                .collect(),
            workspace_path_dep_hashes: [
                (
                    "libs/synthetic/Cargo.toml",
                    "legacy-estate",
                    "cloud/legacy-estate-crate",
                ),
                (
                    "libs/synthetic/Cargo.toml",
                    "legacy-dev",
                    "oya/legacy-dev-crate",
                ),
            ]
            .iter()
            .map(|(origin, name, dest)| workspace_path_dep_digest(origin, name, dest))
            .collect(),
            ..Baseline::default()
        };
        let report = evaluate_name_surface(&policy, &baseline, &surface);
        assert_eq!(
            report.verdict(),
            Verdict::Green,
            "findings: {:#?}",
            report.findings
        );
    }

    #[test]
    fn buck_target_names_parse_both_quote_styles_and_skip_non_literals() {
        let text = concat!(
            "rust_binary(\n",
            "    name = \"cloud-synthetic-buck-bin\",\n",
            ")\n",
            "rust_library(\n",
            "    name = 'oya-synthetic-buck-lib',  # trailing comment\n",
            ")\n",
            "weird(\n",
            "    name = \"prefix-\" + suffix,\n",
            ")\n",
        );
        assert_eq!(
            parse_buck_target_names(text),
            vec!["cloud-synthetic-buck-bin", "oya-synthetic-buck-lib"]
        );
    }

    #[test]
    fn name_baseline_is_bound_to_declaration_origin() {
        let policy = test_policy();
        let baseline = Baseline {
            dep_name_hashes: [name_decl_digest("libs/already/BUCK", "oya-existing-target")]
                .into_iter()
                .collect(),
            ..Baseline::default()
        };
        let surface = NameSurface {
            names: vec![NameDecl {
                name: "oya-existing-target".to_owned(),
                origin: "libs/new/BUCK".to_owned(),
            }],
            ..NameSurface::default()
        };
        let report = evaluate_name_surface(&policy, &baseline, &surface);
        assert_eq!(codes(&report), vec![CODE_NEW_TARGET_NAME]);
    }

    #[test]
    fn rust_module_parser_collects_real_nested_and_raw_mod_items_only() {
        let source = r#"
            // mod cloud_comment;
            const TEXT: &str = "mod oya_string;";
            mod cloud_top;
            mod safe {
                mod r#oya_nested {}
            }
            macro_rules! emits_tokens { () => { mod cloud_macro; } }
        "#;
        assert_eq!(
            parse_rust_module_declarations(source).unwrap(),
            vec![
                ("cloud_top".to_owned(), String::new()),
                ("safe".to_owned(), String::new()),
                ("oya_nested".to_owned(), "safe".to_owned()),
            ]
        );
    }

    #[test]
    fn name_surface_collector_feeds_rust_modules_into_arm_b() {
        let root = std::env::temp_dir().join(format!(
            "rtd-rust-module-surface-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("libs/safe/src")).unwrap();
        fs::write(
            root.join("libs/safe/src/lib.rs"),
            "mod cloud_new;\nconst TEXT: &str = \"mod oya_not_real;\";\n",
        )
        .unwrap();
        let policy = test_policy();
        let surface = collect_name_surface(&root, &policy).unwrap();
        let report = evaluate_name_surface(&policy, &Baseline::default(), &surface);
        assert_eq!(codes(&report), vec![CODE_NEW_TARGET_NAME]);
        assert!(report.findings[0].subject.contains("cloud_new"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn regeneration_refuses_baseline_expansion_and_allows_shrink() {
        let prior = Baseline {
            path_hashes: ["cloud/legacy/kernel.rs", "cloud/legacy/gone.rs"]
                .iter()
                .map(|p| entry_digest(p))
                .collect(),
            anchors: ["cloud/legacy/kernel.rs"]
                .iter()
                .map(|a| (*a).to_owned())
                .collect(),
            ..Baseline::default()
        };
        // Shrink: strictly fewer entries — admissible.
        let shrink = BaselineCandidate {
            paths: ["cloud/legacy/kernel.rs"]
                .iter()
                .map(|p| (*p).to_owned())
                .collect(),
            anchors: ["cloud/legacy/kernel.rs"]
                .iter()
                .map(|a| (*a).to_owned())
                .collect(),
            ..BaselineCandidate::default()
        };
        assert!(enforce_shrink_only(&prior, &shrink).is_ok());
        // Expansion: a NEW path rides along with a legitimate removal — refused.
        let expansion = BaselineCandidate {
            paths: ["cloud/legacy/kernel.rs", "cloud/newly-minted/lib.rs"]
                .iter()
                .map(|p| (*p).to_owned())
                .collect(),
            anchors: ["cloud/legacy/kernel.rs"]
                .iter()
                .map(|a| (*a).to_owned())
                .collect(),
            ..BaselineCandidate::default()
        };
        let error = enforce_shrink_only(&prior, &expansion).unwrap_err();
        assert!(
            error.to_string().contains(CODE_BASELINE_EXPANSION)
                && error.to_string().contains("cloud/newly-minted/lib.rs"),
            "the refusal must carry the code and name the attempted addition: {error}"
        );
        // New anchors are expansions too.
        let anchor_expansion = BaselineCandidate {
            paths: ["cloud/legacy/kernel.rs"]
                .iter()
                .map(|p| (*p).to_owned())
                .collect(),
            anchors: ["cloud/legacy/kernel.rs", "oya/new-anchor"]
                .iter()
                .map(|a| (*a).to_owned())
                .collect(),
            ..BaselineCandidate::default()
        };
        assert!(enforce_shrink_only(&prior, &anchor_expansion).is_err());
    }

    #[test]
    fn arm_d_refuses_non_numeric_and_non_reducing_counts() {
        let policy = test_policy();
        let growth = json!({
            "claims": [{
                "claim": "net_target_surface_reduction",
                "before_count": 10,
                "after_count": 20,
                "census_snapshot_ref": "reorg-census@deadbeef"
            }]
        });
        let report = evaluate_reduction_claims(&policy, &growth);
        assert_eq!(codes(&report), vec![CODE_UNPROVEN_REDUCTION_CLAIM]);
        assert!(report.findings[0].detail.contains("does not reduce"));

        let stringly = json!({
            "claims": [{
                "claim": "net_target_surface_reduction",
                "before_count": "many",
                "after_count": "few",
                "census_snapshot_ref": "reorg-census@deadbeef"
            }]
        });
        let report = evaluate_reduction_claims(&policy, &stringly);
        assert_eq!(codes(&report), vec![CODE_UNPROVEN_REDUCTION_CLAIM]);

        let equal = json!({
            "claims": [{
                "claim": "net_target_surface_reduction",
                "before_count": 10,
                "after_count": 10,
                "census_snapshot_ref": "reorg-census@deadbeef"
            }]
        });
        let report = evaluate_reduction_claims(&policy, &equal);
        assert_eq!(codes(&report), vec![CODE_UNPROVEN_REDUCTION_CLAIM]);
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
            anchors: ["oya/synthetic-legacy-module"]
                .iter()
                .map(|p| (*p).to_owned())
                .collect(),
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

        let numeric_ref = json!({
            "claims": [{
                "claim": "net_target_surface_reduction",
                "before_count": 4981,
                "after_count": 4600,
                "census_snapshot_ref": 123
            }]
        });
        let report = evaluate_reduction_claims(&policy, &numeric_ref);
        assert_eq!(codes(&report), vec![CODE_UNPROVEN_REDUCTION_CLAIM]);
        assert!(
            report.findings[0]
                .detail
                .contains("must be a non-empty string ref")
        );
    }

    #[test]
    fn arm_d_census_ref_must_resolve_and_bind_the_claim_measurement() {
        let root = std::env::temp_dir().join(format!(
            "rtd-census-snapshot-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("evidence")).unwrap();
        let policy = test_policy();
        let claim = json!({
            "claims": [{
                "claim": "net_target_surface_reduction",
                "before_count": 12,
                "after_count": 9,
                "census_snapshot_ref": "evidence/census.json"
            }]
        });

        let missing =
            evaluate_census_snapshot_refs_at(&root, &policy, "evidence/claim.json", &claim);
        assert_eq!(codes(&missing), vec![CODE_UNPROVEN_REDUCTION_CLAIM]);

        fs::write(
            root.join("evidence/census.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "ci-reorg-target-debt-census-snapshot.v1",
                "measurement_claim": "net_target_surface_reduction",
                "before_count": 12,
                "after_count": 10
            }))
            .unwrap(),
        )
        .unwrap();
        let mismatched =
            evaluate_census_snapshot_refs_at(&root, &policy, "evidence/claim.json", &claim);
        assert_eq!(codes(&mismatched), vec![CODE_UNPROVEN_REDUCTION_CLAIM]);

        fs::write(
            root.join("evidence/census.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "ci-reorg-target-debt-census-snapshot.v1",
                "measurement_claim": "net_target_surface_reduction",
                "before_count": 12,
                "after_count": 9
            }))
            .unwrap(),
        )
        .unwrap();
        let bound = evaluate_census_snapshot_refs_at(&root, &policy, "evidence/claim.json", &claim);
        assert_eq!(bound.verdict(), Verdict::Green);
        fs::remove_dir_all(&root).unwrap();
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
        assert_eq!(
            report.verdict(),
            Verdict::Green,
            "a remediation record resolves the finding"
        );
        assert_eq!(
            report.findings.len(),
            1,
            "the finding stays reported as evidence"
        );
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
        missing["commits"][0]
            .as_object_mut()
            .unwrap()
            .remove("added_paths");
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
    fn audit_capture_contract_is_merge_commit_aware() {
        let policy: Value =
            serde_json::from_str(include_str!("../reorg-target-debt-policy.json")).unwrap();
        let instructions = policy["audit_mode"]["commit_set_materialization"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            instructions.contains("git diff-tree -m "),
            "merge commits must be diffed against each parent: {instructions}"
        );
        assert!(
            instructions.contains("treats output as a set"),
            "per-parent duplicate paths must collapse deterministically: {instructions}"
        );
    }

    #[test]
    fn audit_fails_closed_on_malformed_dep_facts_and_unbound_range() {
        let policy = test_policy();
        // A malformed added_workspace_path_deps element must never silently become an
        // empty dependency that audits green.
        let mut malformed = audit_input(json!([clean_commit("b2b2b2")]), json!([]));
        malformed["commits"][0]["added_workspace_path_deps"] = json!(["not-an-object"]);
        let error = audit_interval(&policy, &malformed).unwrap_err();
        assert!(
            error.to_string().contains("RTD_AUDIT_INPUT_INVALID"),
            "{error}"
        );
        let mut missing_path = audit_input(json!([clean_commit("b2b2b2")]), json!([]));
        missing_path["commits"][0]["added_workspace_path_deps"] = json!([{ "name": "legacy-dep" }]);
        assert!(audit_interval(&policy, &missing_path).is_err());
        // A capture whose last commit is not range.to is not bound to the declared
        // range: `complete: true` alone must not audit green.
        let unbound = audit_input(json!([clean_commit("a1a1a1")]), json!([]));
        let error = audit_interval(&policy, &unbound).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("not bound to the declared range"),
            "{error}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn tree_walk_records_directory_symlinks_as_leaves_instead_of_following() {
        let root = std::env::temp_dir().join(format!(
            "rtd-symlink-walk-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("cloud/real")).unwrap();
        fs::write(root.join("cloud/real/file.rs"), "// synthetic").unwrap();
        // A directory symlink pointing back up would loop forever if followed.
        std::os::unix::fs::symlink("..", root.join("cloud/loop")).unwrap();

        let policy = test_policy();
        let paths = collect_target_prefix_paths(&root, &policy).unwrap();
        assert!(paths.contains("cloud/real/file.rs"));
        assert!(
            paths.contains("cloud/loop"),
            "a tracked directory symlink under a target prefix is itself a leaf entry: {paths:?}"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn arm_b_new_edge_to_baselined_destination_from_another_manifest_is_refused() {
        let policy = test_policy();
        let dest = "cloud/legacy-estate-crate";
        let baseline = Baseline {
            workspace_path_dep_hashes: [workspace_path_dep_digest(
                "libs/already-wired/Cargo.toml",
                "legacy-estate",
                dest,
            )]
            .into_iter()
            .collect(),
            ..Baseline::default()
        };
        let surface = NameSurface {
            names: Vec::new(),
            member_path_deps: vec![(
                "libs/new-consumer/Cargo.toml".to_owned(),
                WorkspaceDep {
                    name: "legacy-estate".to_owned(),
                    path: "../../cloud/legacy-estate-crate".to_owned(),
                    path_unparseable: false,
                },
            )],
        };
        let report = evaluate_name_surface(&policy, &baseline, &surface);
        assert_eq!(
            codes(&report),
            vec![CODE_NEW_TARGET_PATH_DEP],
            "a new edge from a different manifest to an already-baselined destination is NEW debt: {:#?}",
            report.findings
        );
        // The original edge remains inventory.
        let original = NameSurface {
            names: Vec::new(),
            member_path_deps: vec![(
                "libs/already-wired/Cargo.toml".to_owned(),
                WorkspaceDep {
                    name: "legacy-estate".to_owned(),
                    path: "../../cloud/legacy-estate-crate".to_owned(),
                    path_unparseable: false,
                },
            )],
        };
        let report = evaluate_name_surface(&policy, &baseline, &original);
        assert_eq!(report.verdict(), Verdict::Green);
    }

    #[test]
    fn parse_manifest_facts_collects_target_qualified_dependency_subtables() {
        let manifest = concat!(
            "[package]\n",
            "name = \"fine-crate\"\n\n",
            "[target.'cfg(unix)'.dependencies.hidden-estate]\n",
            "path = \"../../cloud/hidden-estate\"\n\n",
            "[target.x86_64-unknown-linux-gnu.dev-dependencies]\n",
            "legacy-target-dev = { path = \"../../oya/legacy-target-dev\" }\n\n",
            "[profile.release]\n",
            "lto = true\n",
        );
        let facts = parse_manifest_facts(
            manifest,
            &[
                "dependencies".to_owned(),
                "dev-dependencies".to_owned(),
                "build-dependencies".to_owned(),
            ],
        )
        .unwrap();
        assert_eq!(
            facts.path_deps.len(),
            2,
            "target-qualified tables must be collected: {facts:?}"
        );
        assert_eq!(facts.path_deps[0].name, "hidden-estate");
        assert_eq!(facts.path_deps[0].path, "../../cloud/hidden-estate");
        assert_eq!(facts.path_deps[1].name, "legacy-target-dev");
        assert_eq!(facts.path_deps[1].path, "../../oya/legacy-target-dev");

        let origin = "libs/synthetic/Cargo.toml".to_owned();
        let surface = NameSurface {
            names: Vec::new(),
            member_path_deps: facts
                .path_deps
                .into_iter()
                .map(|dep| (origin.clone(), dep))
                .collect(),
        };
        let report = evaluate_name_surface(&test_policy(), &Baseline::default(), &surface);
        assert_eq!(
            codes(&report),
            vec![CODE_NEW_TARGET_PATH_DEP, CODE_NEW_TARGET_PATH_DEP],
            "target-qualified path edges into the estate fail closed: {:#?}",
            report.findings
        );
    }

    #[test]
    fn audit_normalizes_captured_dep_paths_against_origin_and_refuses_unnormalizable() {
        let policy = test_policy();
        let planted = json!({
            "sha": "b2b2b2",
            "added_paths": ["docs/fine.md"],
            "added_workspace_path_deps": [{
                "name": "legacy-estate",
                "path": "../../cloud/planted-estate",
                "origin": "libs/synthetic/Cargo.toml"
            }],
            "added_dep_names": [],
            "added_evidence_anchors": [],
        });
        let input = audit_input(json!([planted]), json!([]));
        let report = audit_interval(&policy, &input).unwrap();
        assert_eq!(report.verdict(), Verdict::Red);
        assert_eq!(report.findings.len(), 1);
        assert!(
            report.findings[0].subject.contains("cloud/planted-estate"),
            "audit must report the normalized destination, not the raw spelling: {}",
            report.findings[0].subject
        );

        let missing_origin = json!({
            "sha": "b2b2b2",
            "added_paths": ["docs/fine.md"],
            "added_workspace_path_deps": [{
                "name": "legacy-estate",
                "path": "../../cloud/planted-estate"
            }],
            "added_dep_names": [],
            "added_evidence_anchors": [],
        });
        let error =
            audit_interval(&policy, &audit_input(json!([missing_origin]), json!([]))).unwrap_err();
        assert!(
            error.to_string().contains("RTD_AUDIT_INPUT_INVALID")
                && error.to_string().contains("origin"),
            "{error}"
        );

        let escaped = json!({
            "sha": "b2b2b2",
            "added_paths": ["docs/fine.md"],
            "added_workspace_path_deps": [{
                "name": "legacy-estate",
                "path": "../../../../outside",
                "origin": "libs/synthetic/Cargo.toml"
            }],
            "added_dep_names": [],
            "added_evidence_anchors": [],
        });
        let error = audit_interval(&policy, &audit_input(json!([escaped]), json!([]))).unwrap_err();
        assert!(
            error.to_string().contains("RTD_AUDIT_INPUT_INVALID")
                && error.to_string().contains("not a normalizable"),
            "{error}"
        );
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
    #[test]
    fn parse_manifest_facts_collects_spaced_quoted_and_escaped_dependency_spellings() {
        let manifest = concat!(
            "[package]\n",
            "name = \"fine-crate\"\n\n",
            "[target . 'cfg(unix)' . dependencies . hidden]\n",
            "path = \"../../cloud/hidden-estate\"\n\n",
            "[dependencies]\n",
            "\"quoted-dev\" = { path = \"../../oya/quoted-estate\" }\n",
            "escaped-build = { path = \"../../cloud/esca\\u0070ed-estate\" }\n",
            "[dev-dependencies]\n",
            "fine-crate = { version = \"1\" }\n",
            "[build-dependencies]\n",
            "legacy-build = { path = \"../../cloud/legacy-build\" }\n",
        );
        let facts = parse_manifest_facts(
            manifest,
            &[
                "dependencies".to_owned(),
                "dev-dependencies".to_owned(),
                "build-dependencies".to_owned(),
            ],
        )
        .unwrap();
        let names: BTreeSet<&str> = facts
            .path_deps
            .iter()
            .map(|dep| dep.name.as_str())
            .collect();
        assert!(
            names.contains("hidden")
                && names.contains("quoted-dev")
                && names.contains("escaped-build")
                && names.contains("legacy-build"),
            "spaced/quoted/escaped Cargo spellings must be collected: {facts:?}"
        );
        let escaped = facts
            .path_deps
            .iter()
            .find(|dep| dep.name == "escaped-build")
            .unwrap();
        assert_eq!(escaped.path, "../../cloud/escaped-estate");
        let surface = NameSurface {
            names: Vec::new(),
            member_path_deps: facts
                .path_deps
                .into_iter()
                .map(|dep| ("libs/synthetic/Cargo.toml".to_owned(), dep))
                .collect(),
        };
        let report = evaluate_name_surface(&test_policy(), &Baseline::default(), &surface);
        assert!(
            codes(&report).contains(&CODE_NEW_TARGET_PATH_DEP),
            "decoded escaped/quoted target edges fail closed: {:#?}",
            report.findings
        );
    }

    #[test]
    fn parse_manifest_facts_fails_closed_on_unsupported_dependency_shape() {
        let manifest = concat!(
            "[package]\n",
            "name = \"fine-crate\"\n",
            "[dependencies]\n",
            "legacy = [\"not-a-supported-dep-value\"]\n",
        );
        let error = parse_manifest_facts(manifest, &["dependencies".to_owned()]).unwrap_err();
        assert!(
            error.to_string().contains(CODE_DEP_PATH_UNPARSEABLE),
            "{error}"
        );
    }

    #[test]
    fn evaluate_name_surface_fails_closed_on_repo_escape_instead_of_raw_fallback() {
        let policy = test_policy();
        let surface = NameSurface {
            names: Vec::new(),
            member_path_deps: vec![(
                "libs/synthetic/Cargo.toml".to_owned(),
                WorkspaceDep {
                    name: "escape-reentry".to_owned(),
                    path: "../../../../outside/then/cloud/hidden".to_owned(),
                    path_unparseable: false,
                },
            )],
        };
        let report = evaluate_name_surface(&policy, &Baseline::default(), &surface);
        assert_eq!(codes(&report), vec![CODE_DEP_PATH_UNPARSEABLE]);
    }

    #[cfg(unix)]
    #[test]
    fn collector_canonicalizes_non_target_symlink_into_target_prefix() {
        let root = std::env::temp_dir().join(format!(
            "rtd-symlink-canon-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("cloud/hidden-estate")).unwrap();
        fs::write(
            root.join("cloud/hidden-estate/Cargo.toml"),
            "[package]\nname = \"hidden\"\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("libs/alias")).unwrap();
        std::os::unix::fs::symlink(
            root.join("cloud/hidden-estate"),
            root.join("libs/alias/via-link"),
        )
        .unwrap();
        fs::create_dir_all(root.join("libs/consumer")).unwrap();
        fs::write(
            root.join("libs/consumer/Cargo.toml"),
            concat!(
                "[package]\n",
                "name = \"consumer\"\n",
                "[dependencies]\n",
                "hidden = { path = \"../alias/via-link\" }\n",
            ),
        )
        .unwrap();
        let dest =
            canonicalize_declared_dep_path(&root, "libs/consumer/Cargo.toml", "../alias/via-link")
                .unwrap();
        assert_eq!(dest, "cloud/hidden-estate");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn audit_rejects_absolute_windows_and_noncanonical_origins() {
        let policy = test_policy();
        for (origin, path) in [
            ("/checkout/libs/x/Cargo.toml", "../../cloud/y"),
            ("C:/checkout/libs/x/Cargo.toml", "../../cloud/y"),
            ("libs/../libs/x/Cargo.toml", "../../cloud/y"),
            ("libs/x/not-cargo.toml", "../../cloud/y"),
            ("libs/synthetic/Cargo.toml", "/checkout/cloud/y"),
            ("libs/synthetic/Cargo.toml", "C:/checkout/cloud/y"),
        ] {
            let planted = json!({
                "sha": "b2b2b2",
                "added_paths": ["docs/fine.md"],
                "added_workspace_path_deps": [{
                    "name": "legacy-estate",
                    "path": path,
                    "origin": origin
                }],
                "added_dep_names": [],
                "added_evidence_anchors": [],
            });
            let error =
                audit_interval(&policy, &audit_input(json!([planted]), json!([]))).unwrap_err();
            assert!(
                error.to_string().contains("RTD_AUDIT_INPUT_INVALID"),
                "origin={origin:?} path={path:?} error={error}"
            );
        }
    }

    #[test]
    fn audit_evaluates_name_only_dependency_object_independently() {
        let policy = test_policy();
        let planted = json!({
            "sha": "b2b2b2",
            "added_paths": ["docs/fine.md"],
            "added_workspace_path_deps": [{
                "name": "oya-new",
                "path": ""
            }],
            "added_dep_names": [],
            "added_evidence_anchors": [],
        });
        let report = audit_interval(&policy, &audit_input(json!([planted]), json!([]))).unwrap();
        assert_eq!(report.verdict(), Verdict::Red);
        assert_eq!(report.findings.len(), 1);
        assert!(
            report.findings[0].subject.contains("oya-new"),
            "{}",
            report.findings[0].subject
        );
    }

    #[test]
    fn arm_b_baseline_exactness_rejects_extra_tuple_and_name_hashes() {
        let policy = test_policy();
        let candidate = BaselineCandidate {
            workspace_path_deps: ["libs/a/Cargo.toml\0legacy\0cloud/legacy"]
                .iter()
                .map(|entry| (*entry).to_owned())
                .collect(),
            dep_names: [name_decl_key("libs/a/BUCK", "cloud-legacy")]
                .into_iter()
                .collect(),
            ..BaselineCandidate::default()
        };
        let extra_tuple = workspace_path_dep_digest("libs/b/Cargo.toml", "sneaky", "cloud/sneaky");
        let extra_name = name_decl_digest("libs/b/BUCK", "oya-preauthorized");
        let mut baseline = candidate.to_baseline();
        baseline.workspace_path_dep_hashes.insert(extra_tuple);
        baseline.dep_name_hashes.insert(extra_name);
        let report = evaluate_arm_b_baseline_exactness(&policy, &baseline, &candidate);
        assert_eq!(
            codes(&report),
            vec![CODE_STALE_BASELINE_PATH, CODE_STALE_BASELINE_PATH]
        );
    }

    #[test]
    fn regeneration_refuses_same_change_edge_and_hash_expansion() {
        let prior = Baseline {
            workspace_path_dep_hashes: [workspace_path_dep_digest(
                "libs/already/Cargo.toml",
                "legacy",
                "cloud/legacy",
            )]
            .into_iter()
            .collect(),
            ..Baseline::default()
        };
        let expansion = BaselineCandidate {
            workspace_path_deps: [
                "libs/already/Cargo.toml\0legacy\0cloud/legacy".to_owned(),
                "libs/new/Cargo.toml\0legacy\0cloud/legacy".to_owned(),
            ]
            .into_iter()
            .collect(),
            ..BaselineCandidate::default()
        };
        let error = enforce_shrink_only(&prior, &expansion).unwrap_err();
        assert!(
            error.to_string().contains(CODE_BASELINE_EXPANSION)
                && error.to_string().contains("libs/new/Cargo.toml"),
            "{error}"
        );
    }

    #[test]
    fn live_gate_refuses_same_change_edge_plus_matching_candidate_baseline_hash() {
        let root = std::env::temp_dir().join(format!(
            "rtd-frozen-baseline-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("ci/facade/reorg-target-debt")).unwrap();
        fs::create_dir_all(root.join("specs")).unwrap();
        fs::write(
            root.join(POLICY_PATH),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&test_policy_value()).unwrap()
            ),
        )
        .unwrap();
        fs::write(
            root.join("ci/facade/reorg-target-debt/reorg-target-debt-baseline.json"),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&Baseline::default().to_json()).unwrap()
            ),
        )
        .unwrap();
        fs::write(
            root.join(FROZEN_REFERENCE_PATH),
            serde_json::to_vec_pretty(&json!({
                "schema": "ci-reorg-target-debt-merge-base-snapshot.v1",
                "base_ref": PROTECTED_BASE_REF,
                "merge_base": "1111111111111111111111111111111111111111",
                "missing_at_merge_base": false,
                "policy": test_policy_value(),
                "baseline": Baseline::default().to_json(),
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = []\n\n[workspace.dependencies]\n",
        )
        .unwrap();
        fs::write(root.join("specs/masterplan.json"), "{}\n").unwrap();

        fs::create_dir_all(root.join("cloud/x")).unwrap();
        fs::write(
            root.join("cloud/x/Cargo.toml"),
            "[package]\nname = \"safe-x\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"cloud/x\"]\n\n[workspace.dependencies]\n\
             legacy = { path = \"cloud/x\" }\n",
        )
        .unwrap();

        let policy = test_policy();
        let candidate = collect_baseline_candidate(&root, &policy).unwrap();
        let candidate_baseline = candidate.to_baseline();
        fs::write(
            root.join(&policy.baseline_file),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&candidate_baseline.to_json()).unwrap()
            ),
        )
        .unwrap();

        let report = check_live_tree(&root, &policy, &candidate_baseline).unwrap();
        assert_eq!(report.verdict(), Verdict::Red);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == CODE_BASELINE_EXPANSION),
            "a matching candidate hash must not waive its same-change edge: {:?}",
            report.findings
        );
        fs::remove_dir_all(&root).unwrap();
    }
}
