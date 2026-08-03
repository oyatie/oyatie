//! # cloud-ci-supply-chain-audit (owned RustSec advisory gate; replaces reverted #974 shell scan)
//!
//! PR #974 added a born-blocking supply-chain gate by shelling out to `cargo-audit` / `cargo-deny`
//! — shell + a network-fetching advisory index, both forbidden by the no-shell + hermetic-gate +
//! rust-purity doctrine. #977 reverted it (keeping the quinn-proto CVE fix). This gate is the owned
//! replacement: a PURE, hermetic predicate that matches the workspace `Cargo.lock` against a
//! VENDORED, content-addressed RustSec advisory snapshot. No shell, no network, no clock, no
//! `rustsec`/`git2`/`libgit2` crates — only `serde_json` + `toml` + `semver` (the last promoted to a
//! direct workspace dep via one reindeer buckify, adding ZERO new crate to `Cargo.lock`).
//!
//! ## Split of concerns (hermetic gate vs network reconciler)
//! The network/clock half — pinning a fresh advisory-db commit, distilling, opening a GitOps PR,
//! and `remove_by` expiry SLOs — lives in a SEPARATE owned reconciler (deferred Slice D). THIS gate
//! only reads committed bytes: the `Cargo.lock` + the vendored `advisory-mirror/{advisories.json,
//! mirror-manifest.json}` produced by `oya-advisory-mirror-kernel`. That keeps the gate buck2-cacheable
//! and deterministic.
//!
//! ## Which lockfiles are scanned
//! `policy.lockfile_path` is EITHER one path or a LIST of paths, and the locked corpus is their
//! UNION with each entry tagged by its source lockfile. This repo has three workspaces (the root
//! plus the ADR-0512 `kernel/` and `cloud/cloud-kernel/` carve-outs, both in the root `Cargo.toml`
//! `exclude`); scanning only the root lock left ~66 (name,version) pairs — including crypto crates
//! like `aws-lc-sys` — permanently unscanned, so a `cargo update` in a carve-out could introduce an
//! affected version with the gate staying green. Every finding names the lockfile it came from.
//!
//! ## Matching
//! For each advisory whose `package` matches a locked crate `name`, the locked [`semver::Version`] is
//! AFFECTED iff it satisfies NO `patched` and NO `unaffected` [`semver::VersionReq`]. Fail-closed: an
//! unparseable locked version or an unparseable advisory req is treated as affected (we cannot prove
//! safe). An affected advisory blocks unless its id is in `policy.ignore[]`:
//! - a security vulnerability (no `informational`) → `SCA-VULN`.
//! - an `informational = "unmaintained"` advisory → `SCA-UNMAINTAINED`, gated by
//!   `policy.unmaintained_policy == "all"`.
//! - `informational = "unsound" | "notice"` are tracked but OUT of this gate's blocking scope
//!   (documented policy extension point; promote via a future policy knob if needed).
//!
//! ## Mirror integrity (fail-closed against a vacuously-green scan)
//! - `SCA-MIRROR-MALFORMED` — the manifest `content_hash` ≠ the recomputed [`canonical_hash`] of the
//!   committed `advisories.json`, OR the manifest `advisory_count` ≠ the actual record count, OR the
//!   advisories payload is missing/non-array. A truncated or desynced mirror cannot pass.
//! - `SCA-MIRROR-UNDERFLOW` — fewer than `policy.min_advisories` records (defends a truncated/empty
//!   mirror that would otherwise make the gate vacuously green).
//!
//! ## Shrink-only self-clean
//! - `SCA-STALE-IGNORE` — a `policy.ignore[]` id that suppresses NO live affected advisory (the
//!   underlying vuln was fixed / the dep dropped / the crate left the tree). The `--write` mode
//!   removes these (shrink-only); it NEVER adds an ignore (a new vuln must be fixed, not baselined).
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `SCA-VULN` — a security advisory affects a locked crate and is not ignored.
//! - `SCA-UNMAINTAINED` — an unmaintained advisory affects a locked crate (policy=all) and is not ignored.
//! - `SCA-STALE-IGNORE` — an ignore id matches no live affected advisory (shrink-only self-clean).
//! - `SCA-MIRROR-MALFORMED` — the vendored mirror is corrupt / desynced (fail-closed).
//! - `SCA-MIRROR-UNDERFLOW` — the mirror has too few advisories (fail-closed against a false-green).
//! - `SCA-POLICY-GATE-ID-MISMATCH` — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `SCA-POLICY-MALFORMED` — the policy is structurally invalid (fail-closed).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::path::Path;

use oya_advisory_mirror_kernel::{Advisory, canonical_hash};
use semver::{Version, VersionReq};
use serde_json::{Value, json};

/// The gate id, matching the buck2 target stem + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-supply-chain-audit";

/// The remediation doctrine pointer findings carry.
pub const REMEDIATION_DOCTRINE: &str =
    "upgrade the affected crate to a patched version (bump the workspace pin / transitive dep), or — \
     for an unmaintained dep with no maintained drop-in — add a time-boxed policy.ignore[] entry with \
     a reason, pull_chain, and remove_by date";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 7] = [
    "SCA-VULN",
    "SCA-UNMAINTAINED",
    "SCA-STALE-IGNORE",
    "SCA-MIRROR-MALFORMED",
    "SCA-MIRROR-UNDERFLOW",
    "SCA-POLICY-GATE-ID-MISMATCH",
    "SCA-POLICY-MALFORMED",
];

/// The sentinel key for codes that are mirror/policy-level rather than per-advisory.
const MIRROR_KEY: &str = "<mirror>";
const POLICY_KEY: &str = "<policy>";

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only)
// ---------------------------------------------------------------------------

/// Errors collecting the observed graph. Returned instead of panicking so the caller decides how to
/// surface them — an unreadable lock/mirror is a fail-closed error, never a silently empty scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    Io(String),
    Parse(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::Io(m) => write!(f, "supply-chain-audit io: {m}"),
            CollectError::Parse(m) => write!(f, "supply-chain-audit parse: {m}"),
        }
    }
}

impl std::error::Error for CollectError {}

/// Resolve `policy.lockfile_path` to the list of lockfiles to scan. Accepts EITHER a single string
/// (back-compatible; the pack-shaped single-workspace repo) or an array of strings (a repo with
/// carve-out workspaces, each with its OWN `Cargo.lock` — this repo has three per ADR-0512). A list
/// on the existing key rather than a second key keeps the policy surface neutral: one knob still
/// answers "which locks does this repo have", and a single-workspace adopter keeps the string form.
///
/// FAIL-CLOSED: a non-string / non-array value, a non-string element, or an EMPTY array is a policy
/// error — never a silently-empty scan (that would be a vacuous green, the exact defect this fixes).
pub fn lockfile_paths(policy: &Value) -> Result<Vec<String>, CollectError> {
    match policy.get("lockfile_path") {
        None => Ok(vec!["Cargo.lock".to_owned()]),
        Some(Value::String(one)) => Ok(vec![one.clone()]),
        Some(Value::Array(items)) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                let Some(path) = item.as_str() else {
                    return Err(CollectError::Parse(format!(
                        "policy.lockfile_path array element {item} is not a string"
                    )));
                };
                out.push(path.to_owned());
            }
            if out.is_empty() {
                return Err(CollectError::Parse(
                    "policy.lockfile_path is an EMPTY array; a gate that scans no lockfile is vacuously green (fail-closed)".to_owned(),
                ));
            }
            Ok(out)
        }
        Some(other) => Err(CollectError::Parse(format!(
            "policy.lockfile_path must be a string or an array of strings; got {other}"
        ))),
    }
}

/// Collect the observed graph: the locked crates from EVERY configured `Cargo.lock` + the vendored
/// advisory snapshot.
///
/// The ONLY I/O. Reads each [`lockfile_paths`] entry (TOML) and `policy.mirror_dir/{advisories.json,
/// mirror-manifest.json}` (JSON). Emits
/// `{ "locked": [ { "name", "version", "lockfile" }, .. ], "advisories": [ <Advisory>, .. ], "manifest": {..} }`.
/// The locked set is the UNION across lockfiles, each entry TAGGED with the lockfile it came from so
/// a finding can name the workspace (a finding that says only "aws-lc-sys is affected" is unactionable
/// in a multi-workspace repo). An unreadable configured lockfile is a hard error, not a skipped scan.
pub fn collect(repo_root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let lockfiles = lockfile_paths(policy)?;
    let mirror_dir = policy
        .get("mirror_dir")
        .and_then(Value::as_str)
        .ok_or_else(|| CollectError::Parse("policy.mirror_dir is required".to_owned()))?;

    let mut locked: Vec<Value> = Vec::new();
    for lockfile in &lockfiles {
        let lock_text = read_file(&repo_root.join(lockfile))?;
        locked.extend(parse_locked(&lock_text, lockfile)?);
    }
    locked.sort_by(|a, b| locked_sort_key(a).cmp(&locked_sort_key(b)));
    locked.dedup();

    let advisories_text = read_file(&repo_root.join(mirror_dir).join("advisories.json"))?;
    let advisories: Value = serde_json::from_str(&advisories_text)
        .map_err(|e| CollectError::Parse(format!("advisories.json: {e}")))?;

    let manifest_text = read_file(&repo_root.join(mirror_dir).join("mirror-manifest.json"))?;
    let manifest: Value = serde_json::from_str(&manifest_text)
        .map_err(|e| CollectError::Parse(format!("mirror-manifest.json: {e}")))?;

    Ok(json!({
        "locked": locked,
        "advisories": advisories,
        "manifest": manifest,
    }))
}

fn read_file(path: &Path) -> Result<String, CollectError> {
    std::fs::read_to_string(path)
        .map_err(|e| CollectError::Io(format!("read {}: {e}", path.display())))
}

/// Parse the `[[package]]` table of one `Cargo.lock` into `[{ "name", "version", "lockfile" }]`.
/// `lockfile` is the repo-relative path the entry came from — the provenance a finding names.
fn parse_locked(lock_text: &str, lockfile: &str) -> Result<Vec<Value>, CollectError> {
    let doc: toml::Value =
        toml::from_str(lock_text).map_err(|e| CollectError::Parse(format!("{lockfile}: {e}")))?;
    let packages = doc
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| CollectError::Parse(format!("{lockfile} has no [[package]] table")))?;
    let mut locked: Vec<Value> = Vec::with_capacity(packages.len());
    for pkg in packages {
        let Some(name) = pkg.get("name").and_then(toml::Value::as_str) else {
            continue;
        };
        let Some(version) = pkg.get("version").and_then(toml::Value::as_str) else {
            continue;
        };
        locked.push(json!({ "name": name, "version": version, "lockfile": lockfile }));
    }
    Ok(locked)
}

fn locked_sort_key(v: &Value) -> (String, String, String) {
    let field = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_owned();
    (field("name"), field("version"), field("lockfile"))
}

// ---------------------------------------------------------------------------
// Pure evaluation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
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
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_findings(findings: &BTreeSet<Finding>) -> Self {
        let violations = findings.iter().map(|f| f.code.clone()).collect::<BTreeSet<_>>();
        Self {
            verdict: if violations.is_empty() {
                Verdict::Green
            } else {
                Verdict::Red
            },
            violations,
        }
    }
}

/// A `policy.ignore[]` entry: the suppressed advisory id + its bookkeeping.
struct IgnoreEntry {
    id: String,
}

fn parse_ignore(policy: &Value) -> Option<Vec<IgnoreEntry>> {
    let arr = policy.get("ignore").and_then(Value::as_array)?;
    let mut out = Vec::with_capacity(arr.len());
    for entry in arr {
        let id = entry.get("id").and_then(Value::as_str)?;
        out.push(IgnoreEntry { id: id.to_owned() });
    }
    Some(out)
}

/// Whether `version_str` (a locked crate version) is AFFECTED by `advisory`. FAIL-CLOSED: an
/// unparseable locked version, or an advisory with NO parseable patched/unaffected req that the
/// version satisfies, is treated as affected.
fn version_affected(version_str: &str, advisory: &Advisory) -> bool {
    let Ok(version) = Version::parse(version_str) else {
        return true; // fail-closed: cannot parse the locked version → treat as affected.
    };
    let safe = advisory
        .patched
        .iter()
        .chain(advisory.unaffected.iter())
        .any(|req_str| match VersionReq::parse(req_str) {
            Ok(req) => req.matches(&version),
            Err(_) => false, // fail-closed: an unparseable req does NOT prove the version safe.
        });
    !safe
}

/// Every locked site AFFECTED by `advisory`, rendered as `` `<lockfile>` <version> `` and deduped.
/// Empty ⇒ the advisory does not apply. Naming the lockfile is load-bearing in a multi-workspace
/// repo: "aws-lc-sys is affected" without the workspace is not an actionable finding.
fn affected_sites(locked: &[Value], advisory: &Advisory) -> Vec<String> {
    let mut sites: BTreeSet<String> = BTreeSet::new();
    for pkg in locked {
        if pkg.get("name").and_then(Value::as_str) != Some(advisory.package.as_str()) {
            continue;
        }
        let Some(version) = pkg.get("version").and_then(Value::as_str) else {
            continue;
        };
        if !version_affected(version, advisory) {
            continue;
        }
        let lockfile = pkg
            .get("lockfile")
            .and_then(Value::as_str)
            .unwrap_or("<unknown lockfile>");
        sites.insert(format!("`{lockfile}` {version}"));
    }
    sites.into_iter().collect()
}

/// Pure evaluator. `policy` is DATA (`supply-chain-audit-policy.json`); `observed` is the graph
/// shaped by [`collect`]. Unit-testable without a filesystem.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "SCA-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    // Fail CLOSED on a structurally invalid policy: a missing/non-array `ignore`, or an entry
    // lacking an `id`, means the suppression vocabulary is corrupt — flag loudly, do not proceed.
    let Some(ignore) = parse_ignore(policy) else {
        findings.insert(Finding::new(
            "SCA-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `ignore` must be an array of { id, reason, pull_chain, remove_by } objects (each with a string `id`); correct the policy before the gate can evaluate",
        ));
        return findings;
    };
    let ignore_ids: BTreeSet<&str> = ignore.iter().map(|e| e.id.as_str()).collect();

    // Parse the vendored advisories; a missing/non-array/malformed payload fails closed.
    let Some(advisories_val) = observed.get("advisories").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "SCA-MIRROR-MALFORMED",
            MIRROR_KEY,
            "vendored advisories.json is missing or not a JSON array (fail-closed against a vacuously-green scan)",
        ));
        return findings;
    };
    let advisories: Vec<Advisory> =
        match serde_json::from_value(Value::Array(advisories_val.clone())) {
            Ok(a) => a,
            Err(e) => {
                findings.insert(Finding::new(
                    "SCA-MIRROR-MALFORMED",
                    MIRROR_KEY,
                    format!("vendored advisories.json does not match the Advisory schema: {e}"),
                ));
                return findings;
            }
        };

    // Mirror integrity: recomputed content hash + count consistency.
    let manifest = observed.get("manifest").cloned().unwrap_or(Value::Null);
    let recomputed = canonical_hash(&advisories);
    let declared_hash = manifest.get("content_hash").and_then(Value::as_str).unwrap_or("");
    if declared_hash != recomputed {
        findings.insert(Finding::new(
            "SCA-MIRROR-MALFORMED",
            MIRROR_KEY,
            format!(
                "mirror-manifest.json content_hash `{declared_hash}` ≠ recomputed canonical_hash `{recomputed}` of advisories.json — the vendored mirror is corrupt or was regenerated without re-stamping the manifest. Regenerate via the oya-advisory-mirror-producer."
            ),
        ));
    }
    let declared_count = manifest.get("advisory_count").and_then(Value::as_u64);
    let actual_count = advisories.len() as u64;
    if declared_count != Some(actual_count) {
        findings.insert(Finding::new(
            "SCA-MIRROR-MALFORMED",
            MIRROR_KEY,
            format!(
                "mirror-manifest.json advisory_count {declared_count:?} ≠ actual record count {actual_count} in advisories.json (count desync — fail-closed)"
            ),
        ));
    }
    let min_advisories = policy.get("min_advisories").and_then(Value::as_u64).unwrap_or(0);
    if actual_count < min_advisories {
        findings.insert(Finding::new(
            "SCA-MIRROR-UNDERFLOW",
            MIRROR_KEY,
            format!(
                "vendored mirror has {actual_count} advisories, below the policy floor of {min_advisories}; a truncated/empty mirror would make the gate vacuously green (fail-closed)"
            ),
        ));
    }

    // The locked crate (name -> versions) corpus.
    let locked = observed.get("locked").and_then(Value::as_array).cloned().unwrap_or_default();
    let unmaintained_all =
        policy.get("unmaintained_policy").and_then(Value::as_str) == Some("all");

    // The set of ignore ids that actually suppressed a LIVE affected advisory (for stale detection).
    let mut live_affected_ids: BTreeSet<String> = BTreeSet::new();

    for advisory in &advisories {
        let sites = affected_sites(&locked, advisory);
        if sites.is_empty() {
            continue;
        }
        let sites = sites.join(", ");

        match advisory.informational.as_deref() {
            None => {
                // A security vulnerability.
                live_affected_ids.insert(advisory.id.clone());
                if !ignore_ids.contains(advisory.id.as_str()) {
                    findings.insert(Finding::new(
                        "SCA-VULN",
                        &advisory.id,
                        format!(
                            "security advisory {} affects locked crate `{}` at {} (no installed version satisfies patched {:?} / unaffected {:?}). {REMEDIATION_DOCTRINE}.",
                            advisory.id, advisory.package, sites, advisory.patched, advisory.unaffected
                        ),
                    ));
                }
            }
            Some("unmaintained") => {
                if unmaintained_all {
                    live_affected_ids.insert(advisory.id.clone());
                    if !ignore_ids.contains(advisory.id.as_str()) {
                        findings.insert(Finding::new(
                            "SCA-UNMAINTAINED",
                            &advisory.id,
                            format!(
                                "unmaintained advisory {} affects locked crate `{}` at {} (unmaintained_policy=all). Migrate off the crate, or {REMEDIATION_DOCTRINE}.",
                                advisory.id, advisory.package, sites
                            ),
                        ));
                    }
                }
            }
            Some(_other) => {
                // `unsound` / `notice`: tracked but out of this gate's blocking scope (documented).
            }
        }
    }

    // Shrink-only self-clean: an ignore id that suppressed nothing live is stale.
    for entry in &ignore {
        if !live_affected_ids.contains(&entry.id) {
            findings.insert(Finding::new(
                "SCA-STALE-IGNORE",
                &entry.id,
                format!(
                    "policy.ignore[] id `{}` matches no live affected advisory (the vuln was fixed, the crate left the tree, or the id is wrong). Remove it from policy.ignore[] — the ignore list is shrink-only (run the gate binary with --write).",
                    entry.id
                ),
            ));
        }
    }

    findings
}

/// Bare-code projection of [`evaluate_keyed`]; the single source of truth for the verdict.
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

/// The shrink-only set of ignore ids to KEEP: every prior ignore id that still suppresses a live
/// affected advisory. `--write` rewrites `policy.ignore[]` to this subset (drops stale entries). It
/// never adds an id — a new affected advisory must be FIXED, not auto-ignored. Returns
/// `(kept_ids_sorted, dropped_ids_sorted)`.
pub fn shrink_only_ignore(policy: &Value, observed: &Value) -> (Vec<String>, Vec<String>) {
    let findings = evaluate_keyed(policy, observed);
    let stale: BTreeSet<String> = findings
        .iter()
        .filter(|f| f.code == "SCA-STALE-IGNORE")
        .map(|f| f.key.clone())
        .collect();
    let mut kept = Vec::new();
    let mut dropped = Vec::new();
    if let Some(ignore) = parse_ignore(policy) {
        for entry in ignore {
            if stale.contains(&entry.id) {
                dropped.push(entry.id);
            } else {
                kept.push(entry.id);
            }
        }
    }
    kept.sort();
    kept.dedup();
    dropped.sort();
    dropped.dedup();
    (kept, dropped)
}

/// Human-readable render of the findings. Never a bare FAIL — every finding prints its detail.
pub fn render_findings(findings: &BTreeSet<Finding>) -> String {
    if findings.is_empty() {
        return "supply-chain-audit gate passed: no locked crate is affected by an un-ignored RustSec advisory; the vendored mirror is intact".to_owned();
    }
    let mut out = String::from("supply-chain-audit gate failed (owned RustSec advisory scan):\n");
    for finding in findings {
        out.push_str(&format!("    - {} {}\n        {}\n", finding.code, finding.key, finding.detail));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "lockfile_path": "Cargo.lock",
            "mirror_dir": "x",
            "unmaintained_policy": "all",
            "min_advisories": 0,
            "ignore": []
        })
    }

    /// Build an observed graph from synthetic locked crates + advisories, with a CONSISTENT manifest
    /// (correct content_hash + count) so the integrity checks pass and the matching logic is isolated.
    fn observed(locked: &[(&str, &str)], advisories: Vec<Advisory>) -> Value {
        let tagged: Vec<(&str, &str, &str)> =
            locked.iter().map(|(n, v)| (*n, *v, "Cargo.lock")).collect();
        observed_multi(&tagged, advisories)
    }

    /// As [`observed`], but each locked entry carries its own source lockfile (the multi-workspace shape).
    fn observed_multi(locked: &[(&str, &str, &str)], advisories: Vec<Advisory>) -> Value {
        let hash = canonical_hash(&advisories);
        let count = advisories.len();
        json!({
            "locked": locked.iter().map(|(n, v, l)| json!({"name": n, "version": v, "lockfile": l})).collect::<Vec<_>>(),
            "advisories": serde_json::to_value(&advisories).unwrap(),
            "manifest": { "content_hash": hash, "advisory_count": count },
        })
    }

    fn vuln(id: &str, package: &str, patched: &[&str]) -> Advisory {
        Advisory {
            id: id.to_owned(),
            package: package.to_owned(),
            patched: patched.iter().map(|s| s.to_string()).collect(),
            unaffected: vec![],
            informational: None,
        }
    }

    fn unmaintained(id: &str, package: &str) -> Advisory {
        Advisory {
            id: id.to_owned(),
            package: package.to_owned(),
            patched: vec![],
            unaffected: vec![],
            informational: Some("unmaintained".to_owned()),
        }
    }

    fn codes(findings: &BTreeSet<Finding>) -> Vec<String> {
        findings.iter().map(|f| format!("{} {}", f.code, f.key)).collect()
    }

    #[test]
    fn quinn_proto_0_11_14_is_sca_vuln_keyed_to_the_exact_id() {
        let adv = vuln("RUSTSEC-2026-0185", "quinn-proto", &[">= 0.11.15"]);
        let obs = observed(&[("quinn-proto", "0.11.14")], vec![adv]);
        let findings = evaluate_keyed(&policy(), &obs);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "SCA-VULN" && f.key == "RUSTSEC-2026-0185"),
            "0.11.14 must be SCA-VULN keyed to RUSTSEC-2026-0185; got {:?}",
            codes(&findings)
        );
    }

    #[test]
    fn quinn_proto_0_11_15_is_clean() {
        let adv = vuln("RUSTSEC-2026-0185", "quinn-proto", &[">= 0.11.15"]);
        let obs = observed(&[("quinn-proto", "0.11.15")], vec![adv]);
        let findings = evaluate_keyed(&policy(), &obs);
        assert!(
            findings.is_empty(),
            "0.11.15 satisfies the patched range and must be clean; got {:?}",
            codes(&findings)
        );
    }

    #[test]
    fn unmaintained_absent_from_ignore_is_flagged_present_is_clean() {
        let adv = unmaintained("RUSTSEC-2024-0436", "paste");
        let obs = observed(&[("paste", "1.0.15")], vec![adv.clone()]);

        // Absent from ignore → SCA-UNMAINTAINED.
        let findings = evaluate_keyed(&policy(), &obs);
        assert!(
            findings.iter().any(|f| f.code == "SCA-UNMAINTAINED" && f.key == "RUSTSEC-2024-0436"),
            "unmaintained crate absent from ignore must be flagged; got {:?}",
            codes(&findings)
        );

        // Present in ignore → clean (no stale, because it IS live-affected).
        let mut p = policy();
        p["ignore"] = json!([{ "id": "RUSTSEC-2024-0436", "reason": "no drop-in", "remove_by": "2026-12-31" }]);
        let findings = evaluate_keyed(&p, &obs);
        assert!(
            findings.is_empty(),
            "an ignored, live-affected unmaintained crate must be clean; got {:?}",
            codes(&findings)
        );
    }

    #[test]
    fn unmaintained_policy_off_does_not_flag() {
        let adv = unmaintained("RUSTSEC-2024-0436", "paste");
        let obs = observed(&[("paste", "1.0.15")], vec![adv]);
        let mut p = policy();
        p["unmaintained_policy"] = json!("none");
        assert!(evaluate_keyed(&p, &obs).is_empty());
    }

    #[test]
    fn stale_ignore_is_self_clean_flagged() {
        // An ignore id whose crate is NOT in the lock → suppresses nothing → SCA-STALE-IGNORE.
        let adv = vuln("RUSTSEC-2099-0001", "absent-crate", &[">= 9.9.9"]);
        let obs = observed(&[("present-crate", "1.0.0")], vec![adv]);
        let mut p = policy();
        p["ignore"] = json!([{ "id": "RUSTSEC-2099-0001", "reason": "x", "remove_by": "2026-12-31" }]);
        let findings = evaluate_keyed(&p, &obs);
        assert!(
            findings.iter().any(|f| f.code == "SCA-STALE-IGNORE" && f.key == "RUSTSEC-2099-0001"),
            "a non-suppressing ignore must be SCA-STALE-IGNORE; got {:?}",
            codes(&findings)
        );
        // --write drops it (shrink-only).
        let (kept, dropped) = shrink_only_ignore(&p, &obs);
        assert!(kept.is_empty());
        assert_eq!(dropped, vec!["RUSTSEC-2099-0001".to_owned()]);
    }

    #[test]
    fn tampered_manifest_is_mirror_malformed() {
        let adv = vuln("RUSTSEC-2099-0001", "x", &[">= 1.0.0"]);
        let mut obs = observed(&[("x", "1.0.0")], vec![adv]);
        obs["manifest"]["content_hash"] = json!("deadbeefdeadbeef");
        let findings = evaluate_keyed(&policy(), &obs);
        assert!(
            findings.iter().any(|f| f.code == "SCA-MIRROR-MALFORMED"),
            "a tampered content_hash must be SCA-MIRROR-MALFORMED; got {:?}",
            codes(&findings)
        );
    }

    #[test]
    fn underflow_is_flagged() {
        let obs = observed(&[("x", "1.0.0")], vec![vuln("RUSTSEC-2099-0001", "y", &[">= 1.0.0"])]);
        let mut p = policy();
        p["min_advisories"] = json!(1000);
        let findings = evaluate_keyed(&p, &obs);
        assert!(
            findings.iter().any(|f| f.code == "SCA-MIRROR-UNDERFLOW"),
            "a below-floor mirror must be SCA-MIRROR-UNDERFLOW; got {:?}",
            codes(&findings)
        );
    }

    #[test]
    fn unparseable_locked_version_fails_closed() {
        // A locked version that does not parse as semver is treated as affected (fail-closed).
        let adv = vuln("RUSTSEC-2099-0001", "weird", &[">= 1.0.0"]);
        let obs = observed(&[("weird", "not-a-version")], vec![adv]);
        let findings = evaluate_keyed(&policy(), &obs);
        assert!(findings.iter().any(|f| f.code == "SCA-VULN"));
    }

    /// THE multi-lockfile regression: a crate SAFE in the root lock but AFFECTED in a carve-out lock
    /// must be flagged, and the finding must name the carve-out lock (and NOT the root lock).
    /// This is the exact shape of the blindness this gate had while `lockfile_path` was one string.
    #[test]
    fn affected_only_in_a_nested_lock_is_flagged_and_names_that_lockfile() {
        let adv = vuln("RUSTSEC-2026-0044", "aws-lc-sys", &[">= 0.39.0"]);
        let obs = observed_multi(
            &[
                ("aws-lc-sys", "0.41.0", "Cargo.lock"),
                ("aws-lc-sys", "0.37.0", "kernel/Cargo.lock"),
            ],
            vec![adv],
        );
        let findings = evaluate_keyed(&policy(), &obs);
        let vuln_finding = findings
            .iter()
            .find(|f| f.code == "SCA-VULN" && f.key == "RUSTSEC-2026-0044")
            .unwrap_or_else(|| panic!("nested-lock vuln must be flagged; got {:?}", codes(&findings)));
        assert!(
            vuln_finding.detail.contains("`kernel/Cargo.lock` 0.37.0"),
            "the finding must name the offending lockfile + version; got {}",
            vuln_finding.detail
        );
        assert!(
            !vuln_finding.detail.contains("`Cargo.lock` 0.41.0"),
            "the patched root-lock version is NOT an affected site; got {}",
            vuln_finding.detail
        );
    }

    #[test]
    fn lockfile_path_accepts_string_list_or_absent_and_fails_closed_otherwise() {
        assert_eq!(
            lockfile_paths(&json!({ "lockfile_path": "Cargo.lock" })).unwrap(),
            vec!["Cargo.lock".to_owned()]
        );
        assert_eq!(
            lockfile_paths(&json!({ "lockfile_path": ["a/Cargo.lock", "b/Cargo.lock"] })).unwrap(),
            vec!["a/Cargo.lock".to_owned(), "b/Cargo.lock".to_owned()]
        );
        assert_eq!(lockfile_paths(&json!({})).unwrap(), vec!["Cargo.lock".to_owned()]);
        // Fail-closed: an empty list would scan nothing and be vacuously green.
        assert!(lockfile_paths(&json!({ "lockfile_path": [] })).is_err());
        assert!(lockfile_paths(&json!({ "lockfile_path": [7] })).is_err());
        assert!(lockfile_paths(&json!({ "lockfile_path": 7 })).is_err());
    }

    #[test]
    fn gate_id_mismatch_is_flagged() {
        let mut p = policy();
        p["gate_id"] = json!("wrong");
        let obs = observed(&[], vec![]);
        assert!(
            evaluate_keyed(&p, &obs)
                .iter()
                .any(|f| f.code == "SCA-POLICY-GATE-ID-MISMATCH")
        );
    }
}
