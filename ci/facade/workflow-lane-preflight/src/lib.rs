//! # cloud-ci-workflow-lane-preflight — a lane result is a deliverable only if it is COMMITTED, VISIBLE and DISJOINT
//!
//! Three lane failures, all measured on real runs, all of which this gate makes impossible to
//! report as success:
//!
//! 1. **LOST DELIVERABLE.** Three lanes wrote files, reported them, and were auto-torn-down with
//!    their worktrees. Every artifact was destroyed, including the only critic-approved lane's.
//!    A lane that declares changed paths and no commit SHA has produced NOTHING, and the run
//!    must say so before the worktree is reaped, not after.
//! 2. **INVISIBLE DELIVERABLE.** One lane's sole output went to `.omc/specs/`, which is
//!    gitignored. The disjointness check reads git status, so the artifact was invisible to
//!    collision detection AND to review. Its "zero collisions" was TRIVIALLY TRUE rather than
//!    earned — the most dangerous shape a green verdict can take.
//! 3. **HOTFILE COLLISION.** 23 parallel PRs each ASSERTED path-disjointness in the PR body.
//!    1937 distinct paths, only 21 collide — but those 21 touch 20 of the 23 PRs, so 21 of 23
//!    conflicted. The assertions were true of what each PR MOVED and false of what each PR
//!    RETARGETED. Author-asserted disjointness is not evidence; the intersection is COMPUTED
//!    here, from declarations, and never read off a claim.
//!
//! ## What is DATA and what is code
//! The hotfile set is policy DATA in a sibling `workflow-lane-preflight-policy.json`, seeded with the
//! measured set. Another repo adopts this preflight by repointing those strings. Nothing in this
//! kernel names an oyatie path.
//!
//! ## PURE
//! No I/O, no clock, no rand, no subprocess. The CALLER runs `git rev-parse` / `git cat-file -e`
//! / `git check-ignore` / `git ls-files` and passes the ANSWERS in as `LaneDeclaration` data.
//! That is what lets the preflight be tested exhaustively without a repo, and — more importantly
//! — what keeps the measurement independent of the thing being measured. A gate that ran the git
//! queries itself would be trusting its own walk in exactly the place failure 2 hid.
//!
//! ## Every finding is BLOCKING
//! Unlike a debt ratchet, this gate has no legitimate backlog to burn down. Each condition below
//! is a lane that will silently produce nothing or silently conflict, so there is no advisory
//! tier and `failed()` is simply "any finding at all".
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// The gate's stable identifier.
pub const GATE_ID: &str = "cloud-ci-workflow-lane-preflight";

/// The run, a lane, or the policy itself declared nothing to check, so a green verdict would be
/// vacuous rather than earned. This is the failure-2 code: an empty declared set makes "zero
/// collisions" trivially true.
pub const CODE_VACUOUS: &str = "workflow_lane_preflight_vacuous";

/// The lane declares changed paths but no commit SHA. Its worktree is about to be reaped and the
/// work will not survive it. This is failure 1.
pub const CODE_UNCOMMITTED: &str = "workflow_lane_preflight_uncommitted";

/// The lane declares a commit SHA that does not resolve in the object store. A reported SHA that
/// nobody can `git cat-file -e` is indistinguishable from no commit at all.
pub const CODE_MISSING_COMMIT: &str = "workflow_lane_preflight_missing_commit";

/// A declared path is gitignored or otherwise not git-visible, so it is invisible both to
/// collision detection and to review. This is failure 2.
pub const CODE_INVISIBLE_PATH: &str = "workflow_lane_preflight_invisible_path";

/// A declared path is, or lives under, a POLICY hotfile — a path measured to be touched by most
/// concurrent lanes. Hotfiles never belong in an owner cell.
pub const CODE_HOTFILE_CLAIMED: &str = "workflow_lane_preflight_hotfile_claimed";

/// Two lanes in the same run declare intersecting path sets. COMPUTED, never asserted. This is
/// failure 3.
pub const CODE_LANE_COLLISION: &str = "workflow_lane_preflight_lane_collision";

/// One path a lane declares it changed, plus the caller's git answer about it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathClaim {
    /// Repo-relative path as the lane declared it.
    pub path: String,
    /// Did the CALLER confirm git can see this path? False for anything `git check-ignore`
    /// matched, and for anything absent from the commit's tree. A lane cannot self-certify this;
    /// that is the whole point of the field being an input rather than a computation.
    pub git_visible: bool,
}

/// One lane's result declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneDeclaration {
    /// Lane identifier, used only to name findings.
    pub lane: String,
    /// The commit the lane produced. Empty means the lane did not commit.
    pub commit_sha: String,
    /// Did the CALLER resolve `commit_sha` in the object store?
    pub commit_exists: bool,
    /// Every path the lane declares it changed.
    pub paths: Vec<PathClaim>,
}

/// The frozen policy. All repo-specifics are DATA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    /// Paths measured to be touched by many concurrent lanes at once. A lane declaring one of
    /// these is claiming a mutex the run cannot grant in parallel.
    pub hotfiles: Vec<String>,
    /// Anti-vacuity floor: a run with fewer lanes than this is a collapsed run, and its "no
    /// collisions" result is measurement failure rather than disjointness.
    pub min_expected_lanes: usize,
    /// Anti-vacuity floor on the declared paths across the whole run.
    pub min_expected_declared_paths: usize,
    /// Anti-vacuity floor on the POLICY. An empty or truncated hotfile list makes the hotfile
    /// intersection trivially empty — the gate would pass every lane while checking nothing.
    /// This is the guard the corpus gate learned the hard way: the dangerous failure is not a
    /// false red, it is a probe that silently sees nothing and reads as perfect.
    pub min_expected_hotfiles: usize,
    /// Lanes the run has SERIALIZED, and which may therefore claim a hotfile.
    ///
    /// The hotfile rule says "hotfiles never belong in an owner cell", and an owner cell is a
    /// member of a parallel fan-out. A lane the run has agreed to run alone is not in the fan-out,
    /// so the mutex it claims is one the run CAN grant. Without this knob the integrator lane whose
    /// entire job is to update a shared registry is blocked by the rule that exists to protect that
    /// registry, and the only remedies are to delete the hotfile entry (disarming the guard for
    /// everyone) or to stop running the gate. Naming the lane is narrower than either.
    ///
    /// Scope is deliberately ONE code: an exempt lane still fails on missing commits, invisible
    /// paths, vacuity and — critically — COLLISION. Serialization is a claim about ordering, not a
    /// licence to overlap another lane, and taking the exemption wider would re-open failure 3 for
    /// exactly the lanes touching the most contended paths in the repo.
    #[serde(default)]
    pub serialized_owner_lanes: Vec<String>,
}

/// One gate finding. Every finding blocks; see the module docs for why there is no advisory tier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// The violation code.
    pub code: String,
    /// The lane the finding is about (empty for run-wide findings).
    pub lane: String,
    /// Human-readable detail.
    pub detail: String,
}

/// The gate verdict: what was observed, plus every finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verdict {
    /// Lanes observed in this run.
    pub lanes: usize,
    /// Declared paths summed across every lane.
    pub declared_paths: usize,
    /// Distinct normalized paths, so a reader can see how much of the run is duplicated.
    pub distinct_paths: usize,
    /// All findings.
    pub findings: Vec<Finding>,
}

impl Verdict {
    /// Does the preflight fail? True iff there is any finding.
    #[must_use]
    pub fn failed(&self) -> bool {
        !self.findings.is_empty()
    }

    /// Findings carrying a given code.
    #[must_use]
    pub fn with_code<'a>(&'a self, code: &str) -> Vec<&'a Finding> {
        self.findings.iter().filter(|f| f.code == code).collect()
    }
}

/// Canonical form of a declared path: trimmed, no `./` prefix, no trailing slash.
///
/// Declarations arrive from PR bodies, `git diff --name-only`, and hand-written evidence JSON, and
/// those three disagree about decoration. Comparing undecorated strings would let `./specs/x.json`
/// and `specs/x.json` read as disjoint.
#[must_use]
pub fn normalize(path: &str) -> String {
    path.trim()
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_owned()
}

/// Does `outer` cover `inner` — same path, or `inner` sitting underneath `outer` as a directory?
///
/// Directory containment is not decoration. Failure 3's PR claims were "true of what each PR
/// MOVED and false of what each PR RETARGETED": a lane declaring the directory
/// `ci/facade/corpus-index-coverage` retargets every file under it, and exact-string comparison
/// would call that disjoint from a lane declaring one of those files. The intersection has to be
/// over what the change REACHES, not over the strings it happens to spell.
///
/// An EMPTY side reaches nothing, in either role. Without this guard the equality arm makes
/// `covers("", "")` true, so two lanes that declared a blank path — which is to say, two lanes that
/// declared nothing — read as COLLIDING with each other on a path that does not exist. That is a
/// fabricated finding about genuinely disjoint work, and it is the mirror of the false pass the
/// blank produces everywhere else. The empty string is not a path and never reaches one.
#[must_use]
pub fn covers(outer: &str, inner: &str) -> bool {
    if outer.is_empty() || inner.is_empty() {
        return false;
    }
    outer == inner || (inner.len() > outer.len() && inner.starts_with(outer) && inner.as_bytes()[outer.len()] == b'/')
}

/// Do two declared paths reach each other in either direction?
#[must_use]
pub fn overlaps(left: &str, right: &str) -> bool {
    covers(left, right) || covers(right, left)
}

/// Evaluate a run's lane declarations against the frozen policy.
///
/// PURE. `lanes` already carries the caller's git answers; nothing here touches a filesystem.
#[must_use]
pub fn evaluate(lanes: &[LaneDeclaration], policy: &Policy) -> Verdict {
    let mut findings = Vec::new();

    let hotfiles: BTreeSet<String> = policy.hotfiles.iter().map(|p| normalize(p)).collect();
    // BLANK DECLARATIONS ARE NOT PATHS. A whitespace-only or `./`-only declaration normalizes to
    // "", which reaches no hotfile and is git-visible by vacuous default, so it would sail through
    // every per-path check while still counting toward min_expected_declared_paths — one input
    // buying a false pass AND inflating the floor that exists to detect exactly that collapse.
    // They are discarded here and reported below, so no downstream set ever contains one.
    let declared: Vec<Vec<String>> = lanes
        .iter()
        .map(|lane| {
            lane.paths
                .iter()
                .map(|c| normalize(&c.path))
                .filter(|p| !p.is_empty())
                .collect()
        })
        .collect();
    let declared_paths: usize = declared.iter().map(Vec::len).sum();
    let distinct_paths = declared.iter().flatten().collect::<BTreeSet<_>>().len();

    // ANTI-VACUITY FIRST. Every verdict below is meaningless if the run, or the policy, is empty:
    // zero lanes intersect in zero paths, which is indistinguishable from perfect disjointness.
    if hotfiles.len() < policy.min_expected_hotfiles {
        findings.push(Finding {
            code: CODE_VACUOUS.to_owned(),
            lane: String::new(),
            detail: format!(
                "policy carries {} hotfiles, expected at least {} — a truncated hotfile set makes \
                 every hotfile check trivially pass while checking nothing",
                hotfiles.len(),
                policy.min_expected_hotfiles
            ),
        });
    }
    if lanes.len() < policy.min_expected_lanes {
        findings.push(Finding {
            code: CODE_VACUOUS.to_owned(),
            lane: String::new(),
            detail: format!(
                "observed {} lanes, expected at least {} — a run with no lanes has no collisions \
                 for the same reason it has no deliverables",
                lanes.len(),
                policy.min_expected_lanes
            ),
        });
    }
    if declared_paths < policy.min_expected_declared_paths {
        findings.push(Finding {
            code: CODE_VACUOUS.to_owned(),
            lane: String::new(),
            detail: format!(
                "observed {declared_paths} declared paths across the run, expected at least {} — \
                 an empty declared set makes disjointness trivially true rather than earned",
                policy.min_expected_declared_paths
            ),
        });
    }

    for (lane, paths) in lanes.iter().zip(&declared) {
        // A lane declaring nothing is failure 2's exact shape: the work went somewhere git cannot
        // see, so the lane contributes no paths, collides with nobody, and reviews clean.
        if paths.is_empty() {
            findings.push(Finding {
                code: CODE_VACUOUS.to_owned(),
                lane: lane.lane.clone(),
                detail:
                    "lane declares zero paths — a lane that changed nothing git can see produced \
                     nothing, and its clean disjointness verdict is vacuous"
                        .to_owned(),
            });
        } else if lane.commit_sha.trim().is_empty() {
            // THE OWN-PHASE COMMIT RULE, made executable. Worktrees are reaped on completion; an
            // uncommitted lane result does not survive the reap.
            findings.push(Finding {
                code: CODE_UNCOMMITTED.to_owned(),
                lane: lane.lane.clone(),
                detail: format!(
                    "lane declares {} changed paths and no commit SHA — the worktree is reaped on \
                     completion and these paths are destroyed with it",
                    paths.len()
                ),
            });
        } else if !lane.commit_exists {
            findings.push(Finding {
                code: CODE_MISSING_COMMIT.to_owned(),
                lane: lane.lane.clone(),
                detail: format!(
                    "declared commit {} does not resolve in the object store — a SHA nobody can \
                     read back is not evidence that anything was committed",
                    lane.commit_sha.trim()
                ),
            });
        }

        let serialized_owner = policy.serialized_owner_lanes.iter().any(|l| l == &lane.lane);

        for claim in &lane.paths {
            // Re-normalized rather than zipped against `declared`: the blank filter above makes the
            // two sequences different lengths, and a zip that silently truncates would drop the
            // git-visibility check for the tail of every lane that declared one.
            let path = normalize(&claim.path);
            if path.is_empty() {
                findings.push(Finding {
                    code: CODE_VACUOUS.to_owned(),
                    lane: lane.lane.clone(),
                    detail: format!(
                        "declared path {:?} normalizes to the empty string — it names nothing, so \
                         it reaches no hotfile and collides with nothing while still counting as a \
                         declaration",
                        claim.path
                    ),
                });
                continue;
            }
            if !claim.git_visible {
                findings.push(Finding {
                    code: CODE_INVISIBLE_PATH.to_owned(),
                    lane: lane.lane.clone(),
                    detail: format!(
                        "declared path {path} is not git-visible (gitignored or absent from the \
                         tree) — it is invisible to collision detection and to review alike"
                    ),
                });
            }
            // A serialized lane is not in the fan-out, so the mutex it claims is grantable. Every
            // other rule still applies to it, collision included.
            if !serialized_owner {
                for hot in &hotfiles {
                    if overlaps(&path, hot) {
                        findings.push(Finding {
                            code: CODE_HOTFILE_CLAIMED.to_owned(),
                            lane: lane.lane.clone(),
                            detail: format!(
                                "declared path {path} reaches policy hotfile {hot} — hotfiles are \
                                 touched by most concurrent lanes and never belong in an owner cell"
                            ),
                        });
                    }
                }
            }
        }
    }

    // COMPUTED intersection. ponytail: O(lanes^2) pairwise; a run is tens of lanes, not thousands.
    // If a run ever fans out far enough for this to matter, invert to a path -> lanes index.
    for left in 0..lanes.len() {
        for right in (left + 1)..lanes.len() {
            let shared: BTreeSet<&str> = declared[left]
                .iter()
                .filter(|l| declared[right].iter().any(|r| overlaps(l, r)))
                .map(String::as_str)
                .collect();
            if shared.is_empty() {
                continue;
            }
            findings.push(Finding {
                code: CODE_LANE_COLLISION.to_owned(),
                lane: lanes[left].lane.clone(),
                detail: format!(
                    "lane {} and lane {} both reach {}: {}",
                    lanes[left].lane,
                    lanes[right].lane,
                    shared.len(),
                    shared.into_iter().collect::<Vec<_>>().join(", ")
                ),
            });
        }
    }

    Verdict {
        lanes: lanes.len(),
        declared_paths,
        distinct_paths,
        findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Policy {
        Policy {
            hotfiles: vec!["specs/capability-registry.json".to_owned(), "Cargo.toml".to_owned()],
            min_expected_lanes: 1,
            min_expected_declared_paths: 1,
            min_expected_hotfiles: 2,
            serialized_owner_lanes: Vec::new(),
        }
    }

    fn lane(name: &str, sha: &str, paths: &[&str]) -> LaneDeclaration {
        LaneDeclaration {
            lane: name.to_owned(),
            commit_sha: sha.to_owned(),
            commit_exists: !sha.is_empty(),
            paths: paths
                .iter()
                .map(|p| PathClaim {
                    path: (*p).to_owned(),
                    git_visible: true,
                })
                .collect(),
        }
    }

    #[test]
    fn disjoint_committed_visible_lanes_pass() {
        let lanes = [lane("a", "aaa", &["ci/a/x.rs"]), lane("b", "bbb", &["ci/b/y.rs"])];
        let verdict = evaluate(&lanes, &policy());
        assert!(!verdict.failed(), "{:?}", verdict.findings);
        assert_eq!(verdict.declared_paths, 2);
    }

    #[test]
    fn an_empty_run_fails_closed_instead_of_reporting_perfect_disjointness() {
        let verdict = evaluate(&[], &policy());
        assert!(verdict.failed());
        // Two, not three: this fixture's hotfile list satisfies its own floor, so the lanes floor
        // and the declared-paths floor are what fire.
        assert_eq!(verdict.with_code(CODE_VACUOUS).len(), 2);
    }

    #[test]
    fn a_truncated_hotfile_policy_fails_closed() {
        let mut policy = policy();
        policy.hotfiles.clear();
        let verdict = evaluate(&[lane("a", "aaa", &["ci/a/x.rs"])], &policy);
        assert!(!verdict.with_code(CODE_VACUOUS).is_empty());
    }

    #[test]
    fn declared_paths_without_a_commit_sha_fail_closed() {
        let verdict = evaluate(&[lane("a", "", &["ci/a/x.rs"])], &policy());
        assert_eq!(verdict.with_code(CODE_UNCOMMITTED).len(), 1);
    }

    #[test]
    fn a_sha_that_does_not_resolve_fails_closed() {
        let mut only = lane("a", "deadbeef", &["ci/a/x.rs"]);
        only.commit_exists = false;
        let verdict = evaluate(&[only], &policy());
        assert_eq!(verdict.with_code(CODE_MISSING_COMMIT).len(), 1);
    }

    #[test]
    fn a_lane_declaring_nothing_is_vacuous_not_disjoint() {
        let verdict = evaluate(&[lane("a", "aaa", &[]), lane("b", "bbb", &["ci/b/y.rs"])], &policy());
        assert_eq!(verdict.with_code(CODE_VACUOUS).len(), 1);
        assert_eq!(verdict.with_code(CODE_VACUOUS)[0].lane, "a");
    }

    #[test]
    fn a_declared_hotfile_fails_closed() {
        let verdict = evaluate(&[lane("a", "aaa", &["specs/capability-registry.json"])], &policy());
        assert_eq!(verdict.with_code(CODE_HOTFILE_CLAIMED).len(), 1);
    }

    #[test]
    fn a_directory_declaration_reaching_a_hotfile_fails_closed() {
        // The retarget hole from failure 3: exact-string comparison would call this disjoint.
        let verdict = evaluate(&[lane("a", "aaa", &["specs"])], &policy());
        assert_eq!(verdict.with_code(CODE_HOTFILE_CLAIMED).len(), 1);
    }

    #[test]
    fn decoration_does_not_make_two_lanes_disjoint() {
        let lanes = [lane("a", "aaa", &["./ci/a/x.rs"]), lane("b", "bbb", &["ci/a/x.rs"])];
        assert_eq!(evaluate(&lanes, &policy()).with_code(CODE_LANE_COLLISION).len(), 1);
    }

    #[test]
    fn a_sibling_prefix_is_not_containment() {
        assert!(!overlaps("ci/facade", "ci/facade-other/x.rs"));
        assert!(overlaps("ci/facade", "ci/facade/x.rs"));
    }

    // -- Finding 2: the blank declared path -----------------------------------------------------

    #[test]
    fn a_blank_declared_path_is_vacuous_rather_than_silently_clean() {
        // Pre-fix this lane passed every per-path check: "" reaches no hotfile, and the lane was
        // neither pathless nor uncommitted, so the verdict was GREEN on a declaration naming
        // nothing.
        let verdict = evaluate(&[lane("a", "aaa", &["   "])], &policy());
        assert!(verdict.failed(), "a blank declaration must not read as a clean lane");
        assert!(
            verdict
                .with_code(CODE_VACUOUS)
                .iter()
                .any(|f| f.lane == "a" && f.detail.contains("empty string")),
            "{:?}",
            verdict.findings
        );
    }

    #[test]
    fn a_blank_declared_path_does_not_count_toward_the_declared_paths_floor() {
        // The floor exists to detect a collapsed run. A blank inflating it lets the collapse hide.
        let verdict = evaluate(&[lane("a", "aaa", &["./", "ci/a/x.rs"])], &policy());
        assert_eq!(verdict.declared_paths, 1, "{:?}", verdict.findings);
        assert_eq!(verdict.distinct_paths, 1);
    }

    #[test]
    fn two_lanes_declaring_blanks_do_not_collide_on_nothing() {
        // The false-collision arm: covers("", "") was true, so two disjoint lanes were reported as
        // sharing a path that does not exist.
        let lanes = [lane("a", "aaa", &[" ", "ci/a/x.rs"]), lane("b", "bbb", &["", "ci/b/y.rs"])];
        let verdict = evaluate(&lanes, &policy());
        assert!(
            verdict.with_code(CODE_LANE_COLLISION).is_empty(),
            "genuinely disjoint lanes fabricated a collision: {:?}",
            verdict.findings
        );
    }

    #[test]
    fn an_empty_path_reaches_nothing_in_either_direction() {
        assert!(!overlaps("", ""));
        assert!(!overlaps("", "ci/a/x.rs"));
        assert!(!overlaps("ci/a/x.rs", ""));
    }

    #[test]
    fn a_blank_path_cannot_smuggle_a_hotfile_claim_past_the_rule() {
        // Belt and braces: the blank must not be treated as a directory covering the hotfile set.
        let verdict = evaluate(&[lane("a", "aaa", &[""])], &policy());
        assert!(verdict.with_code(CODE_HOTFILE_CLAIMED).is_empty(), "{:?}", verdict.findings);
    }

    // -- Finding 3: the serialized-owner exemption ----------------------------------------------

    #[test]
    fn a_serialized_owner_lane_may_claim_the_hotfile_it_owns() {
        let mut policy = policy();
        policy.serialized_owner_lanes = vec!["registry-integrator".to_owned()];
        let verdict = evaluate(
            &[lane("registry-integrator", "aaa", &["specs/capability-registry.json"])],
            &policy,
        );
        assert!(
            verdict.with_code(CODE_HOTFILE_CLAIMED).is_empty(),
            "the lane whose job is the registry is blocked by the rule protecting it: {:?}",
            verdict.findings
        );
        assert!(!verdict.failed(), "{:?}", verdict.findings);
    }

    #[test]
    fn the_exemption_names_one_lane_and_does_not_leak_to_its_neighbours() {
        let mut policy = policy();
        policy.serialized_owner_lanes = vec!["registry-integrator".to_owned()];
        let verdict = evaluate(
            &[
                lane("registry-integrator", "aaa", &["specs/capability-registry.json"]),
                lane("some-other-lane", "bbb", &["specs/capability-registry.json"]),
            ],
            &policy,
        );
        let hot = verdict.with_code(CODE_HOTFILE_CLAIMED);
        assert_eq!(hot.len(), 1, "{:?}", verdict.findings);
        assert_eq!(hot[0].lane, "some-other-lane");
    }

    #[test]
    fn a_serialized_owner_lane_is_still_held_to_every_other_rule() {
        // Serialization is a claim about ORDERING. It is not a licence to overlap, to skip the
        // commit, or to write somewhere git cannot see.
        let mut policy = policy();
        policy.serialized_owner_lanes = vec!["integrator".to_owned()];

        let uncommitted = evaluate(&[lane("integrator", "", &["specs/capability-registry.json"])], &policy);
        assert_eq!(uncommitted.with_code(CODE_UNCOMMITTED).len(), 1);

        let mut invisible = lane("integrator", "aaa", &["specs/capability-registry.json"]);
        invisible.paths[0].git_visible = false;
        assert_eq!(evaluate(&[invisible], &policy).with_code(CODE_INVISIBLE_PATH).len(), 1);

        let collides = evaluate(
            &[
                lane("integrator", "aaa", &["specs/capability-registry.json"]),
                lane("other", "bbb", &["specs"]),
            ],
            &policy,
        );
        assert_eq!(
            collides.with_code(CODE_LANE_COLLISION).len(),
            1,
            "an exempt lane must still be checked for overlap: {:?}",
            collides.findings
        );
    }

    #[test]
    fn no_lane_is_exempt_by_default() {
        // The knob must be opt-in. A policy that omits it cannot silently disarm the hotfile rule.
        assert!(policy().serialized_owner_lanes.is_empty());
        let verdict = evaluate(&[lane("registry-integrator", "aaa", &["specs/capability-registry.json"])], &policy());
        assert_eq!(verdict.with_code(CODE_HOTFILE_CLAIMED).len(), 1);
    }

    #[test]
    fn an_absent_serialized_owner_list_deserializes_as_empty_rather_than_failing() {
        // Policy JSON predating the knob must keep parsing, and must parse as "nobody is exempt"
        // rather than refusing to load — a policy that fails to parse is a gate that does not run.
        let policy: Policy = serde_json::from_str(
            r#"{"hotfiles":["a"],"min_expected_lanes":1,"min_expected_declared_paths":1,"min_expected_hotfiles":1}"#,
        )
        .expect("policy without the knob still parses");
        assert!(policy.serialized_owner_lanes.is_empty());
    }
}
