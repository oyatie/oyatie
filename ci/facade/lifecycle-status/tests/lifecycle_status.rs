//! ADR-0109 lifecycle-status live-corpus gate.
//!
//! ONE required lane over EVERY config in `specs/lifecycle-configs/`. This replaces the nine
//! `tools/oya-governance-*-lifecycle-app/` dev-CLI crates, which built, had tests, and were never
//! referenced by `.github/workflows/**` — they enforced nothing for their whole lifetime.
//!
//! This file holds exactly ONE `#[test]` on purpose: it must `set_current_dir` to the repo root
//! because the ADR-0109 kernel expands source globs relative to the process working directory, and
//! that is process-global state. Splitting it would race. Pure per-dimension coverage (including
//! every RED fixture) lives in the library unittest target instead.
//!
//! ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use ci_lifecycle_status::{LaneObservation, compare, parse_policy};
use oya_governance_lifecycle_kernel::{NaiveDate, discovery, evaluate};

/// Walk up to the dir holding the canonical `specs/root-hub-pointers.json`. Mirrors the helper
/// every other gate lane uses so all lanes resolve the root identically.
fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

/// A fixed evaluation date. `now` must NOT be the wall clock: an `OverdueTransition` that appears
/// purely because the calendar advanced would turn this required lane into a time bomb that reds
/// the branch with no change to the tree. Deadline drift is real debt, but it belongs to a lane
/// that can be scheduled, not to the merge-blocking context.
const EVALUATED_AT: NaiveDate = NaiveDate::ymd(2026, 1, 1);

#[test]
fn every_lifecycle_config_is_evaluated_and_holds_its_frozen_violation_baseline() {
    let root = repo_root();
    let policy_path = root.join("ci/facade/lifecycle-status/lifecycle-status-policy.json");
    let policy = parse_policy(
        &fs::read_to_string(&policy_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", policy_path.display())),
    )
    .unwrap_or_else(|e| panic!("{}: {e}", policy_path.display()));

    let configs_dir = root.join(&policy.configs_dir);
    let mut configs: Vec<PathBuf> = fs::read_dir(&configs_dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", configs_dir.display()))
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    configs.sort();
    assert!(
        !configs.is_empty(),
        "no lifecycle configs under {} — the ADR-0109 config layout moved and this lane went blind",
        configs_dir.display()
    );

    let discovered_lanes: Vec<String> = configs
        .iter()
        .map(|path| {
            path.file_stem()
                .expect("config file_stem")
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    // The kernel expands source globs relative to the process cwd.
    std::env::set_current_dir(&root).expect("chdir to repo root");

    // Surface-all: record EVERY lane's outcome before asserting, so one broken config does not mask
    // the state of the other eight.
    let mut observations: BTreeMap<String, LaneObservation> = BTreeMap::new();
    for (lane, path) in discovered_lanes.iter().zip(&configs) {
        let config = discovery::load_config(path)
            .unwrap_or_else(|e| panic!("{lane}: load config {}: {e}", path.display()));
        // A discovery error (notably the kernel's `missing source root`, which is how a reorg move
        // silently unhooks a lane) is RECORDED, never swallowed into an empty green lane.
        let observation = match discovery::discover(&config, EVALUATED_AT) {
            Err(error) => LaneObservation::DiscoveryFailed(error),
            Ok(artifacts) => {
                let report = evaluate(&config, &artifacts, EVALUATED_AT, &[]);
                let mut violations: BTreeMap<String, usize> = BTreeMap::new();
                for violation in &report.violations {
                    *violations
                        .entry(violation.kind.as_str().to_owned())
                        .or_insert(0) += 1;
                }
                LaneObservation::Observed {
                    artifacts: report.artifacts_observed,
                    violations,
                }
            }
        };
        eprintln!("lifecycle-status: {lane} {observation:?}");
        observations.insert(lane.clone(), observation);
    }

    let findings = compare(&discovered_lanes, &observations, &policy);
    assert!(
        findings.is_empty(),
        "lifecycle-status gate RED ({} finding(s)) against {}:\n{}",
        findings.len(),
        policy_path.display(),
        findings
            .iter()
            .map(|finding| format!("  - {}", finding.message()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
