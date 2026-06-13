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

use serde_json::Value;

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
        Ok(Policy {
            gate_id,
            universe: str_field(&v, "universe")?,
            full_run_targets,
            full_trigger_patterns: str_list_field(&v, "full_trigger_patterns")?,
            require_owner_patterns: str_list_field(&v, "require_owner_patterns")?,
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
            default_base_ref: str_field(&v, "default_base_ref")?,
        })
    }
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

/// One diff entry, as parsed by the adapter from `git diff --name-status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// File exists at HEAD (added, modified, type-changed, copy/rename destination).
    Present(String),
    /// File no longer exists at HEAD (deleted, rename source).
    Deleted(String),
}

impl Change {
    pub fn path(&self) -> &str {
        match self {
            Change::Present(p) | Change::Deleted(p) => p,
        }
    }
}

/// Why a path was classified the way it was — carried verbatim into the transparency output
/// (the founder automation directive: FAIL output must say exactly what ran and why).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathClass {
    /// Matched an escape-trigger pattern -> FULL.
    FullTrigger(String),
    /// Deleted file in a graph-relevant class -> FULL (its cone is uncomputable at HEAD).
    DeletedGraphFile,
    /// Buildfile (BUCK/BUCK.v2/PACKAGE) changed or deleted -> FULL (blast radius exceeds its
    /// own package: it can add/remove targets or shadow the file dependents load).
    Buildfile,
    /// Package-definition file -> expands to this package target pattern.
    PackagePattern(String),
    /// Sent to `owner()` resolution.
    OwnerQuery,
    /// Deleted file outside every graph-relevant class -> no targets.
    DeletedIrrelevant,
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
            let verb = match change {
                Change::Deleted(_) => "deleted",
                Change::Present(_) => "changed",
            };
            plan.full_reasons.push(format!(
                "buildfile `{path}` {verb} (blast radius exceeds its own package)"
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
            match change {
                Change::Deleted(_) => {
                    plan.full_reasons
                        .push(format!("package sibling `{path}` was deleted"));
                    plan.classified
                        .push((path.to_owned(), PathClass::DeletedGraphFile));
                }
                Change::Present(_) => match package_pattern(path, policy) {
                    Some(pat) => {
                        plan.package_patterns.push(pat.clone());
                        plan.classified
                            .push((path.to_owned(), PathClass::PackagePattern(pat)));
                    }
                    None => {
                        plan.full_reasons.push(format!(
                            "package sibling `{path}` maps to no configured cell root (derivation uncertainty)"
                        ));
                        plan.classified
                            .push((path.to_owned(), PathClass::DeletedGraphFile));
                    }
                },
            }
            continue;
        }
        match change {
            Change::Deleted(_) => {
                if policy
                    .require_owner_patterns
                    .iter()
                    .any(|pat| glob_match(pat, path))
                {
                    plan.full_reasons
                        .push(format!("graph-relevant file `{path}` was deleted"));
                    plan.classified
                        .push((path.to_owned(), PathClass::DeletedGraphFile));
                } else {
                    plan.classified
                        .push((path.to_owned(), PathClass::DeletedIrrelevant));
                }
            }
            Change::Present(_) => {
                plan.owner_paths.push(path.to_owned());
                plan.classified
                    .push((path.to_owned(), PathClass::OwnerQuery));
            }
        }
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
    for path in &plan.owner_paths {
        let owners = owner_results.get(path).map(Vec::as_slice).unwrap_or(&[]);
        if owners.is_empty() {
            if policy
                .require_owner_patterns
                .iter()
                .any(|pat| glob_match(pat, path))
            {
                refusals.push(path.clone());
            }
            // else: provably outside the graph (docs class) — fine.
        } else {
            seeds.extend(owners.iter().cloned());
        }
    }
    if !refusals.is_empty() {
        refusals.sort();
        return Decision::RefuseUnowned { paths: refusals };
    }
    if !plan.full_reasons.is_empty() {
        return Decision::Full {
            reasons: plan.full_reasons.clone(),
        };
    }
    if !seeds.is_empty() {
        return Decision::Affected {
            seeds: seeds.into_iter().collect(),
        };
    }
    Decision::NoGraphTargets
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
            default_base_ref: "origin/main".to_owned(),
        }
    }

    #[test]
    fn policy_rejects_wrong_gate_id() {
        let err = Policy::from_json(r#"{"gate_id": "other"}"#).unwrap_err();
        assert!(err.0.contains("does not match engine"), "{err}");
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
}
