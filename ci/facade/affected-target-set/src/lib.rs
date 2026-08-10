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
//!
//! Also hosts [`hub_exclusivity`] — mechanical REFUSE when open integ PRs multi-own hubs at
//! `specs/integ-branch-envelopes.json#hubs.paths` (ADR-0711; colocated to avoid Cargo.lock hub
//! churn from a new package).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod hub_exclusivity;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// The lane id, matching the buck2 target + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-affected-set";

/// Build the exact Buck2 test argv and replayable display command used by every affected-set test
/// path. Buck2 test actions intentionally receive a small environment whitelist. The owned CI
/// runner keeps its pinned rustup installation outside `$HOME`, so tests that invoke Cargo after
/// changing into throwaway workspaces need that non-secret path forwarded through the executor.
/// `CARGO_HOME` remains isolated; hosted and developer environments without a non-empty
/// `RUSTUP_HOME` keep Buck2's default test environment unchanged.
pub fn buck2_test_invocation<T: AsRef<str>>(
    buck2: &str,
    targets: &[T],
    keep_going: bool,
) -> (Vec<String>, String) {
    buck2_test_invocation_from(buck2, targets, keep_going, |key| std::env::var(key).ok())
}

fn buck2_test_invocation_from<T: AsRef<str>>(
    buck2: &str,
    targets: &[T],
    keep_going: bool,
    mut lookup: impl FnMut(&str) -> Option<String>,
) -> (Vec<String>, String) {
    let mut args = Vec::with_capacity(targets.len() + 5);
    args.push("test".to_owned());
    args.extend(targets.iter().map(|target| target.as_ref().to_owned()));
    if keep_going {
        args.push("--keep-going".to_owned());
    }
    if let Some(rustup_home) = lookup("RUSTUP_HOME").filter(|value| !value.trim().is_empty()) {
        args.extend([
            "--".to_owned(),
            "--env".to_owned(),
            format!("RUSTUP_HOME={rustup_home}"),
        ]);
    }

    let display = std::iter::once(buck2)
        .chain(args.iter().map(String::as_str))
        .map(shell_quote)
        .collect::<Vec<_>>()
        .join(" ");
    (args, display)
}

fn shell_quote(value: &str) -> String {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_@%+=:,./-".contains(&byte))
    {
        return value.to_owned();
    }
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

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
    /// Micro-glob path pattern -> synthetic seed targets: the declared graph edges from a changed
    /// file to targets that read it from the LIVE TREE rather than through a buck2 `srcs` edge.
    /// Either it seeds specific targets (non-empty list) or it is EXPLICITLY declared inert
    /// (empty list `[]` = "this class affects no buck target"; e.g. docs assets).
    ///
    /// ADDITIVE, NOT A FALLBACK: a declaration UNIONS with `owner()` rather than applying only
    /// when `owner()` is empty. A buck2 owner proves a path is a graph input; it does NOT prove
    /// the owner's rdeps cone covers every target that reads the path. The adr-citation-closure
    /// gate is the standing counter-example — it equality-pins a whole-tree census while its own
    /// package is untouched by an ADR edit, and 458 of the tree's ADR records ARE owned. An empty
    /// list still contributes no seed, so declaring a class inert never shadows a real owner.
    ///
    /// A changed owner-query path with NO owner that matches NO synthetic pattern AND is not
    /// owner-required is DERIVATION UNCERTAINTY -> FULL (never silently ignored) — that rule is
    /// unchanged. This is the [`resolve`] "owner OR explicit synthetic dependency, otherwise
    /// FULL" rule. Optional in the pack (absent = empty map = every unowned non-owner-required
    /// path escalates to FULL).
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

/// The merge-base diff argv, minus the two revisions (PURE, so the rename-detection flags are a
/// fact a test can read rather than a string buried in the adapter).
///
/// WHY THE FLAGS ARE EXPLICIT. Rename detection used to be left to ambient git config. It happened
/// to work only because `diff.renames` defaults to true and neither `diff.renames` nor
/// `diff.renameLimit` is set in this repo — that is a property of the MACHINE, not of the tool. A
/// runner with `diff.renames=false` (or a rename set large enough to blow `diff.renameLimit`, where
/// git silently degrades to add+delete and only warns) turns a move into `A`+`D`. That is not a
/// cosmetic difference: [`resolve`] returns `RefuseUnowned` BEFORE it reads `full_reasons`, by
/// design ("a graph-invisibility refusal dominates"), so the `D` half never gets to escalate to
/// FULL. The destination arrives as an unowned `Present` and the verdict is a REFUSAL — every
/// capability-move PR would wedge with no in-band exit, on a config the PR author cannot see.
///
/// `--find-renames` pins detection on regardless of `diff.renames`. `-l0` removes the rename-limit
/// cap so a large capability move cannot silently degrade. Neither changes behavior under this
/// repo's current config — they make today's behavior INDEPENDENT of it. Copy detection is
/// deliberately NOT enabled: `-C` is off by default, and turning it on would reclassify ordinary
/// copies as `Structural` -> FULL, buying a ~55-60 min run for no correctness gain.
pub fn merge_base_diff_args<'a>(merge_base: &'a str, head: &'a str) -> Vec<&'a str> {
    vec![
        "diff",
        "--name-status",
        "-z",
        "--find-renames",
        "-l0",
        merge_base,
        head,
    ]
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
    // Package-sibling manifests (`Cargo.toml`, `build.rs`) never enter `owner_paths` —
    // `plan_changes` records only the enclosing package pattern and continues. Synthetic
    // declarations for those paths (e.g. `os/**/Cargo.toml` → k8s-program-docs census) must
    // still union here; otherwise a manifest-only leaf edit selects the package but never the
    // gate that reads the leaf set.
    for (path, class) in &plan.classified {
        if matches!(class, PathClass::PackagePattern(_))
            && let Some(synth) = synthetic_seeds(path, policy)
        {
            seeds.extend(synth);
        }
    }
    let mut unmapped: Vec<String> = Vec::new();
    for path in &plan.owner_paths {
        let owners = owner_results.get(path).map(Vec::as_slice).unwrap_or(&[]);
        if !owners.is_empty() {
            seeds.extend(owners.iter().cloned());
            // A buck2 owner does not EXHAUST a path's blast radius. A gate may read a file from
            // the LIVE TREE with no graph edge to it: adr-citation-closure walks the whole tree
            // and equality-pins a census over it, while its own package
            // (`srcs = glob(["tests/**/*.rs", "**/*.json"])`) is untouched by an ADR edit, so the
            // rdeps cone of the ADR's own owners never reaches it. Measured on this tree: 463
            // tracked `.md` files ARE buck2-owned (448 docs/adr-archive + 10 docs/decisions), and
            // those 458 ADR records are exactly the files whose edits move the pinned census.
            // Synthetic declarations are therefore ADDITIVE, never a no-owner FALLBACK.
            //
            // FAIL-SAFE DIRECTION: this can only ADD seeds, so the worst case is over-selection
            // (slower), never the under-selection that produces a false green. An inert `[]`
            // declaration unions to exactly the owners, so "declaring docs inert never shadows a
            // real owner" — the property the fallback was protecting — still holds.
            if let Some(synth) = synthetic_seeds(path, policy) {
                seeds.extend(synth);
            }
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
    baseline_reuse_outcome: Option<&Value>,
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
            // WHY the fast path did or did not run. `source` alone cannot distinguish "no baseline
            // was published for this merge-base" from "the runner could not ask" — and for the
            // whole life of the owned arm64 fleet it was always the latter, invisibly. Null here
            // means the consumer never ran at all (a non-FULL tier), which is not a degrade.
            "reuse_outcome": baseline_reuse_outcome.cloned().unwrap_or(Value::Null),
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

/// The only job allowed to publish the build/test pair consumed by the trusted-baseline fast
/// path. A workflow run is an aggregate: an unrelated hosted lane may fail after this owned,
/// isolated producer completed successfully. Binding reuse to this exact job preserves the
/// producer verdict without laundering the aggregate run's other failures.
pub const AFFECTED_SET_PRODUCER_JOB_NAME: &str =
    "gate · affected-set (ADR-0554, binding workspace coverage)";

/// Provenance of the unique canonical workflow run selected for one merge base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedWorkflowRun {
    pub id: u64,
    pub attempt: u64,
    pub head_sha: String,
}

/// Provenance of the unique trusted baseline producer job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedProducerJob {
    pub id: u64,
    pub run_id: u64,
    pub head_sha: String,
    pub conclusion: String,
}

/// Immutable artifact identity returned by the GitHub Actions API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedBaselineArtifact {
    pub id: u64,
    pub name: String,
    pub digest: String,
    pub size_in_bytes: u64,
    pub workflow_run_id: u64,
    pub head_sha: String,
}

/// Sidecar the trusted-baseline consumer writes beside a reused baseline pair.
///
/// Presence means "trusted-artifact"; absence means "cold-rebuild". The sidecar records the
/// jointly bound workflow run, attempt, exact producer job, immutable artifact ids, and digests.
pub const BASELINE_PROVENANCE_FILENAME: &str = "baseline-provenance.json";

/// Sidecar recording WHY the trusted-baseline fast path did or did not run — written on EVERY
/// outcome. Failure to persist this file refuses reuse and runs the cold fallback.
pub const BASELINE_REUSE_OUTCOME_FILENAME: &str = "baseline-reuse-outcome.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaselineReuseState {
    Reused,
    Unavailable,
    Refused,
    CapabilityFault,
}

impl BaselineReuseState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reused => "reused",
            Self::Unavailable => "unavailable",
            Self::Refused => "refused",
            Self::CapabilityFault => "capability-fault",
        }
    }

    pub const fn is_capability_fault(self) -> bool {
        matches!(self, Self::CapabilityFault)
    }

    pub const fn exit_code(self) -> u8 {
        match self {
            Self::Reused => 0,
            Self::Refused => 2,
            Self::Unavailable => 3,
            Self::CapabilityFault => 4,
        }
    }
}

pub const fn classify_api_status(status: u16) -> Option<BaselineReuseState> {
    match status {
        200..=299 => None,
        404 | 410 => Some(BaselineReuseState::Unavailable),
        401 | 403 | 429 | 500..=599 => Some(BaselineReuseState::CapabilityFault),
        _ => Some(BaselineReuseState::Refused),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaselineKind {
    Build,
    Test,
}

impl BaselineKind {
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Test => "test",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "build" => Ok(Self::Build),
            "test" => Ok(Self::Test),
            other => Err(format!(
                "baseline kind must be `build` or `test`, got `{other}`"
            )),
        }
    }
}

pub fn validated_merge_base_sha(merge_base_sha: &str) -> Result<&str, String> {
    let sha = merge_base_sha.trim();
    if sha.len() != 40 || !sha.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return Err(format!(
            "merge-base SHA must be a 40-character hex object id, got `{merge_base_sha}`"
        ));
    }
    Ok(sha)
}

/// Exact immutable name. The attempt and canonical producer key prevent artifacts from an older
/// rerun attempt or another job from entering the selected pair.
pub fn baseline_artifact_name(
    kind: BaselineKind,
    merge_base_sha: &str,
    run_id: u64,
    run_attempt: u64,
) -> Result<String, String> {
    let sha = validated_merge_base_sha(merge_base_sha)?;
    if run_id == 0 || run_attempt == 0 {
        return Err("artifact run id and attempt must both be positive".to_owned());
    }
    Ok(format!(
        "{}-health-baseline-{sha}-{run_id}-{run_attempt}-gate-affected-target-set",
        kind.prefix()
    ))
}

/// Select exactly one completed canonical push-to-dev run at the merge base. Duplicate matching
/// records and incomplete numeric bindings are malformed provenance and therefore errors.
pub fn trusted_dev_push_run(
    runs_json: &str,
    merge_base_sha: &str,
    expected_workflow_path: &str,
) -> Result<Option<TrustedWorkflowRun>, String> {
    let sha = validated_merge_base_sha(merge_base_sha)?;
    let payload: Value = serde_json::from_str(runs_json)
        .map_err(|e| format!("workflow-runs payload is not valid JSON: {e}"))?;
    let runs = payload
        .get("workflow_runs")
        .and_then(Value::as_array)
        .ok_or("workflow-runs payload has no `workflow_runs` array")?;
    let matching = runs
        .iter()
        .filter(|run| {
            run.get("head_sha").and_then(Value::as_str) == Some(sha)
                && run.get("event").and_then(Value::as_str) == Some("push")
                && run.get("head_branch").and_then(Value::as_str) == Some("dev")
                && run.get("status").and_then(Value::as_str) == Some("completed")
                && run.get("path").and_then(Value::as_str) == Some(expected_workflow_path)
        })
        .collect::<Vec<_>>();
    let [run] = matching.as_slice() else {
        return if matching.is_empty() {
            Ok(None)
        } else {
            Err(format!(
                "found {} matching trusted workflow runs; expected exactly one",
                matching.len()
            ))
        };
    };
    let id = run
        .get("id")
        .and_then(Value::as_u64)
        .ok_or("matching trusted workflow run has no numeric `id`")?;
    let attempt = run
        .get("run_attempt")
        .and_then(Value::as_u64)
        .filter(|attempt| *attempt > 0)
        .ok_or("matching trusted workflow run has no positive numeric `run_attempt`")?;
    Ok(Some(TrustedWorkflowRun {
        id,
        attempt,
        head_sha: sha.to_owned(),
    }))
}

pub fn trusted_dev_push_run_id(
    runs_json: &str,
    merge_base_sha: &str,
    expected_workflow_path: &str,
) -> Result<Option<u64>, String> {
    trusted_dev_push_run(runs_json, merge_base_sha, expected_workflow_path)
        .map(|run| run.map(|run| run.id))
}

/// Select the unique exact producer returned by the selected run-attempt endpoint.
pub fn trusted_affected_set_producer_job(
    jobs_json: &str,
    expected_run_id: u64,
    expected_head_sha: &str,
) -> Result<Option<TrustedProducerJob>, String> {
    let expected_head_sha = validated_merge_base_sha(expected_head_sha)?;
    let payload: Value = serde_json::from_str(jobs_json)
        .map_err(|e| format!("workflow-jobs payload is not valid JSON: {e}"))?;
    let jobs = payload
        .get("jobs")
        .and_then(Value::as_array)
        .ok_or("workflow-jobs payload has no `jobs` array")?;
    let matching = jobs
        .iter()
        .filter(|job| {
            job.get("name").and_then(Value::as_str) == Some(AFFECTED_SET_PRODUCER_JOB_NAME)
        })
        .collect::<Vec<_>>();
    let [job] = matching.as_slice() else {
        return if matching.is_empty() {
            Ok(None)
        } else {
            Err(format!(
                "found {} exact affected-set producer jobs; expected exactly one",
                matching.len()
            ))
        };
    };
    if job.get("status").and_then(Value::as_str) != Some("completed")
        || job.get("conclusion").and_then(Value::as_str) != Some("success")
    {
        return Ok(None);
    }
    let id = job
        .get("id")
        .and_then(Value::as_u64)
        .ok_or("trusted producer job has no numeric `id`")?;
    let run_id = job
        .get("run_id")
        .and_then(Value::as_u64)
        .ok_or("trusted producer job has no numeric `run_id`")?;
    let head_sha = job
        .get("head_sha")
        .and_then(Value::as_str)
        .ok_or("trusted producer job has no string `head_sha`")?;
    if run_id != expected_run_id || head_sha != expected_head_sha {
        return Err(format!(
            "trusted producer binding mismatch: run_id={run_id}, head_sha={head_sha}"
        ));
    }
    Ok(Some(TrustedProducerJob {
        id,
        run_id,
        head_sha: head_sha.to_owned(),
        conclusion: "success".to_owned(),
    }))
}

/// Select one immutable, unexpired, exact-name artifact bound to the selected run and head.
pub fn trusted_baseline_artifact(
    artifacts_json: &str,
    artifact_name: &str,
    expected_run_id: u64,
    expected_head_sha: &str,
) -> Result<Option<TrustedBaselineArtifact>, String> {
    let expected_head_sha = validated_merge_base_sha(expected_head_sha)?;
    let payload: Value = serde_json::from_str(artifacts_json)
        .map_err(|e| format!("workflow-artifacts payload is not valid JSON: {e}"))?;
    let artifacts = payload
        .get("artifacts")
        .and_then(Value::as_array)
        .ok_or("workflow-artifacts payload has no `artifacts` array")?;
    let matching = artifacts
        .iter()
        .filter(|artifact| artifact.get("name").and_then(Value::as_str) == Some(artifact_name))
        .collect::<Vec<_>>();
    let [artifact] = matching.as_slice() else {
        return if matching.is_empty() {
            Ok(None)
        } else {
            Err(format!(
                "found {} artifacts named `{artifact_name}`; expected exactly one",
                matching.len()
            ))
        };
    };
    if artifact
        .get("expired")
        .and_then(Value::as_bool)
        .unwrap_or(true)
    {
        return Ok(None);
    }
    let id = artifact
        .get("id")
        .and_then(Value::as_u64)
        .ok_or("matching trusted artifact has no numeric `id`")?;
    let digest = artifact
        .get("digest")
        .and_then(Value::as_str)
        .filter(|digest| {
            digest.strip_prefix("sha256:").is_some_and(|hex| {
                hex.len() == 64 && hex.as_bytes().iter().all(u8::is_ascii_hexdigit)
            })
        })
        .ok_or("matching trusted artifact has no valid `sha256:<64-hex>` digest")?;
    let size_in_bytes = artifact
        .get("size_in_bytes")
        .and_then(Value::as_u64)
        .filter(|size| *size > 0)
        .ok_or("matching trusted artifact has no positive numeric `size_in_bytes`")?;
    let workflow_run = artifact
        .get("workflow_run")
        .and_then(Value::as_object)
        .ok_or("matching trusted artifact has no `workflow_run` provenance")?;
    let workflow_run_id = workflow_run
        .get("id")
        .and_then(Value::as_u64)
        .ok_or("matching trusted artifact workflow_run has no numeric `id`")?;
    let head_sha = workflow_run
        .get("head_sha")
        .and_then(Value::as_str)
        .ok_or("matching trusted artifact workflow_run has no string `head_sha`")?;
    if workflow_run_id != expected_run_id || head_sha != expected_head_sha {
        return Err(format!(
            "artifact `{artifact_name}` binding mismatch: workflow_run_id={workflow_run_id}, head_sha={head_sha}"
        ));
    }
    Ok(Some(TrustedBaselineArtifact {
        id,
        name: artifact_name.to_owned(),
        digest: digest.to_owned(),
        size_in_bytes,
        workflow_run_id,
        head_sha: head_sha.to_owned(),
    }))
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
    run_id: u64,
    run_attempt: u64,
    report_json: &str,
) -> Result<usize, String> {
    let expected = baseline_artifact_name(kind, merge_base_sha, run_id, run_attempt)?;
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

/// Maximum trusted failed-run job log accepted by the dormant partial-negative parser.
///
/// The bootstrap fixture is 576,565 bytes. Stage B must enforce this bound while fetching too.
pub const PARTIAL_NEGATIVE_LOG_MAX_BYTES: usize = 2 * 1024 * 1024;

/// Maximum number of literal `BUILD-FAIL` labels accepted from one canonical producer block.
pub const PARTIAL_NEGATIVE_MAX_FAILURES: usize = 16_384;
pub const PARTIAL_NEGATIVE_SCHEMA_VERSION: u64 = 2;
pub const PARTIAL_NEGATIVE_SOURCE: &str = "trusted-negative-receipt";
pub const PARTIAL_NEGATIVE_COMPLETENESS: &str = "observed-failure-lower-bound";
pub const PARTIAL_NEGATIVE_TEST_POLICY: &str = "hard-no-grandfathering";

const PARTIAL_NEGATIVE_BLOCK_PREFIX: &str = "affected-set: RED — admission FULL build failed on ";
const PARTIAL_NEGATIVE_FAILURE_PREFIX: &str = "affected-set:   BUILD-FAIL ";
const PARTIAL_NEGATIVE_BLOCK_TERMINATOR: &str = "affected-set: REPRODUCE:";

/// Identity of the trusted failed required-context job whose log carries a receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialNegativeJobBinding {
    pub workflow_path: String,
    pub run_id: u64,
    pub run_attempt: u64,
    pub job_id: u64,
    pub job_name: String,
    pub step_number: u64,
    pub step_name: String,
}

/// Typed action-local terminal. The bootstrap fixture reports no action exit code; this is distinct
/// from both success and the enclosing workflow step's exit 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialNegativeActionTerminal {
    Fail,
    NoExitCode,
}

impl PartialNegativeActionTerminal {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fail => "FAIL",
            Self::NoExitCode => "<no exit code>",
        }
    }
}

/// Exact failed build action observed inside the bound job step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialNegativeBuildAction {
    pub label: String,
    /// Opaque configured-platform identity from Buck2 output; it is not claimed to be a digest.
    pub configured_platform_token: String,
    pub rule: String,
    /// Action-local terminal, kept distinct from the enclosing workflow step exit code.
    pub action_terminal: PartialNegativeActionTerminal,
}

/// Complete dormant Stage A receipt contract. Stage B must populate and validate every binding
/// from trusted GitHub responses and a validator built from the immutable base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialNegativeReceipt {
    pub schema_version: u64,
    pub source: String,
    pub completeness: String,
    pub merge_base: String,
    pub job: PartialNegativeJobBinding,
    pub build_action: PartialNegativeBuildAction,
    pub observed_failures: BTreeSet<String>,
    pub test_policy: String,
}

impl PartialNegativeReceipt {
    /// Validate the architecture contract after trusted transport/provenance fields are populated.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != PARTIAL_NEGATIVE_SCHEMA_VERSION
            || self.source != PARTIAL_NEGATIVE_SOURCE
            || self.completeness != PARTIAL_NEGATIVE_COMPLETENESS
            || self.test_policy != PARTIAL_NEGATIVE_TEST_POLICY
        {
            return Err("partial-negative receipt contract token mismatch".to_owned());
        }
        validated_merge_base_sha(&self.merge_base)?;
        if self.job.workflow_path != REQUIRED_CONTEXT_WORKFLOW_PATH
            || self.job.run_id == 0
            || self.job.run_attempt == 0
            || self.job.job_id == 0
            || self.job.step_number == 0
            || self.job.job_name.trim().is_empty()
            || self.job.step_name.trim().is_empty()
        {
            return Err("partial-negative receipt has invalid job/step binding".to_owned());
        }
        if self.build_action.label.trim().is_empty()
            || self
                .build_action
                .configured_platform_token
                .trim()
                .is_empty()
            || self.build_action.rule.trim().is_empty()
        {
            return Err("partial-negative receipt has incomplete build-action binding".to_owned());
        }
        if self.observed_failures.is_empty()
            || self.observed_failures.len() > PARTIAL_NEGATIVE_MAX_FAILURES
        {
            return Err(
                "partial-negative receipt needs a bounded non-empty failure set".to_owned(),
            );
        }
        if !self.observed_failures.contains(&self.build_action.label) {
            return Err("bound failed action is absent from observed failures".to_owned());
        }
        Ok(())
    }
}

/// A validated lower-bound failure set from one trusted failed required-context run.
///
/// This is BUILD-only. It never carries a test baseline: candidate test failures remain hard
/// failures. The type is dormant in Stage A; no workflow or binary calls it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialNegativeFailures {
    pub observed_failures: BTreeSet<String>,
}

/// Parse exactly one canonical FULL-build failure block from a bounded trusted job log.
///
/// Only literal `BUILD-FAIL` records are admitted. `SKIPPED`, `CANCELLED`, summaries, and enclosing
/// workflow exit codes cannot enter the lower-bound set. Transport prefixes before the first
/// `affected-set:` token are ignored; text after an exact label is rejected rather than normalized.
pub fn parse_partial_negative_failures(job_log: &str) -> Result<PartialNegativeFailures, String> {
    if job_log.len() > PARTIAL_NEGATIVE_LOG_MAX_BYTES {
        return Err(format!(
            "trusted failed-run log is {} bytes; maximum is {PARTIAL_NEGATIVE_LOG_MAX_BYTES}",
            job_log.len()
        ));
    }

    let lines: Vec<&str> = job_log.lines().collect();
    let starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            line.find("affected-set:")
                .map(|start| &line[start..])
                .filter(|payload| payload.starts_with(PARTIAL_NEGATIVE_BLOCK_PREFIX))
                .map(|_| index)
        })
        .collect();
    if starts.len() != 1 {
        return Err(format!(
            "trusted failed-run log must contain exactly one canonical FULL-build failure block; found {}",
            starts.len()
        ));
    }

    let start = starts[0];
    let header = lines[start]
        .find("affected-set:")
        .map(|payload_start| &lines[start][payload_start..])
        .ok_or("canonical FULL-build failure header disappeared")?;
    let count_suffix = header
        .strip_prefix(PARTIAL_NEGATIVE_BLOCK_PREFIX)
        .and_then(|suffix| {
            suffix.strip_suffix(" target(s) (integration tip must be green, no grandfathering):")
        })
        .ok_or("canonical FULL-build failure header is malformed")?;
    let declared: usize = count_suffix
        .parse()
        .map_err(|_| "canonical FULL-build failure count is not an unsigned integer".to_owned())?;
    if declared == 0 || declared > PARTIAL_NEGATIVE_MAX_FAILURES {
        return Err(format!(
            "canonical FULL-build failure count must be in 1..={PARTIAL_NEGATIVE_MAX_FAILURES}, got {declared}"
        ));
    }

    let mut failures = BTreeSet::new();
    let mut terminated = false;
    for raw_line in lines.iter().skip(start + 1) {
        let marker_start = raw_line
            .find("affected-set:")
            .ok_or("non-producer line interrupts canonical failure block")?;
        let line = &raw_line[marker_start..];
        if line.starts_with(PARTIAL_NEGATIVE_BLOCK_TERMINATOR) {
            terminated = true;
            break;
        }
        let label = line
            .strip_prefix(PARTIAL_NEGATIVE_FAILURE_PREFIX)
            .ok_or_else(|| format!("unexpected line inside canonical failure block: `{line}`"))?;
        if label.is_empty()
            || label.trim() != label
            || label.split_whitespace().count() != 1
            || !label.contains("//")
            || !label.contains(':')
        {
            return Err(format!("invalid exact Buck2 failure label `{label}`"));
        }
        if !failures.insert(label.to_owned()) {
            return Err(format!("duplicate BUILD-FAIL label `{label}`"));
        }
        if failures.len() > PARTIAL_NEGATIVE_MAX_FAILURES {
            return Err("canonical failure block exceeds entry ceiling".to_owned());
        }
    }
    if !terminated {
        return Err("canonical FULL-build failure block has no REPRODUCE terminator".to_owned());
    }
    if failures.len() != declared {
        return Err(format!(
            "canonical failure block declared {declared} target(s) but contained {} literal BUILD-FAIL record(s)",
            failures.len()
        ));
    }

    Ok(PartialNegativeFailures {
        observed_failures: failures,
    })
}

/// State of the preferred trusted successful-run artifact pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositiveBaselineState {
    Absent,
    Valid,
    /// An exact successful run exists, but its artifact pair is unusable for any reason.
    Invalid,
}

/// State of the optional trusted failed-run BUILD receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NegativeBaselineState {
    Absent,
    Valid(PartialNegativeReceipt),
    Invalid,
}

/// Dormant selection result for Stage B's future trusted-baseline consumer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartialNegativeSelection {
    Positive,
    Negative(PartialNegativeFailures),
    Cold,
}

/// Apply the positive-first anti-downgrade decision table without I/O.
///
/// A known successful run dominates. Missing, partial, malformed, expired, unavailable, or
/// mismatched artifacts from it force a cold rebuild; they may never downgrade to an older failed
/// run. Negative selection requires genuine positive absence, a valid non-empty receipt, and this
/// validator in the exact immutable base.
pub fn select_partial_negative_baseline(
    positive: PositiveBaselineState,
    negative: NegativeBaselineState,
    validator_base_sha: Option<&str>,
) -> PartialNegativeSelection {
    match (positive, negative, validator_base_sha) {
        (PositiveBaselineState::Valid, _, _) => PartialNegativeSelection::Positive,
        (
            PositiveBaselineState::Absent,
            NegativeBaselineState::Valid(receipt),
            Some(validator_base_sha),
        ) if receipt.validate().is_ok()
            && validated_merge_base_sha(validator_base_sha).is_ok()
            && receipt.merge_base == validator_base_sha =>
        {
            PartialNegativeSelection::Negative(PartialNegativeFailures {
                observed_failures: receipt.observed_failures,
            })
        }
        _ => PartialNegativeSelection::Cold,
    }
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
    let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
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
    fn buck2_test_invocation_forwards_only_rustup_home_and_renders_exact_argv() {
        let (args, display) = buck2_test_invocation_from(
            "/opt/bin/buck2",
            &["//one:unit", "//two:fixture"],
            true,
            |key| match key {
                "RUSTUP_HOME" => Some("/opt/rust/rustup with space".to_owned()),
                "CARGO_HOME" => Some("/opt/rust/cargo".to_owned()),
                _ => None,
            },
        );

        assert_eq!(
            args,
            [
                "test",
                "//one:unit",
                "//two:fixture",
                "--keep-going",
                "--",
                "--env",
                "RUSTUP_HOME=/opt/rust/rustup with space",
            ]
        );
        assert_eq!(
            display,
            "/opt/bin/buck2 test //one:unit //two:fixture --keep-going -- --env 'RUSTUP_HOME=/opt/rust/rustup with space'"
        );
    }

    #[test]
    fn buck2_test_invocation_keeps_default_environment_without_valid_rustup_home() {
        for rustup_home in [None, Some(String::new()), Some("  ".to_owned())] {
            let (args, display) = buck2_test_invocation_from("buck2", &["//..."], false, |key| {
                (key == "RUSTUP_HOME")
                    .then(|| rustup_home.clone())
                    .flatten()
            });
            assert_eq!(args, ["test", "//..."]);
            assert_eq!(display, "buck2 test //...");
        }
    }

    #[test]
    fn buck2_test_invocation_shell_quotes_single_quotes_without_changing_argv() {
        let (args, display) = buck2_test_invocation_from("buck2", &["//..."], false, |key| {
            (key == "RUSTUP_HOME").then(|| "/tmp/rust'up".to_owned())
        });
        assert_eq!(args[4], "RUSTUP_HOME=/tmp/rust'up");
        assert_eq!(
            display,
            "buck2 test //... -- --env 'RUSTUP_HOME=/tmp/rust'\"'\"'up'"
        );
    }

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
    const TRUST_SHA: &str = "0123456789abcdef0123456789abcdef01234567";
    const TRUST_RUN: u64 = 11;
    const TRUST_ATTEMPT: u64 = 2;
    const TRUST_DIGEST: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn trusted_baseline_artifact_names_are_jointly_bound() {
        assert_eq!(
            baseline_artifact_name(BaselineKind::Build, TRUST_SHA, TRUST_RUN, TRUST_ATTEMPT)
                .unwrap(),
            format!(
                "build-health-baseline-{TRUST_SHA}-{TRUST_RUN}-{TRUST_ATTEMPT}-gate-affected-target-set"
            )
        );
        assert_ne!(
            baseline_artifact_name(BaselineKind::Build, TRUST_SHA, TRUST_RUN, TRUST_ATTEMPT)
                .unwrap(),
            baseline_artifact_name(BaselineKind::Test, TRUST_SHA, TRUST_RUN, TRUST_ATTEMPT)
                .unwrap()
        );
        assert!(BaselineKind::parse("build").is_ok());
        assert!(BaselineKind::parse("test").is_ok());
        assert!(BaselineKind::parse("Build").is_err());
        assert!(baseline_artifact_name(BaselineKind::Build, TRUST_SHA, 0, 1).is_err());
    }

    #[test]
    fn trusted_health_artifacts_accept_exact_non_empty_kind_scoped_reports() {
        let build_name =
            baseline_artifact_name(BaselineKind::Build, TRUST_SHA, TRUST_RUN, TRUST_ATTEMPT)
                .unwrap();
        let test_name =
            baseline_artifact_name(BaselineKind::Test, TRUST_SHA, TRUST_RUN, TRUST_ATTEMPT)
                .unwrap();
        let json = r#"{"results":{"root//a:a":{"success":"SUCCESS"}}}"#;
        assert_eq!(
            validate_trusted_baseline_artifact(
                BaselineKind::Build,
                &build_name,
                TRUST_SHA,
                TRUST_RUN,
                TRUST_ATTEMPT,
                json,
            ),
            Ok(1)
        );
        assert!(
            validate_trusted_baseline_artifact(
                BaselineKind::Test,
                &build_name,
                TRUST_SHA,
                TRUST_RUN,
                TRUST_ATTEMPT,
                json,
            )
            .is_err()
        );
        assert_eq!(
            validate_trusted_baseline_artifact(
                BaselineKind::Test,
                &test_name,
                TRUST_SHA,
                TRUST_RUN,
                TRUST_ATTEMPT,
                json,
            ),
            Ok(1)
        );
    }

    #[test]
    fn trusted_health_artifacts_refuse_bad_identity_or_payload() {
        let name = baseline_artifact_name(BaselineKind::Build, TRUST_SHA, TRUST_RUN, TRUST_ATTEMPT)
            .unwrap();
        for payload in ["not json", r#"{"ok":true}"#, r#"{"results":{}}"#] {
            assert!(
                validate_trusted_baseline_artifact(
                    BaselineKind::Build,
                    &name,
                    TRUST_SHA,
                    TRUST_RUN,
                    TRUST_ATTEMPT,
                    payload,
                )
                .is_err()
            );
        }
        assert!(baseline_artifact_name(BaselineKind::Build, "dev", TRUST_RUN, 1).is_err());
        assert!(
            validate_trusted_baseline_artifact(
                BaselineKind::Build,
                "build-health-baseline-stale",
                TRUST_SHA,
                TRUST_RUN,
                TRUST_ATTEMPT,
                r#"{"results":{"root//a:a":{"success":"SUCCESS"}}}"#,
            )
            .is_err()
        );
    }

    #[test]
    fn trusted_push_run_selection_accepts_one_completed_dev_push_even_when_aggregate_failed() {
        let runs = format!(
            r#"{{"workflow_runs":[
            {{"id":{TRUST_RUN},"run_attempt":{TRUST_ATTEMPT},"head_sha":"{TRUST_SHA}","event":"push","head_branch":"dev","status":"completed","conclusion":"failure","path":"{REQUIRED_CONTEXT_WORKFLOW_PATH}"}}
        ]}}"#
        );
        assert_eq!(
            trusted_dev_push_run(&runs, TRUST_SHA, REQUIRED_CONTEXT_WORKFLOW_PATH),
            Ok(Some(TrustedWorkflowRun {
                id: TRUST_RUN,
                attempt: TRUST_ATTEMPT,
                head_sha: TRUST_SHA.to_owned(),
            }))
        );
    }

    #[test]
    fn duplicate_or_malformed_exact_run_provenance_is_refused() {
        let duplicate = format!(
            r#"{{"workflow_runs":[
            {{"id":11,"run_attempt":1,"head_sha":"{TRUST_SHA}","event":"push","head_branch":"dev","status":"completed","conclusion":"failure","path":"{REQUIRED_CONTEXT_WORKFLOW_PATH}"}},
            {{"id":12,"run_attempt":1,"head_sha":"{TRUST_SHA}","event":"push","head_branch":"dev","status":"completed","conclusion":"success","path":"{REQUIRED_CONTEXT_WORKFLOW_PATH}"}}
        ]}}"#
        );
        assert!(
            trusted_dev_push_run(&duplicate, TRUST_SHA, REQUIRED_CONTEXT_WORKFLOW_PATH).is_err()
        );
        let malformed = format!(
            r#"{{"workflow_runs":[{{"id":11,"head_sha":"{TRUST_SHA}","event":"push","head_branch":"dev","status":"completed","conclusion":"success","path":"{REQUIRED_CONTEXT_WORKFLOW_PATH}"}}]}}"#
        );
        assert!(
            trusted_dev_push_run(&malformed, TRUST_SHA, REQUIRED_CONTEXT_WORKFLOW_PATH).is_err()
        );
    }

    #[test]
    fn foreign_workflow_and_nonterminal_runs_are_unavailable() {
        let runs = format!(
            r#"{{"workflow_runs":[
            {{"id":14,"run_attempt":1,"head_sha":"{TRUST_SHA}","event":"push","head_branch":"dev","status":"completed","conclusion":"success","path":".github/workflows/attacker-lane.yml"}},
            {{"id":15,"run_attempt":1,"head_sha":"{TRUST_SHA}","event":"push","head_branch":"dev","status":"in_progress","conclusion":null,"path":"{REQUIRED_CONTEXT_WORKFLOW_PATH}"}}
        ]}}"#
        );
        assert_eq!(
            trusted_dev_push_run(&runs, TRUST_SHA, REQUIRED_CONTEXT_WORKFLOW_PATH),
            Ok(None)
        );
    }

    #[test]
    fn trusted_run_parser_refuses_malformed_payloads() {
        assert!(
            trusted_dev_push_run("not json", TRUST_SHA, REQUIRED_CONTEXT_WORKFLOW_PATH).is_err()
        );
        assert!(
            trusted_dev_push_run(r#"{"ok":true}"#, TRUST_SHA, REQUIRED_CONTEXT_WORKFLOW_PATH)
                .is_err()
        );
        assert!(
            trusted_dev_push_run(
                r#"{"workflow_runs":[]}"#,
                "HEAD",
                REQUIRED_CONTEXT_WORKFLOW_PATH
            )
            .is_err()
        );
    }

    #[test]
    fn trusted_producer_requires_unique_terminal_green_run_and_head_binding() {
        let jobs = format!(
            r#"{{"jobs":[
            {{"id":40,"name":"unrelated","status":"completed","conclusion":"success"}},
            {{"id":41,"run_id":{TRUST_RUN},"head_sha":"{TRUST_SHA}","name":"{AFFECTED_SET_PRODUCER_JOB_NAME}","status":"completed","conclusion":"success"}}
        ]}}"#
        );
        assert_eq!(
            trusted_affected_set_producer_job(&jobs, TRUST_RUN, TRUST_SHA),
            Ok(Some(TrustedProducerJob {
                id: 41,
                run_id: TRUST_RUN,
                head_sha: TRUST_SHA.to_owned(),
                conclusion: "success".to_owned(),
            }))
        );
        for conclusion in ["failure", "cancelled"] {
            let jobs = format!(
                r#"{{"jobs":[{{"id":41,"run_id":{TRUST_RUN},"head_sha":"{TRUST_SHA}","name":"{AFFECTED_SET_PRODUCER_JOB_NAME}","status":"completed","conclusion":"{conclusion}"}}]}}"#
            );
            assert_eq!(
                trusted_affected_set_producer_job(&jobs, TRUST_RUN, TRUST_SHA),
                Ok(None)
            );
        }
        let duplicate = format!(
            r#"{{"jobs":[
            {{"id":41,"run_id":{TRUST_RUN},"head_sha":"{TRUST_SHA}","name":"{AFFECTED_SET_PRODUCER_JOB_NAME}","status":"completed","conclusion":"success"}},
            {{"id":42,"run_id":{TRUST_RUN},"head_sha":"{TRUST_SHA}","name":"{AFFECTED_SET_PRODUCER_JOB_NAME}","status":"completed","conclusion":"success"}}
        ]}}"#
        );
        assert!(trusted_affected_set_producer_job(&duplicate, TRUST_RUN, TRUST_SHA).is_err());
        let mismatched = format!(
            r#"{{"jobs":[{{"id":41,"run_id":99,"head_sha":"{TRUST_SHA}","name":"{AFFECTED_SET_PRODUCER_JOB_NAME}","status":"completed","conclusion":"success"}}]}}"#
        );
        assert!(trusted_affected_set_producer_job(&mismatched, TRUST_RUN, TRUST_SHA).is_err());
    }

    fn artifact_fixture(name: &str, expired: bool) -> String {
        format!(
            r#"{{"artifacts":[{{
            "id":22,"name":"{name}","expired":{expired},"digest":"{TRUST_DIGEST}",
            "size_in_bytes":42,"workflow_run":{{"id":{TRUST_RUN},"head_sha":"{TRUST_SHA}"}}
        }}]}}"#
        )
    }

    #[test]
    fn trusted_artifact_selection_accepts_one_exact_bound_immutable_artifact() {
        let name = baseline_artifact_name(BaselineKind::Build, TRUST_SHA, TRUST_RUN, TRUST_ATTEMPT)
            .unwrap();
        assert_eq!(
            trusted_baseline_artifact(&artifact_fixture(&name, false), &name, TRUST_RUN, TRUST_SHA),
            Ok(Some(TrustedBaselineArtifact {
                id: 22,
                name,
                digest: TRUST_DIGEST.to_owned(),
                size_in_bytes: 42,
                workflow_run_id: TRUST_RUN,
                head_sha: TRUST_SHA.to_owned(),
            }))
        );
    }

    #[test]
    fn missing_or_expired_artifact_is_unavailable() {
        let name = baseline_artifact_name(BaselineKind::Build, TRUST_SHA, TRUST_RUN, TRUST_ATTEMPT)
            .unwrap();
        assert_eq!(
            trusted_baseline_artifact(r#"{"artifacts":[]}"#, &name, TRUST_RUN, TRUST_SHA),
            Ok(None)
        );
        assert_eq!(
            trusted_baseline_artifact(&artifact_fixture(&name, true), &name, TRUST_RUN, TRUST_SHA),
            Ok(None)
        );
    }

    #[test]
    fn duplicate_malformed_or_unbound_artifacts_are_refused() {
        let name = baseline_artifact_name(BaselineKind::Build, TRUST_SHA, TRUST_RUN, TRUST_ATTEMPT)
            .unwrap();
        let one = artifact_fixture(&name, false);
        let artifact = serde_json::from_str::<Value>(&one).unwrap()["artifacts"][0].clone();
        let duplicate = json!({"artifacts": [artifact.clone(), artifact]}).to_string();
        assert!(trusted_baseline_artifact(&duplicate, &name, TRUST_RUN, TRUST_SHA).is_err());
        let missing_digest = format!(
            r#"{{"artifacts":[{{"id":21,"name":"{name}","expired":false,"size_in_bytes":42,"workflow_run":{{"id":{TRUST_RUN},"head_sha":"{TRUST_SHA}"}}}}]}}"#
        );
        assert!(trusted_baseline_artifact(&missing_digest, &name, TRUST_RUN, TRUST_SHA).is_err());
        let wrong_run =
            artifact_fixture(&name, false).replace(&format!(r#""id":{TRUST_RUN}"#), r#""id":99"#);
        assert!(trusted_baseline_artifact(&wrong_run, &name, TRUST_RUN, TRUST_SHA).is_err());
    }
    fn trusted_partial_negative_fixture() -> PartialNegativeReceipt {
        PartialNegativeReceipt {
            schema_version: PARTIAL_NEGATIVE_SCHEMA_VERSION,
            source: PARTIAL_NEGATIVE_SOURCE.to_owned(),
            completeness: PARTIAL_NEGATIVE_COMPLETENESS.to_owned(),
            merge_base: "b6cebdaa897912a0bb29a55406375f4bb0109cd6".to_owned(),
            job: PartialNegativeJobBinding {
                workflow_path: REQUIRED_CONTEXT_WORKFLOW_PATH.to_owned(),
                run_id: 30_747_487_757,
                run_attempt: 1,
                job_id: 91_495_435_478,
                job_name: "gate · affected-set (ADR-0554, binding workspace coverage)".to_owned(),
                step_number: 8,
                step_name: "Binding affected-set build + test (cone-binding; FULL tier = build-health ratchet)".to_owned(),
            },
            build_action: PartialNegativeBuildAction {
                label: "root//oya:corpus-yaml-facts".to_owned(),
                configured_platform_token: "e39f0472c2e09c96".to_owned(),
                rule: "genrule".to_owned(),
                action_terminal: PartialNegativeActionTerminal::NoExitCode,
            },
            observed_failures: set(&["root//oya:corpus-yaml-facts"]),
            test_policy: PARTIAL_NEGATIVE_TEST_POLICY.to_owned(),
        }
    }

    #[test]
    fn partial_negative_receipt_validates_the_trusted_fixture_bindings() {
        let receipt = trusted_partial_negative_fixture();
        assert_eq!(receipt.validate(), Ok(()));

        let mut invalid = receipt;
        invalid.schema_version = 1;
        assert!(invalid.validate().is_err());

        let mut missing_action = trusted_partial_negative_fixture();
        missing_action.build_action.label = "root//oya:not-observed".to_owned();
        assert!(missing_action.validate().is_err());
    }

    #[test]
    fn partial_negative_parser_accepts_one_literal_fail_block() {
        let log = concat!(
            "2026-08-01T00:00:00Z setup\n",
            "2026-08-01T00:00:01Z affected-set: RED — admission FULL build failed on 1 target(s) (integration tip must be green, no grandfathering):\n",
            "2026-08-01T00:00:01Z affected-set:   BUILD-FAIL root//oya:corpus-yaml-facts\n",
            "2026-08-01T00:00:01Z affected-set: REPRODUCE: buck2 build //...\n",
        );
        assert_eq!(
            parse_partial_negative_failures(log)
                .unwrap()
                .observed_failures,
            set(&["root//oya:corpus-yaml-facts"])
        );
    }

    #[test]
    fn partial_negative_parser_refuses_non_literal_or_incomplete_evidence() {
        let header = "affected-set: RED — admission FULL build failed on 1 target(s) (integration tip must be green, no grandfathering):\n";
        for log in [
            format!(
                "{header}affected-set:   SKIPPED root//oya:not-failed\naffected-set: REPRODUCE:"
            ),
            format!(
                "{header}affected-set:   BUILD-FAIL root//oya:one\naffected-set:   BUILD-FAIL root//oya:two\naffected-set: REPRODUCE:"
            ),
            format!("{header}affected-set:   BUILD-FAIL root//oya:one"),
            format!(
                "{header}affected-set:   BUILD-FAIL root//oya:one extra\naffected-set: REPRODUCE:"
            ),
        ] {
            assert!(
                parse_partial_negative_failures(&log).is_err(),
                "accepted `{log}`"
            );
        }
    }

    #[test]
    fn partial_negative_parser_refuses_duplicate_blocks_and_oversize_logs() {
        let block = concat!(
            "affected-set: RED — admission FULL build failed on 1 target(s) (integration tip must be green, no grandfathering):\n",
            "affected-set:   BUILD-FAIL root//oya:one\n",
            "affected-set: REPRODUCE:\n",
        );
        assert!(parse_partial_negative_failures(&format!("{block}{block}")).is_err());
        assert!(
            parse_partial_negative_failures(&"x".repeat(PARTIAL_NEGATIVE_LOG_MAX_BYTES + 1))
                .is_err()
        );
    }

    #[test]
    fn partial_negative_selection_is_positive_first_and_base_bounded() {
        let receipt = trusted_partial_negative_fixture();
        let failures = PartialNegativeFailures {
            observed_failures: receipt.observed_failures.clone(),
        };
        let base = receipt.merge_base.clone();
        assert_eq!(
            select_partial_negative_baseline(
                PositiveBaselineState::Valid,
                NegativeBaselineState::Valid(receipt.clone()),
                Some(&base),
            ),
            PartialNegativeSelection::Positive
        );
        assert_eq!(
            select_partial_negative_baseline(
                PositiveBaselineState::Absent,
                NegativeBaselineState::Valid(receipt.clone()),
                Some(&base),
            ),
            PartialNegativeSelection::Negative(failures.clone())
        );

        let mut malformed = receipt.clone();
        malformed.source = "candidate-log".to_owned();
        assert_eq!(
            select_partial_negative_baseline(
                PositiveBaselineState::Absent,
                NegativeBaselineState::Valid(malformed),
                Some(&base),
            ),
            PartialNegativeSelection::Cold
        );
        let different_base = "0123456789abcdef0123456789abcdef01234567";
        for (positive, negative, validator_base) in [
            (
                PositiveBaselineState::Invalid,
                NegativeBaselineState::Valid(receipt.clone()),
                Some(base.as_str()),
            ),
            (
                PositiveBaselineState::Absent,
                NegativeBaselineState::Invalid,
                Some(base.as_str()),
            ),
            (
                PositiveBaselineState::Absent,
                NegativeBaselineState::Valid(receipt.clone()),
                None,
            ),
            (
                PositiveBaselineState::Absent,
                NegativeBaselineState::Valid(receipt.clone()),
                Some(different_base),
            ),
        ] {
            assert_eq!(
                select_partial_negative_baseline(positive, negative, validator_base),
                PartialNegativeSelection::Cold
            );
        }
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
            Some(&json!({"state": "reused"})),
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
    fn operator_artifact_records_a_dark_fast_path_as_a_typed_degrade() {
        // THE REGRESSION THIS PINS (CI job 91383250718): `gh` was absent from the owned runner
        // image, so the consumer degraded to the cold rebuild on EVERY run. `source` alone reads
        // "cold-rebuild" for that, which is byte-identical to the healthy "no baseline published
        // for this merge-base" case — the artifact could not distinguish a broken fleet from a
        // working one, so nobody did. The typed outcome must survive into the artifact.
        let outcome = json!({
            "state": "capability-fault",
            "capability_fault": true,
            "reason": "GET repos/o/r/actions/...: the GitHub API is unreachable",
        });
        let artifact = affected_set_operator_artifact(
            "auto",
            "0123456789abcdef0123456789abcdef01234567",
            "89abcdef0123456789abcdef0123456789abcdef",
            true,
            None,
            Some(&outcome),
            &Decision::Full {
                reasons: vec!["escape trigger".to_owned()],
            },
            &[],
        );
        let baseline = &artifact["merge_base_build_health_baseline"];
        assert_eq!(baseline["source"], "cold-rebuild");
        assert_eq!(baseline["reuse_outcome"]["state"], "capability-fault");
        assert_eq!(baseline["reuse_outcome"]["capability_fault"], true);

        // A non-FULL run never invokes the consumer at all; that is an ABSENCE of a decision, not
        // a degrade, and must not be reported as one.
        let never_ran = affected_set_operator_artifact(
            "auto",
            "0123456789abcdef0123456789abcdef01234567",
            "89abcdef0123456789abcdef0123456789abcdef",
            false,
            None,
            None,
            &Decision::NoGraphTargets,
            &[],
        );
        assert_eq!(
            never_ran["merge_base_build_health_baseline"]["reuse_outcome"],
            Value::Null
        );
    }

    #[test]
    fn a_dead_capability_is_a_distinct_state_from_a_healthy_miss() {
        // The four states must stay mutually distinguishable by BOTH of the channels that consume
        // them: the machine token in the sidecar/artifact, and the process exit code the workflow
        // sees. Collapsing any pair is how the original defect hid.
        let all = [
            BaselineReuseState::Reused,
            BaselineReuseState::Unavailable,
            BaselineReuseState::Refused,
            BaselineReuseState::CapabilityFault,
        ];
        let tokens: BTreeSet<&str> = all.iter().map(|s| s.as_str()).collect();
        assert_eq!(tokens.len(), all.len(), "state tokens must be unique");
        let codes: BTreeSet<u8> = all.iter().map(|s| s.exit_code()).collect();
        assert_eq!(codes.len(), all.len(), "exit codes must be unique");

        // Only a real environment fault is loud. `Unavailable` is the NORMAL path (a merge-base
        // whose dev push never published, or whose artifacts expired) and `Refused` is the gate
        // working as designed; warning on either would drown the one signal that means "fix me".
        assert!(BaselineReuseState::CapabilityFault.is_capability_fault());
        assert!(!BaselineReuseState::Unavailable.is_capability_fault());
        assert!(!BaselineReuseState::Refused.is_capability_fault());
        assert!(!BaselineReuseState::Reused.is_capability_fault());

        // Fail-closed contract the workflow depends on: anything short of a validated pair is
        // non-zero, so the cold rebuild runs.
        assert_eq!(BaselineReuseState::Reused.exit_code(), 0);
        for state in [
            BaselineReuseState::Unavailable,
            BaselineReuseState::Refused,
            BaselineReuseState::CapabilityFault,
        ] {
            assert_ne!(state.exit_code(), 0, "{state:?} must fall back");
        }
    }

    #[test]
    fn http_statuses_split_answers_from_faults() {
        assert_eq!(classify_api_status(200), None);
        assert_eq!(classify_api_status(204), None);
        // A working API saying "not there" — the cold rebuild is the RIGHT answer, quietly.
        for status in [404, 410] {
            assert_eq!(
                classify_api_status(status),
                Some(BaselineReuseState::Unavailable),
                "HTTP {status}"
            );
        }
        // No answer at all: absent/expired token, missing `actions: read`, rate limit, outage.
        // Every one of these is an operator's problem, and none is a property of the PR.
        for status in [401, 403, 429, 500, 502, 503] {
            assert_eq!(
                classify_api_status(status),
                Some(BaselineReuseState::CapabilityFault),
                "HTTP {status}"
            );
        }
        // Anything unmodelled is refused rather than guessed at.
        for status in [301, 400, 422] {
            assert_eq!(
                classify_api_status(status),
                Some(BaselineReuseState::Refused),
                "HTTP {status}"
            );
        }
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
