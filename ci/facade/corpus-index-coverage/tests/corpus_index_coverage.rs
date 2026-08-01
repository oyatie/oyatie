// cloud-ci-corpus-index-coverage live-corpus gate.
//
// 1. LIVE: walk the real tree, assign every tracked-shaped YAML file to the buck2 package that
//    OWNS it (nearest ancestor BUCK), detect which of those packages declare a corpus-yaml-facts
//    extraction target, COMPUTE coverage, and evaluate against the frozen policy.
// 2. RED FIXTURE: a synthetic new uncovered package MUST fail the ratchet — the gate is proven
//    capable of failing, not merely observed passing.
// 3. FLOOR: the walk must see the real corpus, so a broken walk cannot report perfect coverage.
// 4. BASELINE FIDELITY: the frozen ceiling must equal today's uncovered count, so the ratchet has
//    no slack to absorb a regression.
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ci_corpus_index_coverage::{
    CODE_COVERAGE_REGRESSION, CODE_VACUOUS_SCAN, PackageObservation, Policy, evaluate,
};

const POLICY_PATH: &str = "ci/facade/corpus-index-coverage/corpus-index-coverage-policy.json";

/// The rule name every extraction genrule carries. A package is INDEXED iff its BUCK declares it.
const EXTRACTION_TARGET: &str = "corpus-yaml-facts";

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
        baseline_uncovered_packages: field("baseline_uncovered_packages"),
        baseline_unpackaged_yaml_files: field("baseline_unpackaged_yaml_files"),
        min_expected_yaml_packages: field("min_expected_yaml_packages"),
        min_expected_yaml_files: field("min_expected_yaml_files"),
    }
}

/// Directories never part of the source corpus: build output, vendored trees, and every dot-dir
/// (which is also what keeps sibling git worktrees under `.claude/` from being double-counted).
fn skip_dir(name: &str) -> bool {
    name.starts_with('.') || matches!(name, "buck-out" | "target" | "node_modules")
}

/// Walk the tree once, collecting every BUCK package directory and every YAML file.
fn walk(root: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut packages = Vec::new();
    let mut yamls = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                if !skip_dir(name) {
                    stack.push(path);
                }
            } else if name == "BUCK" {
                packages.push(dir.clone());
            } else if name.ends_with(".yaml") || name.ends_with(".yml") {
                yamls.push(path);
            }
        }
    }
    (packages, yamls)
}

/// Observe the live corpus: YAML-owning packages, whether each is indexed, and how many YAML files
/// belong to NO buck2 package at all.
///
/// That last number is the northstar term. It is returned separately because those files have no
/// package to be observed under — which is precisely why they are invisible to the build graph.
fn observe(root: &Path) -> (Vec<PackageObservation>, usize) {
    let (packages, yamls) = walk(root);

    let mut owned: BTreeMap<PathBuf, usize> = BTreeMap::new();
    let package_set: std::collections::BTreeSet<&PathBuf> = packages.iter().collect();
    let mut unpackaged = 0usize;

    for yaml in &yamls {
        // Ownership is NEAREST-ancestor, so a YAML file inside a nested package belongs to that
        // nested package and is never double-counted against an outer one.
        let mut cursor = yaml.parent();
        let mut found = false;
        while let Some(dir) = cursor {
            if package_set.contains(&dir.to_path_buf()) {
                *owned.entry(dir.to_path_buf()).or_default() += 1;
                found = true;
                break;
            }
            if dir == root {
                break;
            }
            cursor = dir.parent();
        }
        if !found {
            unpackaged += 1;
        }
    }

    let observations = owned
        .into_iter()
        .map(|(dir, yaml_files)| {
            let buck = std::fs::read_to_string(dir.join("BUCK")).unwrap_or_default();
            PackageObservation {
                package: dir
                    .strip_prefix(root)
                    .unwrap_or(&dir)
                    .to_string_lossy()
                    .replace('\\', "/"),
                yaml_files,
                indexed: buck.contains(&format!("name = \"{EXTRACTION_TARGET}\"")),
            }
        })
        .collect();
    (observations, unpackaged)
}

#[test]
fn live_corpus_is_within_the_frozen_ceiling() {
    let root = repo_root();
    let (observations, unpackaged) = observe(&root);
    let policy = load_policy(&root);
    let verdict = evaluate(&observations, unpackaged, &policy);

    // The measured number, printed so the burn-down is visible in the CI log rather than inferred.
    println!(
        "corpus-index-coverage: packages {}/{} indexed ({} bps); \
         files {}/{} indexed ({} bps); unpackaged={} (ceiling {}); uncovered={} (ceiling {})",
        verdict.coverage.indexed_packages,
        verdict.coverage.total_packages,
        verdict.coverage.package_coverage_bps(),
        verdict.coverage.indexed_yaml_files,
        verdict.coverage.total_yaml_files,
        verdict.coverage.file_coverage_bps(),
        verdict.coverage.unpackaged_yaml_files,
        policy.baseline_unpackaged_yaml_files,
        verdict.coverage.uncovered_packages,
        policy.baseline_uncovered_packages,
    );

    assert!(
        !verdict.failed(),
        "corpus index coverage regressed: {:#?}",
        verdict.blocking()
    );
}

// RED FIXTURE. Without this the gate is only ever observed passing, which proves nothing.
#[test]
fn a_new_uncovered_package_fails_the_ratchet() {
    let root = repo_root();
    let (mut observations, unpackaged) = observe(&root);
    let policy = load_policy(&root);

    // Exactly the change the gate exists to catch: someone adds a package with YAML in it and no
    // extraction target.
    observations.push(PackageObservation {
        package: "synthetic/new-service".to_owned(),
        yaml_files: 4,
        indexed: false,
    });

    let verdict = evaluate(&observations, unpackaged, &policy);
    assert!(verdict.failed(), "a new uncovered package must fail");
    assert!(
        verdict
            .blocking()
            .iter()
            .any(|f| f.code == CODE_COVERAGE_REGRESSION)
    );
}

// The same synthetic package, WITH an extraction target, must pass — otherwise the gate would be
// blocking all new packages rather than all new UNINDEXED ones.
#[test]
fn a_new_indexed_package_passes_the_ratchet() {
    let root = repo_root();
    let (mut observations, unpackaged) = observe(&root);
    let policy = load_policy(&root);
    observations.push(PackageObservation {
        package: "synthetic/new-service".to_owned(),
        yaml_files: 4,
        indexed: true,
    });
    assert!(!evaluate(&observations, unpackaged, &policy).failed());
}

#[test]
fn the_walk_sees_the_real_corpus() {
    let root = repo_root();
    let (observations, unpackaged) = observe(&root);
    let policy = load_policy(&root);

    assert!(
        observations.len() >= policy.min_expected_yaml_packages,
        "only {} YAML-owning packages found — the walk is broken",
        observations.len()
    );
    // A walk that finds packages but no FILES is equally broken, and the unpackaged term is the one
    // the northstar ratchet rides on, so it gets its own floor.
    let packaged: usize = observations.iter().map(|o| o.yaml_files).sum();
    assert!(packaged >= 100, "only {packaged} YAML files attributed to packages");
    assert!(
        unpackaged >= policy.min_expected_yaml_files / 2,
        "only {unpackaged} unpackaged YAML files — the out-of-package census collapsed"
    );
}

// A vacuous walk must fail closed rather than report perfect coverage. Proven against the LIVE
// policy so the floor is the real one, not a test-local invention.
#[test]
fn a_vacuous_scan_fails_against_the_live_policy() {
    let root = repo_root();
    let policy = load_policy(&root);
    let verdict = evaluate(&[], 0, &policy);
    assert!(verdict.failed());
    assert!(verdict.blocking().iter().any(|f| f.code == CODE_VACUOUS_SCAN));
}

// The ceiling must have NO slack: a ratchet frozen above today's number silently absorbs the next
// regression.
#[test]
fn the_frozen_ceilings_equal_todays_counts() {
    let root = repo_root();
    let (observations, unpackaged) = observe(&root);
    let policy = load_policy(&root);
    let verdict = evaluate(&observations, unpackaged, &policy);
    assert_eq!(
        verdict.coverage.uncovered_packages, policy.baseline_uncovered_packages,
        "lower baseline_uncovered_packages to {} so the ratchet keeps biting",
        verdict.coverage.uncovered_packages
    );
    assert_eq!(
        verdict.coverage.unpackaged_yaml_files, policy.baseline_unpackaged_yaml_files,
        "lower baseline_unpackaged_yaml_files to {} so the northstar ratchet keeps biting",
        verdict.coverage.unpackaged_yaml_files
    );
}
