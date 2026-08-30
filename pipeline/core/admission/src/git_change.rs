//! Fail-closed parser for `git diff --name-status -z -M` output.
//!
//! Occupancy and layout admission need different views of the same change.
//! Occupancy claims every endpoint, including deletes and rename/copy sources.
//! Layout checks only changed paths that remain after the diff is applied, so
//! deleting legacy debt or renaming it into the canonical layout stays green.

use std::collections::BTreeSet;
use std::str;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GitChangePaths {
    pub occupied: BTreeSet<String>,
    pub layout_candidates: BTreeSet<String>,
    /// Destinations that arrived byte-identical, as a rename with similarity
    /// 100. Their content is unchanged, so content rules have nothing of this
    /// change's making to grade.
    pub moved_unchanged: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathSetParseError {
    Unterminated,
    InvalidStatus(String),
    MissingPath(String),
    EmptyPath,
    NonUtf8Path,
}

impl PathSetParseError {
    pub fn message(&self) -> String {
        match self {
            Self::Unterminated => "git name-status output is not NUL-terminated".to_owned(),
            Self::InvalidStatus(status) => format!("invalid git name-status field {status:?}"),
            Self::MissingPath(status) => {
                format!("git name-status field {status:?} is missing a path")
            }
            Self::EmptyPath => "git name-status output contains an empty path".to_owned(),
            Self::NonUtf8Path => {
                "git name-status output contains a non-UTF-8 path; refusing closed".to_owned()
            }
        }
    }
}

fn path(field: &[u8]) -> Result<String, PathSetParseError> {
    if field.is_empty() {
        return Err(PathSetParseError::EmptyPath);
    }
    str::from_utf8(field)
        .map(str::to_owned)
        .map_err(|_| PathSetParseError::NonUtf8Path)
}

/// Parse one complete NUL-delimited Git name-status stream.
///
/// Rename and copy records occupy both endpoints but expose only their
/// destination to layout admission. Deletes remain occupied but have no layout
/// candidate. Unknown, truncated, or lossy records refuse closed.
pub fn git_change_paths_from_name_status_z(
    input: &[u8],
) -> Result<GitChangePaths, PathSetParseError> {
    if input.is_empty() {
        return Ok(GitChangePaths::default());
    }
    if input.last() != Some(&0) {
        return Err(PathSetParseError::Unterminated);
    }

    let fields: Vec<&[u8]> = input[..input.len() - 1].split(|byte| *byte == 0).collect();
    let mut changes = GitChangePaths::default();
    let mut index = 0;
    while index < fields.len() {
        let status = str::from_utf8(fields[index])
            .map_err(|_| PathSetParseError::InvalidStatus("<non-UTF-8>".to_owned()))?;
        index += 1;
        let Some(code) = status.as_bytes().first().copied() else {
            return Err(PathSetParseError::InvalidStatus(status.to_owned()));
        };
        let scored = matches!(code, b'R' | b'C');
        let valid = if scored {
            status.len() > 1 && status.as_bytes()[1..].iter().all(u8::is_ascii_digit)
        } else {
            status.len() == 1 && matches!(code, b'A' | b'D' | b'M' | b'T' | b'U' | b'X' | b'B')
        };
        if !valid {
            return Err(PathSetParseError::InvalidStatus(status.to_owned()));
        }

        let Some(first) = fields.get(index) else {
            return Err(PathSetParseError::MissingPath(status.to_owned()));
        };
        let first = path(first)?;
        changes.occupied.insert(first.clone());
        index += 1;

        if scored {
            let Some(second) = fields.get(index) else {
                return Err(PathSetParseError::MissingPath(status.to_owned()));
            };
            let second = path(second)?;
            // A rename at similarity 100 moved the bytes and wrote none. A copy
            // does add a second body, so it is not treated as unchanged.
            if code == b'R' && &status[1..] == "100" {
                changes.moved_unchanged.insert(second.clone());
            }
            changes.occupied.insert(second.clone());
            changes.layout_candidates.insert(second);
            index += 1;
        } else if code != b'D' {
            changes.layout_candidates.insert(first);
        }
    }
    Ok(changes)
}

/// Compatibility view used by path occupancy callers.
pub fn paths_from_name_status_z(input: &[u8]) -> Result<BTreeSet<String>, PathSetParseError> {
    git_change_paths_from_name_status_z(input).map(|changes| changes.occupied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_identical_rename_is_recorded_as_unchanged_content() {
        let changes = git_change_paths_from_name_status_z(
            b"R100\0old/big.rs\0new/big.rs\0R087\0old/edited.rs\0new/edited.rs\0C100\0src.rs\0copy.rs\0M\0plain.rs\0",
        )
        .unwrap();
        // Only the identical rename. An edited rename still wrote content, and a
        // copy adds a second body that did not exist before.
        assert_eq!(
            changes.moved_unchanged,
            ["new/big.rs"].map(str::to_owned).into()
        );
        // Every destination is still graded for path-derived rules.
        assert!(changes.layout_candidates.contains("new/edited.rs"));
        assert!(changes.layout_candidates.contains("copy.rs"));
    }

    #[test]
    fn rename_and_copy_occupy_both_ends_but_check_only_destinations() {
        let changes = git_change_paths_from_name_status_z(
            b"R100\0old/name.rs\0new/name.rs\0C090\0source.rs\0copy.rs\0",
        )
        .unwrap();
        assert_eq!(
            changes.occupied,
            ["copy.rs", "new/name.rs", "old/name.rs", "source.rs"]
                .map(str::to_owned)
                .into()
        );
        assert_eq!(
            changes.layout_candidates,
            ["copy.rs", "new/name.rs"].map(str::to_owned).into()
        );
    }

    #[test]
    fn deletion_is_occupied_without_becoming_a_layout_candidate() {
        let changes = git_change_paths_from_name_status_z(b"D\0legacy/path.rs\0").unwrap();
        assert!(changes.occupied.contains("legacy/path.rs"));
        assert!(changes.layout_candidates.is_empty());
    }

    #[test]
    fn parser_has_no_three_thousand_file_ceiling() {
        let mut input = Vec::new();
        for index in 0..=3_000 {
            input.extend_from_slice(b"A\0");
            input.extend_from_slice(format!("cap/core/item-{index}.rs").as_bytes());
            input.push(0);
        }
        let changes = git_change_paths_from_name_status_z(&input).unwrap();
        assert_eq!(changes.occupied.len(), 3_001);
        assert_eq!(changes.layout_candidates.len(), 3_001);
    }

    #[test]
    fn nul_protocol_preserves_newlines_and_tabs_in_paths() {
        let changes =
            git_change_paths_from_name_status_z(b"M\0cap/core/line\nwith\ttab.rs\0").unwrap();
        assert!(changes.occupied.contains("cap/core/line\nwith\ttab.rs"));
        assert!(
            changes
                .layout_candidates
                .contains("cap/core/line\nwith\ttab.rs")
        );
    }

    #[test]
    fn malformed_git_records_refuse_closed() {
        assert_eq!(
            git_change_paths_from_name_status_z(b"R100\0old.rs\0"),
            Err(PathSetParseError::MissingPath("R100".into()))
        );
        assert_eq!(
            git_change_paths_from_name_status_z(b"M\0unterminated.rs"),
            Err(PathSetParseError::Unterminated)
        );
        assert_eq!(
            git_change_paths_from_name_status_z(b"M\0bad-\xff.rs\0"),
            Err(PathSetParseError::NonUtf8Path)
        );
    }
}
