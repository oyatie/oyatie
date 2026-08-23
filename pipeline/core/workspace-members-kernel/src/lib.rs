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
//! * An unexcluded matched directory without `Cargo.toml` is an error, just as it is for Cargo.
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

/// A directory is excluded if it equals an `exclude` entry or lives beneath one.
#[must_use]
pub fn is_excluded(dir: &str, exclude: &[String]) -> bool {
    exclude
        .iter()
        .any(|entry| dir == entry || dir.starts_with(&format!("{entry}/")))
}

/// True iff the `members` glob `pattern` covers the repo-relative directory `dir` by Cargo's
/// member-glob shape — each `*` matches exactly one path component (never spanning `/`) and every
/// segment is anchored. PURE string logic (no filesystem): unlike [`resolve_member_dirs_from_str`]
/// this does NOT consult the tree, so it answers "could this pattern match this dir" rather than
/// "does this concrete member exist". A caller that already knows `dir` is a real crate dir (it has
/// a `Cargo.toml`) can therefore reuse this as a pure coverage predicate without re-deriving the
/// `*`-per-component semantics (the workspace-members module is the single source of that truth).
///
/// `dir` and `pattern` are normalized by trimming any trailing `/`; a component count mismatch is an
/// immediate non-match (a 2-segment glob can never cover a 3-segment dir, since `*` never spans `/`).
#[must_use]
pub fn pattern_covers_dir(pattern: &str, dir: &str) -> bool {
    let pattern_segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let dir_segments: Vec<&str> = dir.split('/').filter(|s| !s.is_empty()).collect();
    if pattern_segments.len() != dir_segments.len() {
        return false;
    }
    pattern_segments
        .iter()
        .zip(dir_segments.iter())
        .all(|(pat, name)| segment_matches(pat, name))
}

/// True iff `dir` is covered by `entries`: some `members` pattern covers it (via
/// [`pattern_covers_dir`]) AND no `exclude` entry removes it (via [`is_excluded`]). PURE — mirrors
/// the [`resolve_member_dirs_from_str`] coverage rule minus the filesystem `Cargo.toml`-presence
/// check, which is irrelevant when the caller already knows `dir` is a real crate directory.
#[must_use]
pub fn member_entries_cover_dir(entries: &WorkspaceManifestEntries, dir: &str) -> bool {
    entries
        .members
        .iter()
        .any(|pattern| pattern_covers_dir(pattern, dir))
        && !is_excluded(dir, &entries.exclude)
}

/// Expand one `members` pattern into all matching repo-relative directories. A path component may
/// be a literal (`libs`), a bare wildcard (`*`), or a
/// partial wildcard (`oya-*`). A `*` never spans `/`; each component is matched
/// independently against one directory level, mirroring Cargo's `glob`-crate semantics.
fn expand_pattern(
    repo_root: &Path,
    pattern: &str,
    exclude: &[String],
) -> Result<Vec<String>, ResolveError> {
    let segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let mut frontier: Vec<PathBuf> = vec![PathBuf::new()];
    for segment in segments {
        let mut next: Vec<PathBuf> = Vec::new();
        for base in &frontier {
            if segment.contains('*') {
                let parent = repo_root.join(base);
                let entries = std::fs::read_dir(&parent)
                    .map_err(|error| inspect_member_path_error(&parent, error))?;
                for entry in entries {
                    let entry = entry.map_err(|error| inspect_member_path_error(&parent, error))?;
                    let name = entry.file_name();
                    if !segment_matches(segment, &name.to_string_lossy()) {
                        continue;
                    }
                    let candidate = base.join(&name);
                    if is_excluded(&relative_path_string(&candidate), exclude) {
                        continue;
                    }
                    let entry_path = repo_root.join(&candidate);
                    let file_type = entry
                        .file_type()
                        .map_err(|error| inspect_member_path_error(&entry_path, error))?;
                    let is_directory = if file_type.is_dir() {
                        true
                    } else if file_type.is_symlink() {
                        match std::fs::metadata(&entry_path) {
                            Ok(metadata) => metadata.is_dir(),
                            Err(error) if cargo_skips_symlink_target_error(&error) => false,
                            Err(error) => {
                                return Err(inspect_member_path_error(&entry_path, error));
                            }
                        }
                    } else {
                        false
                    };
                    if !is_directory {
                        continue;
                    }
                    next.push(candidate);
                }
            } else {
                let candidate = base.join(segment);
                if is_excluded(&relative_path_string(&candidate), exclude) {
                    continue;
                }
                let candidate_path = repo_root.join(&candidate);
                match std::fs::metadata(&candidate_path) {
                    Ok(metadata) if metadata.is_dir() => next.push(candidate),
                    Ok(_) => {}
                    Err(error) if cargo_skips_symlink_target_error(&error) => {}
                    Err(error) => {
                        return Err(inspect_member_path_error(&candidate_path, error));
                    }
                }
            }
        }
        frontier = next;
    }
    Ok(frontier
        .into_iter()
        .map(|relative| relative_path_string(&relative))
        .collect())
}

fn relative_path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn inspect_member_path_error(path: &Path, error: std::io::Error) -> ResolveError {
    ResolveError::InspectMemberPath {
        path: path.display().to_string(),
        message: error.to_string(),
    }
}

/// Cargo's member-glob expansion treats dangling and cyclic symlinks as unmatched paths.
/// Other inspection failures remain hard errors so an incomplete member scan cannot go green.
fn cargo_skips_symlink_target_error(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::NotFound {
        return true;
    }

    // `ErrorKind::FilesystemLoop` remains unstable on the pinned toolchain, so recognize the
    // platform error codes returned by metadata(2) / CreateFile for a cyclic symlink.
    #[cfg(unix)]
    if matches!(error.raw_os_error(), Some(40 | 62)) {
        return true;
    }
    #[cfg(windows)]
    if is_windows_filesystem_loop_error_code(error.raw_os_error()) {
        return true;
    }

    false
}

/// Win32's `ERROR_CANT_RESOLVE_FILENAME` is how `metadata` reports a cyclic symlink.
/// Keep this pure so the platform-specific branch has a host-independent regression test.
#[must_use]
#[cfg(any(windows, test))]
fn is_windows_filesystem_loop_error_code(raw_os_error: Option<i32>) -> bool {
    raw_os_error == Some(1921)
}

/// Match one path component against a single-segment glob containing zero or more `*`
/// wildcards (each `*` matches any run of characters within the component). `*` matches
/// the empty string, so `oya-*` matches `oya-`. Anchored at both ends.
#[must_use]
pub fn segment_matches(pattern: &str, name: &str) -> bool {
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
            "wsm-{}-{}",
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
            format!("[package]\nname = \"{}\"\n", relative.replace('/', "-")),
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
    fn star_matches_one_component_and_honors_non_crate_excludes() {
        let root = fixture_root();
        make_crate(&root, "libs/a-kernel");
        make_crate(&root, "libs/b-kernel");
        // An explicitly excluded directory under libs/ does not need a Cargo.toml.
        std::fs::create_dir_all(root.join("libs/not-a-crate")).unwrap();
        // A crate one level deeper must NOT be swept by the single-`*` glob.
        make_crate(&root, "libs/group/nested-kernel");

        let manifest = root_manifest(&["libs/*"], &["libs/not-a-crate", "libs/group"]);
        let resolved = resolve_member_dirs_from_str(&manifest, &root).expect("resolve");

        assert_eq!(
            resolved,
            vec!["libs/a-kernel".to_string(), "libs/b-kernel".to_string()]
        );
    }

    #[test]
    fn diagnostic_scan_reports_every_unexcluded_missing_manifest() {
        let root = fixture_root();
        std::fs::create_dir_all(root.join("comms/messenger/chaos"))
            .expect("create non-crate member match");
        std::fs::create_dir_all(root.join("comms/messenger/resilience"))
            .expect("create second non-crate member match");
        std::fs::create_dir_all(root.join("comms/messenger/fixtures"))
            .expect("create excluded non-crate member match");

        let manifest = root_manifest(&["comms/*/*"], &["comms/messenger/fixtures"]);
        let scan = scan_member_dirs_from_str(&manifest, &root).expect("scan diagnostics");
        assert!(scan.member_dirs.is_empty());
        assert_eq!(
            scan.missing_manifests,
            vec![
                "comms/messenger/chaos".to_owned(),
                "comms/messenger/resilience".to_owned(),
            ]
        );

        let error = resolve_member_dirs_from_str(&manifest, &root)
            .expect_err("Cargo rejects an unexcluded member-glob match without Cargo.toml");

        assert_eq!(
            error,
            ResolveError::MissingManifests(vec![
                "comms/messenger/chaos".to_owned(),
                "comms/messenger/resilience".to_owned(),
            ])
        );
    }

    #[test]
    fn explicit_exclude_suppresses_non_manifest_glob_match() {
        let root = fixture_root();
        std::fs::create_dir_all(root.join("comms/messenger/chaos"))
            .expect("create excluded non-crate member match");

        let manifest = root_manifest(&["comms/*/*"], &["comms/messenger/chaos"]);
        let resolved = resolve_member_dirs_from_str(&manifest, &root).expect("resolve");

        assert!(resolved.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn wildcard_includes_directory_symlink_like_cargo() {
        use std::os::unix::fs::symlink;

        let root = fixture_root();
        make_crate(&root, "real");
        std::fs::create_dir_all(root.join("libs")).unwrap();
        symlink("../real", root.join("libs/link")).unwrap();

        let manifest = root_manifest(&["libs/*"], &[]);
        let resolved = resolve_member_dirs_from_str(&manifest, &root).expect("resolve symlink");

        assert_eq!(resolved, vec!["libs/link".to_owned()]);
    }

    #[cfg(unix)]
    #[test]
    fn wildcard_reports_directory_symlink_missing_manifest_like_cargo() {
        use std::os::unix::fs::symlink;

        let root = fixture_root();
        std::fs::create_dir_all(root.join("real")).unwrap();
        std::fs::create_dir_all(root.join("libs")).unwrap();
        symlink("../real", root.join("libs/link")).unwrap();

        let manifest = root_manifest(&["libs/*"], &[]);
        let scan = scan_member_dirs_from_str(&manifest, &root).expect("scan symlink");

        assert!(scan.member_dirs.is_empty());
        assert_eq!(scan.missing_manifests, vec!["libs/link".to_owned()]);
    }

    #[cfg(unix)]
    #[test]
    fn wildcard_exclude_precedes_directory_symlink_manifest_check() {
        use std::os::unix::fs::symlink;

        let root = fixture_root();
        std::fs::create_dir_all(root.join("real")).unwrap();
        std::fs::create_dir_all(root.join("libs")).unwrap();
        symlink("../real", root.join("libs/link")).unwrap();

        let manifest = root_manifest(&["libs/*"], &["libs/link"]);
        let scan = scan_member_dirs_from_str(&manifest, &root).expect("scan excluded symlink");

        assert!(scan.member_dirs.is_empty());
        assert!(scan.missing_manifests.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn wildcard_skips_dangling_symlink_like_cargo() {
        use std::os::unix::fs::symlink;

        let root = fixture_root();
        std::fs::create_dir_all(root.join("libs")).unwrap();
        symlink("../missing", root.join("libs/link")).unwrap();

        let manifest = root_manifest(&["libs/*"], &[]);
        let scan = scan_member_dirs_from_str(&manifest, &root).expect("scan dangling symlink");

        assert!(scan.member_dirs.is_empty());
        assert!(scan.missing_manifests.is_empty());
    }

    #[test]
    fn exclude_drops_a_nested_workspace_subtree() {
        let root = fixture_root();
        make_crate(&root, "cloud/cloud-data/crates/data-kernel");
        // Nested workspace crates that the glob would otherwise sweep in.
        make_crate(&root, "cloud/cloud-kernel/crates/kernel-frame-kernel");
        make_crate(&root, "cloud/cloud-kernel/crates/kernel-hal-kernel");

        let manifest = root_manifest(&["cloud/*/crates/*"], &["cloud/cloud-kernel"]);
        let resolved = resolve_member_dirs_from_str(&manifest, &root).expect("resolve");

        assert_eq!(
            resolved,
            vec!["cloud/cloud-data/crates/data-kernel".to_string()],
            "the excluded nested-workspace subtree must not appear as a member"
        );
    }

    #[test]
    fn literal_member_path_is_supported_alongside_globs() {
        let root = fixture_root();
        make_crate(&root, "tools/one-cli");
        make_crate(&root, "libs/x-kernel");

        let manifest = root_manifest(&["libs/*", "tools/one-cli"], &[]);
        let resolved = resolve_member_dirs_from_str(&manifest, &root).expect("resolve");

        assert_eq!(
            resolved,
            vec!["libs/x-kernel".to_string(), "tools/one-cli".to_string()]
        );
    }

    #[test]
    fn partial_segment_glob_filters_non_crate_siblings() {
        let root = fixture_root();
        make_crate(&root, "tools/one-cli");
        make_crate(&root, "tools/two-cli");
        // Non-crate sibling dirs (scripts, completions) must NOT break or appear.
        std::fs::create_dir_all(root.join("tools/hooks")).unwrap();
        std::fs::write(root.join("tools/hooks/run.sh"), "#!/bin/sh\n").unwrap();
        std::fs::create_dir_all(root.join("tools/completions/bash")).unwrap();

        let manifest = root_manifest(&["tools/oya-*"], &[]);
        let resolved = resolve_member_dirs_from_str(&manifest, &root).expect("resolve");

        assert_eq!(
            resolved,
            vec!["tools/one-cli".to_string(), "tools/two-cli".to_string()],
            "tools/oya-* must capture only prefixed crate dirs, never sibling tool dirs"
        );
    }

    #[test]
    fn segment_matches_anchors_both_ends() {
        assert!(segment_matches("oya-*", "foo"));
        assert!(segment_matches("oya-*", "oya-"));
        assert!(!segment_matches("oya-*", "xoya-foo"));
        assert!(!segment_matches("oya-*", "completions"));
        assert!(segment_matches("*", "anything"));
        assert!(segment_matches("*-app", "gate-app"));
        assert!(!segment_matches("*-app", "gate-lib"));
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
            vec!["libs/oya-*".to_owned(), "cloud/*/crates/oya-*".to_owned()]
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

    #[test]
    fn malformed_root_manifest_is_a_parse_error() {
        let root = fixture_root();
        let error = scan_member_dirs_from_str("[workspace\nmembers = []\n", &root)
            .expect_err("must reject malformed root manifest");
        assert!(matches!(error, ResolveError::Parse(_)));
    }

    #[test]
    fn malformed_exclude_is_a_shape_error() {
        let root = fixture_root();
        let error = scan_member_dirs_from_str(
            "[workspace]\nmembers = []\nexclude = \"libs/skip\"\n",
            &root,
        )
        .expect_err("must reject a non-array [workspace].exclude value");

        assert!(matches!(error, ResolveError::Shape(_)));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_loops_are_skipped_like_cargo_for_glob_and_literal_members() {
        use std::os::unix::fs::symlink;

        let root = fixture_root();
        symlink("loop", root.join("loop")).unwrap();

        for member in ["*", "loop"] {
            let manifest =
                format!("[workspace]\nmembers = [\"{member}\"]\nexclude = []\nresolver = \"2\"\n");
            let scan = scan_member_dirs_from_str(&manifest, &root)
                .expect("Cargo skips a self-referential symlink member");

            assert!(scan.member_dirs.is_empty(), "member pattern: {member}");
            assert!(
                scan.missing_manifests.is_empty(),
                "member pattern: {member}"
            );
        }
    }

    #[test]
    fn windows_filesystem_loop_error_code_is_recognized_without_windows_host() {
        assert!(is_windows_filesystem_loop_error_code(Some(1921)));
        assert!(!is_windows_filesystem_loop_error_code(Some(40)));
        assert!(!is_windows_filesystem_loop_error_code(None));
    }

    #[test]
    fn expansion_read_dir_errors_fail_closed() {
        let root = fixture_root();
        let not_a_directory = root.join("not-a-directory");
        std::fs::write(&not_a_directory, "fixture").unwrap();

        let result = scan_member_dirs_from_str(
            "[workspace]\nmembers = [\"*\"]\nexclude = []\n",
            &not_a_directory,
        );

        assert!(
            matches!(&result, Err(ResolveError::InspectMemberPath { .. })),
            "a member directory read error must fail closed: {result:?}"
        );
    }

    #[test]
    fn pattern_covers_dir_honors_per_component_glob_semantics() {
        // A narrowed leaf glob covers a matching leaf, never a sibling that fails the prefix.
        assert!(pattern_covers_dir("libs/oya-*", "libs/foo-kernel"));
        assert!(!pattern_covers_dir("libs/oya-*", "libs/registry-drift"));
        // `*` never spans `/`: a 2-segment glob cannot cover a 3-segment dir.
        assert!(!pattern_covers_dir("libs/*", "libs/group/nested-kernel"));
        // A 3-segment capability glob covers a 3-segment face/leaf dir.
        assert!(pattern_covers_dir("messaging/*/*", "messaging/core/domain"));
        assert!(!pattern_covers_dir("messaging/*/*", "messaging/core"));
        // Trailing slashes are normalized away on both sides.
        assert!(pattern_covers_dir("libs/oya-*", "libs/foo-kernel/"));
    }

    #[test]
    fn member_entries_cover_dir_applies_excludes() {
        let entries = WorkspaceManifestEntries {
            members: vec!["cloud/*/crates/*".to_owned()],
            exclude: vec!["cloud/cloud-kernel".to_owned()],
        };
        // Covered by the glob and not excluded.
        assert!(member_entries_cover_dir(
            &entries,
            "cloud/cloud-data/crates/data-kernel"
        ));
        // Matched by the glob but removed by the exclude subtree.
        assert!(!member_entries_cover_dir(
            &entries,
            "cloud/cloud-kernel/crates/kernel-frame-kernel"
        ));
        // Outside every members glob.
        assert!(!member_entries_cover_dir(&entries, "libs/foo-kernel"));
    }
}
