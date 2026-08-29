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

/// The open hops `this` must yield to: those opened before it.
///
/// Ordering is by the forge-assigned pull-request number — never reused, and
/// ordered by when the hop was opened, so both sides compute it identically.
/// Comparing against *every* other open hop instead makes the rule symmetric:
/// two hops sharing one path each refuse the other, so neither can ever land
/// and the pair is breakable only by closing one. A total order has no such
/// standoff — the lowest-numbered hop finds nothing ahead of it, lands, and
/// the rest queue behind it.
pub fn hops_ahead(open: &BTreeSet<u64>, this: u64) -> Vec<u64> {
    open.range(..this).copied().collect()
}

/// `in_flight` is the open PRs ahead of this one (see [`hops_ahead`]).
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
    fn a_hop_yields_only_to_hops_opened_before_it() {
        let open = BTreeSet::from([2272, 2279, 2280, 2282]);
        assert_eq!(hops_ahead(&open, 2279), vec![2272]);
        assert_eq!(hops_ahead(&open, 2282), vec![2272, 2279, 2280]);
    }

    #[test]
    fn hops_sharing_one_path_cannot_refuse_each_other() {
        // The symmetric rule turned a shared file into a deadlock: every hop
        // saw every other, so a set all touching `Cargo.lock` refused one
        // another and none could land.
        let open = BTreeSet::from([2272, 2279, 2280, 2282]);
        assert!(
            hops_ahead(&open, 2272).is_empty(),
            "the lowest-numbered hop must always be admissible"
        );
        for number in [2279, 2280, 2282] {
            assert!(
                hops_ahead(&open, number).contains(&2272),
                "hop {number} must still yield to the one ahead of it"
            );
        }
    }

    #[test]
    fn a_hop_never_yields_to_itself_or_to_later_hops() {
        let open = BTreeSet::from([2272, 2279, 2280]);
        let ahead = hops_ahead(&open, 2279);
        assert!(!ahead.contains(&2279));
        assert!(!ahead.contains(&2280));
    }

    #[test]
    fn empty_this_refuses_closed() {
        assert_eq!(
            admit(&BTreeSet::new(), &[]),
            Err(OccupancyRefused::EmptyPathSet)
        );
    }
}
