//! # cloud-ci-affected-set (ADR-0554)
//!
//! Binding workspace-coverage derivation: the pure decision kernel that turns a merge-base
//! diff into the buck2 target set a PR MUST build and test. Closes FRIC-1781310000 — the
//! pipeline's largest false-green channel: the only binding buck2 lane was scoped to
//! `//cloud/cloud-ci/...`, so code anywhere else (oya/*, libs/*, cloud/* services) could merge
//! broken (proven live: PR #651 head cf16525 did not compile yet its buck2 lane was green;
//! `//oya/ci-webhook-gateway:oya-ci-webhook-gateway-tests` carried an E0428 on dev itself).
//!
//! Precedent (proven patterns, Rust reimplementation): Bazel "target determination" /
//! bazel-diff (Tinder) — derive the affected-target cone from the VCS diff via the build
//! graph; Meta/Google CI runs the reverse-dependency closure of changed sources, with
//! fail-closed escalation to the full workspace when the diff touches graph-semantic files
//! the cone cannot model (macros, toolchains, vendored third-party).
//!
//! ## Born pack-shaped (ADR-0548 R0)
//! The kernel hardcodes NO repo facts: escape-trigger path classes, owner-required path
//! classes, the query universe, the full-run target patterns, and the cell roots are all DATA
//! in the policy JSON. Another buck2 repo adopts this lane by writing its own policy pack.
//!
//! ## Decision contract (fully mechanical — zero manual escape hatches)
//! - [`plan_changes`] `(changes, policy) -> Plan` is PURE: classifies every diff entry.
//! - [`resolve`] `(plan, owner_results, policy) -> Decision` is PURE: folds per-file
//!   `owner()` results into the final verdict.
//! - Verdict dominance (fail-closed): `RefuseUnowned` > `Full` > `Affected` > `NoGraphTargets`.
//!   - `RefuseUnowned`: a file in the owner-required class has NO owning target — the file is
//!     invisible to the build graph, so even a full-workspace run would not compile it.
//!     Running anything would be a false-green; the lane FAILS with the path list.
//!   - `Full`: an escape-trigger path class matched, a graph-relevant file was deleted, or the
//!     adapter reported derivation uncertainty — the rdeps cone cannot be trusted, so the
//!     ENTIRE workspace runs. Escalation is the automation; nothing is ever skipped.
//!   - `Affected`: owners + the reverse-dependency closure (computed by the adapter).
//!   - `NoGraphTargets`: every changed file is unowned AND not in any owner-required class
//!     (docs/config-text outside the buildfile + escape-trigger classes).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

/// The lane id, matching the buck2 target + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-affected-set";

/// Policy pack: ALL repo facts live here (parsed from the policy JSON).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// `gate_id` must equal [`GATE_ID`] (pack/engine pairing check).
    pub gate_id: String,
    /// The uquery universe for the reverse-dependency closure (e.g. `//...`).
    pub universe: String,
    /// Target patterns built+tested in a FULL run (e.g. `["//..."]`).
    pub full_run_targets: Vec<String>,
    /// Path classes that escape the rdeps cone -> FULL run (micro-glob patterns).
    pub full_trigger_patterns: Vec<String>,
    /// Path classes that MUST map to an owning target (micro-glob patterns); an existing
    /// file in this class with no owner is a graph-invisibility defect -> refuse.
    pub require_owner_patterns: Vec<String>,
    /// Buildfile basenames buck2 honors, in PRECEDENCE order (buck2 default: `["BUCK.v2",
    /// "BUCK"]` — `BUCK.v2` SHADOWS `BUCK` when both exist). `owner()` is empty for these BY
    /// DESIGN, so a change to one expands to its package target pattern AND the basename is in
    /// `full_trigger_patterns` (any buildfile edit escalates to FULL, because changing the
    /// package definition can break arbitrary dependents the rdeps cone of the package alone
    /// does not bound). This list is the ground-truth buck2 buildfile-name set, not a single
    /// hand-set name — F2: a stray `BUCK.v2` shadowing a real `BUCK` must not be a plain file.
    pub package_definition_basenames: Vec<String>,
    /// Basenames of package-SIBLING manifests (`Cargo.toml`, `build.rs`): not build-graph
    /// inputs (buck2 never reads them — vendoring/BUCK mirror them) yet semantically bound to
    /// their crate, so they ALSO expand to the enclosing package target pattern. If that
    /// package does not exist, the seed query fails -> the adapter escalates to FULL.
    pub package_sibling_basenames: Vec<String>,
    /// Repo-dir prefix -> buck2 cell root, longest-prefix wins; `""` is the repo root cell
    /// (e.g. `{"": "//"}`). A package file under no mapped cell is derivation uncertainty.
    pub cell_roots: BTreeMap<String, String>,
    /// Micro-glob path pattern -> synthetic seed targets, for changed files that have NO buck2
    /// `owner()` but ARE accounted for: either they seed specific targets (non-empty list) or
    /// they are EXPLICITLY declared inert (empty list `[]` = "this class affects no buck target";
    /// e.g. docs). A changed owner-query path with NO owner that matches NO synthetic pattern AND
    /// is not owner-required is DERIVATION UNCERTAINTY -> FULL (never silently ignored). This is
    /// the [`resolve`] "owner OR explicit synthetic dependency, otherwise FULL" rule. Optional in
    /// the pack (absent = empty map = every unowned non-owner-required path escalates to FULL).
    pub synthetic_dependencies: BTreeMap<String, Vec<String>>,
    /// Default base ref for the merge-base anchor (e.g. `origin/dev`); CLI `--base` overrides.
    pub default_base_ref: String,
}

/// Policy parse/shape error (message is the contract surface; codes stay simple here).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyError(pub String);

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for PolicyError {}

fn str_field(v: &Value, key: &str) -> Result<String, PolicyError> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            PolicyError(format!(
                "policy field `{key}` missing, empty, or not a string"
            ))
        })
}

fn str_list_field(v: &Value, key: &str) -> Result<Vec<String>, PolicyError> {
    let arr = v
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| PolicyError(format!("policy field `{key}` missing or not an array")))?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let s = item.as_str().filter(|s| !s.is_empty()).ok_or_else(|| {
            PolicyError(format!("policy field `{key}` has a non-string/empty entry"))
        })?;
        out.push(s.to_owned());
    }
    Ok(out)
}

impl Policy {
    /// Parse the policy pack from JSON bytes. Fail-closed: any missing/odd field is an error
    /// (the adapter treats a policy error as a hard failure — without the pack not even the
    /// full-run target set is known).
    pub fn from_json(bytes: &str) -> Result<Self, PolicyError> {
        let v: Value = serde_json::from_str(bytes)
            .map_err(|e| PolicyError(format!("policy is not valid JSON: {e}")))?;
        let gate_id = str_field(&v, "gate_id")?;
        if gate_id != GATE_ID {
            return Err(PolicyError(format!(
                "policy gate_id `{gate_id}` does not match engine `{GATE_ID}` — wrong pack for this engine"
            )));
        }
        let cell_roots_v = v
            .get("cell_roots")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                PolicyError("policy field `cell_roots` missing or not an object".into())
            })?;
        let mut cell_roots = BTreeMap::new();
        for (k, val) in cell_roots_v {
            let cell = val.as_str().filter(|s| !s.is_empty()).ok_or_else(|| {
                PolicyError(format!("cell_roots[`{k}`] is not a non-empty string"))
            })?;
            cell_roots.insert(k.clone(), cell.to_owned());
        }
        if cell_roots.is_empty() {
            return Err(PolicyError(
                "policy `cell_roots` must map at least one prefix".into(),
            ));
        }
        let full_run_targets = str_list_field(&v, "full_run_targets")?;
        if full_run_targets.is_empty() {
            return Err(PolicyError(
                "policy `full_run_targets` must be non-empty".into(),
            ));
        }
        let full_trigger_patterns = str_list_field(&v, "full_trigger_patterns")?;
        if full_trigger_patterns.is_empty() {
            return Err(PolicyError(
                "policy `full_trigger_patterns` must be non-empty".into(),
            ));
        }
        let require_owner_patterns = str_list_field(&v, "require_owner_patterns")?;
        if require_owner_patterns.is_empty() {
            return Err(PolicyError(
                "policy `require_owner_patterns` must be non-empty".into(),
            ));
        }
        Ok(Policy {
            gate_id,
            universe: str_field(&v, "universe")?,
            full_run_targets,
            full_trigger_patterns,
            require_owner_patterns,
            package_definition_basenames: {
                let names = str_list_field(&v, "package_definition_basenames")?;
                if names.is_empty() {
                    return Err(PolicyError(
                        "policy `package_definition_basenames` must list at least one buildfile name".into(),
                    ));
                }
                names
            },
            package_sibling_basenames: str_list_field(&v, "package_sibling_basenames")?,
            cell_roots,
            synthetic_dependencies: parse_synthetic_dependencies(&v)?,
            default_base_ref: str_field(&v, "default_base_ref")?,
        })
    }
}

/// Parse the optional `synthetic_dependencies` object (`{pattern: [seed,...]}`). Absent -> empty
/// map. Fail-closed on a malformed shape (a non-object, an empty pattern key, or a non-string/
/// empty seed) — a broken synthetic map must not silently disable the "otherwise FULL" rule.
fn parse_synthetic_dependencies(v: &Value) -> Result<BTreeMap<String, Vec<String>>, PolicyError> {
    let mut map = BTreeMap::new();
    let Some(raw) = v.get("synthetic_dependencies") else {
        return Ok(map);
    };
    let obj = raw.as_object().ok_or_else(|| {
        PolicyError("policy field `synthetic_dependencies` must be an object".into())
    })?;
    for (pattern, targets) in obj {
        if pattern.is_empty() {
            return Err(PolicyError(
                "synthetic_dependencies has an empty pattern key".into(),
            ));
        }
        let arr = targets.as_array().ok_or_else(|| {
            PolicyError(format!(
                "synthetic_dependencies[`{pattern}`] must be an array of target strings"
            ))
        })?;
        let mut seeds = Vec::with_capacity(arr.len());
        for t in arr {
            let s = t.as_str().filter(|s| !s.is_empty()).ok_or_else(|| {
                PolicyError(format!(
                    "synthetic_dependencies[`{pattern}`] has a non-string/empty target"
                ))
            })?;
            seeds.push(s.to_owned());
        }
        map.insert(pattern.clone(), seeds);
    }
    Ok(map)
}

/// Micro-glob match over '/'-separated repo-relative paths.
/// Semantics: `**` matches zero or more whole segments; `*` matches any run of characters
/// WITHIN one segment; everything else is literal. Deliberately tiny + dependency-free so the
/// pack format is fully specified by these three rules.
pub fn glob_match(pattern: &str, path: &str) -> bool {
    let pat: Vec<&str> = pattern.split('/').collect();
    let segs: Vec<&str> = path.split('/').collect();
    match_segments(&pat, &segs)
}

fn match_segments(pat: &[&str], segs: &[&str]) -> bool {
    match pat.split_first() {
        None => segs.is_empty(),
        Some((&"**", rest)) => {
            // `**` consumes zero..all leading segments.
            (0..=segs.len()).any(|skip| match_segments(rest, &segs[skip..]))
        }
        Some((p, rest)) => match segs.split_first() {
            None => false,
            Some((s, seg_rest)) => match_one_segment(p, s) && match_segments(rest, seg_rest),
        },
    }
}

fn match_one_segment(pat: &str, seg: &str) -> bool {
    // Iterative wildcard match (`*` = any run of chars), linear backtracking.
    let p: Vec<char> = pat.chars().collect();
    let s: Vec<char> = seg.chars().collect();
    let (mut pi, mut si) = (0usize, 0usize);
    let (mut star, mut mark) = (None::<usize>, 0usize);
    while si < s.len() {
        if pi < p.len() && (p[pi] == s[si]) {
            pi += 1;
            si += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = si;
            pi += 1;
        } else if let Some(sp) = star {
            pi = sp + 1;
            mark += 1;
            si = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// A structural diff kind whose blast radius the HEAD-only `owner()`/`rdeps()` cone cannot
/// bound. First safe version (ADR-0554 round-6): every one escalates to FULL. A later version
/// may union the base and head owner graphs instead of escalating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuralKind {
    /// `git diff` status `R`: a rename moves a file between packages — the OLD package loses a
    /// source and the NEW package gains one; the head-only cone models neither move soundly.
    Rename,
    /// `git diff` status `C`: a copy adds a source whose provenance the cone cannot attribute.
    Copy,
    /// `git diff` status `T`: a type change (e.g. file <-> symlink, blob <-> gitlink/submodule)
    /// is graph-semantic — buck2 resolves a symlink/gitlink differently from a regular file.
    TypeChange,
}

impl StructuralKind {
    /// Human-facing label for the transparency block / FULL reason.
    pub fn describe(self) -> &'static str {
        match self {
            StructuralKind::Rename => "rename",
            StructuralKind::Copy => "copy",
            StructuralKind::TypeChange => "type-change",
        }
    }
}

/// One diff entry, as parsed by the adapter from `git diff --name-status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Added (`A`) or modified (`M`): a file present at HEAD, routed to `owner()` resolution.
    Present(String),
    /// Deleted (`D`): the file is gone at HEAD, so `owner()` cannot resolve it and deleting a
    /// source can break every dependent of its former target -> FULL, unconditionally.
    Deleted(String),
    /// A rename/copy/type-change (`R`/`C`/`T`): a structural change the head-only cone cannot
    /// bound -> FULL. `path` is the file present at HEAD (destination for R/C) for the reason.
    Structural { path: String, kind: StructuralKind },
}

impl Change {
    pub fn path(&self) -> &str {
        match self {
            Change::Present(p) | Change::Deleted(p) => p,
            Change::Structural { path, .. } => path,
        }
    }
}

/// Parse `git diff --name-status -z` output into `Change`s (PURE; moved from the adapter so the
/// diff-kind -> FULL escalation is unit-testable). NUL-separated records; `R`/`C` carry two
/// paths. Rename/copy/type-change map to a single [`Change::Structural`] -> FULL (their blast
/// radius is not bounded by the head-only cone). A submodule change surfaces as a gitlink path
/// with no buck2 owner and is escalated by the unowned-unmapped rule in [`resolve`], or as a
/// type change (`T`) when a path flips to/from a gitlink.
pub fn parse_name_status_z(raw: &str) -> Result<Vec<Change>, String> {
    let mut fields = raw.split('\0').filter(|s| !s.is_empty());
    let mut changes = Vec::new();
    while let Some(status) = fields.next() {
        let kind = status.chars().next().ok_or("empty status field")?;
        match kind {
            'A' | 'M' => {
                let p = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without a path"))?;
                changes.push(Change::Present(p.to_owned()));
            }
            'D' => {
                let p = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without a path"))?;
                changes.push(Change::Deleted(p.to_owned()));
            }
            'T' => {
                let p = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without a path"))?;
                changes.push(Change::Structural {
                    path: p.to_owned(),
                    kind: StructuralKind::TypeChange,
                });
            }
            'R' => {
                let _old = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without source path"))?;
                let new = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without destination path"))?;
                changes.push(Change::Structural {
                    path: new.to_owned(),
                    kind: StructuralKind::Rename,
                });
            }
            'C' => {
                let _src = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without source path"))?;
                let dst = fields
                    .next()
                    .ok_or_else(|| format!("status `{status}` without destination path"))?;
                changes.push(Change::Structural {
                    path: dst.to_owned(),
                    kind: StructuralKind::Copy,
                });
            }
            // U (unmerged), X (unknown), B (broken pair): states a clean CI checkout cannot
            // produce — surface as uncertainty rather than guessing.
            other => return Err(format!("unsupported diff status `{other}`")),
        }
    }
    Ok(changes)
}

/// Why a path was classified the way it was — carried verbatim into the transparency output
/// (the founder automation directive: FAIL output must say exactly what ran and why).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathClass {
    /// Matched an escape-trigger pattern -> FULL.
    FullTrigger(String),
    /// Deleted at HEAD -> FULL (its cone is uncomputable at HEAD; deleting a source can break
    /// every dependent of its former target). ALL deletions escalate (first safe version).
    Deleted,
    /// Rename/copy/type-change -> FULL (structural change the head-only cone cannot bound).
    Structural(StructuralKind),
    /// Buildfile (BUCK/BUCK.v2) changed -> FULL (blast radius exceeds its own package: it can
    /// add/remove targets or shadow the file dependents load).
    Buildfile,
    /// Package-sibling manifest -> expands to this package target pattern.
    PackagePattern(String),
    /// Sent to `owner()` resolution.
    OwnerQuery,
}

/// The pure classification of a diff (phase A). The adapter answers `owner_paths` with buck2
/// `owner()` results, then [`resolve`] (phase B) folds them into the verdict.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Plan {
    /// Reasons forcing a FULL run (escape triggers, graph deletions, uncertainty).
    pub full_reasons: Vec<String>,
    /// Package target patterns from package-definition files (seeds).
    pub package_patterns: Vec<String>,
    /// Existing files whose owning targets must be queried.
    pub owner_paths: Vec<String>,
    /// Per-path classification, for the transparency block.
    pub classified: Vec<(String, PathClass)>,
}

/// Classify every change (PURE). Order per path: escape-trigger -> package-definition ->
/// deletion handling -> owner query. EVERY existing file goes to `owner()` regardless of
/// extension: a non-source file can be a declared src of a target (`include_str!` assets),
/// so extension pre-filtering would be a false-negative hole.
pub fn plan_changes(changes: &[Change], policy: &Policy) -> Plan {
    let mut plan = Plan::default();
    for change in changes {
        let path = change.path();
        // Structural diff kinds escalate to FULL unconditionally — the head-only owner()/rdeps()
        // cone cannot bound their blast radius (first safe version, ADR-0554 round-6). A deletion
        // removes a source the cone cannot see at HEAD; a rename/copy/type-change moves or
        // reshapes graph inputs the head snapshot cannot attribute. Handled BEFORE the path-class
        // checks so, e.g., a renamed `Cargo.toml` cannot be mistaken for a plain sibling edit.
        match change {
            Change::Deleted(_) => {
                plan.full_reasons.push(format!(
                    "file `{path}` was deleted (deletion blast radius is not bounded by the head-only rdeps cone)"
                ));
                plan.classified.push((path.to_owned(), PathClass::Deleted));
                continue;
            }
            Change::Structural { kind, .. } => {
                plan.full_reasons.push(format!(
                    "`{path}` is a {} (structural change the head-only cone cannot bound)",
                    kind.describe()
                ));
                plan.classified
                    .push((path.to_owned(), PathClass::Structural(*kind)));
                continue;
            }
            Change::Present(_) => {}
        }
        if let Some(pat) = policy
            .full_trigger_patterns
            .iter()
            .find(|pat| glob_match(pat, path))
        {
            plan.full_reasons
                .push(format!("`{path}` matches escape-trigger `{pat}`"));
            plan.classified
                .push((path.to_owned(), PathClass::FullTrigger(pat.clone())));
            continue;
        }
        let basename = path.rsplit('/').next().unwrap_or(path);
        // Buildfile change (BUCK.v2/BUCK/PACKAGE) -> FULL, ALWAYS. Its blast radius is NOT
        // bounded by its own package's rdeps: a new BUCK.v2 SHADOWS the BUCK that dependents
        // load (F2), a new/edited buildfile can add/remove targets dependents resolve, and a
        // PACKAGE file mutates parse-time values for the whole subtree. owner() is empty for a
        // buildfile by design, so seeding "its package" would silently miss every dependent —
        // the exact F2 false-negative. Escalate to the full workspace.
        if policy
            .package_definition_basenames
            .iter()
            .any(|b| b == basename)
        {
            plan.full_reasons.push(format!(
                "buildfile `{path}` changed (blast radius exceeds its own package)"
            ));
            plan.classified
                .push((path.to_owned(), PathClass::Buildfile));
            continue;
        }
        if policy
            .package_sibling_basenames
            .iter()
            .any(|b| b == basename)
        {
            match package_pattern(path, policy) {
                Some(pat) => {
                    plan.package_patterns.push(pat.clone());
                    plan.classified
                        .push((path.to_owned(), PathClass::PackagePattern(pat)));
                }
                None => {
                    plan.full_reasons.push(format!(
                        "package sibling `{path}` maps to no configured cell root (derivation uncertainty)"
                    ));
                    plan.classified.push((
                        path.to_owned(),
                        PathClass::FullTrigger("(no cell root)".to_owned()),
                    ));
                }
            }
            continue;
        }
        plan.owner_paths.push(path.to_owned());
        plan.classified
            .push((path.to_owned(), PathClass::OwnerQuery));
    }
    plan.package_patterns.sort();
    plan.package_patterns.dedup();
    plan
}

/// Map a package-sibling manifest to its enclosing package target pattern via the cell roots
/// (longest prefix wins). `cloud/x/Cargo.toml` + `{"": "//"}` -> `//cloud/x:`.
fn package_pattern(path: &str, policy: &Policy) -> Option<String> {
    let dir = match path.rsplit_once('/') {
        Some((d, _)) => d,
        None => "",
    };
    let mut best: Option<(&str, &str)> = None;
    for (prefix, cell) in &policy.cell_roots {
        let applies = prefix.is_empty() || dir == prefix || dir.starts_with(&format!("{prefix}/"));
        if applies && best.is_none_or(|(bp, _)| prefix.len() > bp.len()) {
            best = Some((prefix, cell));
        }
    }
    best.map(|(prefix, cell)| {
        let rel = dir.strip_prefix(prefix).unwrap_or(dir);
        let rel = rel.strip_prefix('/').unwrap_or(rel);
        format!("{cell}{rel}:")
    })
}

/// The final verdict. Dominance: `RefuseUnowned` > `Full` > `Affected` > `NoGraphTargets`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Owner-required files with NO owning target: graph-invisible code. Even a full run
    /// would not compile these — running anything would be false-green, so the lane fails.
    RefuseUnowned { paths: Vec<String> },
    /// Run the policy's full-run target patterns.
    Full { reasons: Vec<String> },
    /// Run these seed targets + their reverse-dependency closure.
    Affected { seeds: Vec<String> },
    /// Every change is unowned AND not in any owner-required class.
    NoGraphTargets,
}

/// Fold per-file `owner()` results into the verdict (PURE).
pub fn resolve(
    plan: &Plan,
    owner_results: &BTreeMap<String, Vec<String>>,
    policy: &Policy,
) -> Decision {
    let mut refusals: Vec<String> = Vec::new();
    let mut seeds: BTreeSet<String> = plan.package_patterns.iter().cloned().collect();
    let mut unmapped: Vec<String> = Vec::new();
    for path in &plan.owner_paths {
        let owners = owner_results.get(path).map(Vec::as_slice).unwrap_or(&[]);
        if !owners.is_empty() {
            seeds.extend(owners.iter().cloned());
            continue;
        }
        // Unowned Present file. A graph-invisibility refusal dominates: even a full-workspace run
        // would not compile an owner-required source with no owning target.
        if policy
            .require_owner_patterns
            .iter()
            .any(|pat| glob_match(pat, path))
        {
            refusals.push(path.clone());
            continue;
        }
        // Otherwise the path must map to an EXPLICIT synthetic-dependency declaration (specific
        // seed targets, or `[]` = declared inert). No owner and no declaration is derivation
        // uncertainty -> FULL (the old silent "docs class — fine" default was the selector hole).
        match synthetic_seeds(path, policy) {
            Some(synth) => seeds.extend(synth),
            None => unmapped.push(path.clone()),
        }
    }
    if !refusals.is_empty() {
        refusals.sort();
        return Decision::RefuseUnowned { paths: refusals };
    }
    let mut full_reasons = plan.full_reasons.clone();
    for path in &unmapped {
        full_reasons.push(format!(
            "unowned path `{path}` has no buck2 owner and no synthetic-dependency declaration (derivation uncertainty)"
        ));
    }
    if !full_reasons.is_empty() {
        return Decision::Full {
            reasons: full_reasons,
        };
    }
    if !seeds.is_empty() {
        return Decision::Affected {
            seeds: seeds.into_iter().collect(),
        };
    }
    Decision::NoGraphTargets
}

/// The union of synthetic seed targets from EVERY `synthetic_dependencies` pattern matching
/// `path`, or `None` if NO pattern matched. `Some(vec![])` means the path matched at least one
/// EXPLICIT inert declaration (`[]`) — accounted for, seeds no target. Used only for unowned,
/// non-owner-required paths (owned paths are seeded by `owner()`; owner-required unowned paths
/// refuse), so a synthetic declaration never shadows a real owner.
fn synthetic_seeds(path: &str, policy: &Policy) -> Option<Vec<String>> {
    let mut matched = false;
    let mut seeds = Vec::new();
    for (pattern, targets) in &policy.synthetic_dependencies {
        if glob_match(pattern, path) {
            matched = true;
            seeds.extend(targets.iter().cloned());
        }
    }
    matched.then_some(seeds)
}

/// One actual phase outcome recorded by the affected-set composition root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatePhaseOutcome {
    /// Stable phase id.
    pub phase: String,
    /// Observed outcome (`completed`, `failed-escalated`, `not-run`, `completed-check-exit-code`, ...).
    pub status: String,
    /// Operator-facing pointer to the relevant artifact field, command, or reason.
    pub operator_signal: String,
}

impl GatePhaseOutcome {
    pub fn new(
        phase: impl Into<String>,
        status: impl Into<String>,
        operator_signal: impl Into<String>,
    ) -> Self {
        Self {
            phase: phase.into(),
            status: status.into(),
            operator_signal: operator_signal.into(),
        }
    }
}
/// Render a live long-step telemetry line for CI logs.
///
/// The line is intentionally plain text instead of JSON so GitHub Actions displays it while the
/// child command is still running. Machine-readable end-state evidence remains in the uploaded
/// operator artifact.
pub fn long_step_telemetry_line(
    component: &str,
    phase: &str,
    status: &str,
    elapsed_seconds: u64,
    detail: &str,
) -> String {
    format!("{component}: phase={phase} status={status} elapsed_seconds={elapsed_seconds} {detail}")
}

/// Machine-readable operator artifact for the affected-set tier decision.
///
/// The artifact is redaction-safe: it contains refs, target labels, path classifications already
/// printed by the gate, and policy state; it never embeds secrets, DSNs, or raw environment values.
pub fn affected_set_operator_artifact(
    mode: &str,
    resolved_base_ref: &str,
    resolved_head_ref: &str,
    baseline_report_present: bool,
    decision: &Decision,
    phases: &[GatePhaseOutcome],
) -> Value {
    let decision_value = match decision {
        Decision::RefuseUnowned { paths } => json!({
            "tier": "REFUSE_UNOWNED",
            "will_run": false,
            "paths": paths,
        }),
        Decision::Full { reasons } => json!({
            "tier": "FULL",
            "will_run": true,
            "reasons": reasons,
        }),
        Decision::Affected { seeds } => json!({
            "tier": "AFFECTED",
            "will_run": true,
            "seed_count": seeds.len(),
            "seeds": seeds,
        }),
        Decision::NoGraphTargets => json!({
            "tier": "NO_GRAPH_TARGETS",
            "will_run": false,
            "reasons": ["every changed file is unowned and not owner-required"],
        }),
    };

    json!({
        "schema_version": 1,
        "artifact_type": "cloud_ci_operator_artifact",
        "artifact_id": "affected-set-tier-decision",
        "gate_id": GATE_ID,
        "mode": mode,
        "resolved_refs": {
            "base": resolved_base_ref,
            "head": resolved_head_ref,
        },
        "decision": decision_value,
        "merge_base_build_health_baseline": {
            "required": matches!(decision, Decision::Full { .. }) && mode == "auto",
            "report_present": baseline_report_present,
            "anti_laundering": "baseline report must be produced from the merge-base committed tree, never the candidate tree"
        },
        "long_running_gate_phases": phases
            .iter()
            .map(|phase| json!({
                "phase": phase.phase.as_str(),
                "status": phase.status.as_str(),
                "operator_signal": phase.operator_signal.as_str(),
            }))
            .collect::<Vec<_>>(),
        "retention_and_pii": {
            "retention_days": 30,
            "pii": "none; refs, repo paths, and target labels only",
            "secret_redaction": "no tenant, idempotency, DSN, token, or password material is emitted"
        }
    })
}

// ── BUILD-HEALTH RATCHET (ADR-0554 round-3; reuses the ADR-0551/#698 merge-base frozen-baseline
//    pattern). The FULL tier hard-failing on ANY `//...` build failure is a flag-day requirement
//    (the whole workspace must compile before any BUCK-touching PR can merge) — it violates the
//    founder merge-base-ratchet doctrine (block NEW debt, grandfather pre-existing; FRIC-1781112000
//    / #698). The ratchet compares the set of build FAILURES at the PR head against the set at the
//    merge-base: a head failure that was ALSO failing at the merge-base is GRANDFATHERED (shrink-only
//    burn-down); a head failure that built at the merge-base — or a brand-new target that fails — is a
//    REGRESSION and BLOCKS. Soundness (the #698 F1 lesson): the baseline is the merge-base build
//    result, materialized out-of-band from the merge-base checkout, NEVER the candidate tree, so a PR
//    cannot launder a regression by growing its own baseline.

/// One target's build status, parsed from a buck2 `--build-report` JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetBuildStatus {
    Success,
    Fail,
}

/// Parse a buck2 `--build-report` JSON document into `{ target_label -> status }`.
///
/// buck2 emits `results.<unconfigured-label>.success = "SUCCESS"|"FAIL"|...`. We KEY ON THE
/// UNCONFIGURED LABEL (the `results` map key) so the baseline/head sets are comparable across
/// runs regardless of the per-run configuration hash (`#<hash>`) that appears in `failures`.
/// Any `success` value other than the literal `"SUCCESS"` is treated as a FAILURE (fail-closed:
/// `FAIL`, `SKIPPED`, `CANCELLED`, or an unexpected value all count as not-known-good).
pub fn parse_build_report(json: &str) -> Result<BTreeMap<String, TargetBuildStatus>, String> {
    let v: Value =
        serde_json::from_str(json).map_err(|e| format!("build-report is not valid JSON: {e}"))?;
    let results = v
        .get("results")
        .and_then(Value::as_object)
        .ok_or("build-report has no `results` object")?;
    let mut map = BTreeMap::new();
    for (label, entry) in results {
        let status = match entry.get("success").and_then(Value::as_str) {
            Some("SUCCESS") => TargetBuildStatus::Success,
            // FAIL / SKIPPED / CANCELLED / anything-else -> not-known-good (fail-closed).
            _ => TargetBuildStatus::Fail,
        };
        map.insert(label.clone(), status);
    }
    Ok(map)
}

/// The set of target labels that FAILED in a build-report (the failure set used by the ratchet).
pub fn failing_targets(report: &BTreeMap<String, TargetBuildStatus>) -> BTreeSet<String> {
    report
        .iter()
        .filter(|(_, s)| **s == TargetBuildStatus::Fail)
        .map(|(label, _)| label.clone())
        .collect()
}

/// Expected trusted dev-push artifact name for a build-health baseline produced at `merge_base_sha`.
///
/// The consumer workflow still validates GitHub Actions provenance (push-to-dev, successful
/// `oya-ci-required` run, exact `head_sha`) before download. This pure helper pins the artifact name
/// and SHA shape so stale/wrong artifacts cannot be confused with an exact merge-base baseline.
pub fn build_health_baseline_artifact_name(merge_base_sha: &str) -> Result<String, String> {
    let sha = merge_base_sha.trim();
    if sha.len() != 40 || !sha.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return Err(format!(
            "merge-base SHA must be a 40-character hex object id, got `{merge_base_sha}`"
        ));
    }
    Ok(format!("build-health-baseline-{sha}"))
}

/// Expected trusted dev-push artifact name for a normalized test-health baseline.
pub fn test_health_baseline_artifact_name(merge_base_sha: &str) -> Result<String, String> {
    let sha = merge_base_sha.trim();
    let _ = build_health_baseline_artifact_name(sha)?;
    Ok(format!("test-health-baseline-{sha}"))
}

/// Select the trusted push-to-dev workflow run whose head SHA is the exact merge-base.
pub fn trusted_dev_push_run_id(
    runs_json: &str,
    merge_base_sha: &str,
) -> Result<Option<u64>, String> {
    trusted_dev_push_run(runs_json, merge_base_sha).map(|run| run.map(|trusted_run| trusted_run.id))
}

/// Exact trusted workflow-run provenance used by the baseline-pair selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedDevPushRun {
    pub id: u64,
    pub repository_id: u64,
}

fn trusted_dev_push_runs(
    runs_json: &str,
    merge_base_sha: &str,
) -> Result<Vec<TrustedDevPushRun>, String> {
    const REQUIRED_WORKFLOW_PATH: &str = ".github/workflows/oya-ci-required.yml";

    let sha = merge_base_sha.trim();
    let _ = build_health_baseline_artifact_name(sha)?;
    let payload: Value = serde_json::from_str(runs_json)
        .map_err(|e| format!("workflow-runs payload is not valid JSON: {e}"))?;
    let runs = payload
        .get("workflow_runs")
        .and_then(Value::as_array)
        .ok_or("workflow-runs payload has no `workflow_runs` array")?;

    let mut trusted_runs = Vec::new();
    for run in runs {
        let exact_trusted_run = run.get("head_sha").and_then(Value::as_str) == Some(sha)
            && run.get("event").and_then(Value::as_str) == Some("push")
            && run.get("head_branch").and_then(Value::as_str) == Some("dev")
            && run.get("status").and_then(Value::as_str) == Some("completed")
            && run.get("conclusion").and_then(Value::as_str) == Some("success")
            && run.get("path").and_then(Value::as_str) == Some(REQUIRED_WORKFLOW_PATH);
        if !exact_trusted_run {
            continue;
        }

        let repository_id = run
            .pointer("/repository/id")
            .and_then(Value::as_u64)
            .ok_or("matching trusted workflow run has no numeric `repository.id`")?;
        let head_repository_id = run
            .pointer("/head_repository/id")
            .and_then(Value::as_u64)
            .ok_or("matching trusted workflow run has no numeric `head_repository.id`")?;
        if repository_id != head_repository_id {
            return Err(format!(
                "matching workflow run head repository {head_repository_id} does not match trusted repository {repository_id}"
            ));
        }
        let id = run
            .get("id")
            .and_then(Value::as_u64)
            .ok_or("matching trusted workflow run has no numeric `id`")?;
        trusted_runs.push(TrustedDevPushRun { id, repository_id });
    }

    trusted_runs.sort_unstable_by_key(|run| std::cmp::Reverse(run.id));
    if trusted_runs.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(
            "workflow-runs payload contains a duplicate trusted workflow run id".to_owned(),
        );
    }
    if let Some(repository_id) = trusted_runs.first().map(|run| run.repository_id)
        && trusted_runs
            .iter()
            .any(|run| run.repository_id != repository_id)
    {
        return Err(
            "trusted workflow runs span multiple repository identities; refusing ambiguous provenance"
                .to_owned(),
        );
    }

    Ok(trusted_runs)
}

/// Select the newest exact successful `oya-ci-required` push-to-dev run.
///
/// Canonical ordering is descending immutable GitHub run ID, independent of API payload order.
/// The #1323 pair selector can continue to an older trusted rerun if the newest run lacks a
/// complete BUILD + TEST pair.
pub fn trusted_dev_push_run(
    runs_json: &str,
    merge_base_sha: &str,
) -> Result<Option<TrustedDevPushRun>, String> {
    trusted_dev_push_runs(runs_json, merge_base_sha).map(|runs| runs.into_iter().next())
}

/// Select the unexpired exact-name build-health baseline artifact from a trusted run.
pub fn trusted_build_health_baseline_artifact_id(
    artifacts_json: &str,
    artifact_name: &str,
) -> Result<Option<u64>, String> {
    let payload: Value = serde_json::from_str(artifacts_json)
        .map_err(|e| format!("workflow-artifacts payload is not valid JSON: {e}"))?;
    let artifacts = payload
        .get("artifacts")
        .and_then(Value::as_array)
        .ok_or("workflow-artifacts payload has no `artifacts` array")?;

    for artifact in artifacts {
        if artifact.get("name").and_then(Value::as_str) == Some(artifact_name) {
            if artifact
                .get("expired")
                .and_then(Value::as_bool)
                .unwrap_or(true)
            {
                return Ok(None);
            }
            return artifact
                .get("id")
                .and_then(Value::as_u64)
                .map(Some)
                .ok_or("matching trusted artifact has no numeric `id`".to_owned());
        }
    }

    Ok(None)
}

/// Metadata for one selected immutable GitHub Actions artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedBaselineArtifact {
    pub id: u64,
    pub name: String,
}

/// Atomic BUILD + TEST baseline pair from one exact trusted dev-push run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedBaselineSelection {
    pub merge_base_sha: String,
    pub run_id: u64,
    pub repository_id: u64,
    pub build_artifact: TrustedBaselineArtifact,
    pub test_artifact: TrustedBaselineArtifact,
}

fn trusted_artifact_for_run(
    artifacts: &[Value],
    expected_name: &str,
    run: &TrustedDevPushRun,
    merge_base_sha: &str,
) -> Result<Option<TrustedBaselineArtifact>, String> {
    let mut matching = Vec::new();
    for artifact in artifacts
        .iter()
        .filter(|artifact| artifact.get("name").and_then(Value::as_str) == Some(expected_name))
    {
        let workflow_run = artifact.get("workflow_run").ok_or_else(|| {
            format!("artifact `{expected_name}` has no `workflow_run` provenance")
        })?;
        let artifact_run_id = workflow_run
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("artifact `{expected_name}` has no numeric workflow run id"))?;
        if artifact_run_id == run.id {
            matching.push(artifact);
        }
    }
    if matching.len() > 1 {
        return Err(format!(
            "workflow-artifacts payload contains duplicate exact-name artifact `{expected_name}` for trusted workflow run {}",
            run.id
        ));
    }
    let Some(artifact) = matching.first().copied() else {
        return Ok(None);
    };
    if artifact
        .get("expired")
        .and_then(Value::as_bool)
        .unwrap_or(true)
    {
        return Ok(None);
    }

    let workflow_run = artifact
        .get("workflow_run")
        .ok_or_else(|| format!("artifact `{expected_name}` has no `workflow_run` provenance"))?;
    if workflow_run.get("head_sha").and_then(Value::as_str) != Some(merge_base_sha) {
        return Err(format!(
            "artifact `{expected_name}` head SHA does not match merge-base `{merge_base_sha}`"
        ));
    }
    if workflow_run.get("head_branch").and_then(Value::as_str) != Some("dev") {
        return Err(format!(
            "artifact `{expected_name}` was not produced from the trusted `dev` branch"
        ));
    }
    let repository_id = workflow_run
        .get("repository_id")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("artifact `{expected_name}` has no numeric `repository_id`"))?;
    let head_repository_id = workflow_run
        .get("head_repository_id")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("artifact `{expected_name}` has no numeric `head_repository_id`"))?;
    if repository_id != run.repository_id || head_repository_id != run.repository_id {
        return Err(format!(
            "artifact `{expected_name}` repository provenance does not match trusted repository {}",
            run.repository_id
        ));
    }

    let id = artifact
        .get("id")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("artifact `{expected_name}` has no numeric `id`"))?;
    Ok(Some(TrustedBaselineArtifact {
        id,
        name: expected_name.to_owned(),
    }))
}

fn validate_exact_artifact_run_membership(
    artifacts: &[Value],
    expected_names: &[&str],
    trusted_runs: &[TrustedDevPushRun],
) -> Result<(), String> {
    let trusted_run_ids: BTreeSet<u64> = trusted_runs.iter().map(|run| run.id).collect();
    for artifact in artifacts.iter().filter(|artifact| {
        artifact
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| expected_names.contains(&name))
    }) {
        let Some(name) = artifact.get("name").and_then(Value::as_str) else {
            continue;
        };
        let artifact_run_id = artifact
            .pointer("/workflow_run/id")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("artifact `{name}` has no numeric workflow run id"))?;
        if !trusted_run_ids.contains(&artifact_run_id) {
            return Err(format!(
                "artifact `{name}` workflow run {artifact_run_id} is not an exact trusted dev-push run"
            ));
        }
    }
    Ok(())
}

/// Select an atomic trusted BUILD + TEST baseline pair for the exact merge-base.
///
/// Missing or expired artifacts return `Ok(None)` so the workflow can execute its clean-worktree
/// cold fallback. Malformed or mismatched provenance is an error, never a fallback.
pub fn select_trusted_baseline_artifacts(
    runs_json: &str,
    artifacts_json: &str,
    merge_base_sha: &str,
) -> Result<Option<TrustedBaselineSelection>, String> {
    let sha = merge_base_sha.trim();
    let build_name = build_health_baseline_artifact_name(sha)?;
    let test_name = test_health_baseline_artifact_name(sha)?;
    let trusted_runs = trusted_dev_push_runs(runs_json, sha)?;
    if trusted_runs.is_empty() {
        return Ok(None);
    }
    let payload: Value = serde_json::from_str(artifacts_json)
        .map_err(|e| format!("workflow-artifacts payload is not valid JSON: {e}"))?;
    let artifacts = payload
        .get("artifacts")
        .and_then(Value::as_array)
        .ok_or("workflow-artifacts payload has no `artifacts` array")?;
    validate_exact_artifact_run_membership(
        artifacts,
        &[build_name.as_str(), test_name.as_str()],
        &trusted_runs,
    )?;

    for run in trusted_runs {
        let Some(build_artifact) = trusted_artifact_for_run(artifacts, &build_name, &run, sha)?
        else {
            continue;
        };
        let Some(test_artifact) = trusted_artifact_for_run(artifacts, &test_name, &run, sha)?
        else {
            continue;
        };

        return Ok(Some(TrustedBaselineSelection {
            merge_base_sha: sha.to_owned(),
            run_id: run.id,
            repository_id: run.repository_id,
            build_artifact,
            test_artifact,
        }));
    }

    Ok(None)
}

/// Validate a trusted build-health baseline artifact payload after provenance selection.
///
/// Returns the number of build-report results. Empty/invalid reports are refused because an empty
/// baseline would launder every head failure into "brand-new but unproven" ambiguity.
pub fn validate_trusted_build_health_baseline_artifact(
    artifact_name: &str,
    merge_base_sha: &str,
    report_json: &str,
) -> Result<usize, String> {
    let expected = build_health_baseline_artifact_name(merge_base_sha)?;
    if artifact_name != expected {
        return Err(format!(
            "build-health baseline artifact name `{artifact_name}` does not match expected `{expected}`"
        ));
    }
    let report = parse_build_report(report_json)?;
    if report.is_empty() {
        return Err("build-health baseline artifact has an empty `results` object".to_owned());
    }
    Ok(report.len())
}

/// Validate a trusted normalized test-health baseline artifact after provenance selection.
pub fn validate_trusted_test_health_baseline_artifact(
    artifact_name: &str,
    merge_base_sha: &str,
    report_json: &str,
) -> Result<usize, String> {
    let expected = test_health_baseline_artifact_name(merge_base_sha)?;
    if artifact_name != expected {
        return Err(format!(
            "test-health baseline artifact name `{artifact_name}` does not match expected `{expected}`"
        ));
    }
    let report = parse_build_report(report_json)?;
    if report.is_empty() {
        return Err("test-health baseline artifact has an empty `results` object".to_owned());
    }
    Ok(report.len())
}

/// The build-health verdict: regressions BLOCK, pre-existing failures are GRANDFATHERED.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildHealthVerdict {
    /// Targets that FAIL at head but did NOT fail at the merge-base (built there, or are brand
    /// new): REGRESSIONS. Non-empty => BLOCK.
    pub regressions: Vec<String>,
    /// Targets that fail at head AND failed at the merge-base: GRANDFATHERED (shrink-only).
    pub grandfathered: Vec<String>,
    /// Targets that failed at the merge-base but now BUILD at head: burned-down (informational).
    pub fixed: Vec<String>,
}

impl BuildHealthVerdict {
    /// The required-context verdict: green IFF there are no regressions.
    pub fn is_green(&self) -> bool {
        self.regressions.is_empty()
    }
}

/// Compute the build-health verdict (PURE).
///
/// `baseline_failures` = failing-target set at the MERGE-BASE (frozen, out-of-band).
/// `head_failures`     = failing-target set at the PR HEAD.
///
/// A head failure is a REGRESSION iff it is not in the baseline failure set (it built at the
/// merge-base, or the target is brand-new). A head failure that IS in the baseline is
/// grandfathered. This is exactly set-difference, so a PR can only ever SHRINK the grandfathered
/// set or ADD a regression — it can never launder a regression into the baseline, because the
/// baseline is supplied from the merge-base build, not from any candidate-controlled input.
pub fn build_health_verdict(
    baseline_failures: &BTreeSet<String>,
    head_failures: &BTreeSet<String>,
) -> BuildHealthVerdict {
    let regressions: Vec<String> = head_failures
        .difference(baseline_failures)
        .cloned()
        .collect();
    let grandfathered: Vec<String> = head_failures
        .intersection(baseline_failures)
        .cloned()
        .collect();
    let fixed: Vec<String> = baseline_failures
        .difference(head_failures)
        .cloned()
        .collect();
    BuildHealthVerdict {
        regressions,
        grandfathered,
        fixed,
    }
}

// ── TEST-HEALTH RATCHET (ADR-0554 round-6; the FULL-tier test-health follow-up ADR-0554 §"declared
//    next IP"). A FULL fallback that only BUILDS is "checking less": a target can BUILD and still
//    FAIL its tests at runtime, and buck2's `--build-report` marks such a target `"success":
//    "SUCCESS"` (verified live — the report captures BUILD status only). So the test-health ratchet
//    cannot reuse the build-report; it parses buck2 `test`'s per-target verdict lines instead, then
//    reuses `build_health_verdict` over the resulting failure sets (block test REGRESSIONS,
//    grandfather pre-existing test debt — the same merge-base ratchet the build side uses).

/// One test target's outcome, distilled from buck2 `test`'s per-target console verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    /// buck2 printed `Pass: <label>`.
    Pass,
    /// buck2 printed `Fail:`/`Timeout:`/`Fatal:` for the label — not-known-good (fail-closed).
    Fail,
}

/// buck2 `test`'s `Tests finished:` summary counts (the reconciliation ground truth).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TestSummary {
    pass: usize,
    fail: usize,
    timeout: usize,
    fatal: usize,
    infra_failure: usize,
}

/// Strip ANSI SGR escape sequences (`\x1b[...m`) so the parser matches on plain text.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Consume up to and including the final byte of a CSI sequence (`m`, or any letter).
            for e in chars.by_ref() {
                if e.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Read the non-negative integer immediately following `keyword` in `hay` (first occurrence).
fn count_after(hay: &str, keyword: &str) -> Option<usize> {
    let idx = hay.find(keyword)? + keyword.len();
    let digits: String = hay[idx..]
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// Parse the `Tests finished: Pass P. Fail F. Timeout T. Fatal Ft. Skip S. Omit O. Infra Failure
/// IF. Build failure BF` summary line. `Fail ` never matches inside `Infra Failure`/`Build failure`
/// (the space after `Fail` is absent in `Failure`).
fn parse_test_summary(line: &str) -> Option<TestSummary> {
    if !line.contains("Tests finished:") {
        return None;
    }
    Some(TestSummary {
        pass: count_after(line, "Pass ")?,
        fail: count_after(line, "Fail ")?,
        timeout: count_after(line, "Timeout ")?,
        fatal: count_after(line, "Fatal ")?,
        infra_failure: count_after(line, "Infra Failure ")?,
    })
}

/// Parse a per-target verdict line: `<glyph/timestamp...> <Verdict>: <label> (<t>s)`. Returns the
/// target label + status for `Pass:`/`Fail:`/`Timeout:`/`Fatal:`; `None` for any other line
/// (including `Skip:`/`Omit:`, which are not ratcheted target outcomes). The `<Verdict>: ` token
/// (a colon) disambiguates from the summary's `Pass 1.` (a space + digit).
fn parse_target_verdict_line(line: &str) -> Option<(String, TestStatus)> {
    for (token, status) in [
        ("Pass: ", TestStatus::Pass),
        ("Fail: ", TestStatus::Fail),
        ("Timeout: ", TestStatus::Fail),
        ("Fatal: ", TestStatus::Fail),
    ] {
        if let Some(idx) = line.find(token) {
            let rest = &line[idx + token.len()..];
            // The label runs up to the trailing ` (<duration>)`; if absent, take the whole rest.
            let label = rest
                .rsplit_once(" (")
                .map(|(l, _)| l)
                .unwrap_or(rest)
                .trim();
            if !label.is_empty() {
                return Some((label.to_owned(), status));
            }
        }
    }
    None
}

/// Parse buck2 `test`'s console stream into `{ test_target_label -> TestStatus }`, RECONCILED
/// against the `Tests finished:` summary. Fail-closed: returns `Err` when the summary is missing,
/// when the parsed per-target Pass/Fail counts do not equal the summary counts (an incomplete
/// verdict set could under-count head failures and false-green a regression), or when the summary
/// reports a build/infra failure (those are build-health's job or derivation uncertainty — a test
/// report is only trustworthy over a cleanly-built workspace).
pub fn parse_test_verdicts(console: &str) -> Result<BTreeMap<String, TestStatus>, String> {
    let clean = strip_ansi(console);
    let mut verdicts: BTreeMap<String, TestStatus> = BTreeMap::new();
    let mut summary: Option<TestSummary> = None;
    for line in clean.lines() {
        if let Some(s) = parse_test_summary(line) {
            summary = Some(s);
            continue;
        }
        if let Some((label, status)) = parse_target_verdict_line(line) {
            // A duplicate label with a WORSE status must not be masked (fail-closed): once a label
            // is Fail it stays Fail.
            verdicts
                .entry(label)
                .and_modify(|existing| {
                    if status == TestStatus::Fail {
                        *existing = TestStatus::Fail;
                    }
                })
                .or_insert(status);
        }
    }
    let summary = summary.ok_or(
        "buck2 test console has no `Tests finished:` summary — cannot reconcile the per-target \
         verdict set (fail-closed; refusing to grandfather against an unverifiable report)",
    )?;
    // Build failures are the BUILD-health ratchet's domain, not the test ratchet's: a build-failing
    // target emits no Pass/Fail verdict line (it is counted only in `Build failure`), so it is
    // simply absent from the runtime-verdict map. We therefore do NOT fail-closed on
    // `Build failure` here — the composition root runs build-health FIRST, which blocks build
    // regressions and grandfathers pre-existing build debt, so any remaining build failure is
    // already accounted for. We DO fail-closed on `Infra Failure` (genuine derivation uncertainty).
    if summary.infra_failure > 0 {
        return Err(format!(
            "buck2 test summary reports {} infra failure(s) — test derivation uncertainty (fail-closed)",
            summary.infra_failure
        ));
    }
    let parsed_pass = verdicts
        .values()
        .filter(|s| **s == TestStatus::Pass)
        .count();
    let parsed_fail = verdicts
        .values()
        .filter(|s| **s == TestStatus::Fail)
        .count();
    let expected_fail = summary.fail + summary.timeout + summary.fatal;
    if parsed_pass != summary.pass || parsed_fail != expected_fail {
        return Err(format!(
            "buck2 test verdict reconciliation mismatch: parsed pass={parsed_pass} \
             fail={parsed_fail}, summary pass={} fail+timeout+fatal={expected_fail} — refusing to \
             grandfather against an incomplete verdict set (fail-closed)",
            summary.pass
        ));
    }
    Ok(verdicts)
}

/// The set of test target labels that FAILED (the failure set the test-health ratchet diffs).
pub fn failing_test_targets(verdicts: &BTreeMap<String, TestStatus>) -> BTreeSet<String> {
    verdicts
        .iter()
        .filter(|(_, s)| **s == TestStatus::Fail)
        .map(|(label, _)| label.clone())
        .collect()
}

/// Serialize a parsed test-verdict map into the SAME `{"results": {label: {"success": ...}}}`
/// shape as a buck2 `--build-report`, so the merge-base TEST baseline artifact flows through the
/// exact `parse_build_report`/`failing_targets` machinery the build baseline already uses.
pub fn test_verdicts_to_report_value(verdicts: &BTreeMap<String, TestStatus>) -> Value {
    let results: serde_json::Map<String, Value> = verdicts
        .iter()
        .map(|(label, status)| {
            let success = match status {
                TestStatus::Pass => "SUCCESS",
                TestStatus::Fail => "FAIL",
            };
            (label.clone(), json!({ "success": success }))
        })
        .collect();
    json!({ "results": results })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_star_within_segment() {
        assert!(glob_match("*.bzl", "macros.bzl"));
        assert!(!glob_match("*.bzl", "dir/macros.bzl"));
        assert!(glob_match("**/*.bzl", "a/b/macros.bzl"));
        assert!(glob_match("**/*.bzl", "macros.bzl"));
    }

    #[test]
    fn glob_double_star_prefix_and_exact() {
        assert!(glob_match("toolchains/**", "toolchains/BUCK"));
        assert!(glob_match("toolchains/**", "toolchains/a/b.bzl"));
        assert!(!glob_match("toolchains/**", "toolchainsx/a"));
        assert!(glob_match(".buckconfig", ".buckconfig"));
        assert!(!glob_match(".buckconfig", "x/.buckconfig"));
    }

    #[test]
    fn glob_double_star_matches_zero_segments() {
        assert!(glob_match("third-party/**", "third-party/BUCK"));
        assert!(glob_match("a/**/b", "a/b"));
        assert!(glob_match("a/**/b", "a/x/y/b"));
    }

    #[test]
    fn package_pattern_root_cell() {
        let policy = test_policy();
        assert_eq!(
            package_pattern("cloud/x/Cargo.toml", &policy),
            Some("//cloud/x:".to_owned())
        );
        assert_eq!(
            package_pattern("Cargo.toml", &policy),
            Some("//:".to_owned())
        );
    }

    fn test_policy() -> Policy {
        Policy {
            gate_id: GATE_ID.to_owned(),
            universe: "//...".to_owned(),
            full_run_targets: vec!["//...".to_owned()],
            full_trigger_patterns: vec![
                ".buckconfig".to_owned(),
                "toolchains/**".to_owned(),
                "third-party/**".to_owned(),
                "**/*.bzl".to_owned(),
                "Cargo.lock".to_owned(),
            ],
            require_owner_patterns: vec!["**/*.rs".to_owned()],
            package_definition_basenames: vec!["BUCK.v2".to_owned(), "BUCK".to_owned()],
            package_sibling_basenames: vec!["Cargo.toml".to_owned(), "build.rs".to_owned()],
            cell_roots: BTreeMap::from([(String::new(), "//".to_owned())]),
            synthetic_dependencies: BTreeMap::new(),
            default_base_ref: "origin/main".to_owned(),
        }
    }

    #[test]
    fn policy_rejects_wrong_gate_id() {
        let err = Policy::from_json(r#"{"gate_id": "other"}"#).unwrap_err();
        assert!(err.0.contains("does not match engine"), "{err}");
    }

    fn minimal_policy_json(full_triggers: &str, require_owners: &str) -> String {
        format!(
            r#"{{
                "gate_id": "{GATE_ID}",
                "universe": "//...",
                "full_run_targets": ["//..."],
                "full_trigger_patterns": {full_triggers},
                "require_owner_patterns": {require_owners},
                "package_definition_basenames": ["BUCK.v2", "BUCK"],
                "package_sibling_basenames": ["Cargo.toml", "build.rs"],
                "cell_roots": {{"": "//"}},
                "default_base_ref": "origin/dev"
            }}"#
        )
    }

    #[test]
    fn policy_rejects_empty_full_trigger_patterns() {
        let json = minimal_policy_json("[]", r#"["**/*.rs"]"#);
        let err = Policy::from_json(&json).unwrap_err();
        assert!(
            err.0.contains("full_trigger_patterns") && err.0.contains("non-empty"),
            "{err}"
        );
    }

    #[test]
    fn policy_rejects_empty_require_owner_patterns() {
        let json = minimal_policy_json(r#"["third-party/**"]"#, "[]");
        let err = Policy::from_json(&json).unwrap_err();
        assert!(
            err.0.contains("require_owner_patterns") && err.0.contains("non-empty"),
            "{err}"
        );
    }

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn build_report_parses_success_and_fail_keyed_on_unconfigured_label() {
        let json = r#"{
            "results": {
                "root//a:a": {"success": "SUCCESS"},
                "root//b:b": {"success": "FAIL"},
                "root//c:c": {"success": "SKIPPED"}
            }
        }"#;
        let report = parse_build_report(json).unwrap();
        assert_eq!(report.get("root//a:a"), Some(&TargetBuildStatus::Success));
        // FAIL and SKIPPED both count as not-known-good (fail-closed).
        assert_eq!(report.get("root//b:b"), Some(&TargetBuildStatus::Fail));
        assert_eq!(report.get("root//c:c"), Some(&TargetBuildStatus::Fail));
        assert_eq!(failing_targets(&report), set(&["root//b:b", "root//c:c"]));
    }
    #[test]
    fn trusted_build_health_artifact_accepts_exact_non_empty_baseline() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let name = build_health_baseline_artifact_name(sha).unwrap();
        let json = r#"{"results":{"root//a:a":{"success":"SUCCESS"}}}"#;
        assert_eq!(
            validate_trusted_build_health_baseline_artifact(&name, sha, json),
            Ok(1)
        );
    }

    #[test]
    fn trusted_build_health_artifact_rejects_stale_name() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let stale = "build-health-baseline-89abcdef0123456789abcdef0123456789abcdef";
        let json = r#"{"results":{"root//a:a":{"success":"SUCCESS"}}}"#;
        let err = validate_trusted_build_health_baseline_artifact(stale, sha, json).unwrap_err();
        assert!(err.contains("does not match expected"), "{err}");
    }

    #[test]
    fn trusted_build_health_artifact_rejects_invalid_or_empty_report() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let name = build_health_baseline_artifact_name(sha).unwrap();

        let invalid =
            validate_trusted_build_health_baseline_artifact(&name, sha, "not json").unwrap_err();
        assert!(invalid.contains("not valid JSON"), "{invalid}");

        let empty =
            validate_trusted_build_health_baseline_artifact(&name, sha, r#"{"results":{}}"#)
                .unwrap_err();
        assert!(empty.contains("empty `results`"), "{empty}");
    }

    #[test]
    fn trusted_build_health_artifact_rejects_bad_sha_shape() {
        let err = build_health_baseline_artifact_name("dev").unwrap_err();
        assert!(err.contains("40-character hex"), "{err}");
    }

    #[test]
    fn trusted_push_run_selection_accepts_exact_successful_dev_push() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let runs = r#"{
            "workflow_runs": [
                {"id": 11, "head_sha": "fedcba9876543210fedcba9876543210fedcba98", "event": "push", "head_branch": "dev", "conclusion": "success"},
                {"id": 12, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "pull_request", "head_branch": "dev", "conclusion": "success"},
                {
                    "id": 13,
                    "head_sha": "0123456789abcdef0123456789abcdef01234567",
                    "event": "push",
                    "head_branch": "dev",
                    "status": "completed",
                    "conclusion": "success",
                    "path": ".github/workflows/oya-ci-required.yml",
                    "repository": {"id": 99},
                    "head_repository": {"id": 99}
                }
            ]
        }"#;
        assert_eq!(trusted_dev_push_run_id(runs, sha), Ok(Some(13)));
    }

    #[test]
    fn trusted_push_run_selection_falls_back_on_missing_or_untrusted() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let runs = r#"{
            "workflow_runs": [
                {"id": 12, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "push", "head_branch": "feature", "conclusion": "success"},
                {"id": 13, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "push", "head_branch": "dev", "conclusion": "failure"}
            ]
        }"#;
        assert_eq!(trusted_dev_push_run_id(runs, sha), Ok(None));
    }

    #[test]
    fn strict_trusted_run_rejects_wrong_workflow_and_repository() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let wrong_workflow = json!({
            "workflow_runs": [{
                "id": 13,
                "head_sha": sha,
                "event": "push",
                "head_branch": "dev",
                "status": "completed",
                "conclusion": "success",
                "path": ".github/workflows/untrusted.yml",
                "repository": {"id": 99},
                "head_repository": {"id": 99}
            }]
        });
        assert_eq!(
            trusted_dev_push_run(&wrong_workflow.to_string(), sha),
            Ok(None)
        );

        let fork_head = json!({
            "workflow_runs": [{
                "id": 13,
                "head_sha": sha,
                "event": "push",
                "head_branch": "dev",
                "status": "completed",
                "conclusion": "success",
                "path": ".github/workflows/oya-ci-required.yml",
                "repository": {"id": 99},
                "head_repository": {"id": 100}
            }]
        });
        let err = trusted_dev_push_run(&fork_head.to_string(), sha).unwrap_err();
        assert!(err.contains("does not match trusted repository"), "{err}");
    }

    #[test]
    fn trusted_baseline_artifact_selection_accepts_unexpired_exact_match() {
        let artifacts = r#"{
            "artifacts": [
                {"id": 21, "name": "build-health-baseline-fedcba9876543210fedcba9876543210fedcba98", "expired": false},
                {"id": 22, "name": "build-health-baseline-0123456789abcdef0123456789abcdef01234567", "expired": false}
            ]
        }"#;
        assert_eq!(
            trusted_build_health_baseline_artifact_id(
                artifacts,
                "build-health-baseline-0123456789abcdef0123456789abcdef01234567",
            ),
            Ok(Some(22))
        );
    }

    #[test]
    fn trusted_baseline_artifact_selection_falls_back_on_missing_or_stale() {
        let artifact_name = "build-health-baseline-0123456789abcdef0123456789abcdef01234567";
        assert_eq!(
            trusted_build_health_baseline_artifact_id(r#"{"artifacts":[]}"#, artifact_name),
            Ok(None)
        );
        assert_eq!(
            trusted_build_health_baseline_artifact_id(
                r#"{"artifacts":[{"id":22,"name":"build-health-baseline-0123456789abcdef0123456789abcdef01234567","expired":true}]}"#,
                artifact_name,
            ),
            Ok(None)
        );
    }

    fn trusted_pair_payloads() -> (String, String) {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let run_id = 30_144_110_793_u64;
        let repository_id = 1_236_575_706_u64;
        let runs = json!({
            "workflow_runs": [{
                "id": run_id,
                "head_sha": sha,
                "event": "push",
                "head_branch": "dev",
                "status": "completed",
                "conclusion": "success",
                "path": ".github/workflows/oya-ci-required.yml",
                "repository": {"id": repository_id},
                "head_repository": {"id": repository_id}
            }]
        });
        let artifact = |id: u64, name: String| {
            json!({
                "id": id,
                "name": name,
                "expired": false,
                "workflow_run": {
                    "id": run_id,
                    "head_sha": sha,
                    "head_branch": "dev",
                    "repository_id": repository_id,
                    "head_repository_id": repository_id
                }
            })
        };
        let artifacts = json!({
            "artifacts": [
                artifact(41, build_health_baseline_artifact_name(sha).unwrap()),
                artifact(42, test_health_baseline_artifact_name(sha).unwrap())
            ]
        });
        (runs.to_string(), artifacts.to_string())
    }

    #[test]
    fn trusted_baseline_pair_is_atomic_and_exact() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let (runs, artifacts) = trusted_pair_payloads();
        let selected = select_trusted_baseline_artifacts(&runs, &artifacts, sha)
            .unwrap()
            .expect("trusted pair");
        assert_eq!(selected.merge_base_sha, sha);
        assert_eq!(selected.run_id, 30_144_110_793);
        assert_eq!(selected.repository_id, 1_236_575_706);
        assert_eq!(selected.build_artifact.id, 41);
        assert_eq!(selected.test_artifact.id, 42);
    }

    #[test]
    fn trusted_reruns_are_order_independent_and_use_the_newest_complete_pair() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let (runs, artifacts) = trusted_pair_payloads();
        let mut payload: Value = serde_json::from_str(&runs).unwrap();
        let old_run = payload["workflow_runs"][0].clone();
        let mut newer_incomplete_run = old_run.clone();
        newer_incomplete_run["id"] = json!(30_144_110_794_u64);
        payload["workflow_runs"]
            .as_array_mut()
            .unwrap()
            .push(newer_incomplete_run);

        let canonical = trusted_dev_push_run(&payload.to_string(), sha)
            .unwrap()
            .expect("newest trusted run");
        assert_eq!(canonical.id, 30_144_110_794);

        let selected = select_trusted_baseline_artifacts(&payload.to_string(), &artifacts, sha)
            .unwrap()
            .expect("older complete pair");
        assert_eq!(selected.run_id, 30_144_110_793);

        payload["workflow_runs"].as_array_mut().unwrap().reverse();
        let reversed = select_trusted_baseline_artifacts(&payload.to_string(), &artifacts, sha)
            .unwrap()
            .expect("order-independent pair");
        assert_eq!(reversed, selected);
    }

    #[test]
    fn duplicate_trusted_run_id_is_rejected() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let (runs, _) = trusted_pair_payloads();
        let mut payload: Value = serde_json::from_str(&runs).unwrap();
        let duplicate = payload["workflow_runs"][0].clone();
        payload["workflow_runs"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        let err = trusted_dev_push_run(&payload.to_string(), sha).unwrap_err();
        assert!(err.contains("duplicate trusted workflow run id"), "{err}");
    }

    #[test]
    fn trusted_baseline_pair_rejects_duplicate_exact_name() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let (runs, artifacts) = trusted_pair_payloads();
        let mut payload: Value = serde_json::from_str(&artifacts).unwrap();
        let duplicate = payload["artifacts"][0].clone();
        payload["artifacts"].as_array_mut().unwrap().push(duplicate);
        let err = select_trusted_baseline_artifacts(&runs, &payload.to_string(), sha).unwrap_err();
        assert!(err.contains("duplicate exact-name"), "{err}");
    }

    #[test]
    fn trusted_test_health_artifact_requires_exact_non_empty_report() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let name = test_health_baseline_artifact_name(sha).unwrap();
        let report = r#"{"results":{"root//a:a":{"success":"SUCCESS"}}}"#;
        assert_eq!(
            validate_trusted_test_health_baseline_artifact(&name, sha, report),
            Ok(1)
        );
        let err = validate_trusted_test_health_baseline_artifact(&name, sha, r#"{"results":{}}"#)
            .unwrap_err();
        assert!(err.contains("empty `results`"), "{err}");
    }

    #[test]
    fn build_health_regression_blocks_grandfathered_does_not() {
        // baseline (merge-base) red: {blake3, sqlx}. head red: {blake3, sqlx, NEW}.
        // blake3+sqlx grandfathered; NEW is a regression -> BLOCK.
        let baseline = set(&["root//tp:blake3", "root//libs:sqlx"]);
        let head = set(&["root//tp:blake3", "root//libs:sqlx", "root//oya:new-break"]);
        let v = build_health_verdict(&baseline, &head);
        assert_eq!(v.regressions, vec!["root//oya:new-break".to_string()]);
        // BTreeSet intersection yields sorted order.
        assert_eq!(
            v.grandfathered,
            vec!["root//libs:sqlx".to_string(), "root//tp:blake3".to_string()]
        );
        assert!(!v.is_green(), "a regression must block");
    }

    #[test]
    fn build_health_only_pre_existing_red_is_green_via_grandfather() {
        // This is the #702 shape: the FULL run is red ONLY on the 4 pre-existing breakages, all
        // present at the merge-base -> all grandfathered -> GREEN (no flag-day requirement).
        let baseline = set(&[
            "root//third-party:blake3",
            "root//libs/oya-data-sql-adapter-sqlx:oya-data-sql-adapter-sqlx-unittest",
            "root//oya/ci-controller/crates/oya-ci-controller-app:oya-ci-controller",
            "root//libs/oya-shared-backbone-grpc-generated-adapter:oya-shared-backbone-grpc-generated-adapter-build-script-run",
        ]);
        let head = baseline.clone();
        let v = build_health_verdict(&baseline, &head);
        assert!(v.regressions.is_empty());
        assert_eq!(v.grandfathered.len(), 4);
        assert!(
            v.is_green(),
            "pre-existing-only red must be green via grandfather"
        );
    }

    #[test]
    fn build_health_a_target_that_built_at_merge_base_then_fails_is_a_regression() {
        // The core ratchet semantics + the laundering guard: a target NOT in the baseline
        // failure set that fails at head is a regression, FULL STOP. There is no candidate input
        // that can add it to `baseline` (the baseline comes from the merge-base build), so a PR
        // cannot reclassify its own regression as pre-existing (#698 F1 lesson).
        let baseline = set(&["root//tp:blake3"]); // only blake3 was red at merge-base
        let head = set(&["root//tp:blake3", "root//libs:was-green-now-red"]);
        let v = build_health_verdict(&baseline, &head);
        assert_eq!(
            v.regressions,
            vec!["root//libs:was-green-now-red".to_string()]
        );
        assert!(!v.is_green());
    }

    #[test]
    fn build_health_fixed_target_is_burned_down_not_a_failure() {
        // A baseline-red target that now BUILDS at head: burn-down (informational), green.
        let baseline = set(&["root//tp:blake3", "root//libs:sqlx"]);
        let head = set(&["root//tp:blake3"]); // sqlx fixed
        let v = build_health_verdict(&baseline, &head);
        assert!(v.regressions.is_empty());
        assert_eq!(v.fixed, vec!["root//libs:sqlx".to_string()]);
        assert!(v.is_green());
    }

    #[test]
    fn build_health_clean_workspace_is_green() {
        let v = build_health_verdict(&BTreeSet::new(), &BTreeSet::new());
        assert!(v.is_green());
        assert!(v.regressions.is_empty() && v.grandfathered.is_empty());
    }

    // ── D7 admission PRODUCER verdict (round-4): the admission FULL tier emits a build-report as
    //    a byproduct and derives the HARD verdict from the report's failure set being EMPTY — no
    //    grandfathering (the integration tip MUST be green). These pin the kernel semantic the
    //    admission producer (run_full_admission_producer in main.rs) reads off the parsed report;
    //    it is deliberately STRICTER than the PR ratchet (build_health_verdict), which
    //    grandfathers pre-existing reds.

    #[test]
    fn admission_producer_passes_only_on_empty_failure_set() {
        // A clean admission build-report -> empty failure set -> the producer's HARD verdict is
        // GREEN. (Build green is the precondition for running the full test suite.)
        let json = r#"{
            "results": {
                "root//a:a": {"success": "SUCCESS"},
                "root//b:b": {"success": "SUCCESS"}
            }
        }"#;
        let report = parse_build_report(json).unwrap();
        assert!(
            failing_targets(&report).is_empty(),
            "an all-SUCCESS admission report has an empty failure set -> producer PASS"
        );
    }

    #[test]
    fn admission_producer_hard_fails_on_any_failure_no_grandfathering() {
        // The integration-tip semantic: ANY build failure in the admission report is a hard fail —
        // there is NO baseline and NO grandfathering at admission (unlike the PR ratchet). A single
        // FAIL makes the failure set non-empty, so the producer blocks.
        let json = r#"{
            "results": {
                "root//a:a": {"success": "SUCCESS"},
                "root//pre-existing:red": {"success": "FAIL"}
            }
        }"#;
        let report = parse_build_report(json).unwrap();
        let failures = failing_targets(&report);
        assert_eq!(failures, set(&["root//pre-existing:red"]));
        assert!(
            !failures.is_empty(),
            "a non-empty admission failure set must hard-fail (no grandfathering at admission)"
        );
    }

    #[test]
    fn affected_set_operator_artifact_records_full_tier_and_phases() {
        let decision = Decision::Full {
            reasons: vec!["buildfile `BUCK` changed".to_owned()],
        };

        let phases = vec![
            GatePhaseOutcome::new("derive-affected-set-tier", "completed", "decision.tier"),
            GatePhaseOutcome::new(
                "rdeps-closure",
                "failed-escalated",
                "rdeps returned an empty closure for non-empty seeds",
            ),
            GatePhaseOutcome::new(
                "binding-build-test",
                "completed-check-exit-code",
                "FULL escalation executed after rdeps failure",
            ),
        ];
        let artifact = affected_set_operator_artifact(
            "auto",
            "0123456789abcdef0123456789abcdef01234567",
            "89abcdef0123456789abcdef0123456789abcdef",
            false,
            &decision,
            &phases,
        );

        assert_eq!(
            artifact["resolved_refs"]["base"],
            "0123456789abcdef0123456789abcdef01234567"
        );
        assert_eq!(
            artifact["resolved_refs"]["head"],
            "89abcdef0123456789abcdef0123456789abcdef"
        );
        assert_eq!(artifact["artifact_type"], "cloud_ci_operator_artifact");
        assert_eq!(artifact["artifact_id"], "affected-set-tier-decision");
        assert_eq!(artifact["decision"]["tier"], "FULL");
        assert_eq!(artifact["decision"]["will_run"], true);
        assert_eq!(
            artifact["merge_base_build_health_baseline"]["required"],
            true
        );
        assert_eq!(
            artifact["long_running_gate_phases"][1]["phase"],
            "rdeps-closure"
        );
        assert_eq!(
            artifact["long_running_gate_phases"][1]["status"],
            "failed-escalated"
        );
        let rendered = artifact.to_string();
        assert!(
            !rendered.contains("postgres://") && !rendered.contains("postgres:postgres"),
            "affected-set artifact must not leak DSNs or credentials: {rendered}"
        );
    }

    #[test]
    fn affected_set_operator_artifact_records_affected_seed_count() {
        let decision = Decision::Affected {
            seeds: vec![
                "root//tenancy/core/domain:tenancy-domain".to_owned(),
                "root//iam/facade/identity-service:iam-identity-service".to_owned(),
            ],
        };

        let phases = vec![
            GatePhaseOutcome::new("derive-affected-set-tier", "completed", "decision.tier"),
            GatePhaseOutcome::new("rdeps-closure", "completed", "2 affected targets"),
            GatePhaseOutcome::new(
                "binding-build-test",
                "pending-after-decision",
                "gate exit code and build-health verdict",
            ),
        ];
        let artifact = affected_set_operator_artifact(
            "auto",
            "0123456789abcdef0123456789abcdef01234567",
            "89abcdef0123456789abcdef0123456789abcdef",
            false,
            &decision,
            &phases,
        );

        assert_eq!(artifact["decision"]["tier"], "AFFECTED");
        assert_eq!(artifact["decision"]["seed_count"], 2);
        assert_eq!(
            artifact["long_running_gate_phases"][2]["status"],
            "pending-after-decision"
        );
    }

    #[test]
    fn long_step_telemetry_line_records_phase_and_elapsed_seconds() {
        let line = long_step_telemetry_line(
            "affected-set",
            "binding-build-test",
            "running",
            42,
            "command=buck2 test @targets",
        );

        assert!(line.contains("affected-set: phase=binding-build-test"));
        assert!(line.contains("status=running"));
        assert!(line.contains("elapsed_seconds=42"));
        assert!(line.contains("command=buck2 test @targets"));
    }

    // ── Defect 1: the diff parser maps rename/copy/type-change to Structural -> FULL ─────────

    #[test]
    fn parse_name_status_maps_structural_kinds_and_present_and_deleted() {
        // `-z` NUL-separated: A/M -> Present, D -> Deleted, T -> Structural(TypeChange),
        // R<score> -> Structural(Rename) on the DESTINATION, C<score> -> Structural(Copy).
        let raw = "M\0a.rs\0D\0b.rs\0T\0c.rs\0R100\0old.rs\0new.rs\0C075\0src.rs\0dst.rs\0";
        let changes = parse_name_status_z(raw).unwrap();
        assert_eq!(
            changes,
            vec![
                Change::Present("a.rs".into()),
                Change::Deleted("b.rs".into()),
                Change::Structural {
                    path: "c.rs".into(),
                    kind: StructuralKind::TypeChange
                },
                Change::Structural {
                    path: "new.rs".into(),
                    kind: StructuralKind::Rename
                },
                Change::Structural {
                    path: "dst.rs".into(),
                    kind: StructuralKind::Copy
                },
            ]
        );
    }

    #[test]
    fn rename_of_non_source_file_escalates_to_full() {
        // RED on the pre-round-6 parser: a rename was split into Deleted(old)+Present(new); a
        // rename of a NON-owner-required file with an owned destination resolved to AFFECTED, not
        // FULL — the structural move (old package loses the source) went unmodeled. Now FULL.
        let p = test_policy();
        let changes = parse_name_status_z("R100\0old/data.txt\0new/data.txt\0").unwrap();
        let plan = plan_changes(&changes, &p);
        assert!(matches!(
            resolve(&plan, &BTreeMap::new(), &p),
            Decision::Full { .. }
        ));
    }

    // ── Defect 3: runtime test failures are invisible to --build-report; the verdict parser
    //    catches them, reconciled fail-closed against the `Tests finished:` summary ────────────

    fn buck2_test_console(pass: &[&str], fail: &[&str]) -> String {
        let mut s = String::new();
        for label in pass {
            s.push_str(&format!("[ts] \u{2713} Pass: {label} (0.1s)\n"));
        }
        for label in fail {
            s.push_str(&format!("[ts] \u{2717} Fail: {label} (0.1s)\n"));
        }
        // Summary carries ANSI colour codes in real output; include them to exercise strip_ansi.
        s.push_str(&format!(
            "Tests finished: \u{1b}[38;5;10mPass {}\u{1b}[39m. \u{1b}[38;5;9mFail {}\u{1b}[39m. \
             Timeout 0. Fatal 0. Skip 0. Omit 0. Infra Failure 0. Build failure 0\n",
            pass.len(),
            fail.len()
        ));
        s
    }

    #[test]
    fn test_verdict_ratchet_catches_a_runtime_failure_the_build_report_calls_success() {
        // The exact hole: a target BUILDS (build-report `success: SUCCESS`) but its test FAILS at
        // runtime. build_health_verdict over the BUILD report would grandfather/miss it; the
        // TEST-verdict ratchet sees it as a regression.
        let build_report = r#"{"results":{"root//svc:svc-test":{"success":"SUCCESS"}}}"#;
        assert!(
            failing_targets(&parse_build_report(build_report).unwrap()).is_empty(),
            "build-report marks a build-OK-but-runtime-failed target SUCCESS (the hole)"
        );

        let baseline = buck2_test_console(&["root//svc:svc-test"], &[]);
        let head = buck2_test_console(&[], &["root//svc:svc-test"]);
        let baseline_fails = failing_test_targets(&parse_test_verdicts(&baseline).unwrap());
        let head_fails = failing_test_targets(&parse_test_verdicts(&head).unwrap());
        let verdict = build_health_verdict(&baseline_fails, &head_fails);
        assert_eq!(verdict.regressions, vec!["root//svc:svc-test".to_string()]);
        assert!(!verdict.is_green(), "a runtime test regression must block");
    }

    #[test]
    fn test_verdict_grandfathers_pre_existing_runtime_failure() {
        let baseline = buck2_test_console(&["root//a:a"], &["root//b:b"]);
        let head = buck2_test_console(&["root//a:a"], &["root//b:b"]);
        let baseline_fails = failing_test_targets(&parse_test_verdicts(&baseline).unwrap());
        let head_fails = failing_test_targets(&parse_test_verdicts(&head).unwrap());
        let verdict = build_health_verdict(&baseline_fails, &head_fails);
        assert!(verdict.is_green());
        assert_eq!(verdict.grandfathered, vec!["root//b:b".to_string()]);
    }

    #[test]
    fn test_verdict_reconciliation_fails_closed_on_undercount() {
        // If the parser sees FEWER Fail lines than the summary claims, the failure set is
        // incomplete — grandfathering against it could false-green a regression. Refuse.
        let console = "[ts] \u{2713} Pass: root//a:a (0.0s)\n\
             Tests finished: Pass 1. Fail 2. Timeout 0. Fatal 0. Skip 0. Omit 0. \
             Infra Failure 0. Build failure 0\n";
        let err = parse_test_verdicts(console).unwrap_err();
        assert!(err.contains("reconciliation mismatch"), "{err}");
    }

    #[test]
    fn test_verdict_refuses_a_report_missing_the_summary() {
        let console = "[ts] \u{2717} Fail: root//a:a (0.0s)\n";
        let err = parse_test_verdicts(console).unwrap_err();
        assert!(err.contains("no `Tests finished:` summary"), "{err}");
    }

    #[test]
    fn test_verdict_tolerates_grandfathered_build_failures_but_refuses_infra_failures() {
        // Build failures are the BUILD-health ratchet's domain (it runs first): a build-failing
        // target emits no Pass/Fail verdict line, so the runtime-verdict set still reconciles. An
        // INFRA failure is genuine derivation uncertainty and must fail-closed.
        let ok = "[ts] \u{2713} Pass: root//a:a (0.0s)\n\
             Tests finished: Pass 1. Fail 0. Timeout 0. Fatal 0. Skip 0. Omit 0. \
             Infra Failure 0. Build failure 3\n";
        assert!(
            parse_test_verdicts(ok).is_ok(),
            "build failures are build-health's concern, not the test parser's"
        );

        let infra = "Tests finished: Pass 0. Fail 0. Timeout 0. Fatal 0. Skip 0. Omit 0. \
             Infra Failure 2. Build failure 0\n";
        let err = parse_test_verdicts(infra).unwrap_err();
        assert!(err.contains("infra failure"), "{err}");
    }
}
