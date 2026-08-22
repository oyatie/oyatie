//! # cloud-ci-capability-membership (Phase-0 capability-first reorg; ADR-0562 §6)
//!
//! The MEMBERSHIP lint — the anti-junk-drawer authority. It is the closed-registry enforcement
//! that makes a capability-first source tree mechanically safe (an open capability set degrades
//! into a util/ junk-drawer). ADR-0562 §6 mandates that every crate map to EXACTLY ONE registered
//! capability (or a meta directory), that no top-level dir appear outside the closed set, and that
//! `base/` admission be gated.
//!
//! ## What this gate asserts (the contract)
//! 1. EXACTLY-ONE MEMBERSHIP: every crate in the tree maps to exactly one home — a registered
//!    capability OR a meta directory (kernel/os/base/governance/build/app). No crate is unmapped
//!    (beyond the frozen baseline, see §ratchet); no crate maps to two homes.
//! 2. CLOSED TOP-LEVEL SET: no top-level directory exists outside the closed set
//!    (`allowed_top_level_dirs` = meta directories ∪ the current capability-home roots ∪ the known
//!    non-crate dirs). A NEW top-level dir (e.g. `common/`) outside the set FAILS.
//! 3. BASE/-ADMISSION RULE: a `base/` crate must be depended-on by `>=3` capabilities AND be
//!    strictly below all of them in the ADR-0280 DAG. It is the structural backstop against `base/`
//!    becoming a util dumping ground. (PRE-MOVE there is no `base/` dir, so this is vacuously green
//!    on the live tree and enforced from RED fixtures.)
//!
//! ## PRE-MOVE vs POST-move mapping
//! PRE-MOVE (today) a crate is mapped by the CURRENT dir it lives under, resolved against the
//! registry's `capabilities[].absorbs_current_dirs` + the `membership_lint_coverage` block
//! (`app_products`, `retired_v1_products`, `meta_directory_absorbs`, `absorbs_current_crate_globs`,
//! `frozen_unmapped_baseline`). POST-move the path itself is the namespace (the gate would map by
//! the top-level dir directly); the policy `scan_roots` shrink as the strangler completes.
//!
//! ## Ratchet semantics — BORN-ADVISORY with a FROZEN BASELINE
//! After extending the registry, some current crates remain genuinely ambiguous (the
//! `libs/shared-*` cross-capability junk-drawer + `libs/http-*` middleware + base/
//! candidates). They are captured as the registry's `frozen_unmapped_baseline` and are NOT
//! force-mapped to a wrong capability. The gate is ADVISORY for those crates but enforces NO
//! REGRESSION: a crate unmapped AND not in the frozen baseline is a NEW unmapped crate and FAILS
//! ([`MEM-NEW-UNMAPPED-CRATE`]); a NEW top-level dir outside the closed set FAILS
//! ([`MEM-NEW-TOP-LEVEL-DIR`]). The frozen baseline burns DOWN as the strangler homes crates.
//! FLIP-TO-BLOCKING TRIGGER: when the frozen baseline reaches 0 (post-strangler), a remaining
//! unmapped crate becomes a hard failure with zero advisory slack.
//!
//! ## STOP ACCRUAL — the legacy-root freeze
//! Membership alone does not stop the reorg's debt from GROWING: a brand-new crate born under
//! `oya/`, `libs/`, `tools/` or `cloud/` maps cleanly to a registered capability (those dirs are
//! exactly what `absorbs_current_dirs` covers), so the lint is green while the pile the strangler
//! has to move gets bigger. The policy's `legacy_root_freeze` block closes that: the crate census
//! of each frozen root is FROZEN SHRINK-ONLY at the freeze commit. A crate dir under a frozen root
//! that is not in the census is a NEW legacy-root crate ([`MEM-NEW-LEGACY-ROOT-CRATE`]) — it
//! belongs in its capability root. A census entry that is no longer a crate is burn-down that must
//! be recorded in the same change ([`MEM-STALE-LEGACY-ROOT-BASELINE`]), so the census tracks the
//! moves instead of accumulating slack. The census is PRODUCER-EMITTED (`--emit-legacy-freeze`
//! renders it from this gate's own [`collect`]); the producer refuses to grow it without
//! `--allow-new`, so a regen cannot launder a newly-born crate into the tolerated set. A policy
//! with no `legacy_root_freeze` block (fixtures, adopting repos) is inert here.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `MEM-NEW-UNMAPPED-CRATE`     — a crate maps to NO home and is NOT in the frozen baseline (regression).
//! - `MEM-DOUBLE-MAPPED-CRATE`    — a crate maps to >1 home (the closed-set/exactly-one violation).
//! - `MEM-NEW-TOP-LEVEL-DIR`      — a top-level dir exists outside the closed `allowed_top_level_dirs`.
//! - `MEM-BASE-ADMISSION-CONSUMERS` — a `base/` crate has `<3` capability consumers.
//! - `MEM-BASE-ADMISSION-DAG`     — a `base/` crate is not strictly below all its capability consumers in the DAG.
//! - `MEM-NEW-LEGACY-ROOT-CRATE`  — a crate was born under a FROZEN legacy root and is not in the frozen census (accrual).
//! - `MEM-STALE-LEGACY-ROOT-BASELINE` — a legacy-root census entry is no longer a crate (burn-down that must be recorded).
//! - `MEM-STALE-FROZEN-BASELINE`  — a frozen-baseline entry no longer exists / is now mapped (drift; baseline must shrink in lockstep).
//! - `MEM-EMPTY-SCAN`             — fewer crates than the policy floor (false-green guard).
//! - `MEM-POLICY-GATE-ID-MISMATCH`— the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `MEM-POLICY-MALFORMED`       — the policy/registry is missing/wrong-typed a required field (fail-closed).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-capability-membership";

/// The violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 11] = [
    "MEM-NEW-UNMAPPED-CRATE",
    "MEM-DOUBLE-MAPPED-CRATE",
    "MEM-NEW-TOP-LEVEL-DIR",
    "MEM-NEW-LEGACY-ROOT-CRATE",
    "MEM-STALE-LEGACY-ROOT-BASELINE",
    "MEM-BASE-ADMISSION-CONSUMERS",
    "MEM-BASE-ADMISSION-DAG",
    "MEM-STALE-FROZEN-BASELINE",
    "MEM-EMPTY-SCAN",
    "MEM-POLICY-GATE-ID-MISMATCH",
    "MEM-POLICY-MALFORMED",
];

/// The policy block that freezes the legacy-root crate census shrink-only.
pub const LEGACY_ROOT_FREEZE_KEY: &str = "legacy_root_freeze";

/// Home label for `membership_lint_coverage.retired_v1_products` dirs. Not `meta:app/` —
/// D41/D42 retire-in-place until a later PR deletes the crates.
pub const RETIRE_IN_PLACE_HOME: &str = "retire-in-place";

/// Sentinel key for policy-level (non-per-crate) findings.
const POLICY_KEY: &str = "<policy>";

/// Errors collecting the observed crate corpus. Returned (never panicked) so the caller decides how
/// to surface them — an unreadable scan root is a fail-closed error, never a silent skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    Io(String),
    Parse { path: String, message: String },
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::Io(message) => write!(f, "capability-membership io: {message}"),
            CollectError::Parse { path, message } => {
                write!(f, "registry {path} is not valid JSON: {message}")
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
    /// The crate dir / top-level dir (repo-relative) or `<policy>` for policy-level findings.
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
    pub crates_checked: usize,
    pub mapped_to_home: usize,
    pub frozen_unmapped: usize,
    /// Crates still living under a FROZEN legacy root — the strangler's burn-down number. The
    /// freeze holds it shrink-only; the reorg is done for a root when its share reaches 0.
    pub legacy_root_crates: usize,
    pub violations: BTreeSet<String>,
}

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only)
// ---------------------------------------------------------------------------

/// Collect the observed corpus the [`evaluate_keyed`] consumes purely:
/// `{ "crates": [<repo-relative crate dir>, ..], "crate_count": <usize>, "top_level_dirs": [..],
///    "registry": <the parsed capability registry> }`.
///
/// A "crate" is a directory whose `Cargo.toml` declares a `[package]` (a virtual workspace
/// manifest — `[workspace]` with no `[package]` — is NOT a crate). Scans only the policy
/// `scan_roots` (the dirs that hold capability/lib/tool crates today); `top_level_dirs` is the full
/// set of repo top-level directories for the closed-set check. Read-only; writes no temp files.
pub fn collect(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let scan_roots: Vec<String> = string_array(policy, "scan_roots");
    let mut crates: Vec<String> = Vec::new();
    let mut sorted_roots = scan_roots.clone();
    sorted_roots.sort();
    for scan_root in &sorted_roots {
        collect_crate_dirs(&root.join(scan_root), root, &mut crates)?;
    }
    crates.sort();
    crates.dedup();

    let ignored: BTreeSet<String> = string_array(policy, "ignored_top_level_dirs")
        .into_iter()
        .collect();
    let top_level_dirs = collect_top_level_dirs(root, &ignored)?;

    let registry_path = policy
        .get("registry_path")
        .and_then(Value::as_str)
        .unwrap_or("governance/capability-registry.json");
    let abs = root.join(registry_path);
    let text = fs::read_to_string(&abs)
        .map_err(|e| CollectError::Io(format!("read registry {registry_path}: {e}")))?;
    let registry: Value = serde_json::from_str(&text).map_err(|e| CollectError::Parse {
        path: registry_path.to_owned(),
        message: e.to_string(),
    })?;

    Ok(json!({
        "crate_count": crates.len(),
        "crates": crates,
        "top_level_dirs": top_level_dirs,
        "registry": registry,
    }))
}

/// Recursively collect repo-relative crate dirs (a `Cargo.toml` with a `[package]` section) under
/// `dir`. A missing scan root is not an error (repo-portable). `[package]` detection mirrors cargo:
/// a manifest with `[package]` is a crate; a `[workspace]`-only manifest is a virtual workspace.
fn collect_crate_dirs(
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
            // Skip build artifacts. Everything else is descended (nested workspaces included).
            if path.file_name().and_then(|n| n.to_str()) == Some("target") {
                continue;
            }
            collect_crate_dirs(&path, repo_root, out)?;
        } else if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
            let is_crate = manifest_declares_package(&path)?;
            if is_crate && let Some(parent) = path.parent() {
                let rel = parent
                    .strip_prefix(repo_root)
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| parent.to_string_lossy().into_owned());
                out.push(rel);
            }
        }
    }
    Ok(())
}

/// True iff the manifest declares a `[package]` table (cargo's crate test). A `[workspace]`-only
/// manifest returns false. Line-prefix match tolerates trailing whitespace/comments.
fn manifest_declares_package(manifest: &Path) -> Result<bool, CollectError> {
    let text = fs::read_to_string(manifest)
        .map_err(|e| CollectError::Io(format!("read {}: {e}", manifest.display())))?;
    Ok(text.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed == "[package]" || trimmed.starts_with("[package]")
    }))
}

/// The repo's source top-level directory set (for the closed-set check). Skips hidden dirs (`.git`,
/// `.omc`, ...) and any `ignored` build-artifact dir (`buck-out`, `target`, ... — DATA from the
/// policy `ignored_top_level_dirs`), so generated/untracked dirs never trip the closed-set rule.
fn collect_top_level_dirs(
    root: &Path,
    ignored: &BTreeSet<String>,
) -> Result<Vec<String>, CollectError> {
    let mut dirs = Vec::new();
    let entries = fs::read_dir(root)
        .map_err(|e| CollectError::Io(format!("read repo root {}: {e}", root.display())))?;
    for entry in entries {
        let entry = entry.map_err(|e| CollectError::Io(format!("entry in repo root: {e}")))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        if !file_type.is_dir() {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || ignored.contains(name) {
                continue;
            }
            dirs.push(name.to_owned());
        }
    }
    dirs.sort();
    Ok(dirs)
}

// ---------------------------------------------------------------------------
// Pure evaluation
// ---------------------------------------------------------------------------

fn string_array(value: &Value, key: &str) -> Vec<String> {
    value
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

/// Parsed registry mapping DATA. Returns an Err string on any malformed required field so the
/// evaluator emits `MEM-POLICY-MALFORMED` and fails CLOSED rather than silently dropping a check.
///
/// `pub` (additive, same pattern as slice 2.5): the born-accounting register_crate ORCHESTRATOR
/// (`cloud-ci-register-crate-app`) reuses [`parse_mapping`] + [`homes_for`] verbatim so its
/// "is this crate already capability-mapped?" check is DRIFT-PROOF — it never reimplements the home
/// resolution this gate enforces. Fields stay private (the orchestrator treats `Mapping` as opaque,
/// only passing it to [`homes_for`]).
pub struct Mapping {
    /// (prefix dir, home label) pairs from absorbs_current_dirs + app_products +
    /// retired_v1_products + meta_directory_absorbs.
    dir_prefixes: Vec<(String, String)>,
    /// (glob, home label) pairs from absorbs_current_crate_globs (glob ends with `*` for stem-match).
    globs: Vec<(String, String)>,
    /// The frozen unmapped baseline crate dirs.
    frozen: BTreeSet<String>,
}

impl Mapping {
    /// The distinct home labels this registry can express — every `capability:<name>` / `meta:<dir>`
    /// a crate could be mapped to (from `dir_prefixes` ∪ `globs`). The born-accounting orchestrator
    /// uses this as the CapabilitySet slug universe so a genuinely-unmapped crate has a valid home
    /// choice (it must not be forced into a wrong group for lack of an expressible one).
    #[must_use]
    pub fn expressible_homes(&self) -> BTreeSet<String> {
        self.dir_prefixes
            .iter()
            .chain(self.globs.iter())
            .map(|(_, home)| home.clone())
            .collect()
    }
}

/// Parse the capability registry into the [`Mapping`] DATA the home resolution consumes.
///
/// `pub` so the born-accounting orchestrator reuses the SAME parse (no reimplementation / drift).
///
/// # Errors
/// An `Err(String)` naming the malformed required field, so the evaluator emits `MEM-POLICY-MALFORMED`
/// and fails CLOSED rather than silently dropping a check.
pub fn parse_mapping(registry: &Value) -> Result<Mapping, String> {
    let mut dir_prefixes: Vec<(String, String)> = Vec::new();
    let mut globs: Vec<(String, String)> = Vec::new();

    let capabilities = registry
        .get("capabilities")
        .and_then(Value::as_array)
        .ok_or_else(|| "registry `capabilities` must be an array".to_owned())?;
    for cap in capabilities {
        let name = cap
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "a capability is missing `name`".to_owned())?;
        for dir in string_array(cap, "absorbs_current_dirs") {
            dir_prefixes.push((dir, format!("capability:{name}")));
        }
    }

    let coverage = registry
        .get("membership_lint_coverage")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            "registry `membership_lint_coverage` must be present (the membership-lint extension)"
                .to_owned()
        })?;

    // app_products → app/
    if let Some(app) = coverage.get("app_products") {
        let meta = app
            .get("meta_dir")
            .and_then(Value::as_str)
            .unwrap_or("app/");
        for dir in string_array(app, "current_dirs") {
            dir_prefixes.push((dir, format!("meta:{meta}")));
        }
    }

    // retired_v1_products → retire-in-place (NOT meta:app/). meta_dir must be omitted or null.
    if let Some(retired) = coverage.get("retired_v1_products") {
        if let Some(meta) = retired.get("meta_dir").and_then(Value::as_str) {
            return Err(format!(
                "retired_v1_products.meta_dir must be omitted or null (got {meta:?}); these dirs must not absorb into app/"
            ));
        }
        for dir in string_array(retired, "current_dirs") {
            dir_prefixes.push((dir, RETIRE_IN_PLACE_HOME.to_owned()));
        }
    }

    // meta_directory_absorbs → kernel/, os/, ...
    if let Some(arr) = coverage
        .get("meta_directory_absorbs")
        .and_then(Value::as_array)
    {
        for entry in arr {
            let meta = entry
                .get("meta_dir")
                .and_then(Value::as_str)
                .ok_or_else(|| "a meta_directory_absorbs entry lacks `meta_dir`".to_owned())?;
            for dir in string_array(entry, "current_dirs") {
                dir_prefixes.push((dir, format!("meta:{meta}")));
            }
        }
    }

    // absorbs_current_crate_globs → capability or meta
    if let Some(arr) = coverage
        .get("absorbs_current_crate_globs")
        .and_then(Value::as_array)
    {
        for entry in arr {
            let home = if let Some(cap) = entry.get("capability").and_then(Value::as_str) {
                format!("capability:{cap}")
            } else if let Some(meta) = entry.get("meta_dir").and_then(Value::as_str) {
                format!("meta:{meta}")
            } else {
                return Err(
                    "an absorbs_current_crate_globs entry lacks `capability`/`meta_dir`".to_owned(),
                );
            };
            for g in string_array(entry, "globs") {
                globs.push((g, home.clone()));
            }
        }
    }

    let frozen: BTreeSet<String> = coverage
        .get("frozen_unmapped_baseline")
        .and_then(|b| b.get("crates"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    if dir_prefixes.is_empty() {
        return Err(
            "registry produced zero dir→home mappings (capabilities/coverage empty?)".to_owned(),
        );
    }

    Ok(Mapping {
        dir_prefixes,
        globs,
        frozen,
    })
}

/// True iff `crate_dir` is `entry` or a subdir of `entry`.
fn prefix_match(crate_dir: &str, entry: &str) -> bool {
    crate_dir == entry || crate_dir.starts_with(&format!("{entry}/"))
}

/// True iff `crate_dir` matches `glob` (a `*`-suffixed stem match, else exact-or-subdir).
///
/// `pub` so the born-accounting orchestrator's mapping check shares the EXACT glob semantics this
/// gate enforces (a `*`-suffix stem match, not an exact-string compare) — the drift this fixes.
#[must_use]
pub fn glob_match(crate_dir: &str, glob: &str) -> bool {
    match glob.strip_suffix('*') {
        Some(stem) => crate_dir.starts_with(stem),
        None => prefix_match(crate_dir, glob),
    }
}

/// All home labels a crate maps to (for the exactly-one check).
///
/// `pub` so the born-accounting orchestrator reuses this verbatim: a crate is "already capability-
/// mapped" iff `homes_for(&mapping, dir).len() >= 1`. Single source of home resolution → no drift
/// between the gate that BLOCKS and the orchestrator that decides whether to emit a mapping edit.
#[must_use]
pub fn homes_for(mapping: &Mapping, crate_dir: &str) -> Vec<String> {
    let mut homes: Vec<String> = Vec::new();
    for (entry, home) in &mapping.dir_prefixes {
        if prefix_match(crate_dir, entry) {
            homes.push(home.clone());
        }
    }
    for (glob, home) in &mapping.globs {
        if glob_match(crate_dir, glob) {
            homes.push(home.clone());
        }
    }
    homes.sort();
    homes.dedup();
    homes
}

/// Pure evaluator. `policy` + `observed` (shaped by [`collect`]) are DATA. Surface-all: every
/// violation is reported, not just the first.
#[must_use]
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "MEM-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    let registry = observed.get("registry").cloned().unwrap_or(Value::Null);
    let mapping = match parse_mapping(&registry) {
        Ok(mapping) => mapping,
        Err(message) => {
            findings.insert(Finding::new(
                "MEM-POLICY-MALFORMED",
                POLICY_KEY,
                format!("{message}; the registry must be corrected before the gate can evaluate"),
            ));
            return findings;
        }
    };

    let crates: Vec<String> = string_array(observed, "crates");
    let crate_set: BTreeSet<String> = crates.iter().cloned().collect();
    let crate_count = observed
        .get("crate_count")
        .and_then(Value::as_u64)
        .unwrap_or(crates.len() as u64);

    let min_expected = policy
        .get("min_expected_crates")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if crate_count < min_expected {
        findings.insert(Finding::new(
            "MEM-EMPTY-SCAN",
            POLICY_KEY,
            format!(
                "scan found {crate_count} crate(s), below the policy floor of {min_expected}; the scan roots, CWD, or collection is likely broken (fail-closed against a silent false-green)"
            ),
        ));
    }

    // 1. Exactly-one membership for every crate (advisory for the frozen baseline).
    for crate_dir in &crates {
        let homes = homes_for(&mapping, crate_dir);
        match homes.len() {
            0 => {
                if !mapping.frozen.contains(crate_dir) {
                    findings.insert(Finding::new(
                        "MEM-NEW-UNMAPPED-CRATE",
                        crate_dir,
                        "crate maps to NO registered capability/meta home and is NOT in the frozen unmapped baseline — a NEW unmapped crate is a regression (map it to a capability via the registry, or it must not be added)",
                    ));
                }
            }
            1 => {}
            _ => {
                findings.insert(Finding::new(
                    "MEM-DOUBLE-MAPPED-CRATE",
                    crate_dir,
                    format!(
                        "crate maps to {} homes ({}) — every crate must map to EXACTLY ONE capability/meta home (the closed-set guarantee)",
                        homes.len(),
                        homes.join(", ")
                    ),
                ));
            }
        }
    }

    // 6. The frozen baseline must shrink in lockstep with the moves: a stale entry (no longer a
    //    crate, or now mapped) is drift that would otherwise hide a regression.
    for frozen in &mapping.frozen {
        if !crate_set.contains(frozen) {
            findings.insert(Finding::new(
                "MEM-STALE-FROZEN-BASELINE",
                frozen,
                "frozen-baseline entry is no longer a crate in the tree — the baseline must shrink in lockstep with the strangler moves (remove the stale entry)",
            ));
        } else if !homes_for(&mapping, frozen).is_empty() {
            findings.insert(Finding::new(
                "MEM-STALE-FROZEN-BASELINE",
                frozen,
                "frozen-baseline entry is now mapped to a home — remove it from frozen_unmapped_baseline (the baseline burns DOWN as crates are homed)",
            ));
        }
    }

    // 2. Closed top-level-dir set: no NEW top-level dir outside allowed_top_level_dirs.
    let allowed: BTreeSet<String> = string_array(policy, "allowed_top_level_dirs")
        .into_iter()
        .collect();
    for dir in string_array(observed, "top_level_dirs") {
        if !allowed.contains(&dir) {
            findings.insert(Finding::new(
                "MEM-NEW-TOP-LEVEL-DIR",
                &dir,
                format!(
                    "top-level dir {dir:?} is outside the closed set (meta directories ∪ capability homes ∪ known non-crate dirs); a new top-level dir is a junk-drawer regression — register it as a capability or place its crates under an existing home"
                ),
            ));
        }
    }

    // 3. base/-admission rule over any crate under base/.
    evaluate_base_admission(observed, &registry, &crates, &mut findings);

    // 4. STOP ACCRUAL: the legacy-root census is frozen shrink-only.
    evaluate_legacy_root_freeze(policy, &crates, &crate_set, &mut findings);

    findings
}

/// The frozen legacy roots the policy declares. Empty when the `legacy_root_freeze` block is absent
/// or declares no roots — the freeze is then INERT, which is what a fixture / adopting-repo policy
/// wants (the committed-policy self-test is what proves the live gate is not inert).
#[must_use]
pub fn frozen_legacy_roots(policy: &Value) -> Vec<String> {
    policy
        .get(LEGACY_ROOT_FREEZE_KEY)
        .map(|freeze| string_array(freeze, "frozen_roots"))
        .unwrap_or_default()
}

/// The crate dirs that live under a frozen legacy root, sorted and deduped.
///
/// The SINGLE source of the "is this crate in a legacy root?" predicate: the `--emit-legacy-freeze`
/// producer renders exactly this list, and the evaluator below checks exactly this list against the
/// committed census — so a producer run and a gate run can never disagree about what the census
/// should contain.
#[must_use]
pub fn legacy_root_census(policy: &Value, crates: &[String]) -> Vec<String> {
    let roots = frozen_legacy_roots(policy);
    if roots.is_empty() {
        return Vec::new();
    }
    let mut census: Vec<String> = crates
        .iter()
        .filter(|dir| roots.iter().any(|root| prefix_match(dir, root)))
        .cloned()
        .collect();
    census.sort();
    census.dedup();
    census
}

/// STOP ACCRUAL. The legacy-root crate census is FROZEN SHRINK-ONLY: today's crates are tolerated,
/// a crate BORN under a frozen root is a regression, and a census entry that is no longer a crate is
/// burn-down that must be recorded in the same change (otherwise the census keeps slack the reorg
/// already earned back, and a crate later re-created at that exact path lands pre-forgiven).
fn evaluate_legacy_root_freeze(
    policy: &Value,
    crates: &[String],
    crate_set: &BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) {
    let roots = frozen_legacy_roots(policy);
    if roots.is_empty() {
        return;
    }
    let Some(freeze) = policy.get(LEGACY_ROOT_FREEZE_KEY) else {
        return;
    };
    let census: BTreeSet<String> = string_array(freeze, "crates").into_iter().collect();

    for crate_dir in legacy_root_census(policy, crates) {
        if census.contains(&crate_dir) {
            continue;
        }
        findings.insert(Finding::new(
            "MEM-NEW-LEGACY-ROOT-CRATE",
            &crate_dir,
            format!(
                "crate was born under the FROZEN legacy root {:?}, whose census is shrink-only ({} entries) — the capability-first reorg exists to EMPTY that root, so a new crate there is fresh migration debt. Create it under its capability root instead (<capability>/{{core,ports,adapters,facade}}/ or app/<product>/). If this crate genuinely cannot live anywhere else yet, the census entry must be added by hand and reviewed: the producer refuses to grow it (`--emit-legacy-freeze` without `--allow-new`).",
                roots
                    .iter()
                    .find(|root| prefix_match(&crate_dir, root))
                    .map(String::as_str)
                    .unwrap_or_default(),
                census.len()
            ),
        ));
    }

    for entry in &census {
        if !crate_set.contains(entry) {
            findings.insert(Finding::new(
                "MEM-STALE-LEGACY-ROOT-BASELINE",
                entry,
                "legacy-root census entry is no longer a crate — that is burn-down, and it must be recorded in the SAME change that moved or deleted the crate (re-run `--emit-legacy-freeze`). A census that keeps slack it no longer needs silently pre-forgives the next crate born at that path.",
            ));
        }
    }
}

/// Enforce the ADR-0562 §6 `base/`-admission rule: a `base/` crate must be depended-on by `>=3`
/// capabilities AND be strictly below all of them in the ADR-0280 DAG. PRE-MOVE there is no `base/`
/// dir on the live tree, so this is vacuously green; RED fixtures (a `base/` crate with `<3`
/// consumers or not-below-all) exercise it. The consumer set + DAG relation are read from the
/// optional `observed.base_admission_facts` block (supplied by fixtures / a future producer); when
/// absent for a `base/` crate, the gate fails CLOSED on the consumer check (a `base/` crate with no
/// declared facts cannot prove admission).
fn evaluate_base_admission(
    observed: &Value,
    _registry: &Value,
    crates: &[String],
    findings: &mut BTreeSet<Finding>,
) {
    let facts = observed
        .get("base_admission_facts")
        .and_then(Value::as_object);
    for crate_dir in crates {
        if !(crate_dir == "base" || crate_dir.starts_with("base/")) {
            continue;
        }
        let crate_facts = facts.and_then(|f| f.get(crate_dir));
        let consumers: Vec<String> = crate_facts
            .map(|f| string_array(f, "capability_consumers"))
            .unwrap_or_default();
        if consumers.len() < 3 {
            findings.insert(Finding::new(
                "MEM-BASE-ADMISSION-CONSUMERS",
                crate_dir,
                format!(
                    "base/ crate has {} declared capability consumer(s); admission requires >=3 (base/ is irreducible cross-capability primitives, NOT a util/ junk-drawer)",
                    consumers.len()
                ),
            ));
        }
        let strictly_below = crate_facts
            .and_then(|f| f.get("strictly_below_all_consumers"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if consumers.len() >= 3 && !strictly_below {
            findings.insert(Finding::new(
                "MEM-BASE-ADMISSION-DAG",
                crate_dir,
                "base/ crate is not strictly below all its capability consumers in the ADR-0280 DAG; admission requires it sit beneath every consumer",
            ));
        }
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

    let registry = observed.get("registry").cloned().unwrap_or(Value::Null);
    let crates: Vec<String> = string_array(observed, "crates");
    let crates_checked = observed
        .get("crate_count")
        .and_then(Value::as_u64)
        .map(|n| n as usize)
        .unwrap_or(crates.len());

    let (mapped_to_home, frozen_unmapped) = match parse_mapping(&registry) {
        Ok(mapping) => {
            let mut mapped = 0usize;
            let mut frozen = 0usize;
            for c in &crates {
                if homes_for(&mapping, c).len() == 1 {
                    mapped += 1;
                } else if mapping.frozen.contains(c) {
                    frozen += 1;
                }
            }
            (mapped, frozen)
        }
        Err(_) => (0, 0),
    };

    Report {
        verdict: if violations.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        },
        crates_checked,
        mapped_to_home,
        frozen_unmapped,
        legacy_root_crates: legacy_root_census(policy, &crates).len(),
        violations,
    }
}

/// Render findings for a human / CI log. Empty findings render the GREEN line.
#[must_use]
pub fn render_findings(findings: &BTreeSet<Finding>) -> String {
    if findings.is_empty() {
        return "capability-membership: GREEN — every crate maps to exactly one registered \
                capability/meta home (no NEW unmapped crate, no NEW top-level dir); the frozen \
                unmapped baseline holds steady"
            .to_owned();
    }
    let mut per_code: BTreeMap<String, usize> = BTreeMap::new();
    for f in findings {
        *per_code.entry(f.code.clone()).or_insert(0) += 1;
    }
    let mut lines = vec![format!(
        "capability-membership: RED — {} finding(s) across {} violation class(es):",
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
