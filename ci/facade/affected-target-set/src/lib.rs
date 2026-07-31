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
//! - Verdict dominance (fail-closed): `RefuseUnowned` > `Full` > `Affected` >
//!   `RefuseEmptySelection` > `NoGraphTargets`.
//!   - `RefuseUnowned`: a file in the owner-required class has NO owning target — the file is
//!     invisible to the build graph, so even a full-workspace run would not compile it.
//!     Running anything would be a false-green; the lane FAILS with the path list.
//!   - `Full`: an escape-trigger path class matched, a graph-relevant file was deleted, or the
//!     adapter reported derivation uncertainty — the rdeps cone cannot be trusted, so the
//!     ENTIRE workspace runs. Escalation is the automation; nothing is ever skipped.
//!   - `Affected`: owners + the reverse-dependency closure (computed by the adapter).
//!   - `RefuseEmptySelection`: predicate (1) of the selection-totality assertion — the diff is
//!     NON-EMPTY yet the selection is EMPTY and the changed paths carry no inert-selection
//!     licence. Passing here would be green precisely BECAUSE nothing was built or tested.
//!   - `NoGraphTargets`: every changed file is unowned AND not in any owner-required class AND
//!     in a declared `inert_selection_classes` class (docs) — or the diff is empty.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

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
    /// Micro-glob path classes ALLOWED to constitute the ENTIRE selection — the exemption list
    /// for [`unjustified_empty_selection`] (predicate (1) of the selection-totality assertion).
    ///
    /// This is DELIBERATELY a second, single-purpose declaration rather than a re-read of the
    /// `[]` entries in `synthetic_dependencies`, because those two statements are different:
    /// `synthetic_dependencies[X] = []` says "X contributes no seed"; `inert_selection_classes`
    /// says "a diff consisting ONLY of X may build and test NOTHING and still pass". The reverted
    /// PR #1389 (`.github/**: []` -> a workflow-only PR resolved to `NoGraphTargets` and walked
    /// past the no-new-shell ratchet) is exactly the case where the first is a defensible
    /// optimization and the second is a merge-authority hole. Splitting them means a future
    /// `[]` declaration for a broad class fails RED naming the path, instead of passing green.
    ///
    /// Optional in the pack; ABSENT = empty = fail-closed (ANY non-empty diff that selects
    /// nothing is refused).
    pub inert_selection_classes: Vec<String>,
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
            // Absent -> empty -> fail-closed: nothing may be the entire selection.
            inert_selection_classes: if v.get("inert_selection_classes").is_some() {
                str_list_field(&v, "inert_selection_classes")?
            } else {
                Vec::new()
            },
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
    /// PREDICATE (1) OF THE SELECTION-TOTALITY ASSERTION: the diff is NON-EMPTY yet the selection
    /// is EMPTY, and `paths` are the changed files not covered by any declared inert selection
    /// class. A non-empty diff that selects nothing is a bug in target determination, never a
    /// pass: the lane would report success having built and tested NOTHING — green precisely
    /// BECAUSE it checked nothing.
    RefuseEmptySelection { paths: Vec<String> },
    /// Every change is unowned AND not in any owner-required class, AND every changed path is in
    /// a declared inert selection class (or the diff is empty). The only legitimate empty
    /// selection.
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
    // PREDICATE (1). Everything above either selected targets or escalated; reaching here means
    // the lane is about to PASS having built and tested nothing. That is only admissible when
    // every changed path is in a declared inert selection class.
    let unjustified = unjustified_empty_selection(plan, policy);
    if !unjustified.is_empty() {
        return Decision::RefuseEmptySelection { paths: unjustified };
    }
    Decision::NoGraphTargets
}

/// The changed paths that would ride an EMPTY selection to a PASS without an explicit licence
/// (PURE). Empty result = the empty selection is justified (either the diff is empty, or every
/// changed path is in a declared [`Policy::inert_selection_classes`] class).
///
/// WHY IT IS NOT ENOUGH THAT `resolve` ALREADY ESCALATES UNMAPPED PATHS TO FULL. It escalates
/// paths with no `synthetic_dependencies` match at all. It does NOT question a path whose
/// declaration matched and contributed an EMPTY seed union — and that is precisely the reverted
/// PR #1389 shape (`.github/**: []`), where the empty selection was a policy statement nobody had
/// to defend. This function makes "may be the entire selection" a separate, named, reviewable
/// licence, so the `[]`-inert false green cannot be reintroduced for a NEW class by one line.
///
/// ARTIFACT NOTE: `.github/workflows/oya-ci-required.yml` keeps the `affected-set-operator-artifacts`
/// upload on `if: always()` because, before this assertion existed, that artifact was the ONLY
/// witness of an empty affected set. This function is the gate that comment defers to; once it has
/// proven itself in production the upload can drop to `failure()` like its siblings — an empty
/// selection now fails the lane instead of passing it silently.
pub fn unjustified_empty_selection(plan: &Plan, policy: &Policy) -> Vec<String> {
    let mut paths: Vec<String> = plan
        .classified
        .iter()
        .map(|(path, _)| path)
        .filter(|path| {
            !policy
                .inert_selection_classes
                .iter()
                .any(|pattern| glob_match(pattern, path))
        })
        .cloned()
        .collect();
    paths.sort();
    paths.dedup();
    paths
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
    baseline_provenance: Option<&Value>,
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
        Decision::RefuseEmptySelection { paths } => json!({
            "tier": "REFUSE_EMPTY_SELECTION",
            "will_run": false,
            "paths": paths,
            "reasons": ["non-empty diff selected no targets and the paths carry no inert-selection licence"],
        }),
        Decision::NoGraphTargets => json!({
            "tier": "NO_GRAPH_TARGETS",
            "will_run": false,
            "reasons": ["every changed file is unowned, not owner-required, and in a declared inert selection class"],
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
            // WHICH baseline produced this verdict, and therefore what was grandfathered. A
            // reused artifact grandfathers nothing (its source tip passed admission green); a
            // cold rebuild may grandfather env-dependent merge-base failures. Recorded so the
            // difference is auditable per run instead of inferred from wall-clock or logs.
            "source": if baseline_provenance.is_some() { "trusted-artifact" } else { "cold-rebuild" },
            "provenance": baseline_provenance.cloned().unwrap_or(Value::Null),
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

/// The ONLY workflow whose push-to-dev runs may publish a trusted merge-base baseline (ADR-0515
/// single required context). Bound into run selection so the artifact NAME alone is never enough.
pub const REQUIRED_CONTEXT_WORKFLOW_PATH: &str = ".github/workflows/oya-ci-required.yml";

/// Sidecar the trusted-baseline consumer writes beside a reused baseline pair.
///
/// WHY IT EXISTS: the FULL tier's grandfathering set depends on WHICH baseline it got. A reused
/// baseline comes from a dev tip that passed admission, so its failure set is empty and nothing is
/// grandfathered; a cold rebuild can observe env-dependent merge-base failures and grandfather
/// them. The same PR at the same merge-base can therefore be green or red across two runs with no
/// code change, purely on whether the artifact still exists. The direction is safe (a reused
/// baseline is never laxer — see [`build_health_verdict`], a set difference in which a smaller
/// baseline can only ADD regressions), but it must be a recorded DECISION, not an inheritance.
/// Presence of this file means "trusted-artifact"; absence means "cold-rebuild".
pub const BASELINE_PROVENANCE_FILENAME: &str = "baseline-provenance.json";

/// Which merge-base health baseline an artifact carries (GH #1323/#899, ADR-0554 D8).
///
/// A FULL-tier PR needs BOTH: a target that BUILDS can still FAIL its tests, and buck2's
/// `--build-report` records BUILD status only — so the two baselines are distinct artifacts with
/// distinct names, produced by the same trusted push-to-dev run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaselineKind {
    /// `buck2 build //... --keep-going --build-report` results.
    Build,
    /// `buck2 test //...` per-target verdicts, normalized to the build-report shape.
    Test,
}

impl BaselineKind {
    /// The artifact-name discriminator: `build-health-baseline-<sha>` / `test-health-baseline-<sha>`.
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Test => "test",
        }
    }

    /// Parse the CLI spelling. Unknown values fail closed (no silent default kind).
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "build" => Ok(Self::Build),
            "test" => Ok(Self::Test),
            other => Err(format!("baseline kind must be `build` or `test`, got `{other}`")),
        }
    }
}

/// Reject anything that is not a full 40-char hex object id (fail-closed: an abbreviated or
/// symbolic ref could resolve differently across runs, so it must never name a baseline).
///
/// Public because the consumer must run this BEFORE interpolating a merge-base into any API
/// route, not merely before comparing it — the shape check is what keeps a caller-supplied value
/// from smuggling extra path or query segments.
pub fn validated_merge_base_sha(merge_base_sha: &str) -> Result<&str, String> {
    let sha = merge_base_sha.trim();
    if sha.len() != 40 || !sha.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return Err(format!(
            "merge-base SHA must be a 40-character hex object id, got `{merge_base_sha}`"
        ));
    }
    Ok(sha)
}

/// Expected trusted dev-push artifact name for a `kind` baseline produced at `merge_base_sha`.
///
/// The consumer workflow still validates GitHub Actions provenance (push-to-dev, successful
/// `oya-ci-required` run, exact `head_sha`) before download. This pure helper pins the artifact name
/// and SHA shape so stale/wrong artifacts cannot be confused with an exact merge-base baseline.
pub fn baseline_artifact_name(
    kind: BaselineKind,
    merge_base_sha: &str,
) -> Result<String, String> {
    let sha = validated_merge_base_sha(merge_base_sha)?;
    Ok(format!("{}-health-baseline-{sha}", kind.prefix()))
}

/// Select the trusted push-to-dev workflow run whose head SHA is the exact merge-base.
///
/// ANTI-LAUNDERING: every accepted property comes from GitHub Actions PROVENANCE, never from
/// anything a candidate PR controls — the run must be an `event=push` on `head_branch=dev` that
/// `conclusion=success`ed, its `head_sha` must be the EXACT merge-base, and its `path` must be the
/// canonical required-context workflow file.
///
/// The `path` bind is DEFENCE IN DEPTH, not the closing of a live hole: the consumer already
/// queries the per-workflow runs route and then reads artifacts per-run, so a foreign workflow's
/// artifacts were never reachable in the first place. It is asserted here anyway so the guarantee
/// survives a future caller that widens the route to the repo-wide `/actions/runs` list, where
/// selecting on name alone WOULD be reachable.
pub fn trusted_dev_push_run_id(
    runs_json: &str,
    merge_base_sha: &str,
    expected_workflow_path: &str,
) -> Result<Option<u64>, String> {
    let sha = validated_merge_base_sha(merge_base_sha)?;
    let payload: Value = serde_json::from_str(runs_json)
        .map_err(|e| format!("workflow-runs payload is not valid JSON: {e}"))?;
    let runs = payload
        .get("workflow_runs")
        .and_then(Value::as_array)
        .ok_or("workflow-runs payload has no `workflow_runs` array")?;

    for run in runs {
        if run.get("head_sha").and_then(Value::as_str) == Some(sha)
            && run.get("event").and_then(Value::as_str) == Some("push")
            && run.get("head_branch").and_then(Value::as_str) == Some("dev")
            && run.get("conclusion").and_then(Value::as_str) == Some("success")
            && run.get("path").and_then(Value::as_str) == Some(expected_workflow_path)
        {
            return run
                .get("id")
                .and_then(Value::as_u64)
                .map(Some)
                .ok_or("matching trusted workflow run has no numeric `id`".to_owned());
        }
    }

    Ok(None)
}

/// Select the unexpired exact-name health baseline artifact from a trusted run.
pub fn trusted_baseline_artifact_id(
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

/// Validate a trusted health baseline artifact payload after provenance selection.
///
/// `artifact_name` is the name the GitHub API reports for the artifact id that was actually
/// downloaded — checking it here closes the "selected one artifact, downloaded another" loop
/// against a live server response rather than against a locally-computed string.
///
/// Returns the number of report results. Empty/invalid reports are refused because an empty
/// baseline would launder every head failure into "brand-new but unproven" ambiguity.
pub fn validate_trusted_baseline_artifact(
    kind: BaselineKind,
    artifact_name: &str,
    merge_base_sha: &str,
    report_json: &str,
) -> Result<usize, String> {
    let expected = baseline_artifact_name(kind, merge_base_sha)?;
    if artifact_name != expected {
        return Err(format!(
            "{}-health baseline artifact name `{artifact_name}` does not match expected `{expected}`",
            kind.prefix()
        ));
    }
    let report = parse_build_report(report_json)?;
    if report.is_empty() {
        return Err(format!(
            "{}-health baseline artifact has an empty `results` object",
            kind.prefix()
        ));
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

/// Directory names never walked when deriving consumers: VCS + build scratch. Generic to any
/// buck2/cargo repo (R0: no repo-specific path lives in this kernel — the scanned class is a
/// PARAMETER, and the class root itself is skipped as a consequence of that parameter).
/// Directories the consumer scan never descends into.
///
/// `.claude` is tracked (it carries BUCK/OWNERS/settings.json) but also hosts the per-lane
/// isolated worktrees the operating contract mandates. Those are full nested repo copies, so
/// scanning them derives packages like `.claude/worktrees/agent-*/ci/facade/...` that no
/// declaration will ever list, and the gate REDs on tens of thousands of files that are not
/// this checkout. The direction is fail-closed, but a gate that REDs bogusly is how someone
/// gets talked into weakening the scan — which is precisely how the reverted #1389 happened.
const CONSUMER_SCAN_SKIP_DIRS: [&str; 5] =
    [".git", ".claude", "buck-out", "target", "node_modules"];

/// One derived consumer of a whole-tree-scanner path class: the buck2 package that can produce a
/// test verdict, and the file whose quote-anchored path literal put it there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathLiteralConsumer {
    /// Repo-root-relative package directory, e.g. `ci/facade/baseline-ratchet`.
    pub package: String,
    /// Repo-root-relative path of the first file in the package naming a `class_dir` path.
    pub evidence: String,
}

/// Derive, FROM THE TREE, the buck2 packages whose test verdict a change under `class_dir` can
/// flip — the accountability check for a `synthetic_dependencies` seed list.
///
/// A hand-maintained seed list rots silently: consumer N+1 lands unwired and the affected cone
/// narrows with nothing going red — the same class of hand-maintained safety property that
/// produced the reverted `[]`-inert declaration for `.github/**`.
///
/// `class_dir` is the repo-root-relative directory whose consumers are wanted (the caller's repo
/// fact, e.g. `.github`). The needle is a double quote immediately followed by `class_dir`, i.e.
/// a Rust/JSON string literal that STARTS at that path. Prose mentions (`` `.github/x.yml` `` in
/// a doc comment or a JSON `_comment`) are not quote-anchored and so do not match — the wanted
/// discrimination, and why no comment-stripping is needed for either language.
///
/// A package qualifies when BOTH hold:
/// 1. some `.rs`/`.json` file under it contains that needle; and
/// 2. its buildfile declares at least one `rust_test` — only a package that can produce a
///    verdict can produce a FALSE-GREEN one. Data-only packages (`specs/`, `registry/`) are full
///    of such strings and are excluded here mechanically, with no allowlist to maintain.
///
/// `class_dir` itself is never walked: its files are the SUBJECT of the declaration, never a
/// consumer of it.
///
/// Deliberately CONSERVATIVE: literal presence, not a proven filesystem read. A package that
/// merely embeds the string is over-seeded, which costs build time; missing one is a
/// merge-authority hole. Over-seeding is the safe direction, so no "mentions-only" exemption
/// list exists — there is no hand-maintained judgement anywhere in this derivation.
pub fn scan_path_literal_consumers(
    root: &Path,
    class_dir: &str,
) -> Result<Vec<PathLiteralConsumer>, String> {
    let needle = format!("\"{class_dir}");
    let mut hits: BTreeMap<String, String> = BTreeMap::new();
    collect_path_literal_files(root, root, class_dir, &needle, &mut hits)?;
    Ok(hits
        .into_iter()
        .filter_map(|(package, evidence)| {
            package_declares_rust_test(root, &package)
                .then_some(PathLiteralConsumer { package, evidence })
        })
        .collect())
}

/// Recursive half of [`scan_path_literal_consumers`]: records `package -> first evidence file`
/// for every `.rs`/`.json` containing `needle`. Both are repo-root-relative, `/`-separated.
fn collect_path_literal_files(
    root: &Path,
    dir: &Path,
    class_dir: &str,
    needle: &str,
    hits: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("dir entry under {}: {e}", dir.display()))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("file_type {}: {e}", path.display()))?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            let is_class_root = rel_slash_path(root, &path) == class_dir;
            if !is_class_root && !CONSUMER_SCAN_SKIP_DIRS.contains(&name.as_str()) {
                collect_path_literal_files(root, &path, class_dir, needle, hits)?;
            }
            continue;
        }
        if !(name.ends_with(".rs") || name.ends_with(".json")) {
            continue;
        }
        // Unreadable/non-UTF-8 sources cannot be proven inert, so they are skipped only because
        // they cannot carry a Rust/JSON string literal at all.
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if !text.contains(needle) {
            continue;
        }
        let Some(package) = enclosing_buck_package(root, &path) else {
            continue;
        };
        let rel = rel_slash_path(root, &path);
        hits.entry(package).or_insert(rel);
    }
    Ok(())
}

/// The nearest ancestor directory of `path` (at or below `root`) holding a buildfile, as a
/// repo-root-relative `/`-separated dir. Buildfile precedence mirrors
/// `package_definition_basenames`: `BUCK.v2` shadows `BUCK`.
fn enclosing_buck_package(root: &Path, path: &Path) -> Option<String> {
    let mut dir = path.parent()?;
    loop {
        if dir.join("BUCK.v2").is_file() || dir.join("BUCK").is_file() {
            return Some(rel_slash_path(root, dir));
        }
        if dir == root {
            return None;
        }
        dir = dir.parent()?;
    }
}

/// True iff the package's buildfile declares a `rust_test` — the packages that can go green.
fn package_declares_rust_test(root: &Path, package: &str) -> bool {
    let dir = root.join(package);
    for basename in ["BUCK.v2", "BUCK"] {
        if let Ok(text) = fs::read_to_string(dir.join(basename)) {
            return text.contains("rust_test(");
        }
    }
    false
}

/// Repo-root-relative, `/`-separated rendering of `path` (Windows lanes run these gates too).
fn rel_slash_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
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
            inert_selection_classes: Vec::new(),
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
    fn trusted_baseline_artifact_names_are_kind_scoped() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(
            baseline_artifact_name(BaselineKind::Build, sha).unwrap(),
            format!("build-health-baseline-{sha}")
        );
        assert_eq!(
            baseline_artifact_name(BaselineKind::Test, sha).unwrap(),
            format!("test-health-baseline-{sha}")
        );
        // The two kinds must never collide — a test baseline can never be served as a build one.
        assert_ne!(
            baseline_artifact_name(BaselineKind::Build, sha).unwrap(),
            baseline_artifact_name(BaselineKind::Test, sha).unwrap()
        );
        assert!(BaselineKind::parse("build").is_ok());
        assert!(BaselineKind::parse("test").is_ok());
        assert!(BaselineKind::parse("Build").is_err());
        assert!(BaselineKind::parse("").is_err());
    }

    #[test]
    fn trusted_build_health_artifact_accepts_exact_non_empty_baseline() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let name = baseline_artifact_name(BaselineKind::Build, sha).unwrap();
        let json = r#"{"results":{"root//a:a":{"success":"SUCCESS"}}}"#;
        assert_eq!(
            validate_trusted_baseline_artifact(BaselineKind::Build, &name, sha, json),
            Ok(1)
        );
    }

    #[test]
    fn trusted_test_health_artifact_accepts_exact_non_empty_baseline() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let name = baseline_artifact_name(BaselineKind::Test, sha).unwrap();
        // The test baseline is the normalizer's build-report-shaped output.
        let json = r#"{"results":{"root//a:a-unittest":{"success":"SUCCESS"},"root//b:b-unittest":{"success":"FAIL"}}}"#;
        assert_eq!(
            validate_trusted_baseline_artifact(BaselineKind::Test, &name, sha, json),
            Ok(2)
        );
        // A build-named artifact must NOT validate as the test baseline (kind confusion).
        let build_name = baseline_artifact_name(BaselineKind::Build, sha).unwrap();
        let err = validate_trusted_baseline_artifact(BaselineKind::Test, &build_name, sha, json)
            .unwrap_err();
        assert!(err.contains("does not match expected"), "{err}");
    }

    #[test]
    fn trusted_build_health_artifact_rejects_stale_name() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let stale = "build-health-baseline-89abcdef0123456789abcdef0123456789abcdef";
        let json = r#"{"results":{"root//a:a":{"success":"SUCCESS"}}}"#;
        let err = validate_trusted_baseline_artifact(BaselineKind::Build, stale, sha, json)
            .unwrap_err();
        assert!(err.contains("does not match expected"), "{err}");
    }

    #[test]
    fn trusted_build_health_artifact_rejects_invalid_or_empty_report() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let name = baseline_artifact_name(BaselineKind::Build, sha).unwrap();

        let invalid =
            validate_trusted_baseline_artifact(BaselineKind::Build, &name, sha, "not json")
                .unwrap_err();
        assert!(invalid.contains("not valid JSON"), "{invalid}");

        let empty = validate_trusted_baseline_artifact(
            BaselineKind::Build,
            &name,
            sha,
            r#"{"results":{}}"#,
        )
        .unwrap_err();
        assert!(empty.contains("empty `results`"), "{empty}");

        // A truncated/garbage download with no `results` object at all is refused too.
        let shapeless =
            validate_trusted_baseline_artifact(BaselineKind::Build, &name, sha, r#"{"ok":true}"#)
                .unwrap_err();
        assert!(shapeless.contains("no `results` object"), "{shapeless}");
    }

    #[test]
    fn trusted_build_health_artifact_rejects_bad_sha_shape() {
        let err = baseline_artifact_name(BaselineKind::Build, "dev").unwrap_err();
        assert!(err.contains("40-character hex"), "{err}");
        // An abbreviated SHA is rejected as well — it could resolve differently across runs.
        let abbrev = baseline_artifact_name(BaselineKind::Test, "0123456").unwrap_err();
        assert!(abbrev.contains("40-character hex"), "{abbrev}");
    }

    #[test]
    fn trusted_push_run_selection_accepts_exact_successful_dev_push() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let runs = r#"{
            "workflow_runs": [
                {"id": 11, "head_sha": "fedcba9876543210fedcba9876543210fedcba98", "event": "push", "head_branch": "dev", "conclusion": "success", "path": ".github/workflows/oya-ci-required.yml"},
                {"id": 12, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "pull_request", "head_branch": "dev", "conclusion": "success", "path": ".github/workflows/oya-ci-required.yml"},
                {"id": 13, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "push", "head_branch": "dev", "conclusion": "success", "path": ".github/workflows/oya-ci-required.yml"}
            ]
        }"#;
        assert_eq!(
            trusted_dev_push_run_id(runs, sha, REQUIRED_CONTEXT_WORKFLOW_PATH),
            Ok(Some(13))
        );
    }

    #[test]
    fn trusted_push_run_selection_falls_back_on_missing_or_untrusted() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let runs = r#"{
            "workflow_runs": [
                {"id": 12, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "push", "head_branch": "feature", "conclusion": "success", "path": ".github/workflows/oya-ci-required.yml"},
                {"id": 13, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "push", "head_branch": "dev", "conclusion": "failure", "path": ".github/workflows/oya-ci-required.yml"}
            ]
        }"#;
        assert_eq!(
            trusted_dev_push_run_id(runs, sha, REQUIRED_CONTEXT_WORKFLOW_PATH),
            Ok(None)
        );
    }

    #[test]
    fn trusted_push_run_selection_rejects_a_different_workflow_on_the_same_sha() {
        // DEFENCE IN DEPTH: the live consumer queries the per-workflow runs route, so a foreign
        // workflow's run is not reachable there today. This pins the bind so it still holds if a
        // caller ever widens the query to the repo-wide `/actions/runs` list.
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let runs = r#"{
            "workflow_runs": [
                {"id": 14, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "push", "head_branch": "dev", "conclusion": "success", "path": ".github/workflows/attacker-lane.yml"},
                {"id": 15, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "push", "head_branch": "dev", "conclusion": "success"}
            ]
        }"#;
        assert_eq!(
            trusted_dev_push_run_id(runs, sha, REQUIRED_CONTEXT_WORKFLOW_PATH),
            Ok(None)
        );
    }

    #[test]
    fn trusted_push_run_selection_refuses_malformed_input_fail_closed() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        assert!(
            trusted_dev_push_run_id("not json", sha, REQUIRED_CONTEXT_WORKFLOW_PATH).is_err()
        );
        assert!(
            trusted_dev_push_run_id(r#"{"ok":true}"#, sha, REQUIRED_CONTEXT_WORKFLOW_PATH)
                .is_err()
        );
        assert!(
            trusted_dev_push_run_id(r#"{"workflow_runs":[]}"#, "HEAD", REQUIRED_CONTEXT_WORKFLOW_PATH)
                .is_err()
        );
    }

    #[test]
    fn trusted_baseline_artifact_selection_accepts_unexpired_exact_match() {
        let artifacts = r#"{
            "artifacts": [
                {"id": 21, "name": "build-health-baseline-fedcba9876543210fedcba9876543210fedcba98", "expired": false},
                {"id": 22, "name": "build-health-baseline-0123456789abcdef0123456789abcdef01234567", "expired": false},
                {"id": 23, "name": "test-health-baseline-0123456789abcdef0123456789abcdef01234567", "expired": false}
            ]
        }"#;
        assert_eq!(
            trusted_baseline_artifact_id(
                artifacts,
                "build-health-baseline-0123456789abcdef0123456789abcdef01234567",
            ),
            Ok(Some(22))
        );
        assert_eq!(
            trusted_baseline_artifact_id(
                artifacts,
                "test-health-baseline-0123456789abcdef0123456789abcdef01234567",
            ),
            Ok(Some(23))
        );
    }

    #[test]
    fn trusted_baseline_artifact_selection_falls_back_on_missing_or_stale() {
        let artifact_name = "build-health-baseline-0123456789abcdef0123456789abcdef01234567";
        assert_eq!(
            trusted_baseline_artifact_id(r#"{"artifacts":[]}"#, artifact_name),
            Ok(None)
        );
        assert_eq!(
            trusted_baseline_artifact_id(
                r#"{"artifacts":[{"id":22,"name":"build-health-baseline-0123456789abcdef0123456789abcdef01234567","expired":true}]}"#,
                artifact_name,
            ),
            Ok(None)
        );
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
            None,
            &decision,
            &phases,
        );

        assert_eq!(
            artifact["resolved_refs"]["base"],
            "0123456789abcdef0123456789abcdef01234567"
        );
        // No sidecar => the baseline was rebuilt cold in this job.
        assert_eq!(
            artifact["merge_base_build_health_baseline"]["source"],
            "cold-rebuild"
        );
        assert_eq!(
            artifact["merge_base_build_health_baseline"]["provenance"],
            Value::Null
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
            None,
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
    fn operator_artifact_records_which_baseline_produced_the_verdict() {
        // The same PR at the same merge-base grandfathers differently depending on whether the
        // baseline was REUSED (source tip passed admission => empty failure set => nothing
        // grandfathered) or REBUILT COLD (may grandfather env-dependent merge-base failures).
        // The artifact must say which, so the verdict is an auditable decision.
        let decision = Decision::Full {
            reasons: vec!["escape trigger".to_owned()],
        };
        let provenance = json!({
            "schema_version": 1,
            "source": "trusted-artifact",
            "workflow_run_id": 4242,
        });
        let artifact = affected_set_operator_artifact(
            "auto",
            "0123456789abcdef0123456789abcdef01234567",
            "89abcdef0123456789abcdef0123456789abcdef",
            true,
            Some(&provenance),
            &decision,
            &[],
        );
        assert_eq!(
            artifact["merge_base_build_health_baseline"]["source"],
            "trusted-artifact"
        );
        assert_eq!(
            artifact["merge_base_build_health_baseline"]["provenance"]["workflow_run_id"],
            4242
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
