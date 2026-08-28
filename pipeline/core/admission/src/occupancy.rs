//! Path occupancy. Combine is git; this refuses spawn of overlapping
//! path-sets. Inputs are NUL-delimited git name-status records, not prompts
//! or newline-delimited API projections.

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccupiedSet {
    pub id: String,
    pub paths: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OccupancyRefused {
    EmptyPathSet,
    Overlap { path: String, other: String },
}

impl OccupancyRefused {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyPathSet => "current pull request produced an empty path-set".to_owned(),
            Self::Overlap { path, other } => {
                format!("path {path:?} already occupied by {other}")
            }
        }
    }
}

/// `in_flight` is other open PRs targeting `dev` (not this PR).
pub fn admit(this: &BTreeSet<String>, in_flight: &[OccupiedSet]) -> Result<(), OccupancyRefused> {
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
    fn base_neutral_path_admits_without_other_occupied_sets() {
        let this = set(&["pipeline/core/admission/src/owners.rs"]);
        assert_eq!(admit(&this, &[]), Ok(()));
    }

    #[test]
    fn disjoint_open_paths_admit() {
        let this = set(&["iam/core/pdp-kernel/src/lib.rs"]);
        let other = occupied("pr-1", &["storage/core/domain/src/lib.rs"]);
        assert_eq!(admit(&this, &[other]), Ok(()));
    }

    #[test]
    fn overlap_refuses() {
        let this = set(&["iam/core/pdp-kernel/src/lib.rs"]);
        let other = occupied("pr-9", &["iam/core/pdp-kernel/src/lib.rs"]);
        assert_eq!(
            admit(&this, &[other]),
            Err(OccupancyRefused::Overlap {
                path: "iam/core/pdp-kernel/src/lib.rs".into(),
                other: "pr-9".into(),
            })
        );
    }

    #[test]
    fn empty_this_refuses_closed() {
        assert_eq!(
            admit(&BTreeSet::new(), &[]),
            Err(OccupancyRefused::EmptyPathSet)
        );
    }
}
