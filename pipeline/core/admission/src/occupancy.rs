//! Path occupancy. Combine is git; this refuses spawn of overlapping
//! path-sets and serializes hubs. Inputs are NUL-delimited git name-status
//! records, not prompts or newline-delimited API projections.

use std::collections::BTreeSet;
use std::str;

/// Closed hub set. A hop that touches any of these is N=1 at trunk HEAD.
pub const OYATIE_HUBS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "rustfmt.toml",
    "deny.toml",
    "reindeer.toml",
    "AGENTS.md",
    "CLAUDE.md",
    "docs/AGENTS.md",
    "OWNERS",
    ".github/CODEOWNERS",
    ".github/PULL_REQUEST_TEMPLATE.md",
    ".github/branch-protection.yaml",
    ".github/workflows/presubmit.yml",
    "pipeline/core/admission/src/cadence.rs",
    "pipeline/core/admission/src/fanin.rs",
    "pipeline/core/admission/src/lib.rs",
    "pipeline/core/admission/src/occupancy.rs",
    ".gitignore",
];

/// Repo-root law is a hub even when the individual ADR filename is new.
pub const OYATIE_HUB_PREFIXES: &[&str] = &["docs/decisions/ADR-07"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccupiedSet {
    pub id: String,
    pub paths: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OccupancyRefused {
    EmptyPathSet,
    Overlap { path: String, other: String },
    HubOnStaleBase,
}

impl OccupancyRefused {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyPathSet => "current pull request produced an empty path-set".to_owned(),
            Self::Overlap { path, other } => {
                format!("path {path:?} already occupied by {other}")
            }
            Self::HubOnStaleBase => "hub hop requires merge-base == origin/dev HEAD".to_owned(),
        }
    }
}

pub fn hits_hub(paths: &BTreeSet<String>) -> bool {
    paths.iter().any(|p| {
        OYATIE_HUBS.contains(&p.as_str())
            || OYATIE_HUB_PREFIXES
                .iter()
                .any(|prefix| p.starts_with(prefix))
    })
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

/// Parse `git diff --name-status -z -M`. Rename and copy records carry two
/// paths; both are occupied so a skipped/changed classification cannot admit
/// a writer to either endpoint. Unknown, truncated, or lossy input is red.
pub fn paths_from_name_status_z(input: &[u8]) -> Result<BTreeSet<String>, PathSetParseError> {
    if input.is_empty() {
        return Ok(BTreeSet::new());
    }
    if input.last() != Some(&0) {
        return Err(PathSetParseError::Unterminated);
    }

    let fields: Vec<&[u8]> = input[..input.len() - 1].split(|b| *b == 0).collect();
    let mut paths = BTreeSet::new();
    let mut i = 0;
    while i < fields.len() {
        let status = str::from_utf8(fields[i])
            .map_err(|_| PathSetParseError::InvalidStatus("<non-UTF-8>".to_owned()))?;
        i += 1;
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

        let Some(first) = fields.get(i) else {
            return Err(PathSetParseError::MissingPath(status.to_owned()));
        };
        paths.insert(path(first)?);
        i += 1;
        if scored {
            let Some(second) = fields.get(i) else {
                return Err(PathSetParseError::MissingPath(status.to_owned()));
            };
            paths.insert(path(second)?);
            i += 1;
        }
    }
    Ok(paths)
}

/// `in_flight` is other open PRs targeting `dev` (not this PR).
pub fn admit(
    this: &BTreeSet<String>,
    in_flight: &[OccupiedSet],
    merge_base_is_trunk: bool,
) -> Result<(), OccupancyRefused> {
    if this.is_empty() {
        return Err(OccupancyRefused::EmptyPathSet);
    }
    for other in in_flight {
        if let Some(path) = this.intersection(&other.paths).next() {
            return Err(OccupancyRefused::Overlap {
                path: path.clone(),
                other: other.id.clone(),
            });
        }
    }
    if hits_hub(this) && !merge_base_is_trunk {
        return Err(OccupancyRefused::HubOnStaleBase);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(paths: &[&str]) -> BTreeSet<String> {
        paths.iter().map(|s| (*s).to_string()).collect()
    }

    fn occupied(id: &str, paths: &[&str]) -> OccupiedSet {
        OccupiedSet {
            id: id.to_owned(),
            paths: set(paths),
        }
    }

    #[test]
    fn disjoint_open_paths_admit() {
        let this = set(&["iam/core/pdp-kernel/src/lib.rs"]);
        let other = occupied("pr-1", &["storage/core/domain/src/lib.rs"]);
        assert_eq!(admit(&this, &[other], false), Ok(()));
    }

    #[test]
    fn overlap_refuses() {
        let this = set(&["iam/core/pdp-kernel/src/lib.rs"]);
        let other = occupied("pr-9", &["iam/core/pdp-kernel/src/lib.rs"]);
        assert_eq!(
            admit(&this, &[other], true),
            Err(OccupancyRefused::Overlap {
                path: "iam/core/pdp-kernel/src/lib.rs".into(),
                other: "pr-9".into(),
            })
        );
    }

    #[test]
    fn hub_on_stale_base_refuses() {
        let this = set(&["Cargo.toml"]);
        assert_eq!(
            admit(&this, &[], false),
            Err(OccupancyRefused::HubOnStaleBase)
        );
    }

    #[test]
    fn disjoint_hubs_at_trunk_admit() {
        let this = set(&["AGENTS.md"]);
        let other = occupied("pr-2", &[".github/workflows/presubmit.yml"]);
        assert_eq!(admit(&this, &[other], true), Ok(()));
    }

    #[test]
    fn hub_at_trunk_with_disjoint_open_admits() {
        let this = set(&["AGENTS.md"]);
        let other = occupied("pr-3", &["iam/core/pdp-kernel/src/lib.rs"]);
        assert_eq!(admit(&this, &[other], true), Ok(()));
    }

    #[test]
    fn empty_this_refuses_closed() {
        assert_eq!(
            admit(&BTreeSet::new(), &[], false),
            Err(OccupancyRefused::EmptyPathSet)
        );
    }

    #[test]
    fn rename_occupies_both_ends_and_refuses_an_old_path_editor() {
        let this = paths_from_name_status_z(b"R100\0old/name.rs\0new/name.rs\0").unwrap();
        let other = occupied("pr-7", &["old/name.rs"]);
        assert_eq!(
            admit(&this, &[other], true),
            Err(OccupancyRefused::Overlap {
                path: "old/name.rs".into(),
                other: "pr-7".into(),
            })
        );
        assert!(this.contains("new/name.rs"));
    }

    #[test]
    fn parser_has_no_three_thousand_file_ceiling() {
        let mut input = Vec::new();
        for i in 0..=3_000 {
            input.extend_from_slice(b"A\0");
            input.extend_from_slice(format!("cap/core/item-{i}.rs").as_bytes());
            input.push(0);
        }
        assert_eq!(paths_from_name_status_z(&input).unwrap().len(), 3_001);
    }

    #[test]
    fn nul_protocol_preserves_newlines_and_tabs_in_paths() {
        let paths = paths_from_name_status_z(b"M\0cap/core/line\nwith\ttab.rs\0").unwrap();
        assert!(paths.contains("cap/core/line\nwith\ttab.rs"));
    }

    #[test]
    fn malformed_git_records_refuse_closed() {
        assert_eq!(
            paths_from_name_status_z(b"R100\0old.rs\0"),
            Err(PathSetParseError::MissingPath("R100".into()))
        );
        assert_eq!(
            paths_from_name_status_z(b"M\0unterminated.rs"),
            Err(PathSetParseError::Unterminated)
        );
        assert_eq!(
            paths_from_name_status_z(b"M\0bad-\xff.rs\0"),
            Err(PathSetParseError::NonUtf8Path)
        );
    }

    #[test]
    fn all_live_repo_law_surfaces_are_hubs() {
        for hub in [
            "docs/AGENTS.md",
            "docs/decisions/ADR-0719-eac-serving-control-north-star.md",
            ".github/branch-protection.yaml",
            "pipeline/core/admission/src/lib.rs",
        ] {
            assert!(hits_hub(&set(&[hub])), "missing hub {hub}");
        }
    }
}
