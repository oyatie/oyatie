//! # cloud-ci-supply-chain-audit (owned RustSec advisory gate; replaces reverted #974 shell scan)
//!
//! PR #974 added a born-blocking supply-chain gate by shelling out to `cargo-audit` / `cargo-deny`
//! — shell + a network-fetching advisory index, both forbidden by the no-shell + hermetic-gate +
//! rust-purity doctrine. #977 reverted it (keeping the quinn-proto CVE fix). This gate is the owned
//! replacement: a PURE, hermetic predicate that matches the authoritative lockfile corpus against a
//! VENDORED, content-addressed RustSec advisory snapshot. No shell, no network, no clock, no
//! `rustsec`/`git2`/`libgit2` crates — only `serde_json` + `toml` + `semver` (the last promoted to a
//! direct workspace dep via one reindeer buckify, adding ZERO new crate to `Cargo.lock`).
//!
//! ## Split of concerns (hermetic gate vs network reconciler)
//! The network/clock half — pinning a fresh advisory-db commit, distilling, opening a GitOps PR,
//! and `remove_by` expiry SLOs — lives in a SEPARATE owned reconciler (deferred Slice D). THIS gate
//! only reads candidate-tree bytes: the separately materialized SCM tracked-path snapshot, the
//! configured `Cargo.lock` files, and the vendored
//! `advisory-mirror/{advisories.json,mirror-manifest.json}` produced by
//! `oya-advisory-mirror-kernel`. That keeps the gate buck2-cacheable and deterministic.
//!
//! ## Lockfile corpus
//! `policy.lockfile_corpus` is the reviewed authority: each entry pairs one repo-relative
//! `Cargo.toml` workspace root with its sibling `Cargo.lock`. Before reading packages, collection
//! projects every workspace-owned lock from the independently materialized
//! `policy.scm_facts_path#tracked_paths` universe: a tracked lock is workspace-owned exactly when
//! its tracked sibling manifest declares `[workspace]`. That projection must equal the policy
//! corpus. A newly tracked workspace root therefore fails until declared, while orphan locks and
//! member-local locks cannot expand the scan. Collection performs no tree walk, so ignored files,
//! nested worktrees, build products, and directory iteration order cannot change the result. Every
//! configured component is checked with `symlink_metadata`; absolute paths, `..`, symlinks, missing
//! files, non-files, duplicate entries, and manifest/lockfile parent mismatch fail closed.
//! `policy.min_lockfiles` remains a defense-in-depth shrink floor. The legacy single
//! `policy.lockfile_path` form remains accepted as a one-entry corpus without the SCM projection.
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
//! - `SCA-LOCKFILE-UNDERFLOW` — the configured corpus is smaller than `policy.min_lockfiles`.
//! - `SCA-MIRROR-MALFORMED` — the vendored mirror is corrupt / desynced (fail-closed).
//! - `SCA-MIRROR-UNDERFLOW` — the mirror has too few advisories (fail-closed against a false-green).
//! - `SCA-POLICY-GATE-ID-MISMATCH` — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `SCA-POLICY-MALFORMED` — the policy is structurally invalid (fail-closed).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use advisory_mirror_kernel::{Advisory, canonical_hash};
use semver::{Version, VersionReq};
use serde_json::{Value, json};

/// The gate id, matching the buck2 target stem + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-supply-chain-audit";

/// Stable tracked-tree boundary schema emitted by the out-of-graph SCM facts producer.
const SCM_FACTS_SCHEMA: &str = "oya-ci/scm-facts/v2";

/// The remediation doctrine pointer findings carry.
pub const REMEDIATION_DOCTRINE: &str = "upgrade the affected crate to a patched version (bump the workspace pin / transitive dep), or — \
     for an unmaintained dep with no maintained drop-in — add a time-boxed policy.ignore[] entry with \
     a reason, pull_chain, and remove_by date";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 8] = [
    "SCA-VULN",
    "SCA-UNMAINTAINED",
    "SCA-STALE-IGNORE",
    "SCA-LOCKFILE-UNDERFLOW",
    "SCA-MIRROR-MALFORMED",
    "SCA-MIRROR-UNDERFLOW",
    "SCA-POLICY-GATE-ID-MISMATCH",
    "SCA-POLICY-MALFORMED",
];

/// The sentinel key for codes that are mirror/policy-level rather than per-advisory.
const MIRROR_KEY: &str = "<mirror>";
const POLICY_KEY: &str = "<policy>";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LockfilePackageRow {
    source: String,
    name: String,
    version: String,
}

impl LockfilePackageRow {
    fn as_legacy_record(&self) -> Value {
        json!({
            "name": self.name,
            "version": self.version,
        })
    }

    fn as_provenance_record(&self) -> Value {
        json!({
            "name": self.name,
            "version": self.version,
            "lockfile": self.source,
        })
    }

    fn finding_key(&self, advisory_id: &str) -> String {
        format!("{advisory_id}::{}/{}", self.source, self.version)
    }
}

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

/// One authoritative workspace/package manifest and its sibling lockfile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockfileSource {
    /// Repo-relative `Cargo.toml` that declares `[workspace]` or `[package]`.
    pub manifest_path: String,
    /// Repo-relative sibling `Cargo.lock` scanned for advisory matches.
    pub lockfile_path: String,
}

/// Parse and validate the policy's deterministic lockfile corpus.
///
/// The structured `lockfile_corpus` form is authoritative for multi-workspace repositories. The
/// legacy `lockfile_path` string remains supported as a one-entry corpus and defaults to
/// `Cargo.lock`, preserving the original policy behavior.
///
/// # Errors
///
/// Returns [`CollectError::Parse`] for malformed, ambiguous, duplicate, non-relative, non-canonical,
/// or non-sibling manifest/lockfile declarations.
pub fn configured_lockfiles(policy: &Value) -> Result<Vec<LockfileSource>, CollectError> {
    let mut sources = if let Some(corpus) = policy.get("lockfile_corpus") {
        if policy.get("lockfile_path").is_some() {
            return Err(CollectError::Parse(
                "policy must use either lockfile_corpus or legacy lockfile_path, not both"
                    .to_owned(),
            ));
        }
        let entries = corpus.as_array().ok_or_else(|| {
            CollectError::Parse("policy.lockfile_corpus must be a non-empty array".to_owned())
        })?;
        if entries.is_empty() {
            return Err(CollectError::Parse(
                "policy.lockfile_corpus must not be empty".to_owned(),
            ));
        }
        minimum_lockfiles(policy, true)?;
        scm_facts_path(policy)?;

        let mut parsed = Vec::with_capacity(entries.len());
        for (index, entry) in entries.iter().enumerate() {
            let object = entry.as_object().ok_or_else(|| {
                CollectError::Parse(format!("policy.lockfile_corpus[{index}] must be an object"))
            })?;
            if object.len() != 2 {
                return Err(CollectError::Parse(format!(
                    "policy.lockfile_corpus[{index}] must contain exactly manifest_path and lockfile_path"
                )));
            }
            let manifest_path = object
                .get("manifest_path")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    CollectError::Parse(format!(
                        "policy.lockfile_corpus[{index}].manifest_path must be a string"
                    ))
                })?;
            let lockfile_path = object
                .get("lockfile_path")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    CollectError::Parse(format!(
                        "policy.lockfile_corpus[{index}].lockfile_path must be a string"
                    ))
                })?;
            parsed.push(validate_source(manifest_path, lockfile_path)?);
        }
        parsed
    } else {
        let lockfile_path = policy
            .get("lockfile_path")
            .map(|value| {
                value.as_str().ok_or_else(|| {
                    CollectError::Parse("policy.lockfile_path must be a string".to_owned())
                })
            })
            .transpose()?
            .unwrap_or("Cargo.lock");
        let lock_path = validate_relative_path(lockfile_path, "Cargo.lock")?;
        let manifest_path = lock_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join("Cargo.toml")
            .to_string_lossy()
            .replace('\\', "/");
        vec![validate_source(&manifest_path, lockfile_path)?]
    };

    let mut manifests = BTreeSet::new();
    let mut lockfiles = BTreeSet::new();
    for source in &sources {
        if !manifests.insert(source.manifest_path.as_str()) {
            return Err(CollectError::Parse(format!(
                "duplicate manifest_path `{}` in policy lockfile corpus",
                source.manifest_path
            )));
        }
        if !lockfiles.insert(source.lockfile_path.as_str()) {
            return Err(CollectError::Parse(format!(
                "duplicate lockfile_path `{}` in policy lockfile corpus",
                source.lockfile_path
            )));
        }
    }
    sources.sort_by(|a, b| {
        a.lockfile_path
            .cmp(&b.lockfile_path)
            .then_with(|| a.manifest_path.cmp(&b.manifest_path))
    });
    Ok(sources)
}

fn validate_source(
    manifest_path: &str,
    lockfile_path: &str,
) -> Result<LockfileSource, CollectError> {
    let manifest = validate_relative_path(manifest_path, "Cargo.toml")?;
    let lockfile = validate_relative_path(lockfile_path, "Cargo.lock")?;
    if manifest.parent() != lockfile.parent() {
        return Err(CollectError::Parse(format!(
            "manifest `{manifest_path}` and lockfile `{lockfile_path}` must be siblings"
        )));
    }
    Ok(LockfileSource {
        manifest_path: manifest_path.to_owned(),
        lockfile_path: lockfile_path.to_owned(),
    })
}

fn validate_relative_path(raw: &str, expected_file_name: &str) -> Result<PathBuf, CollectError> {
    if raw.is_empty() || raw.contains('\\') {
        return Err(CollectError::Parse(format!(
            "path `{raw}` must be a non-empty repo-relative `/`-separated path"
        )));
    }
    let path = Path::new(raw);
    if path.file_name().and_then(|name| name.to_str()) != Some(expected_file_name)
        || normalized_relative(path).as_deref() != Some(raw)
    {
        return Err(CollectError::Parse(format!(
            "path `{raw}` must be a normalized repo-relative path ending in {expected_file_name}"
        )));
    }
    Ok(path.to_path_buf())
}

fn minimum_lockfiles(policy: &Value, structured: bool) -> Result<usize, CollectError> {
    match policy.get("min_lockfiles") {
        Some(value) => {
            let floor = value.as_u64().ok_or_else(|| {
                CollectError::Parse(
                    "policy.min_lockfiles must be a non-negative integer".to_owned(),
                )
            })?;
            let floor = usize::try_from(floor).map_err(|_| {
                CollectError::Parse("policy.min_lockfiles exceeds platform capacity".to_owned())
            })?;
            if structured && floor == 0 {
                return Err(CollectError::Parse(
                    "policy.min_lockfiles must be at least 1 with policy.lockfile_corpus"
                        .to_owned(),
                ));
            }
            Ok(floor)
        }
        None if structured => Err(CollectError::Parse(
            "policy.min_lockfiles is required with policy.lockfile_corpus".to_owned(),
        )),
        None => Ok(1),
    }
}

/// Collect the observed graph from the configured lockfile corpus and vendored advisory snapshot.
///
/// The ONLY I/O. For structured policies, reads `policy.scm_facts_path` (JSON) and the tracked
/// sibling manifests needed to project workspace ownership. Then reads the configured
/// lockfiles/manifests (TOML) and `policy.mirror_dir/{advisories.json,mirror-manifest.json}` (JSON).
/// Emits the backward-compatible
/// `{ "locked": [ { "name", "version" }, .. ],
///   "locked_by_source": [ { "name", "version", "lockfile" }, .. ],
///   "advisories": [ <Advisory>, .. ], "manifest": {..} }`.
/// No directory walk occurs.
pub fn collect(repo_root: &Path, policy: &Value) -> Result<Value, CollectError> {
    validate_repo_root(repo_root)?;
    let sources = configured_lockfiles(policy)?;
    if policy.get("lockfile_corpus").is_some() {
        validate_lockfile_corpus_totality(repo_root, policy, &sources)?;
    }
    let mirror_dir = policy
        .get("mirror_dir")
        .and_then(Value::as_str)
        .ok_or_else(|| CollectError::Parse("policy.mirror_dir is required".to_owned()))?;
    let mirror_dir = validate_relative_directory(mirror_dir)?;

    let mut locked = Vec::<LockfilePackageRow>::new();
    for source in &sources {
        let manifest_path = Path::new(&source.manifest_path);
        let manifest_text = read_repo_file(repo_root, manifest_path)?;
        validate_workspace_manifest(&manifest_text, &source.manifest_path)?;
        let lock_text = read_repo_file(repo_root, Path::new(&source.lockfile_path))?;
        locked.extend(parse_locked(&lock_text, &source.lockfile_path)?);
    }
    let mut locked_by_source = locked.clone();
    locked_by_source.sort();
    let locked_by_source = locked_by_source
        .into_iter()
        .map(|row| row.as_provenance_record())
        .collect::<Vec<_>>();

    locked.sort_by_key(locked_sort_key);
    locked.dedup_by_key(|row| (row.name.clone(), row.version.clone()));
    let locked = locked
        .into_iter()
        .map(|row| row.as_legacy_record())
        .collect::<Vec<_>>();

    let advisories_path = mirror_dir.join("advisories.json");
    let advisories_text = read_repo_file(repo_root, &advisories_path)?;
    let advisories: Value = serde_json::from_str(&advisories_text)
        .map_err(|e| CollectError::Parse(format!("{}: {e}", advisories_path.display())))?;

    let mirror_manifest_path = mirror_dir.join("mirror-manifest.json");
    let manifest_text = read_repo_file(repo_root, &mirror_manifest_path)?;
    let manifest: Value = serde_json::from_str(&manifest_text)
        .map_err(|e| CollectError::Parse(format!("{}: {e}", mirror_manifest_path.display())))?;

    Ok(json!({
        "locked": locked,
        "locked_by_source": locked_by_source,
        "advisories": advisories,
        "manifest": manifest,
    }))
}

fn scm_facts_path(policy: &Value) -> Result<PathBuf, CollectError> {
    let raw = policy
        .get("scm_facts_path")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            CollectError::Parse(
                "policy.scm_facts_path is required with policy.lockfile_corpus".to_owned(),
            )
        })?;
    validate_relative_file(raw, "policy.scm_facts_path")
}

fn validate_relative_file(raw: &str, label: &str) -> Result<PathBuf, CollectError> {
    let path = Path::new(raw);
    if raw.is_empty() || raw.contains('\\') || normalized_relative(path).as_deref() != Some(raw) {
        return Err(CollectError::Parse(format!(
            "{label} `{raw}` must be a normalized repo-relative `/`-separated file path"
        )));
    }
    Ok(path.to_path_buf())
}

/// Compare the reviewed lockfile corpus with the independently materialized tracked-tree topology.
///
/// The SCM facts face supplies the exact tracked path universe without a runtime filesystem walk.
/// Within that universe, `[workspace]` is the explicit ownership marker for a sibling `Cargo.lock`.
/// Package-only member locks and locks without a tracked sibling manifest are deliberately excluded.
fn validate_lockfile_corpus_totality(
    repo_root: &Path,
    policy: &Value,
    declared_sources: &[LockfileSource],
) -> Result<(), CollectError> {
    let facts_path = scm_facts_path(policy)?;
    let facts_text = read_repo_file(repo_root, &facts_path)?;
    let facts: Value = serde_json::from_str(&facts_text)
        .map_err(|error| CollectError::Parse(format!("{}: {error}", facts_path.display())))?;
    let schema = facts.get("schema").and_then(Value::as_str).ok_or_else(|| {
        CollectError::Parse(format!("{}: missing string schema", facts_path.display()))
    })?;
    if schema != SCM_FACTS_SCHEMA {
        return Err(CollectError::Parse(format!(
            "{}: unsupported scm-facts schema {schema:?}; expected {SCM_FACTS_SCHEMA}",
            facts_path.display()
        )));
    }
    let tracked_values = facts
        .get("tracked_paths")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CollectError::Parse(format!(
                "{}: missing tracked_paths array",
                facts_path.display()
            ))
        })?;

    let mut tracked_paths = BTreeSet::new();
    let mut previous: Option<&str> = None;
    for (index, value) in tracked_values.iter().enumerate() {
        let raw = value.as_str().ok_or_else(|| {
            CollectError::Parse(format!(
                "{}: tracked_paths[{index}] must be a string",
                facts_path.display()
            ))
        })?;
        if previous.is_some_and(|prior| prior >= raw) {
            return Err(CollectError::Parse(format!(
                "{}: tracked_paths must be strictly sorted and unique; entry {index} is {raw:?}",
                facts_path.display()
            )));
        }
        previous = Some(raw);
        tracked_paths.insert(raw.to_owned());
    }

    let mut projected_sources = BTreeSet::new();
    for lockfile_path in tracked_paths
        .iter()
        .filter(|path| path.as_str() == "Cargo.lock" || path.ends_with("/Cargo.lock"))
    {
        let lockfile = validate_relative_path(lockfile_path, "Cargo.lock")?;
        let manifest_path = lockfile
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join("Cargo.toml")
            .to_string_lossy()
            .replace('\\', "/");
        if !tracked_paths.contains(&manifest_path) {
            continue;
        }

        let manifest_text = read_repo_file(repo_root, Path::new(&manifest_path))?;
        let manifest: toml::Value = toml::from_str(&manifest_text)
            .map_err(|error| CollectError::Parse(format!("{manifest_path}: {error}")))?;
        if !manifest.get("workspace").is_some_and(toml::Value::is_table)
            && !manifest.get("package").is_some_and(toml::Value::is_table)
        {
            return Err(CollectError::Parse(format!(
                "{manifest_path} must declare [workspace] or [package] to classify its tracked Cargo.lock"
            )));
        }
        match manifest.get("workspace") {
            Some(value) if value.is_table() => {
                projected_sources.insert((manifest_path, lockfile_path.clone()));
            }
            Some(_) => {
                return Err(CollectError::Parse(format!(
                    "{manifest_path}: top-level workspace must be a table"
                )));
            }
            None => {
                // A tracked package/member-local Cargo.lock is not a workspace-owned lockfile.
            }
        }
    }

    let declared_sources = declared_sources
        .iter()
        .map(|source| (source.manifest_path.clone(), source.lockfile_path.clone()))
        .collect::<BTreeSet<_>>();
    if projected_sources == declared_sources {
        return Ok(());
    }

    let undeclared = projected_sources
        .difference(&declared_sources)
        .map(|(_, lockfile)| lockfile.as_str())
        .collect::<Vec<_>>();
    let not_workspace_owned = declared_sources
        .difference(&projected_sources)
        .map(|(_, lockfile)| lockfile.as_str())
        .collect::<Vec<_>>();
    Err(CollectError::Parse(format!(
        "lockfile corpus totality mismatch against {}: undeclared workspace-owned lockfiles={undeclared:?}; declared paths absent from the workspace-owned projection={not_workspace_owned:?}",
        facts_path.display()
    )))
}

fn validate_relative_directory(raw: &str) -> Result<PathBuf, CollectError> {
    let path = Path::new(raw);
    if raw.is_empty() || raw.contains('\\') || normalized_relative(path).as_deref() != Some(raw) {
        return Err(CollectError::Parse(format!(
            "directory `{raw}` must be a normalized repo-relative `/`-separated path"
        )));
    }
    Ok(path.to_path_buf())
}

fn normalized_relative(path: &Path) -> Option<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            return None;
        };
        parts.push(component.to_str()?);
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("/"))
}

fn validate_repo_root(repo_root: &Path) -> Result<(), CollectError> {
    let metadata = std::fs::symlink_metadata(repo_root)
        .map_err(|e| CollectError::Io(format!("metadata {}: {e}", repo_root.display())))?;
    if metadata.file_type().is_symlink() {
        return Err(CollectError::Parse(format!(
            "repo root {} must not be a symlink",
            repo_root.display()
        )));
    }
    if !metadata.is_dir() {
        return Err(CollectError::Io(format!(
            "repo root {} is not a directory",
            repo_root.display()
        )));
    }
    Ok(())
}

fn read_repo_file(repo_root: &Path, relative: &Path) -> Result<String, CollectError> {
    let mut current = repo_root.to_path_buf();
    let component_count = relative.components().count();
    for (index, component) in relative.components().enumerate() {
        let Component::Normal(component) = component else {
            return Err(CollectError::Parse(format!(
                "path {} is not normalized and repo-relative",
                relative.display()
            )));
        };
        current.push(component);
        let metadata = std::fs::symlink_metadata(&current)
            .map_err(|e| CollectError::Io(format!("metadata {}: {e}", current.display())))?;
        if metadata.file_type().is_symlink() {
            return Err(CollectError::Parse(format!(
                "configured path {} contains symlink {}",
                relative.display(),
                current.display()
            )));
        }
        let is_last = index + 1 == component_count;
        if is_last && !metadata.is_file() {
            return Err(CollectError::Io(format!(
                "configured path {} is not a regular file",
                current.display()
            )));
        }
        if !is_last && !metadata.is_dir() {
            return Err(CollectError::Io(format!(
                "configured path component {} is not a directory",
                current.display()
            )));
        }
    }
    std::fs::read_to_string(&current)
        .map_err(|e| CollectError::Io(format!("read {}: {e}", current.display())))
}

fn validate_workspace_manifest(text: &str, source: &str) -> Result<(), CollectError> {
    let doc: toml::Value =
        toml::from_str(text).map_err(|e| CollectError::Parse(format!("{source}: {e}")))?;
    if !doc.get("workspace").is_some_and(toml::Value::is_table)
        && !doc.get("package").is_some_and(toml::Value::is_table)
    {
        return Err(CollectError::Parse(format!(
            "{source} must declare [workspace] or [package] to own a Cargo.lock"
        )));
    }
    Ok(())
}

/// Parse one `Cargo.lock`'s `[[package]]` tables; [`collect`] sorts and deduplicates the union.
fn parse_locked(lock_text: &str, source: &str) -> Result<Vec<LockfilePackageRow>, CollectError> {
    let doc: toml::Value =
        toml::from_str(lock_text).map_err(|e| CollectError::Parse(format!("{source}: {e}")))?;
    let packages = doc
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| CollectError::Parse(format!("{source} has no [[package]] table")))?;
    if packages.is_empty() {
        return Err(CollectError::Parse(format!(
            "{source} contains zero [[package]] rows"
        )));
    }
    let mut locked: Vec<LockfilePackageRow> = Vec::with_capacity(packages.len());
    for (index, pkg) in packages.iter().enumerate() {
        let package = pkg.as_table().ok_or_else(|| {
            CollectError::Parse(format!("{source} package[{index}] must be a table"))
        })?;
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .filter(|name| !name.is_empty())
            .ok_or_else(|| {
                CollectError::Parse(format!(
                    "{source} package[{index}].name must be a non-empty string"
                ))
            })?;
        let version = package
            .get("version")
            .and_then(toml::Value::as_str)
            .filter(|version| !version.is_empty())
            .ok_or_else(|| {
                CollectError::Parse(format!(
                    "{source} package[{index}].version must be a non-empty string"
                ))
            })?;
        locked.push(LockfilePackageRow {
            source: source.to_owned(),
            name: name.to_owned(),
            version: version.to_owned(),
        });
    }
    Ok(locked)
}

fn locked_sort_key(row: &LockfilePackageRow) -> (String, String) {
    (row.name.clone(), row.version.clone())
}

fn observed_locked_rows(policy: &Value, observed: &Value) -> (Vec<LockfilePackageRow>, bool) {
    if let Some(rows) = observed.get("locked_by_source").and_then(Value::as_array) {
        let mut observed_rows = Vec::with_capacity(rows.len());
        let mut sources = BTreeSet::new();
        for row in rows {
            let Some(row) = row.as_object() else {
                continue;
            };
            let name = row.get("name").and_then(Value::as_str).unwrap_or("");
            let version = row.get("version").and_then(Value::as_str).unwrap_or("");
            let source = row.get("lockfile").and_then(Value::as_str).unwrap_or("");
            if name.is_empty() || version.is_empty() || source.is_empty() {
                continue;
            }
            sources.insert(source.to_owned());
            observed_rows.push(LockfilePackageRow {
                source: source.to_owned(),
                name: name.to_owned(),
                version: version.to_owned(),
            });
        }
        if !observed_rows.is_empty() {
            return (
                observed_rows,
                if policy
                    .get("lockfile_corpus")
                    .and_then(Value::as_array)
                    .is_some()
                {
                    sources.len() > 1
                } else if policy
                    .get("lockfile_path")
                    .and_then(Value::as_str)
                    .is_none_or(|value| value != "Cargo.lock")
                {
                    // Explicit legacy override means the configured file is authoritative and
                    // historically keyed by advisory id only.
                    false
                } else {
                    sources.len() > 1
                },
            );
        }
    }

    let source = policy
        .get("lockfile_path")
        .and_then(Value::as_str)
        .unwrap_or("Cargo.lock");
    let observed_rows = observed
        .get("locked")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| {
                    Some(LockfilePackageRow {
                        source: source.to_owned(),
                        name: row.get("name")?.as_str()?.to_owned(),
                        version: row.get("version")?.as_str()?.to_owned(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    (observed_rows, false)
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
        let violations = findings
            .iter()
            .map(|f| f.code.clone())
            .collect::<BTreeSet<_>>();
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

fn lockfile_policy_finding(policy: &Value) -> Option<Finding> {
    let sources = match configured_lockfiles(policy) {
        Ok(sources) => sources,
        Err(error) => {
            return Some(Finding::new(
                "SCA-POLICY-MALFORMED",
                POLICY_KEY,
                error.to_string(),
            ));
        }
    };
    let floor = match minimum_lockfiles(policy, policy.get("lockfile_corpus").is_some()) {
        Ok(floor) => floor,
        Err(error) => {
            return Some(Finding::new(
                "SCA-POLICY-MALFORMED",
                POLICY_KEY,
                error.to_string(),
            ));
        }
    };
    (sources.len() < floor).then(|| {
        Finding::new(
            "SCA-LOCKFILE-UNDERFLOW",
            POLICY_KEY,
            format!(
                "configured lockfile corpus has {} entries, below policy.min_lockfiles={floor}; restore the removed workspace lockfile declaration",
                sources.len()
            ),
        )
    })
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

    if let Some(finding) = lockfile_policy_finding(policy) {
        findings.insert(finding);
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
    let declared_hash = manifest
        .get("content_hash")
        .and_then(Value::as_str)
        .unwrap_or("");
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
    let min_advisories = policy
        .get("min_advisories")
        .and_then(Value::as_u64)
        .unwrap_or(0);
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
    let (locked, uses_provenance) = observed_locked_rows(policy, observed);
    let unmaintained_all = policy.get("unmaintained_policy").and_then(Value::as_str) == Some("all");

    // The set of ignore ids that actually suppressed a LIVE affected advisory (for stale detection).
    let mut live_affected_ids: BTreeSet<String> = BTreeSet::new();

    for advisory in &advisories {
        let mut affected_rows = Vec::new();
        for pkg in &locked {
            if pkg.name != advisory.package {
                continue;
            }
            if !version_affected(&pkg.version, advisory) {
                continue;
            }
            affected_rows.push(pkg);
        }
        if affected_rows.is_empty() {
            continue;
        }

        live_affected_ids.insert(advisory.id.clone());

        let mut emitted: BTreeSet<(String, String, String)> = BTreeSet::new();
        for pkg in affected_rows {
            let key = if uses_provenance {
                pkg.finding_key(&advisory.id)
            } else {
                advisory.id.clone()
            };
            if !emitted.insert((key.clone(), pkg.source.clone(), pkg.version.clone())) {
                continue;
            }
            if ignore_ids.contains(advisory.id.as_str()) {
                continue;
            }

            match advisory.informational.as_deref() {
                None => {
                    // A security vulnerability.
                    findings.insert(Finding::new(
                        "SCA-VULN",
                        &key,
                        format!(
                            "security advisory {} affects locked crate `{}` version `{}` in lockfile `{}` (no installed version satisfies patched {:?} / unaffected {:?}). {}",
                            advisory.id,
                            advisory.package,
                            pkg.version,
                            pkg.source,
                            advisory.patched,
                            advisory.unaffected,
                            REMEDIATION_DOCTRINE
                        ),
                    ));
                }
                Some("unmaintained") if unmaintained_all => {
                    findings.insert(Finding::new(
                        "SCA-UNMAINTAINED",
                        &key,
                        format!(
                            "unmaintained advisory {} affects locked crate `{}` version `{}` in lockfile `{}` (unmaintained_policy=all). Migrate off the crate, or {}.",
                            advisory.id,
                            advisory.package,
                            pkg.version,
                            pkg.source,
                            REMEDIATION_DOCTRINE
                        ),
                    ));
                }
                Some(_other) => {
                    // `unsound` / `notice`: tracked but out of this gate's blocking scope (documented).
                }
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
        out.push_str(&format!(
            "    - {} {}\n        {}\n",
            finding.code, finding.key, finding.detail
        ));
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
        let hash = canonical_hash(&advisories);
        let count = advisories.len();
        json!({
            "locked": locked.iter().map(|(n, v)| json!({"name": n, "version": v})).collect::<Vec<_>>(),
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
        findings
            .iter()
            .map(|f| format!("{} {}", f.code, f.key))
            .collect()
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
            findings
                .iter()
                .any(|f| f.code == "SCA-UNMAINTAINED" && f.key == "RUSTSEC-2024-0436"),
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
        p["ignore"] =
            json!([{ "id": "RUSTSEC-2099-0001", "reason": "x", "remove_by": "2026-12-31" }]);
        let findings = evaluate_keyed(&p, &obs);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "SCA-STALE-IGNORE" && f.key == "RUSTSEC-2099-0001"),
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
        let obs = observed(
            &[("x", "1.0.0")],
            vec![vuln("RUSTSEC-2099-0001", "y", &[">= 1.0.0"])],
        );
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
