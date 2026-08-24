use std::path::{Path, PathBuf};

use crate::{ResolveError, WorkspaceManifestEntries};

/// A directory is excluded if it equals an `exclude` entry or lives beneath one.
#[must_use]
pub fn is_excluded(dir: &str, exclude: &[String]) -> bool {
    exclude
        .iter()
        .any(|entry| dir == entry || dir.starts_with(&format!("{entry}/")))
}

/// True iff the `members` glob `pattern` covers the repo-relative directory `dir` by
/// Cargo's member-glob shape. Each `*` matches exactly one path component, a complete
/// `**` component matches zero or more components, and literal `.` / `..` components
/// are normalized before component matching.
///
/// This is pure string logic. It intentionally does not prove that source-marker
/// components such as `src/..` exist; [`crate::resolve_member_dirs_from_str`] performs
/// that filesystem check during concrete expansion.
#[must_use]
pub fn pattern_covers_dir(pattern: &str, dir: &str) -> bool {
    let Some(pattern_segments) = normalized_segments(pattern) else {
        return false;
    };
    let Some(dir_segments) = normalized_segments(dir) else {
        return false;
    };
    components_match(&pattern_segments, &dir_segments)
}

/// True iff `dir` is covered by at least one member entry and no exclude removes it.
#[must_use]
pub fn member_entries_cover_dir(entries: &WorkspaceManifestEntries, dir: &str) -> bool {
    entries
        .members
        .iter()
        .any(|pattern| pattern_covers_dir(pattern, dir))
        && !is_excluded(dir, &entries.exclude)
}

/// Expand one member pattern into concrete repo-relative directories.
///
/// Wildcards are resolved one directory level at a time. Literal parent components are
/// applied only after the preceding component has been inspected, so `*/src/..` both
/// requires the source directory and returns the canonical crate directory.
pub(crate) fn expand_pattern(
    repo_root: &Path,
    pattern: &str,
    exclude: &[String],
) -> Result<Vec<String>, ResolveError> {
    let segments = pattern.split('/').filter(|segment| !segment.is_empty());
    let mut frontier = vec![PathBuf::new()];
    for segment in segments {
        if segment == "." {
            continue;
        }
        if segment == ".." {
            frontier = normalize_parent_components(frontier, pattern, exclude)?;
            continue;
        }

        let mut next = Vec::new();
        for base in &frontier {
            if segment == "**" {
                expand_recursive(repo_root, base, exclude, &mut next)?;
            } else if segment.contains('*') {
                expand_wildcard(repo_root, base, segment, exclude, &mut next)?;
            } else {
                expand_literal(repo_root, base, segment, exclude, &mut next)?;
            }
        }
        frontier = next;
    }
    Ok(frontier
        .into_iter()
        .map(|relative| relative_path_string(&relative))
        .collect())
}

fn components_match(pattern: &[&str], path: &[&str]) -> bool {
    match (pattern.split_first(), path.split_first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some((&"**", rest)), _) => {
            components_match(rest, path)
                || path
                    .split_first()
                    .is_some_and(|(_, tail)| components_match(pattern, tail))
        }
        (Some((_component, _)), None) => false,
        (Some((component, rest)), Some((name, tail))) => {
            segment_matches(component, name) && components_match(rest, tail)
        }
    }
}

fn normalize_parent_components(
    frontier: Vec<PathBuf>,
    pattern: &str,
    exclude: &[String],
) -> Result<Vec<PathBuf>, ResolveError> {
    let mut next = Vec::with_capacity(frontier.len());
    for mut candidate in frontier {
        if !candidate.pop() {
            return Err(ResolveError::Shape(format!(
                "workspace member pattern escapes repository root: {pattern}"
            )));
        }
        if !is_excluded(&relative_path_string(&candidate), exclude) {
            next.push(candidate);
        }
    }
    Ok(next)
}

fn expand_wildcard(
    repo_root: &Path,
    base: &Path,
    segment: &str,
    exclude: &[String],
    next: &mut Vec<PathBuf>,
) -> Result<(), ResolveError> {
    let parent = repo_root.join(base);
    let entries =
        std::fs::read_dir(&parent).map_err(|error| inspect_member_path_error(&parent, error))?;
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
                Err(error) => return Err(inspect_member_path_error(&entry_path, error)),
            }
        } else {
            false
        };
        if is_directory {
            next.push(candidate);
        }
    }
    Ok(())
}

fn expand_recursive(
    repo_root: &Path,
    base: &Path,
    exclude: &[String],
    next: &mut Vec<PathBuf>,
) -> Result<(), ResolveError> {
    next.push(base.to_path_buf());
    let parent = repo_root.join(base);
    let entries =
        std::fs::read_dir(&parent).map_err(|error| inspect_member_path_error(&parent, error))?;
    for entry in entries {
        let entry = entry.map_err(|error| inspect_member_path_error(&parent, error))?;
        let candidate = base.join(entry.file_name());
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
                Err(error) => return Err(inspect_member_path_error(&entry_path, error)),
            }
        } else {
            false
        };
        if !is_directory {
            continue;
        }
        if file_type.is_symlink() {
            next.push(candidate);
        } else {
            expand_recursive(repo_root, &candidate, exclude, next)?;
        }
    }
    Ok(())
}

fn expand_literal(
    repo_root: &Path,
    base: &Path,
    segment: &str,
    exclude: &[String],
    next: &mut Vec<PathBuf>,
) -> Result<(), ResolveError> {
    let candidate = base.join(segment);
    if is_excluded(&relative_path_string(&candidate), exclude) {
        return Ok(());
    }
    let candidate_path = repo_root.join(&candidate);
    match std::fs::metadata(&candidate_path) {
        Ok(metadata) if metadata.is_dir() => next.push(candidate),
        Ok(_) => {}
        Err(error) if cargo_skips_symlink_target_error(&error) => {}
        Err(error) => return Err(inspect_member_path_error(&candidate_path, error)),
    }
    Ok(())
}

fn normalized_segments(path: &str) -> Option<Vec<&str>> {
    let mut normalized = Vec::new();
    for segment in path.split('/').filter(|segment| !segment.is_empty()) {
        match segment {
            "." => {}
            ".." => {
                normalized.pop()?;
            }
            _ => normalized.push(segment),
        }
    }
    Some(normalized)
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

/// Cargo treats dangling and cyclic symlinks encountered during glob expansion as unmatched.
fn cargo_skips_symlink_target_error(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::NotFound {
        return true;
    }
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
#[must_use]
#[cfg(any(windows, test))]
pub(crate) fn is_windows_filesystem_loop_error_code(raw_os_error: Option<i32>) -> bool {
    raw_os_error == Some(1921)
}

/// Match one path component against an anchored glob containing zero or more `*` wildcards.
#[must_use]
pub fn segment_matches(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == name;
    }
    let mut cursor = 0usize;
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if index == 0 {
            if !name[cursor..].starts_with(part) {
                return false;
            }
            cursor += part.len();
        } else if index == parts.len() - 1 {
            return name[cursor..].ends_with(part) && name.len() - cursor >= part.len();
        } else {
            match name[cursor..].find(part) {
                Some(offset) => cursor += offset + part.len(),
                None => return false,
            }
        }
    }
    true
}
