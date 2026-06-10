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
//! * A matched directory is a member only if it contains a `Cargo.toml`.
//! * `exclude` entries remove a directory and everything beneath it from the match set
//!   (this is how the `cloud/cloud-kernel` nested `no_std` workspace stays out of the
//!   root workspace even though `cloud/*/crates/*` would otherwise sweep its crates in).
//!
//! Pure + deterministic: a function of the committed tree only, so committed faces stay
//! byte-identical to regenerated faces (ADR-0083 hermetic-determinism contract).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Raw root workspace manifest entries before glob expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceManifestEntries {
    pub members: Vec<String>,
    pub exclude: Vec<String>,
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
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::Read(message) => write!(f, "read workspace manifest: {message}"),
            ResolveError::Parse(message) => write!(f, "parse workspace manifest: {message}"),
            ResolveError::Shape(message) => write!(f, "workspace manifest shape: {message}"),
        }
    }
}

impl std::error::Error for ResolveError {}

/// Resolve the concrete first-party workspace member directories from
/// `<repo_root>/Cargo.toml`.
///
/// Returns repo-relative, forward-slash directory paths (e.g. `libs/oya-foo-kernel`),
/// sorted and de-duplicated. Each returned directory is guaranteed to contain a
/// `Cargo.toml` and to NOT live under any `[workspace].exclude` entry.
pub fn resolve_member_dirs(repo_root: &Path) -> Result<Vec<String>, ResolveError> {
    let manifest_path = repo_root.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|error| ResolveError::Read(format!("{}: {error}", manifest_path.display())))?;
    resolve_member_dirs_from_str(&text, repo_root)
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
    let document: toml::Value =
        toml::from_str(manifest_text).map_err(|error| ResolveError::Parse(error.to_string()))?;
    let entries = parse_manifest_entries(&document)?;

    let mut resolved: BTreeSet<String> = BTreeSet::new();
    for pattern in &entries.members {
        for dir in expand_pattern(repo_root, pattern) {
            if is_excluded(&dir, &entries.exclude) {
                continue;
            }
            resolved.insert(dir);
        }
    }
    Ok(resolved.into_iter().collect())
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
    let exclude = string_array(workspace.get("exclude")).unwrap_or_default();
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

/// A directory is excluded if it equals an `exclude` entry or lives beneath one.
fn is_excluded(dir: &str, exclude: &[String]) -> bool {
    exclude
        .iter()
        .any(|entry| dir == entry || dir.starts_with(&format!("{entry}/")))
}

/// Expand one `members` pattern into the repo-relative directories that contain a
/// `Cargo.toml`. A path component may be a literal (`libs`), a bare wildcard (`*`), or a
/// partial wildcard (`oya-*`). A `*` never spans `/`; each component is matched
/// independently against one directory level, mirroring Cargo's `glob`-crate semantics.
fn expand_pattern(repo_root: &Path, pattern: &str) -> Vec<String> {
    let segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let mut frontier: Vec<PathBuf> = vec![PathBuf::new()];
    for segment in segments {
        let mut next: Vec<PathBuf> = Vec::new();
        for base in &frontier {
            if segment.contains('*') {
                if let Ok(entries) = std::fs::read_dir(repo_root.join(base)) {
                    for entry in entries.flatten() {
                        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            continue;
                        }
                        let name = entry.file_name();
                        if segment_matches(segment, &name.to_string_lossy()) {
                            next.push(base.join(&name));
                        }
                    }
                }
            } else {
                let candidate = base.join(segment);
                if repo_root.join(&candidate).is_dir() {
                    next.push(candidate);
                }
            }
        }
        frontier = next;
    }
    frontier
        .into_iter()
        .filter(|relative| repo_root.join(relative).join("Cargo.toml").is_file())
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .collect()
}

/// Match one path component against a single-segment glob containing zero or more `*`
/// wildcards (each `*` matches any run of characters within the component). `*` matches
/// the empty string, so `oya-*` matches `oya-`. Anchored at both ends.
fn segment_matches(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    // No wildcard -> exact match (callers only reach here when a `*` is present, but keep
    // the function total for reuse).
    if parts.len() == 1 {
        return pattern == name;
    }
    let mut cursor = 0usize;
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if index == 0 {
            // Leading literal must anchor the start.
            if !name[cursor..].starts_with(part) {
                return false;
            }
            cursor += part.len();
        } else if index == parts.len() - 1 {
            // Trailing literal must anchor the end (and not overlap consumed prefix).
            return name[cursor..].ends_with(part) && name.len() - cursor >= part.len();
        } else {
            // Interior literal: find next occurrence at or after the cursor.
            match name[cursor..].find(part) {
                Some(offset) => cursor += offset + part.len(),
                None => return false,
            }
        }
    }
    // Pattern ended with a `*` (last part empty) -> remainder is unconstrained.
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Build an isolated fixture tree under the OS temp dir and return its root.
    fn fixture_root() -> PathBuf {
        let unique = format!(
            "oya-wsm-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let root = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create fixture root");
        root
    }

    fn make_crate(root: &Path, relative: &str) {
        let dir = root.join(relative);
        std::fs::create_dir_all(&dir).expect("create crate dir");
        std::fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{}\"\n",
                relative.replace('/', "-")
            ),
        )
        .expect("write Cargo.toml");
    }

    fn root_manifest(members: &[&str], exclude: &[&str]) -> String {
        let members = members
            .iter()
            .map(|m| format!("  \"{m}\",\n"))
            .collect::<String>();
        let exclude = exclude
            .iter()
            .map(|m| format!("  \"{m}\",\n"))
            .collect::<String>();
        format!("[workspace]\nmembers = [\n{members}]\nexclude = [\n{exclude}]\nresolver = \"2\"\n")
    }

    #[test]
    fn star_matches_one_component_and_requires_cargo_toml() {
        let root = fixture_root();
        make_crate(&root, "libs/oya-a-kernel");
        make_crate(&root, "libs/oya-b-kernel");
        // A directory under libs/ WITHOUT a Cargo.toml is not a member.
        std::fs::create_dir_all(root.join("libs/not-a-crate")).unwrap();
        // A crate one level deeper must NOT be swept by the single-`*` glob.
        make_crate(&root, "libs/group/oya-nested-kernel");

        let manifest = root_manifest(&["libs/*"], &[]);
        let resolved = resolve_member_dirs_from_str(&manifest, &root).expect("resolve");

        assert_eq!(
            resolved,
            vec![
                "libs/oya-a-kernel".to_string(),
                "libs/oya-b-kernel".to_string()
            ]
        );
    }

    #[test]
    fn exclude_drops_a_nested_workspace_subtree() {
        let root = fixture_root();
        make_crate(&root, "cloud/cloud-data/crates/oya-data-kernel");
        // Nested workspace crates that the glob would otherwise sweep in.
        make_crate(&root, "cloud/cloud-kernel/crates/oya-kernel-frame-kernel");
        make_crate(&root, "cloud/cloud-kernel/crates/oya-kernel-hal-kernel");

        let manifest = root_manifest(&["cloud/*/crates/*"], &["cloud/cloud-kernel"]);
        let resolved = resolve_member_dirs_from_str(&manifest, &root).expect("resolve");

        assert_eq!(
            resolved,
            vec!["cloud/cloud-data/crates/oya-data-kernel".to_string()],
            "the excluded nested-workspace subtree must not appear as a member"
        );
    }

    #[test]
    fn literal_member_path_is_supported_alongside_globs() {
        let root = fixture_root();
        make_crate(&root, "tools/oya-one-cli");
        make_crate(&root, "libs/oya-x-kernel");

        let manifest = root_manifest(&["libs/*", "tools/oya-one-cli"], &[]);
        let resolved = resolve_member_dirs_from_str(&manifest, &root).expect("resolve");

        assert_eq!(
            resolved,
            vec![
                "libs/oya-x-kernel".to_string(),
                "tools/oya-one-cli".to_string()
            ]
        );
    }

    #[test]
    fn partial_segment_glob_filters_non_crate_siblings() {
        let root = fixture_root();
        make_crate(&root, "tools/oya-one-cli");
        make_crate(&root, "tools/oya-two-cli");
        // Non-crate sibling dirs (scripts, completions) must NOT break or appear.
        std::fs::create_dir_all(root.join("tools/hooks")).unwrap();
        std::fs::write(root.join("tools/hooks/run.sh"), "#!/bin/sh\n").unwrap();
        std::fs::create_dir_all(root.join("tools/completions/bash")).unwrap();

        let manifest = root_manifest(&["tools/oya-*"], &[]);
        let resolved = resolve_member_dirs_from_str(&manifest, &root).expect("resolve");

        assert_eq!(
            resolved,
            vec![
                "tools/oya-one-cli".to_string(),
                "tools/oya-two-cli".to_string()
            ],
            "tools/oya-* must capture only oya-prefixed crate dirs, never sibling tool dirs"
        );
    }

    #[test]
    fn segment_matches_anchors_both_ends() {
        assert!(segment_matches("oya-*", "oya-foo"));
        assert!(segment_matches("oya-*", "oya-"));
        assert!(!segment_matches("oya-*", "xoya-foo"));
        assert!(!segment_matches("oya-*", "completions"));
        assert!(segment_matches("*", "anything"));
        assert!(segment_matches("*-app", "oya-gate-app"));
        assert!(!segment_matches("*-app", "oya-gate-lib"));
    }

    #[test]
    fn manifest_entries_reader_returns_raw_members_and_excludes() {
        let manifest = root_manifest(
            &["libs/oya-*", "cloud/*/crates/oya-*"],
            &["cloud/cloud-kernel"],
        );
        let entries = workspace_manifest_entries_from_str(&manifest).expect("entries");
        assert_eq!(
            entries.members,
            vec![
                "libs/oya-*".to_owned(),
                "cloud/*/crates/oya-*".to_owned()
            ]
        );
        assert_eq!(entries.exclude, vec!["cloud/cloud-kernel".to_owned()]);
    }

    #[test]
    fn missing_workspace_table_is_a_shape_error() {
        let root = fixture_root();
        let error = resolve_member_dirs_from_str("[package]\nname = \"x\"\n", &root)
            .expect_err("must reject non-workspace manifest");
        assert!(matches!(error, ResolveError::Shape(_)));
    }
}
