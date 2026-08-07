// cloud-ci-workflow-lane-preflight gate.
//
// Two jobs, and they are deliberately different in kind:
//
//   * the LIVE half reads the frozen policy off the real tree and proves the hotfile set has not
//     rotted. A hotfile whose path no longer exists is a guard aimed at nothing, and a policy full
//     of those passes every lane while checking nothing.
//   * the META half injects the three failures this gate exists to prevent and asserts each is
//     caught. A gate that cannot demonstrate catching its own target cases is not delivered.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use ci_workflow_lane_preflight::{
    CODE_HOTFILE_CLAIMED, CODE_INVISIBLE_PATH, CODE_LANE_COLLISION, CODE_UNCOMMITTED, CODE_VACUOUS,
    LaneDeclaration, PathClaim, Policy, evaluate,
};

const POLICY_PATH: &str = "ci/facade/workflow-lane-preflight/workflow-lane-preflight-policy.json";

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(POLICY_PATH).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root (the dir holding {POLICY_PATH})");
}

fn load_policy(root: &Path) -> Policy {
    let raw = std::fs::read_to_string(root.join(POLICY_PATH)).expect("read policy");
    let doc: serde_json::Value = serde_json::from_str(&raw).expect("policy parses");
    let field = |key: &str| -> usize {
        doc[key]
            .as_u64()
            .unwrap_or_else(|| panic!("policy field {key} missing")) as usize
    };
    Policy {
        hotfiles: doc["hotfiles"]
            .as_array()
            .expect("policy field hotfiles missing")
            .iter()
            .map(|v| v.as_str().expect("hotfile is a string").to_owned())
            .collect(),
        min_expected_lanes: field("min_expected_lanes"),
        min_expected_declared_paths: field("min_expected_declared_paths"),
        min_expected_hotfiles: field("min_expected_hotfiles"),
    }
}

fn committed(lane: &str, sha: &str, paths: &[&str]) -> LaneDeclaration {
    LaneDeclaration {
        lane: lane.to_owned(),
        commit_sha: sha.to_owned(),
        commit_exists: true,
        paths: paths
            .iter()
            .map(|path| PathClaim {
                path: (*path).to_owned(),
                git_visible: true,
            })
            .collect(),
    }
}

// ---------------------------------------------------------------------------------------------
// LIVE: the frozen policy still points at real paths.
// ---------------------------------------------------------------------------------------------

#[test]
fn every_policy_hotfile_exists_on_the_real_tree() {
    let root = repo_root();
    let policy = load_policy(&root);
    let missing: Vec<&String> = policy
        .hotfiles
        .iter()
        .filter(|hot| !root.join(hot).exists())
        .collect();
    assert!(
        missing.is_empty(),
        "hotfile policy has rotted — these paths no longer exist and their guard now protects \
         nothing: {missing:?}"
    );
}

#[test]
fn the_frozen_policy_satisfies_its_own_anti_vacuity_floor() {
    let root = repo_root();
    let policy = load_policy(&root);
    // The policy is the probe. If IT collapses, every hotfile intersection is trivially empty and
    // the gate reports a clean run while measuring nothing.
    assert!(
        policy.hotfiles.len() >= policy.min_expected_hotfiles,
        "policy carries {} hotfiles, below its own floor of {}",
        policy.hotfiles.len(),
        policy.min_expected_hotfiles
    );
    assert!(policy.min_expected_hotfiles > 0, "a zero hotfile floor is not a floor");
    assert!(policy.min_expected_lanes > 0, "a zero lane floor is not a floor");
    assert!(
        policy.min_expected_declared_paths > 0,
        "a zero path floor is not a floor"
    );
}

// ---------------------------------------------------------------------------------------------
// META: the gate catches its own target cases.
// ---------------------------------------------------------------------------------------------

/// THE META-TEST. Two colliding lanes and a gitignored declaration, in one synthetic run, against
/// the REAL frozen policy. Both must be caught, or this gate has rotted into always-green.
///
/// The three lanes reproduce the run this gate was built from:
///   * `reorg-a` and `reorg-b` each declare a disjoint-looking set that in fact retargets the same
///     hotfile — failure 3, where 23 sincere disjointness claims produced 21 conflicts.
///   * `spec-lane` writes its sole output under `.omc/`, which this repo gitignores (verified:
///     `git check-ignore -v .omc/specs/x.json` -> `.gitignore:8:/.omc/*`), so the caller reports
///     `git_visible: false` — failure 2, whose clean verdict was trivially true.
#[test]
fn two_colliding_lanes_and_a_gitignored_declaration_are_both_caught() {
    let root = repo_root();
    let policy = load_policy(&root);
    let lanes = vec![
        committed(
            "reorg-a",
            "1111111111111111111111111111111111111111",
            &["specs/capability-registry.json", "oya/billing/core/src/lib.rs"],
        ),
        committed(
            "reorg-b",
            "2222222222222222222222222222222222222222",
            &["specs/capability-registry.json", "oya/meter/core/src/lib.rs"],
        ),
        LaneDeclaration {
            lane: "spec-lane".to_owned(),
            commit_sha: "3333333333333333333333333333333333333333".to_owned(),
            commit_exists: true,
            paths: vec![PathClaim {
                path: ".omc/specs/lane-output.json".to_owned(),
                git_visible: false,
            }],
        },
    ];

    let verdict = evaluate(&lanes, &policy);
    assert!(verdict.failed());

    let collisions = verdict.with_code(CODE_LANE_COLLISION);
    assert_eq!(collisions.len(), 1, "{:?}", verdict.findings);
    assert!(collisions[0].detail.contains("specs/capability-registry.json"));

    let invisible = verdict.with_code(CODE_INVISIBLE_PATH);
    assert_eq!(invisible.len(), 1, "{:?}", verdict.findings);
    assert_eq!(invisible[0].lane, "spec-lane");

    // And the hotfile rule fires independently of the collision, on BOTH reorg lanes — the
    // collision only happened because a hotfile was in an owner cell at all.
    assert_eq!(verdict.with_code(CODE_HOTFILE_CLAIMED).len(), 2);
}

/// Failure 1: three lanes wrote files, reported them, and were torn down with their worktrees.
#[test]
fn a_lane_that_declared_paths_but_never_committed_is_caught() {
    let root = repo_root();
    let policy = load_policy(&root);
    let lanes = vec![LaneDeclaration {
        lane: "critic-approved".to_owned(),
        commit_sha: String::new(),
        commit_exists: false,
        paths: vec![PathClaim {
            path: "ci/facade/some-gate/src/lib.rs".to_owned(),
            git_visible: true,
        }],
    }];
    let verdict = evaluate(&lanes, &policy);
    assert_eq!(verdict.with_code(CODE_UNCOMMITTED).len(), 1);
}

/// The dangerous shape: a run where nothing was declared reads as flawlessly disjoint.
#[test]
fn an_empty_run_fails_closed_against_the_real_policy() {
    let root = repo_root();
    let policy = load_policy(&root);
    let verdict = evaluate(&[], &policy);
    assert!(verdict.failed());
    assert!(!verdict.with_code(CODE_VACUOUS).is_empty());
}

/// The control: a run that did everything right must PASS against the real frozen policy, or the
/// tests above prove only that the gate is stuck red.
#[test]
fn a_committed_visible_disjoint_hotfile_free_run_passes() {
    let root = repo_root();
    let policy = load_policy(&root);
    let lanes = vec![
        committed(
            "lane-a",
            "1111111111111111111111111111111111111111",
            &["ci/facade/workflow-lane-preflight/src/lib.rs"],
        ),
        committed(
            "lane-b",
            "2222222222222222222222222222222222222222",
            &["oya/meter/core/src/lib.rs"],
        ),
    ];
    let verdict = evaluate(&lanes, &policy);
    assert!(!verdict.failed(), "{:?}", verdict.findings);
    assert_eq!(verdict.lanes, 2);
    assert_eq!(verdict.distinct_paths, 2);
}
