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
//! ## Lockfile COVERAGE (the scan is only as good as the corpus it names)
//! The gate scans exactly the lockfiles `policy.lockfile_paths` names. That declaration used to be a
//! single `lockfile_path` string — "scan THE lockfile" — while the obligation is "scan EVERY
//! lockfile". A repository holds more than one `Cargo.lock` the moment it carries a nested workspace
//! (the ADR-0512 `kernel` / `cloud/cloud-kernel` carve-outs) or a stranded lockfile with no manifest
//! above it. Every crate pinned only in an unnamed lockfile was invisible: a `cargo update` there
//! could introduce an affected version and this gate would stay green, because it never looked.
//!
//! The fix that STAYS fixed is not a longer declaration — it is an assertion that the declaration
//! equals reality. [`discover_lockfiles`] DERIVES the corpus by walking the tree (a `Cargo.lock`
//! either exists on disk or it does not), and [`evaluate_keyed`] asserts set equality against
//! `policy.lockfile_paths` in BOTH directions. A workspace added tomorrow is `SCA-LOCKFILE-UNCOVERED`
//! until it is declared; a declaration whose file is gone is `SCA-LOCKFILE-ABSENT` until it is
//! dropped. The declaration therefore cannot silently drift out of agreement with the repository.
//!
//! Two rejected derivation sources, and why:
//! - **Root `Cargo.toml` `exclude`** — it answers "what is NOT a root-workspace member", not "where
//!   is a lockfile". MEASURED on this tree: `exclude` names 3 entries (2 of them workspaces) while 18
//!   `Cargo.lock` files are tracked; it under-reports the corpus by 16. It is also itself hand-kept,
//!   i.e. the exact defect being fixed one level up.
//! - **The git-tracked file set** — accurate, but obtaining it means spawning `git`, which this
//!   gate's no-shell/hermetic/buck2-cacheable contract forbids (see the module header above).
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `SCA-VULN` — a security advisory affects a locked crate and is not ignored.
//! - `SCA-UNMAINTAINED` — an unmaintained advisory affects a locked crate (policy=all) and is not ignored.
//! - `SCA-STALE-IGNORE` — an ignore id matches no live affected advisory (shrink-only self-clean).
//! - `SCA-LOCKFILE-UNCOVERED` — a `Cargo.lock` exists in the tree that `policy.lockfile_paths` does
//!   not name, so its pins are never scanned (the coverage defect).
//! - `SCA-LOCKFILE-ABSENT` — `policy.lockfile_paths` names a path with no file behind it (a stale
//!   declaration, or a derivation that under-reported — both fail closed).
//! - `SCA-MIRROR-MALFORMED` — the vendored mirror is corrupt / desynced (fail-closed).
//! - `SCA-MIRROR-UNDERFLOW` — the mirror has too few advisories (fail-closed against a false-green).
//! - `SCA-POLICY-GATE-ID-MISMATCH` — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `SCA-POLICY-MALFORMED` — the policy is structurally invalid (fail-closed).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeSet, VecDeque};
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
pub const VIOLATION_CODES: [&str; 9] = [
    "SCA-VULN",
    "SCA-UNMAINTAINED",
    "SCA-STALE-IGNORE",
    "SCA-LOCKFILE-UNCOVERED",
    "SCA-LOCKFILE-ABSENT",
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

/// Directory names the lockfile derivation never descends into: VCS metadata, build/dependency
/// output roots, and `.claude` — which is tracked but hosts the per-lane isolated worktrees the
/// operating contract mandates, i.e. full nested copies of this repository whose lockfiles belong to
/// a different checkout. Same list, same reason, as the affected-set gate's `CONSUMER_SCAN_SKIP_DIRS`.
///
/// Deliberately a CODE constant and not policy DATA: a policy-editable skip list would be a one-line
/// way to silence the coverage assertion, which is precisely the class of defect the assertion
/// exists to prevent. Any other directory that grows a `Cargo.lock` surfaces as
/// `SCA-LOCKFILE-UNCOVERED` — noisy, but the fail-closed direction.
const UNWALKED_DIRS: [&str; 5] = [".git", ".claude", "buck-out", "target", "node_modules"];

/// DERIVE the repository's actual lockfile corpus by walking the tree from `repo_root`.
///
/// This is the source the coverage assertion compares `policy.lockfile_paths` against, and it was
/// chosen because it cannot drift from reality: a `Cargo.lock` is either on disk or it is not. It
/// needs no registry, no manifest parsing, and no second hand-kept list. Returns repo-root-relative,
/// `/`-separated paths, sorted and deduped.
pub fn discover_lockfiles(repo_root: &Path) -> Result<Vec<String>, CollectError> {
    let mut found: Vec<String> = Vec::new();
    let mut queue = VecDeque::from([repo_root.to_path_buf()]);
    while let Some(dir) = queue.pop_front() {
        let entries = std::fs::read_dir(&dir)
            .map_err(|e| CollectError::Io(format!("read_dir {}: {e}", dir.display())))?;
        for entry in entries {
            let entry = entry.map_err(|e| {
                CollectError::Io(format!("read_dir entry under {}: {e}", dir.display()))
            })?;
            let path = entry.path();
            // `DirEntry::file_type` does not follow symlinks, so a symlinked directory is neither
            // dir nor file here and is not descended into: the walk cannot cycle, and it cannot
            // escape the repository. That is also the correct corpus answer — a lockfile "inside" a
            // symlinked directory really lives at the link target, which is either elsewhere in this
            // tree (and derived on its own) or outside it (and not this repository's to scan).
            let file_type = entry
                .file_type()
                .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
            let name = path.file_name().and_then(|n| n.to_str());
            if file_type.is_dir() {
                if !name.is_some_and(|n| UNWALKED_DIRS.contains(&n)) {
                    queue.push_back(path);
                }
            } else if name == Some(LOCKFILE_NAME)
                && let Some(rel) = repo_relative(repo_root, &path)
            {
                found.push(rel);
            }
        }
    }
    found.sort();
    found.dedup();
    Ok(found)
}

/// The file name that defines the corpus. A repository's lockfile set is exactly its `Cargo.lock`s.
const LOCKFILE_NAME: &str = "Cargo.lock";

fn repo_relative(repo_root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(repo_root).ok()?;
    let parts: Vec<&str> = rel.components().filter_map(|c| c.as_os_str().to_str()).collect();
    if parts.len() != rel.components().count() {
        return None; // non-UTF-8 component: cannot be compared against a JSON declaration.
    }
    Some(parts.join("/"))
}

/// The DECLARED lockfile corpus (`policy.lockfile_paths`), sorted and deduped. `None` when the field
/// is missing or is not an array of strings — a structurally invalid declaration fails closed rather
/// than silently degrading to "scan nothing" or "scan the root only".
pub fn configured_lockfiles(policy: &Value) -> Option<Vec<String>> {
    let arr = policy.get("lockfile_paths").and_then(Value::as_array)?;
    let mut out = Vec::with_capacity(arr.len());
    for entry in arr {
        out.push(entry.as_str()?.to_owned());
    }
    out.sort();
    out.dedup();
    Some(out)
}

/// Collect the observed graph: the locked crates from EVERY declared `Cargo.lock`, the DERIVED
/// lockfile corpus, and the vendored advisory snapshot.
///
/// The ONLY I/O. Reads each `policy.lockfile_paths` entry (TOML), walks the tree for the derived
/// corpus, and reads `policy.mirror_dir/{advisories.json,mirror-manifest.json}` (JSON). Emits
/// `{ "locked": [ { "name", "version" }, .. ], "configured_lockfiles": [..],
/// "discovered_lockfiles": [..], "advisories": [ <Advisory>, .. ], "manifest": {..} }`.
///
/// A declared path with no file behind it is NOT a hard error here: it is reported by the pure
/// evaluator as `SCA-LOCKFILE-ABSENT`, so the operator sees the named path instead of an opaque
/// exit-2 abort. A declared path that EXISTS but cannot be read or parsed stays fail-closed.
pub fn collect(repo_root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let configured = configured_lockfiles(policy).ok_or_else(|| {
        CollectError::Parse(
            "policy.lockfile_paths must be an array of repo-root-relative Cargo.lock path strings"
                .to_owned(),
        )
    })?;
    let mirror_dir = policy
        .get("mirror_dir")
        .and_then(Value::as_str)
        .ok_or_else(|| CollectError::Parse("policy.mirror_dir is required".to_owned()))?;

    let mut locked: Vec<Value> = Vec::new();
    for rel in &configured {
        let path = repo_root.join(rel);
        if !path.is_file() {
            continue; // reported as SCA-LOCKFILE-ABSENT by the pure evaluator.
        }
        locked.extend(parse_locked(&read_file(&path)?, rel)?);
    }
    locked.sort_by(|a, b| locked_sort_key(a).cmp(&locked_sort_key(b)));
    locked.dedup();

    let discovered = discover_lockfiles(repo_root)?;

    let advisories_text = read_file(&repo_root.join(mirror_dir).join("advisories.json"))?;
    let advisories: Value = serde_json::from_str(&advisories_text)
        .map_err(|e| CollectError::Parse(format!("advisories.json: {e}")))?;

    let manifest_text = read_file(&repo_root.join(mirror_dir).join("mirror-manifest.json"))?;
    let manifest: Value = serde_json::from_str(&manifest_text)
        .map_err(|e| CollectError::Parse(format!("mirror-manifest.json: {e}")))?;

    Ok(json!({
        "locked": locked,
        "configured_lockfiles": configured,
        "discovered_lockfiles": discovered,
        "advisories": advisories,
        "manifest": manifest,
    }))
}

fn read_file(path: &Path) -> Result<String, CollectError> {
    std::fs::read_to_string(path)
        .map_err(|e| CollectError::Io(format!("read {}: {e}", path.display())))
}

/// Parse the `[[package]]` table of ONE `Cargo.lock` into `[{ "name", "version" }]`. `origin` is the
/// repo-root-relative path, carried only so a parse failure names WHICH lockfile broke — with a
/// multi-lockfile corpus, "Cargo.lock: expected ..." is not a diagnosable message.
fn parse_locked(lock_text: &str, origin: &str) -> Result<Vec<Value>, CollectError> {
    let doc: toml::Value =
        toml::from_str(lock_text).map_err(|e| CollectError::Parse(format!("{origin}: {e}")))?;
    let packages = doc
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| CollectError::Parse(format!("{origin} has no [[package]] table")))?;
    let mut locked: Vec<Value> = Vec::with_capacity(packages.len());
    for pkg in packages {
        let Some(name) = pkg.get("name").and_then(toml::Value::as_str) else {
            continue;
        };
        let Some(version) = pkg.get("version").and_then(toml::Value::as_str) else {
            continue;
        };
        locked.push(json!({ "name": name, "version": version }));
    }
    Ok(locked)
}

fn locked_sort_key(v: &Value) -> (String, String) {
    (
        v.get("name").and_then(Value::as_str).unwrap_or("").to_owned(),
        v.get("version").and_then(Value::as_str).unwrap_or("").to_owned(),
    )
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

    // --- Lockfile COVERAGE: the DECLARED corpus must equal the DERIVED one. ---
    //
    // Runs BEFORE every early return below, because a coverage hole is exactly the failure that
    // must not be masked by an unrelated policy/mirror fault. The gate scans only what
    // `policy.lockfile_paths` names, so a `Cargo.lock` the declaration omits is a workspace whose
    // pins are never checked — a later `cargo update` there can introduce an affected version with
    // this gate still green. `observed.discovered_lockfiles` is DERIVED by walking the tree
    // (`discover_lockfiles`), never read from a second hand-kept list: a hand-kept mirror of the
    // declaration would reproduce the very defect this assertion closes.
    //
    // A missing/non-array `discovered_lockfiles` degrades to the EMPTY derived set, which turns
    // every declared path into `SCA-LOCKFILE-ABSENT` — RED. That is the intended fail-closed
    // direction: an observation that lost the derivation must never read as "coverage is fine".
    match configured_lockfiles(policy) {
        None => {
            findings.insert(Finding::new(
                "SCA-POLICY-MALFORMED",
                POLICY_KEY,
                "policy `lockfile_paths` must be an array of repo-root-relative `Cargo.lock` path strings naming EVERY lockfile in the repository; a single-path declaration cannot express the corpus and leaves nested workspaces unscanned",
            ));
        }
        Some(configured) => {
            let derived: BTreeSet<&str> = observed
                .get("discovered_lockfiles")
                .and_then(Value::as_array)
                .map(|arr| arr.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let declared: BTreeSet<&str> = configured.iter().map(String::as_str).collect();
            for path in derived.difference(&declared) {
                findings.insert(Finding::new(
                    "SCA-LOCKFILE-UNCOVERED",
                    path,
                    format!(
                        "`{path}` is a Cargo.lock in this repository that policy.lockfile_paths does not name, so none of its pinned crates are matched against the advisory mirror — a `cargo update` in that workspace can introduce an affected version while this gate stays green. Add `{path}` to policy.lockfile_paths (or delete the lockfile if it is stranded); the declared set is asserted equal to the set derived from the tree, so it cannot be left out silently."
                    ),
                ));
            }
            for path in declared.difference(&derived) {
                findings.insert(Finding::new(
                    "SCA-LOCKFILE-ABSENT",
                    path,
                    format!(
                        "policy.lockfile_paths names `{path}`, but no such file was derived from the tree — either the lockfile was removed and the declaration was not shrunk with it, or the derivation under-reported (a walk that finds nothing must not read as full coverage). Remove the entry in the same change that removed the file."
                    ),
                ));
            }
        }
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
        let affected = locked.iter().any(|pkg| {
            pkg.get("name").and_then(Value::as_str) == Some(advisory.package.as_str())
                && pkg
                    .get("version")
                    .and_then(Value::as_str)
                    .is_some_and(|v| version_affected(v, advisory))
        });
        if !affected {
            continue;
        }

        match advisory.informational.as_deref() {
            None => {
                // A security vulnerability.
                live_affected_ids.insert(advisory.id.clone());
                if !ignore_ids.contains(advisory.id.as_str()) {
                    findings.insert(Finding::new(
                        "SCA-VULN",
                        &advisory.id,
                        format!(
                            "security advisory {} affects locked crate `{}` (no installed version satisfies patched {:?} / unaffected {:?}). {REMEDIATION_DOCTRINE}.",
                            advisory.id, advisory.package, advisory.patched, advisory.unaffected
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
                                "unmaintained advisory {} affects locked crate `{}` (unmaintained_policy=all). Migrate off the crate, or {REMEDIATION_DOCTRINE}.",
                                advisory.id, advisory.package
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
        return "supply-chain-audit gate passed: every Cargo.lock in the tree is declared in policy.lockfile_paths, no locked crate in any of them is affected by an un-ignored RustSec advisory, and the vendored mirror is intact".to_owned();
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

    /// An EMPTY declared corpus, paired with the empty derived corpus [`observed`] reports, so the
    /// coverage assertion is neutral and these cases isolate the advisory-matching logic. The
    /// coverage assertion has its own cases below.
    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "lockfile_paths": [],
            "mirror_dir": "x",
            "unmaintained_policy": "all",
            "min_advisories": 0,
            "ignore": []
        })
    }

    /// Build an observed graph from synthetic locked crates + advisories, with a CONSISTENT manifest
    /// (correct content_hash + count) so the integrity checks pass and the matching logic is isolated.
    fn observed(locked: &[(&str, &str)], advisories: Vec<Advisory>) -> Value {
        observed_with_lockfiles(locked, advisories, &[])
    }

    fn observed_with_lockfiles(
        locked: &[(&str, &str)],
        advisories: Vec<Advisory>,
        discovered: &[&str],
    ) -> Value {
        let hash = canonical_hash(&advisories);
        let count = advisories.len();
        json!({
            "locked": locked.iter().map(|(n, v)| json!({"name": n, "version": v})).collect::<Vec<_>>(),
            "discovered_lockfiles": discovered,
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

    #[test]
    fn a_lockfile_the_declaration_omits_is_uncovered() {
        // The defect this assertion closes: a nested workspace exists in the tree, the declaration
        // names only the root, and every crate pinned ONLY in the nested lock is unscanned.
        let mut p = policy();
        p["lockfile_paths"] = json!(["Cargo.lock"]);
        let obs = observed_with_lockfiles(
            &[],
            vec![],
            &["Cargo.lock", "kernel/Cargo.lock", "fourth/Cargo.lock"],
        );
        let findings = evaluate_keyed(&p, &obs);
        let uncovered: Vec<&str> = findings
            .iter()
            .filter(|f| f.code == "SCA-LOCKFILE-UNCOVERED")
            .map(|f| f.key.as_str())
            .collect();
        assert_eq!(
            uncovered,
            vec!["fourth/Cargo.lock", "kernel/Cargo.lock"],
            "every derived lockfile the declaration omits must be SCA-LOCKFILE-UNCOVERED, keyed to \
             its path; got {:?}",
            codes(&findings)
        );
    }

    #[test]
    fn a_declaration_matching_the_derived_corpus_is_clean() {
        let mut p = policy();
        p["lockfile_paths"] = json!(["kernel/Cargo.lock", "Cargo.lock"]); // order must not matter
        let obs = observed_with_lockfiles(&[], vec![], &["Cargo.lock", "kernel/Cargo.lock"]);
        assert!(
            evaluate_keyed(&p, &obs).is_empty(),
            "a declaration equal to the derived corpus must be clean"
        );
    }

    #[test]
    fn a_declared_path_with_no_file_is_absent() {
        let mut p = policy();
        p["lockfile_paths"] = json!(["Cargo.lock", "deleted/Cargo.lock"]);
        let obs = observed_with_lockfiles(&[], vec![], &["Cargo.lock"]);
        let findings = evaluate_keyed(&p, &obs);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "SCA-LOCKFILE-ABSENT" && f.key == "deleted/Cargo.lock"),
            "a declared path with no derived file must be SCA-LOCKFILE-ABSENT; got {:?}",
            codes(&findings)
        );
    }

    #[test]
    fn a_lost_derivation_fails_closed_rather_than_reading_as_covered() {
        // If the walk is absent from the observation, the derived set is empty. It must NOT read as
        // "the declaration is fine" — every declared path REDs as absent.
        let mut p = policy();
        p["lockfile_paths"] = json!(["Cargo.lock"]);
        let obs = json!({ "locked": [], "advisories": [], "manifest": {} });
        let findings = evaluate_keyed(&p, &obs);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "SCA-LOCKFILE-ABSENT" && f.key == "Cargo.lock"),
            "a missing derivation must fail closed; got {:?}",
            codes(&findings)
        );
    }

    #[test]
    fn a_single_string_lockfile_path_is_policy_malformed() {
        // The pre-fix shape (`"lockfile_path": "Cargo.lock"`) cannot express a corpus. It must be
        // rejected outright, not silently honoured as a one-element set.
        let mut p = policy();
        p.as_object_mut().unwrap().remove("lockfile_paths");
        p["lockfile_path"] = json!("Cargo.lock");
        let findings = evaluate_keyed(&p, &observed(&[], vec![]));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "SCA-POLICY-MALFORMED" && f.key == POLICY_KEY),
            "a single-string lockfile declaration must be SCA-POLICY-MALFORMED; got {:?}",
            codes(&findings)
        );
    }

    #[test]
    fn coverage_is_reported_even_when_the_mirror_is_malformed() {
        // A coverage hole must not be masked by an unrelated fault that early-returns.
        let mut p = policy();
        p["lockfile_paths"] = json!([]);
        let obs = json!({
            "locked": [],
            "discovered_lockfiles": ["kernel/Cargo.lock"],
            "advisories": "not-an-array",
            "manifest": {},
        });
        let findings = evaluate_keyed(&p, &obs);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "SCA-LOCKFILE-UNCOVERED" && f.key == "kernel/Cargo.lock"),
            "coverage must be evaluated before the mirror early-return; got {:?}",
            codes(&findings)
        );
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
