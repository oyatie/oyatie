//! Path occupancy. Combine is git; this refuses spawn of overlapping
//! path-sets and serializes hubs. Inputs are NUL-delimited git name-status
//! records, not prompts or newline-delimited API projections.

use std::collections::BTreeSet;

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
    "pipeline/core/admission/src/git_change.rs",
    "pipeline/core/admission/src/layout.rs",
    "pipeline/core/admission/src/layout/inner.rs",
    "pipeline/core/admission/src/lib.rs",
    "pipeline/core/admission/src/occupancy.rs",
    "pipeline/core/admission/src/bin/path-layout.rs",
    "pipeline/core/admission/src/bin/path-occupancy.rs",
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
