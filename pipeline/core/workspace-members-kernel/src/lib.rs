//! Canonical workspace-membership resolver: the single source of truth for "which
//! directories are first-party workspace members", expanded from the root `Cargo.toml`
//! `[workspace].members` glob patterns while honoring `[workspace].exclude`.
//!
//! # Why this exists (FRIC-1781069288, ADR-0538)
//!
//! The root manifest lists workspace members as GLOBS (`libs/*`, `cloud/*/crates/*`,
//! ...) so that adding a crate requires ZERO edit to the shared `members` array. That
//! eliminates the merge-conflict class that previously hit every concurrent new-crate
//! lane by construction (two lanes both editing the same `members` array + `Cargo.lock`).
//!
//! Any gate or tool that needs the concrete member set resolves it HERE (reuse, not
//! re-derive) instead of textually parsing the array, which after globbing would only
//! see `*` literals, never real crate directories.
//!
//! # Semantics (mirror Cargo's own member-glob behavior)
//!
//! * Each `*` matches exactly one path component; `*` never matches `/`.
//! * A complete `**` component matches zero or more directory levels.
//! * Literal `.` and `..` components are normalized after their preceding path has been
//!   resolved, matching Cargo's treatment of source-marker globs such as `*/src/..`.
//! * An unexcluded matched directory without `Cargo.toml` is an error, just as it is for Cargo.
//! * `exclude` entries remove a directory and everything beneath it from the match set
//!   (this is how the `cloud/cloud-kernel` nested `no_std` workspace stays out of the
//!   root workspace even though `cloud/*/crates/*` would otherwise sweep its crates in).
//!
//! Pure + deterministic: a function of the committed tree only, so committed faces stay
//! byte-identical to regenerated faces (ADR-0083 hermetic-determinism contract).

use std::collections::BTreeSet;
use std::path::Path;

mod member_glob;

use member_glob::expand_pattern;
pub use member_glob::{is_excluded, member_entries_cover_dir, pattern_covers_dir, segment_matches};

/// Raw root workspace manifest entries before glob expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceManifestEntries {
    pub members: Vec<String>,
    pub exclude: Vec<String>,
}

/// Cargo-faithful expansion diagnostics for root workspace member globs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceMemberScan {
    /// Valid member directories, sorted and de-duplicated.
    pub member_dirs: Vec<String>,
    /// Unexcluded glob matches that do not contain `Cargo.toml`, sorted and de-duplicated.
    pub missing_manifests: Vec<String>,
}

/// Failure resolving the workspace member set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// The root `Cargo.toml` could not be read.
    Read(String),
    /// The root `Cargo.toml` is not valid TOML.
    Parse(String),
    /// The manifest is missing a required `[workspace]` shape.
    Shape(String),
    /// A filesystem path needed for member-glob expansion could not be inspected.
    InspectMemberPath { path: String, message: String },
    /// One or more unexcluded member-glob matches do not contain `Cargo.toml`.
    MissingManifests(Vec<String>),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::Read(message) => write!(f, "read workspace manifest: {message}"),
            ResolveError::Parse(message) => write!(f, "parse workspace manifest: {message}"),
            ResolveError::Shape(message) => write!(f, "workspace manifest shape: {message}"),
            ResolveError::InspectMemberPath { path, message } => {
                write!(f, "inspect workspace member path {path}: {message}")
            }
            ResolveError::MissingManifests(paths) => write!(
                f,
                "workspace member glob matched directories without Cargo.toml: {}",
                paths.join(", ")
            ),
        }
    }
}

impl std::error::Error for ResolveError {}

/// Resolve the concrete first-party workspace member directories from
/// `<repo_root>/Cargo.toml`.
///
/// Returns repo-relative, forward-slash directory paths (e.g. `libs/foo-kernel`),
/// sorted and de-duplicated. Each returned directory is guaranteed to contain a
/// `Cargo.toml` and to NOT live under any `[workspace].exclude` entry. Returns
/// [`ResolveError::MissingManifests`] when an unexcluded glob match has no manifest.
pub fn resolve_member_dirs(repo_root: &Path) -> Result<Vec<String>, ResolveError> {
    let manifest_path = repo_root.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|error| ResolveError::Read(format!("{}: {error}", manifest_path.display())))?;
    resolve_member_dirs_from_str(&text, repo_root)
}

/// Scan concrete member-glob matches while retaining missing-manifest diagnostics.
///
/// Gate producers use this surface to emit every invalid match as policy input. Ordinary callers
/// should use [`resolve_member_dirs`], which fails closed when `missing_manifests` is non-empty.
pub fn scan_member_dirs(repo_root: &Path) -> Result<WorkspaceMemberScan, ResolveError> {
    let manifest_path = repo_root.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|error| ResolveError::Read(format!("{}: {error}", manifest_path.display())))?;
    scan_member_dirs_from_str(&text, repo_root)
}

/// Read the raw root `[workspace].members` and `[workspace].exclude` string arrays from
/// `<repo_root>/Cargo.toml`.
pub fn read_workspace_manifest_entries(
    repo_root: &Path,
) -> Result<WorkspaceManifestEntries, ResolveError> {
    let manifest_path = repo_root.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|error| ResolveError::Read(format!("{}: {error}", manifest_path.display())))?;
    workspace_manifest_entries_from_str(&text)
}

/// As [`read_workspace_manifest_entries`] but with the manifest text already in hand.
pub fn workspace_manifest_entries_from_str(
    manifest_text: &str,
) -> Result<WorkspaceManifestEntries, ResolveError> {
    let document: toml::Value =
        toml::from_str(manifest_text).map_err(|error| ResolveError::Parse(error.to_string()))?;
    parse_manifest_entries(&document)
}

/// As [`resolve_member_dirs`] but with the manifest text already in hand. The
/// filesystem under `repo_root` is still consulted to expand globs (`*`) and to apply
/// the `Cargo.toml`-presence rule, exactly like Cargo.
pub fn resolve_member_dirs_from_str(
    manifest_text: &str,
    repo_root: &Path,
) -> Result<Vec<String>, ResolveError> {
    let scan = scan_member_dirs_from_str(manifest_text, repo_root)?;
    if scan.missing_manifests.is_empty() {
        Ok(scan.member_dirs)
    } else {
        Err(ResolveError::MissingManifests(scan.missing_manifests))
    }
}

/// As [`scan_member_dirs`] but with the manifest text already in hand.
pub fn scan_member_dirs_from_str(
    manifest_text: &str,
    repo_root: &Path,
) -> Result<WorkspaceMemberScan, ResolveError> {
    let document: toml::Value =
        toml::from_str(manifest_text).map_err(|error| ResolveError::Parse(error.to_string()))?;
    let entries = parse_manifest_entries(&document)?;

    let mut resolved: BTreeSet<String> = BTreeSet::new();
    let mut missing_manifests: BTreeSet<String> = BTreeSet::new();
    for pattern in &entries.members {
        for dir in expand_pattern(repo_root, pattern, &entries.exclude)? {
            if is_excluded(&dir, &entries.exclude) {
                continue;
            }
            if repo_root.join(&dir).join("Cargo.toml").is_file() {
                resolved.insert(dir);
            } else {
                missing_manifests.insert(dir);
            }
        }
    }
    Ok(WorkspaceMemberScan {
        member_dirs: resolved.into_iter().collect(),
        missing_manifests: missing_manifests.into_iter().collect(),
    })
}

fn parse_manifest_entries(
    document: &toml::Value,
) -> Result<WorkspaceManifestEntries, ResolveError> {
    let workspace = document
        .get("workspace")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| ResolveError::Shape("missing [workspace] table".to_string()))?;
    let members = string_array(workspace.get("members"))
        .ok_or_else(|| ResolveError::Shape("missing [workspace].members array".to_string()))?;
    let exclude = match workspace.get("exclude") {
        Some(value) => string_array(Some(value)).ok_or_else(|| {
            ResolveError::Shape("[workspace].exclude must be an array of strings".to_string())
        })?,
        None => Vec::new(),
    };
    Ok(WorkspaceManifestEntries { members, exclude })
}

/// Read an array-of-strings TOML value, returning `None` if absent or not a homogeneous
/// string array.
fn string_array(value: Option<&toml::Value>) -> Option<Vec<String>> {
    value?
        .as_array()?
        .iter()
        .map(|entry| entry.as_str().map(str::to_owned))
        .collect()
}

#[cfg(test)]
mod tests;
