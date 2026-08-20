//! # cloud-ci-tier-field-coverage (Phase-0 capability-first reorg; ADR-0562/0536/0245/0280)
//!
//! The born-blocking per-service tier-metadata coverage gate. Phase-0 of the ratified
//! capability-first repo organization (ADR-0562) introduces a clean mandatory dependency-class
//! triple on EVERY service `manifest.json`:
//!
//! - `tier`        — the dependency class, ∈ `tier_enum` (substrate / product / service-cell / reserved);
//! - `tier_subtype`— the refined class, ∈ `tier_subtype_enum` (the platform-architecture canonical list);
//! - `dr_tier`     — the DR/reliability class, ∈ {T0, T1, T2, T3}, kept DISTINCT from `tier`;
//! - and, for substrates, `substrate_dag_position` `{stratum, depends_on, consumed_by_substrates}`.
//!
//! ADR-0245 makes tier a manifest FACET (not a directory split); ADR-0536 decides WHAT each
//! substrate is; the closed ADR-0562 capability registry maps every service dir → capability → tier.
//!
//! ## What this gate asserts (the contract)
//! 1. 100% COVERAGE: every service manifest under the governed roots carries `tier`,
//!    `tier_subtype`, `dr_tier`.
//! 2. ENUM VALIDITY: `tier` ∈ `tier_enum`; `tier_subtype` ∈ `tier_subtype_enum`;
//!    `dr_tier` ∈ `dr_tier_enum`.
//! 3. NO TYPE-OVERLOAD: `tier` is the dependency class ONLY — it must never carry a DR/reliability
//!    value (T0..T3) nor a deployment-mode value (saas/surface/sse/hot/fallback/self-hosted), nor the
//!    historical loose `external-facing`. This is the V3 de-overload guard.
//! 4. `tier`/`dr_tier` SEPARATION: the two are DISTINCT fields and must not be conflated.
//! 5. SUBSTRATE DAG POSITION: a `tier == "substrate"` manifest carries `substrate_dag_position` with a
//!    `stratum` ∈ `substrate_dag_stratum_enum`.
//! 6. ADR-0348 SHARDING AUTOMATION: top-level service manifests under the
//!    policy-governed roots carry a non-manual `sharding_automation` block with
//!    control-plane autosharding, residency-aware auto-rebalance, dynamic-sharding
//!    thresholds, and audit-chain emit declarations.
//! 7. OPENSLO MANIFEST COVERAGE: top-level service manifests either reference existing non-empty
//!    OpenSLO files or carry explicit live exemptions for non-runtime / not-yet-measured services.
//!
//! ## Born pack-shaped
//! The enums, governed roots, the overload denylist, and the scan floor are DATA in
//! `tier-field-coverage-policy.json` (sourced verbatim from `specs/platform-architecture.json`
//! microservice_taxonomy). The kernel hardcodes no enum value; a different repo adopts the gate by
//! repointing the policy.
//!
//! ## Ratchet semantics
//! Phase-0 assigns every live service a tier BEFORE this gate goes blocking, so the gate ships
//! born-blocking GREEN at zero baseline: any NEW service without the triple, with an out-of-enum
//! value, or with an overloaded `tier`, fails closed on arrival.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `TFC-MISSING-TIER`            — manifest lacks `tier`.
//! - `TFC-MISSING-TIER-SUBTYPE`    — manifest lacks `tier_subtype`.
//! - `TFC-MISSING-DR-TIER`         — manifest lacks `dr_tier`.
//! - `TFC-TIER-NOT-IN-ENUM`        — `tier` value ∉ `tier_enum`.
//! - `TFC-TIER-SUBTYPE-NOT-IN-ENUM`— `tier_subtype` value ∉ `tier_subtype_enum`.
//! - `TFC-DR-TIER-NOT-IN-ENUM`     — `dr_tier` value ∉ `dr_tier_enum`.
//! - `TFC-TIER-TYPE-OVERLOAD`      — `tier` carries a denied DR/deployment-mode value (V3 guard).
//! - `TFC-SUBSTRATE-MISSING-DAG-POSITION` — a substrate lacks `substrate_dag_position`.
//! - `TFC-SUBSTRATE-DAG-STRATUM-INVALID`  — `substrate_dag_position.stratum` ∉ stratum enum.
//! - `TFC-SHARDING-MISSING-BLOCK`  — top-level service lacks an ADR-0348 `sharding_automation` block.
//! - `TFC-AUTOSHARDING-MALFORMED`  — autosharding is absent, wrong-typed, or not control-plane-shaped.
//! - `TFC-AUTOSHARDING-MANUAL-MODE`— autosharding declares a manual mode refused by ADR-0348.
//! - `TFC-AUTOREBALANCE-MALFORMED` — auto-rebalance is absent, wrong-typed, or missing thresholds.
//! - `TFC-AUTOREBALANCE-RESIDENCY-MISSING` — enabled auto-rebalance omits residency/compliance honors.
//! - `TFC-DYNAMIC-SHARDING-MALFORMED` — dynamic-sharding is absent or wrong-typed.
//! - `TFC-DYNAMIC-SHARDING-THRESHOLD-MISSING` — enabled dynamic-sharding lacks numeric thresholds.
//! - `TFC-AUTOMATION-AUDIT-CHAIN-EMIT-MISSING` — enabled automation omits audit-chain emission.
//! - `TFC-SLO-MISSING-OR-UNEXEMPT` — no non-empty/resolved SLO coverage and no explicit live exemption.
//! - `TFC-SLO-REFERENCE-UNRESOLVED` — SLO entry references no existing non-empty OpenSLO file.
//! - `TFC-SLO-ENTRY-MALFORMED`     — SLO entry shape is not a resolvable reference/exemption key.
//! - `TFC-SLO-EXEMPTION-MALFORMED` — SLO exemption lacks live structured metadata or uses placeholders.
//! - `TFC-EMPTY-SCAN`              — fewer service manifests than the policy floor (false-green guard).
//! - `TFC-POLICY-GATE-ID-MISMATCH` — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `TFC-POLICY-MALFORMED`        — the policy is missing/wrong-typed a required field (fail-closed).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-tier-field-coverage";

/// The violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 24] = [
    "TFC-MISSING-TIER",
    "TFC-MISSING-TIER-SUBTYPE",
    "TFC-MISSING-DR-TIER",
    "TFC-TIER-NOT-IN-ENUM",
    "TFC-TIER-SUBTYPE-NOT-IN-ENUM",
    "TFC-DR-TIER-NOT-IN-ENUM",
    "TFC-TIER-TYPE-OVERLOAD",
    "TFC-SUBSTRATE-MISSING-DAG-POSITION",
    "TFC-SUBSTRATE-DAG-STRATUM-INVALID",
    "TFC-SHARDING-MISSING-BLOCK",
    "TFC-AUTOSHARDING-MALFORMED",
    "TFC-AUTOSHARDING-MANUAL-MODE",
    "TFC-AUTOREBALANCE-MALFORMED",
    "TFC-AUTOREBALANCE-RESIDENCY-MISSING",
    "TFC-DYNAMIC-SHARDING-MALFORMED",
    "TFC-DYNAMIC-SHARDING-THRESHOLD-MISSING",
    "TFC-AUTOMATION-AUDIT-CHAIN-EMIT-MISSING",
    "TFC-SLO-MISSING-OR-UNEXEMPT",
    "TFC-SLO-REFERENCE-UNRESOLVED",
    "TFC-SLO-ENTRY-MALFORMED",
    "TFC-SLO-EXEMPTION-MALFORMED",
    "TFC-EMPTY-SCAN",
    "TFC-POLICY-GATE-ID-MISMATCH",
    "TFC-POLICY-MALFORMED",
];

/// Sentinel key for policy-level (non-per-manifest) findings.
const POLICY_KEY: &str = "<policy>";

/// Errors collecting the observed manifest corpus. Returned (never panicked) so the caller decides
/// how to surface them — an unreadable governed root is a fail-closed error, never a silent skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    Io(String),
    Parse { path: String, message: String },
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::Io(message) => write!(f, "tier-field-coverage io: {message}"),
            CollectError::Parse { path, message } => {
                write!(f, "manifest {path} is not valid JSON: {message}")
            }
        }
    }
}

impl std::error::Error for CollectError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    /// The manifest path (repo-relative) or `<policy>` for policy-level findings.
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub manifests_checked: usize,
    pub violations: BTreeSet<String>,
}

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only)
// ---------------------------------------------------------------------------

/// Collect every service `manifest.json` under the policy's `governed_service_roots`, returning a
/// `{ "manifests": [ { "path", "manifest": <json> }, .. ], "manifest_count": <usize> }` shape that
/// [`evaluate_keyed`] consumes purely.
///
/// A service manifest is a file named `manifest.json` anywhere within a governed root subtree (so
/// nested pack/sub-context manifests are included). Read-only; writes no temp files. A malformed
/// manifest is a fail-closed `CollectError::Parse`, never a silently skipped file.
pub fn collect_manifests(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let mut roots: Vec<String> = policy
        .get("governed_service_roots")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    roots.sort();

    let mut paths: Vec<String> = Vec::new();
    for service_root in &roots {
        let dir = root.join(service_root);
        collect_manifest_paths(&dir, root, &mut paths)?;
    }
    paths.sort();

    let mut manifests = Vec::with_capacity(paths.len());
    for rel in &paths {
        let abs = root.join(rel);
        let text =
            fs::read_to_string(&abs).map_err(|e| CollectError::Io(format!("read {rel}: {e}")))?;
        let value: Value = serde_json::from_str(&text).map_err(|e| CollectError::Parse {
            path: rel.clone(),
            message: e.to_string(),
        })?;
        manifests.push(json!({ "path": rel, "manifest": value }));
    }

    let mut openslo_files: BTreeMap<String, bool> = BTreeMap::new();
    collect_openslo_files(root, root, &mut openslo_files)?;
    let available_openslo_files: Vec<Value> = openslo_files
        .iter()
        .map(|(path, non_empty)| json!({ "path": path, "non_empty": non_empty }))
        .collect();

    Ok(json!({
        "manifest_count": manifests.len(),
        "manifests": manifests,
        "available_openslo_files": available_openslo_files,
    }))
}

/// Recursively collect repo-relative paths of files named `manifest.json` under `dir`. A missing
/// governed root is not an error (the gate is repo-portable). Deterministic via the sorted caller.
fn collect_manifest_paths(
    dir: &Path,
    repo_root: &Path,
    out: &mut Vec<String>,
) -> Result<(), CollectError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(CollectError::Io(format!("read dir {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry =
            entry.map_err(|e| CollectError::Io(format!("entry in {}: {e}", dir.display())))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        if file_type.is_dir() {
            // A manifest under `tests/` is a FIXTURE, not a service. Widening
            // `governed_service_roots` to include `cell` pulled
            // `cell/core/regional-pack/tests/fixtures/kr/manifest.json` into the coverage
            // corpus, where the gate demanded tier/tier_subtype/dr_tier it will never
            // legitimately carry. Widening a corpus without reconciling its new members
            // trades one red for another, so the fixture class is excluded at the walk.
            if path.file_name().and_then(|n| n.to_str()) == Some("tests") {
                continue;
            }
            collect_manifest_paths(&path, repo_root, out)?;
        } else if path.file_name().and_then(|n| n.to_str()) == Some("manifest.json") {
            let rel = path
                .strip_prefix(repo_root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| path.to_string_lossy().into_owned());
            out.push(rel);
        }
    }
    Ok(())
}

/// Recursively collect repo-relative OpenSLO files and whether they contain non-whitespace content.
/// This is read-only collector work, not policy: the evaluator receives the resulting proof and stays
/// deterministic over its input value.
fn collect_openslo_files(
    dir: &Path,
    repo_root: &Path,
    out: &mut BTreeMap<String, bool>,
) -> Result<(), CollectError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(CollectError::Io(format!("read dir {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry =
            entry.map_err(|e| CollectError::Io(format!("entry in {}: {e}", dir.display())))?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name == ".git" || name == ".gjc" || name == "target" || name == "buck-out" {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        if file_type.is_dir() {
            collect_openslo_files(&path, repo_root, out)?;
        } else if name.ends_with(".openslo.yaml") || name.ends_with(".openslo.yml") {
            let rel = path
                .strip_prefix(repo_root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| path.to_string_lossy().into_owned());
            let text = fs::read_to_string(&path)
                .map_err(|e| CollectError::Io(format!("read {rel}: {e}")))?;
            out.insert(rel, !text.trim().is_empty());
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Pure evaluation
// ---------------------------------------------------------------------------

/// Parsed policy DATA. Returns an Err string on any malformed required field so the evaluator emits
/// `TFC-POLICY-MALFORMED` and fails CLOSED rather than silently dropping a check.
struct ParsedPolicy {
    governed_service_roots: BTreeSet<String>,
    tier_enum: BTreeSet<String>,
    tier_subtype_enum: BTreeSet<String>,
    dr_tier_enum: BTreeSet<String>,
    stratum_enum: BTreeSet<String>,
    denied_overload: BTreeSet<String>,
    substrate_requires_dag_position: bool,
    require_sharding_automation: bool,
    require_openslo_manifest_refs: bool,
    canonical_autosharding_mode: String,
    allowed_disabled_autosharding_modes: BTreeSet<String>,
}

fn string_set(policy: &Value, key: &str) -> Result<BTreeSet<String>, String> {
    let arr = policy
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("policy `{key}` must be a string array"))?;
    let mut out = BTreeSet::new();
    for (i, v) in arr.iter().enumerate() {
        let s = v
            .as_str()
            .ok_or_else(|| format!("policy `{key}`[{i}] must be a string"))?;
        out.insert(s.to_owned());
    }
    if out.is_empty() {
        return Err(format!("policy `{key}` must be non-empty"));
    }
    Ok(out)
}

fn optional_string_set(policy: &Value, key: &str) -> BTreeSet<String> {
    policy
        .get(key)
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn required_bool(policy: &Value, key: &str) -> Result<bool, String> {
    policy
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("policy `{key}` must be a boolean"))
}

fn optional_string(policy: &Value, key: &str, default: &str) -> String {
    policy
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(default)
        .to_owned()
}

fn parse_policy(policy: &Value) -> Result<ParsedPolicy, String> {
    Ok(ParsedPolicy {
        tier_enum: string_set(policy, "tier_enum")?,
        tier_subtype_enum: string_set(policy, "tier_subtype_enum")?,
        dr_tier_enum: string_set(policy, "dr_tier_enum")?,
        stratum_enum: string_set(policy, "substrate_dag_stratum_enum")?,
        denied_overload: string_set(policy, "de_overload_denied_tier_values")?,
        governed_service_roots: string_set(policy, "governed_service_roots")?,
        substrate_requires_dag_position: policy
            .get("substrate_requires_dag_position")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        require_sharding_automation: required_bool(policy, "require_sharding_automation")?,
        require_openslo_manifest_refs: required_bool(policy, "require_openslo_manifest_refs")?,
        canonical_autosharding_mode: optional_string(
            policy,
            "canonical_autosharding_mode",
            "control_plane_driven",
        ),
        allowed_disabled_autosharding_modes: optional_string_set(
            policy,
            "allowed_disabled_autosharding_modes",
        ),
    })
}

/// Pure evaluator. `policy` is DATA (`tier-field-coverage-policy.json`); `observed` is the collected
/// corpus shaped by [`collect_manifests`]. Surface-all: every violation is reported, not just the
/// first.
#[must_use]
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "TFC-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    let parsed = match parse_policy(policy) {
        Ok(parsed) => parsed,
        Err(message) => {
            findings.insert(Finding::new(
                "TFC-POLICY-MALFORMED",
                POLICY_KEY,
                format!("{message}; the policy must be corrected before the gate can evaluate"),
            ));
            return findings;
        }
    };

    let min_expected = policy
        .get("min_expected_service_manifests")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    let manifests = observed
        .get("manifests")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let count = observed
        .get("manifest_count")
        .and_then(Value::as_u64)
        .unwrap_or(manifests.len() as u64);
    if count < min_expected {
        findings.insert(Finding::new(
            "TFC-EMPTY-SCAN",
            POLICY_KEY,
            format!(
                "scan found {count} service manifest(s), below the policy floor of {min_expected}; the governed roots, CWD, or collection is likely broken (fail-closed against a silent false-green)"
            ),
        ));
    }

    let available_openslo_paths: BTreeSet<String> = observed
        .get("available_openslo_files")
        .and_then(Value::as_array)
        .map(|files| {
            files
                .iter()
                .filter(|file| bool_field(file, "non_empty") == Some(true))
                .filter_map(|file| string_field(file, "path").map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();

    for entry in &manifests {
        let path = entry
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        let manifest = entry.get("manifest").cloned().unwrap_or(Value::Null);
        evaluate_one(
            &parsed,
            path,
            &manifest,
            &available_openslo_paths,
            &mut findings,
        );
    }

    findings
}

/// Evaluate one manifest against the parsed policy, pushing every finding it carries.
fn evaluate_one(
    parsed: &ParsedPolicy,
    path: &str,
    manifest: &Value,
    available_openslo_paths: &BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) {
    let tier = manifest.get("tier").and_then(Value::as_str);
    let subtype = manifest.get("tier_subtype").and_then(Value::as_str);
    let dr_tier = manifest.get("dr_tier").and_then(Value::as_str);

    // 1. Coverage.
    if tier.is_none() {
        findings.insert(Finding::new(
            "TFC-MISSING-TIER",
            path,
            "manifest lacks the mandatory dependency-class `tier` field (ADR-0562/0245)",
        ));
    }
    if subtype.is_none() {
        findings.insert(Finding::new(
            "TFC-MISSING-TIER-SUBTYPE",
            path,
            "manifest lacks the mandatory `tier_subtype` field (ADR-0562/0245)",
        ));
    }
    if dr_tier.is_none() {
        findings.insert(Finding::new(
            "TFC-MISSING-DR-TIER",
            path,
            "manifest lacks the mandatory `dr_tier` DR/reliability class (kept DISTINCT from `tier`)",
        ));
    }

    // 2/3. Enum validity + type-overload guard for `tier`.
    if let Some(tier) = tier {
        if parsed.denied_overload.contains(tier) {
            findings.insert(Finding::new(
                "TFC-TIER-TYPE-OVERLOAD",
                path,
                format!(
                    "`tier` carries a denied value {tier:?} — `tier` is the dependency class ONLY; a DR/reliability value (T0..T3) belongs in `dr_tier` and a deployment-mode value belongs in `deployment_mode` (V3 de-overload; tier/dr_tier must stay distinct)"
                ),
            ));
        } else if !parsed.tier_enum.contains(tier) {
            findings.insert(Finding::new(
                "TFC-TIER-NOT-IN-ENUM",
                path,
                format!("`tier` value {tier:?} is not in tier_enum"),
            ));
        }
    }
    if let Some(subtype) = subtype
        && !parsed.tier_subtype_enum.contains(subtype)
    {
        findings.insert(Finding::new(
            "TFC-TIER-SUBTYPE-NOT-IN-ENUM",
            path,
            format!("`tier_subtype` value {subtype:?} is not in tier_subtype_enum"),
        ));
    }
    if let Some(dr_tier) = dr_tier
        && !parsed.dr_tier_enum.contains(dr_tier)
    {
        findings.insert(Finding::new(
            "TFC-DR-TIER-NOT-IN-ENUM",
            path,
            format!("`dr_tier` value {dr_tier:?} is not in dr_tier_enum"),
        ));
    }

    // 5. Substrate DAG position.
    if tier == Some("substrate") && parsed.substrate_requires_dag_position {
        match manifest.get("substrate_dag_position") {
            None => {
                findings.insert(Finding::new(
                    "TFC-SUBSTRATE-MISSING-DAG-POSITION",
                    path,
                    "a substrate manifest must declare `substrate_dag_position` {stratum, depends_on, consumed_by_substrates} sourced from the landed DAG (ADR-0280)",
                ));
            }
            Some(pos) => {
                let stratum = pos.get("stratum").and_then(Value::as_str);
                match stratum {
                    Some(stratum) if parsed.stratum_enum.contains(stratum) => {}
                    Some(stratum) => {
                        findings.insert(Finding::new(
                            "TFC-SUBSTRATE-DAG-STRATUM-INVALID",
                            path,
                            format!("`substrate_dag_position.stratum` value {stratum:?} is not in substrate_dag_stratum_enum"),
                        ));
                    }
                    None => {
                        findings.insert(Finding::new(
                            "TFC-SUBSTRATE-DAG-STRATUM-INVALID",
                            path,
                            "`substrate_dag_position.stratum` is missing or not a string",
                        ));
                    }
                }
            }
        }
    }

    if is_top_level_service_manifest(path, &parsed.governed_service_roots) {
        if parsed.require_sharding_automation {
            evaluate_sharding_automation(parsed, path, manifest, findings);
        }
        if parsed.require_openslo_manifest_refs {
            evaluate_openslo_manifest_refs(
                parsed,
                path,
                manifest,
                available_openslo_paths,
                findings,
            );
        }
    }
}

/// A service manifest that owns the ADR-0348 sharding and OpenSLO-ref obligations.
///
/// TWO shapes are top-level, and missing the first one is a silent coverage loss:
///
///   `<root>/manifest.json`            the capability's OWN service manifest
///   `<root>/<service>/manifest.json`  a sub-service under that capability
///
/// The capability-first rehome turns `cloud/cloud-billing/manifest.json` into
/// `billing/manifest.json`, which has ONE component after the root, not two. Requiring
/// two silently dropped fourteen substrate services out of
/// `evaluate_sharding_automation` and `evaluate_openslo_manifest_refs` — billing,
/// compliance, console, flags, gateway, iac, intelligence, k8s, marketplace, network,
/// observability, secrets, storage, tenancy — while the gate still reported green. All
/// fourteen satisfied this predicate before the move.
///
/// A gate that stops looking at a T0/T1 substrate service and stays green is the exact
/// false-green class this gate exists to prevent, so both shapes are accepted and the
/// depth limit is kept: anything deeper (notably `*/tests/fixtures/**/manifest.json`) is
/// not a service and must stay out.
fn is_top_level_service_manifest(path: &str, governed_roots: &BTreeSet<String>) -> bool {
    governed_roots.iter().any(|root| {
        let root = root.trim_matches('/');
        if root.is_empty() {
            return false;
        }
        let Some(rest) = path.strip_prefix(root) else {
            return false;
        };
        let Some(rest) = rest.strip_prefix('/') else {
            return false;
        };
        let mut parts = rest.split('/');
        let Some(first) = parts.next().filter(|segment| !segment.is_empty()) else {
            return false;
        };
        if first == "manifest.json" {
            return parts.next().is_none();
        }
        parts.next() == Some("manifest.json") && parts.next().is_none()
    })
}

fn bool_field(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
}

fn is_manual_mode(mode: &str) -> bool {
    mode.trim().eq_ignore_ascii_case("manual")
}

fn evaluate_sharding_automation(
    parsed: &ParsedPolicy,
    path: &str,
    manifest: &Value,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(sharding) = manifest.get("sharding_automation") else {
        findings.insert(Finding::new(
            "TFC-SHARDING-MISSING-BLOCK",
            path,
            "top-level service manifest lacks `sharding_automation` (ADR-0348)",
        ));
        return;
    };
    let Some(sharding) = sharding.as_object() else {
        findings.insert(Finding::new(
            "TFC-SHARDING-MISSING-BLOCK",
            path,
            "`sharding_automation` must be an object with autosharding, auto_rebalance, and dynamic_sharding declarations",
        ));
        return;
    };

    match sharding.get("autosharding") {
        Some(Value::String(mode)) => {
            if is_manual_mode(mode) {
                findings.insert(Finding::new(
                    "TFC-AUTOSHARDING-MANUAL-MODE",
                    path,
                    "ADR-0348 refuses manual autosharding mode; use control_plane_driven or an explicit disabled-runtime object",
                ));
            } else if mode.trim() != parsed.canonical_autosharding_mode {
                findings.insert(Finding::new(
                    "TFC-AUTOSHARDING-MALFORMED",
                    path,
                    format!(
                        "`sharding_automation.autosharding` string must be {:?}, got {mode:?}",
                        parsed.canonical_autosharding_mode
                    ),
                ));
            }
        }
        Some(Value::Object(_)) => {
            let autosharding = sharding.get("autosharding").unwrap_or(&Value::Null);
            let enabled = bool_field(autosharding, "enabled");
            let mode = string_field(autosharding, "mode");
            let intended = string_field(autosharding, "intended_control_plane");
            if mode.is_some_and(is_manual_mode) || intended.is_some_and(is_manual_mode) {
                findings.insert(Finding::new(
                    "TFC-AUTOSHARDING-MANUAL-MODE",
                    path,
                    "ADR-0348 refuses manual autosharding mode; disabled foundations must still name the control-plane destination",
                ));
            }
            match enabled {
                Some(true) => {
                    let control_plane_declared = mode
                        == Some(parsed.canonical_autosharding_mode.as_str())
                        || intended == Some(parsed.canonical_autosharding_mode.as_str());
                    if !control_plane_declared {
                        findings.insert(Finding::new(
                            "TFC-AUTOSHARDING-MALFORMED",
                            path,
                            format!(
                                "enabled autosharding must declare {:?} as mode or intended_control_plane",
                                parsed.canonical_autosharding_mode
                            ),
                        ));
                    }
                }
                Some(false) => {
                    let disabled_mode = mode.is_some_and(|m| {
                        m == parsed.canonical_autosharding_mode
                            || parsed.allowed_disabled_autosharding_modes.contains(m)
                    });
                    let intended_control_plane = intended
                        .map(|m| m == parsed.canonical_autosharding_mode)
                        .unwrap_or(true);
                    if !disabled_mode || !intended_control_plane {
                        findings.insert(Finding::new(
                            "TFC-AUTOSHARDING-MALFORMED",
                            path,
                            "disabled autosharding must carry a recognized disabled mode and may only point at control_plane_driven as the intended destination",
                        ));
                    }
                }
                None => {
                    findings.insert(Finding::new(
                        "TFC-AUTOSHARDING-MALFORMED",
                        path,
                        "`sharding_automation.autosharding.enabled` must be a boolean",
                    ));
                }
            };
        }
        _ => {
            findings.insert(Finding::new(
                "TFC-AUTOSHARDING-MALFORMED",
                path,
                "`sharding_automation.autosharding` must be a control_plane_driven string or an object declaration",
            ));
        }
    }

    let Some(auto_rebalance) = sharding.get("auto_rebalance") else {
        findings.insert(Finding::new(
            "TFC-AUTOREBALANCE-MALFORMED",
            path,
            "`sharding_automation.auto_rebalance` sub-block is required (ADR-0348)",
        ));
        return;
    };
    let Some(enabled) = bool_field(auto_rebalance, "enabled") else {
        findings.insert(Finding::new(
            "TFC-AUTOREBALANCE-MALFORMED",
            path,
            "`sharding_automation.auto_rebalance.enabled` must be a boolean",
        ));
        return;
    };
    if enabled {
        if auto_rebalance
            .get("trigger_load_skew_threshold_percent")
            .and_then(Value::as_u64)
            .is_none()
        {
            findings.insert(Finding::new(
                "TFC-AUTOREBALANCE-MALFORMED",
                path,
                "enabled auto_rebalance must declare trigger_load_skew_threshold_percent",
            ));
        }
        if bool_field(auto_rebalance, "honors_residency") != Some(true)
            || bool_field(auto_rebalance, "honors_compliance_packs") != Some(true)
        {
            findings.insert(Finding::new(
                "TFC-AUTOREBALANCE-RESIDENCY-MISSING",
                path,
                "enabled auto_rebalance must explicitly honor residency and compliance packs",
            ));
        }
        if bool_field(auto_rebalance, "audit_chain_emit") != Some(true) {
            findings.insert(Finding::new(
                "TFC-AUTOMATION-AUDIT-CHAIN-EMIT-MISSING",
                path,
                "enabled auto_rebalance must emit audit-chain events",
            ));
        }
    }

    let Some(dynamic_sharding) = sharding.get("dynamic_sharding") else {
        findings.insert(Finding::new(
            "TFC-DYNAMIC-SHARDING-MALFORMED",
            path,
            "`sharding_automation.dynamic_sharding` sub-block is required (ADR-0348)",
        ));
        return;
    };
    let Some(enabled) = bool_field(dynamic_sharding, "enabled") else {
        findings.insert(Finding::new(
            "TFC-DYNAMIC-SHARDING-MALFORMED",
            path,
            "`sharding_automation.dynamic_sharding.enabled` must be a boolean",
        ));
        return;
    };
    if enabled {
        for key in [
            "hot_split_threshold_p99_ms",
            "hot_split_utilization_threshold_percent",
            "cold_merge_utilization_threshold_percent",
            "cold_merge_minimum_quiet_hours",
        ] {
            if dynamic_sharding.get(key).and_then(Value::as_u64).is_none() {
                findings.insert(Finding::new(
                    "TFC-DYNAMIC-SHARDING-THRESHOLD-MISSING",
                    path,
                    format!("enabled dynamic_sharding must declare numeric `{key}`"),
                ));
            }
        }
        if bool_field(dynamic_sharding, "audit_chain_emit") != Some(true) {
            findings.insert(Finding::new(
                "TFC-AUTOMATION-AUDIT-CHAIN-EMIT-MISSING",
                path,
                "enabled dynamic_sharding must emit audit-chain events",
            ));
        }
    }
}

const GENERIC_SLO_EXEMPTION_MARKERS: [&str; 9] = [
    "todo",
    "tbd",
    "fixme",
    "placeholder",
    "test fixture",
    "fixture-only",
    "replace this exemption",
    "before production or hyperscaler-ready promotion",
    "no current openslo artifact resolves",
];

fn is_generic_placeholder(value: &str) -> bool {
    let lower = value.trim().to_ascii_lowercase();
    lower.is_empty()
        || GENERIC_SLO_EXEMPTION_MARKERS
            .iter()
            .any(|marker| lower.contains(marker))
}

fn live_string_field(value: &Value, key: &str, min_len: usize) -> bool {
    string_field(value, key)
        .map(str::trim)
        .is_some_and(|s| s.len() >= min_len && !is_generic_placeholder(s))
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(idx, b)| idx == 4 || idx == 7 || b.is_ascii_digit())
}

fn live_date_field(value: &Value, key: &str) -> bool {
    string_field(value, key)
        .map(str::trim)
        .is_some_and(|s| is_iso_date(s) && !is_generic_placeholder(s))
}

fn slo_exemption_is_live(
    path: &str,
    scope: &str,
    exemption: &Value,
    findings: &mut BTreeSet<Finding>,
) -> bool {
    let is_live = exemption.as_object().is_some()
        && live_string_field(exemption, "status", 3)
        && live_string_field(exemption, "owner", 3)
        && live_string_field(exemption, "rationale", 20)
        && (live_date_field(exemption, "expires_on") || live_date_field(exemption, "cutover_on"))
        && (live_string_field(exemption, "evidence", 3)
            || live_string_field(exemption, "ticket", 3));

    if !is_live {
        findings.insert(Finding::new(
            "TFC-SLO-EXEMPTION-MALFORMED",
            path,
            format!("{scope} must carry live structured metadata: status, owner, rationale, expires_on or cutover_on ISO date, and evidence or ticket; generic placeholders are refused"),
        ));
    }

    is_live
}

fn manifest_has_slo_exemption(
    path: &str,
    manifest: &Value,
    findings: &mut BTreeSet<Finding>,
) -> bool {
    let Some(exemption) = manifest.get("slo_exemption") else {
        return false;
    };
    slo_exemption_is_live(path, "`slo_exemption`", exemption, findings)
}

fn exempt_slo_names(
    path: &str,
    manifest: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    manifest
        .get("slo_exemptions")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .enumerate()
                .filter_map(|(index, row)| {
                    let Some(name) = string_field(row, "name") else {
                        findings.insert(Finding::new(
                            "TFC-SLO-EXEMPTION-MALFORMED",
                            path,
                            format!("`slo_exemptions` row {index} must carry a non-empty name"),
                        ));
                        return None;
                    };
                    if is_generic_placeholder(name) {
                        findings.insert(Finding::new(
                            "TFC-SLO-EXEMPTION-MALFORMED",
                            path,
                            format!("`slo_exemptions` row {index} has a generic or empty name"),
                        ));
                        return None;
                    }
                    if slo_exemption_is_live(
                        path,
                        &format!("`slo_exemptions` row {index}"),
                        row,
                        findings,
                    ) {
                        Some(name.to_owned())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn entry_name(entry: &Value) -> Option<&str> {
    entry
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| string_field(entry, "name"))
}

fn inferred_local_slo_path(manifest_path: &str, name: &str) -> Option<String> {
    let service_dir = manifest_path.strip_suffix("/manifest.json")?;
    Some(format!("{service_dir}/slos/{name}.openslo.yaml"))
}

fn declared_slo_path_resolves(
    manifest_path: &str,
    declared: &str,
    available_openslo_paths: &BTreeSet<String>,
) -> bool {
    let declared = declared.trim();
    if available_openslo_paths.contains(declared) {
        return true;
    }
    let Some(service_dir) = manifest_path.strip_suffix("/manifest.json") else {
        return false;
    };
    let service_relative = format!("{service_dir}/{declared}");
    available_openslo_paths.contains(&service_relative)
}

fn evaluate_openslo_manifest_refs(
    _parsed: &ParsedPolicy,
    path: &str,
    manifest: &Value,
    available_openslo_paths: &BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(slos_value) = manifest.get("slos") else {
        if !manifest_has_slo_exemption(path, manifest, findings) {
            findings.insert(Finding::new(
                "TFC-SLO-MISSING-OR-UNEXEMPT",
                path,
                "top-level service manifest lacks `slos` and has no explicit `slo_exemption`",
            ));
        }
        return;
    };
    let Some(slos) = slos_value.as_array() else {
        findings.insert(Finding::new(
            "TFC-SLO-ENTRY-MALFORMED",
            path,
            "`slos` must be an array of OpenSLO references or explicitly exempted entries",
        ));
        return;
    };
    if slos.is_empty() {
        if !manifest_has_slo_exemption(path, manifest, findings) {
            findings.insert(Finding::new(
                "TFC-SLO-MISSING-OR-UNEXEMPT",
                path,
                "top-level service manifest has an empty `slos` array and no explicit `slo_exemption`",
            ));
        }
        return;
    }

    let exemptions = exempt_slo_names(path, manifest, findings);
    let mut covered = false;
    for (index, entry) in slos.iter().enumerate() {
        let name = entry_name(entry);
        if name.is_some_and(|name| exemptions.contains(name)) {
            covered = true;
            continue;
        }

        if let Some(file) = string_field(entry, "file") {
            if !(file.ends_with(".openslo.yaml") || file.ends_with(".openslo.yml"))
                || !declared_slo_path_resolves(path, file, available_openslo_paths)
            {
                findings.insert(Finding::new(
                    "TFC-SLO-REFERENCE-UNRESOLVED",
                    path,
                    format!(
                        "SLO entry {index} references a missing or empty OpenSLO file {file:?}"
                    ),
                ));
            } else {
                covered = true;
            }
            continue;
        }

        if let Some(raw) = entry.as_str().filter(|s| {
            let s = s.trim();
            s.ends_with(".openslo.yaml") || s.ends_with(".openslo.yml")
        }) {
            if declared_slo_path_resolves(path, raw, available_openslo_paths) {
                covered = true;
            } else {
                findings.insert(Finding::new(
                    "TFC-SLO-REFERENCE-UNRESOLVED",
                    path,
                    format!("SLO entry {index} references a missing or empty OpenSLO file {raw:?}"),
                ));
            }
            continue;
        }

        let Some(name) = name else {
            findings.insert(Finding::new(
                "TFC-SLO-ENTRY-MALFORMED",
                path,
                format!("SLO entry {index} must carry a non-empty `name`, scalar id, or `file`"),
            ));
            continue;
        };
        let Some(inferred) = inferred_local_slo_path(path, name) else {
            findings.insert(Finding::new(
                "TFC-SLO-ENTRY-MALFORMED",
                path,
                format!("SLO entry {index} cannot infer a local OpenSLO path from manifest path"),
            ));
            continue;
        };
        if available_openslo_paths.contains(&inferred) {
            covered = true;
        } else {
            findings.insert(Finding::new(
                "TFC-SLO-REFERENCE-UNRESOLVED",
                path,
                format!("SLO entry {index} lacks a non-empty resolvable OpenSLO file; expected {inferred:?} or an explicit per-entry exemption"),
            ));
        }
    }

    if !covered && !manifest_has_slo_exemption(path, manifest, findings) {
        findings.insert(Finding::new(
            "TFC-SLO-MISSING-OR-UNEXEMPT",
            path,
            "no SLO entry resolves to an existing OpenSLO file and the manifest has no explicit `slo_exemption`",
        ));
    }
}

/// The bare-code projection of [`evaluate_keyed`]: the single source of the verdict + counts.
#[must_use]
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    let findings = evaluate_keyed(policy, observed);
    let violations = findings
        .iter()
        .map(|f| f.code.clone())
        .collect::<BTreeSet<_>>();
    let manifests_checked = observed
        .get("manifest_count")
        .and_then(Value::as_u64)
        .map(|n| n as usize)
        .or_else(|| {
            observed
                .get("manifests")
                .and_then(Value::as_array)
                .map(Vec::len)
        })
        .unwrap_or(0);
    Report {
        verdict: if violations.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        },
        manifests_checked,
        violations,
    }
}

/// Render findings for a human / CI log. Empty findings render the GREEN line.
#[must_use]
pub fn render_findings(findings: &BTreeSet<Finding>) -> String {
    if findings.is_empty() {
        return "tier-field-coverage: GREEN — every governed service manifest carries valid \
                tier/tier_subtype/dr_tier metadata, substrate DAG position, ADR-0348 sharding \
                automation, and resolvable or explicitly exempted OpenSLO coverage"
            .to_owned();
    }
    // Group counts per code for a stable summary, then list each finding.
    let mut per_code: BTreeMap<String, usize> = BTreeMap::new();
    for f in findings {
        *per_code.entry(f.code.clone()).or_insert(0) += 1;
    }
    let mut lines = vec![format!(
        "tier-field-coverage: RED — {} finding(s) across {} violation class(es):",
        findings.len(),
        per_code.len()
    )];
    for f in findings {
        lines.push(format!("  {} {}: {}", f.code, f.key, f.detail));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests;
