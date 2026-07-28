// cloud-ci-crate-catalog-coverage live-corpus gate.
//
// 1. LIVE: collect the REAL crate set (every tracked Cargo.toml `[package] name`) and the
//    REAL catalog row set (registry/catalog/*.yaml stems), evaluate against the frozen
//    baseline, and assert GREEN.
// 2. RED FIXTURE: a synthetic uncatalogued crate MUST fail. A gate only observed passing
//    has not been shown capable of failing.
// 3. BASELINE FIDELITY: every frozen entry is still genuinely uncatalogued — the baseline
//    is neither stale nor over-broad.
// 4. FLOOR: the collector must see the real corpus, so a collection bug cannot present as
//    clean coverage.
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use ci_crate_catalog_coverage::{
    Baseline, CODE_CRATE_WITHOUT_CATALOG_ROW, GATE_ID, Observed, Verdict, evaluate,
};

const POLICY_PATH: &str = "ci/facade/crate-catalog-coverage/crate-catalog-coverage-policy.json";
const CATALOG_DIR: &str = "registry/catalog";

/// Walk up from the test's working directory to the repo root (the dir holding the policy).
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
    panic!("failed to locate repo root (the dir holding {POLICY_PATH}) from the test current_dir");
}

fn load_baseline(root: &Path) -> Baseline {
    let raw = std::fs::read_to_string(root.join(POLICY_PATH)).expect("read policy");
    let doc: serde_json::Value = serde_json::from_str(&raw).expect("policy parses");
    let uncatalogued = doc["uncatalogued"]
        .as_array()
        .expect("uncatalogued array")
        .iter()
        .map(|v| v.as_str().expect("entry is a string").to_owned())
        .collect();
    let min_expected_crates = doc["min_expected_crates"]
        .as_u64()
        .expect("min_expected_crates") as usize;
    Baseline {
        uncatalogued,
        min_expected_crates,
    }
}

/// The tracked file list, from git — the same boundary every other gate uses, so an
/// untracked scratch crate cannot influence the verdict.
fn tracked_files(root: &Path) -> Vec<String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["ls-files"])
        .output()
        .expect("git ls-files");
    assert!(out.status.success(), "git ls-files failed");
    String::from_utf8(out.stdout)
        .expect("utf8")
        .lines()
        .map(str::to_owned)
        .collect()
}

/// Extract `[package] name = "..."` from a manifest. Deliberately narrow: only the
/// `name` key inside the `[package]` table, so a `[dependencies]` entry that happens
/// to be called `name` cannot be mistaken for a package declaration.
fn package_name(manifest: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = line.strip_prefix("name") {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                return Some(rest.trim().trim_matches('"').to_owned());
            }
        }
    }
    None
}

fn collect(root: &Path) -> Observed {
    let mut crates: BTreeMap<String, String> = BTreeMap::new();
    let mut catalog_rows: BTreeSet<String> = BTreeSet::new();

    for path in tracked_files(root) {
        if path.ends_with("Cargo.toml") {
            let Ok(text) = std::fs::read_to_string(root.join(&path)) else {
                continue;
            };
            if let Some(name) = package_name(&text) {
                crates.insert(name, path.clone());
            }
            continue;
        }
        if let Some(stem) = path
            .strip_prefix(&format!("{CATALOG_DIR}/"))
            .and_then(|p| p.strip_suffix(".yaml"))
        {
            if !stem.contains('/') {
                catalog_rows.insert(stem.to_owned());
            }
        }
    }
    Observed {
        crates,
        catalog_rows,
    }
}

#[test]
fn live_corpus_is_green_against_the_frozen_baseline() {
    let root = repo_root();
    let observed = collect(&root);
    let baseline = load_baseline(&root);
    let report = evaluate(&observed, &baseline);

    assert_eq!(
        report.verdict,
        Verdict::Green,
        "the live tree MUST be GREEN. Findings:\n{}",
        report
            .findings
            .iter()
            .map(|f| format!("  [{}] {}\n      {}", f.code, f.subject, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    eprintln!(
        "{GATE_ID} live corpus: GREEN — {} crates / {} catalog rows / {} baselined gaps",
        report.crates_checked,
        report.rows_checked,
        baseline.uncatalogued.len()
    );
}

#[test]
fn red_fixture_uncatalogued_crate_fails_closed() {
    // A synthetic crate with no row and no baseline entry MUST fail. This is the
    // proof the gate can fail at all — the difference between a gate and a decoration.
    let observed = Observed {
        crates: [("synthetic-uncatalogued-crate".to_owned(), "x/Cargo.toml".to_owned())]
            .into_iter()
            .collect(),
        catalog_rows: BTreeSet::new(),
    };
    let baseline = Baseline {
        uncatalogued: BTreeSet::new(),
        min_expected_crates: 0,
    };
    let report = evaluate(&observed, &baseline);
    assert_eq!(report.verdict, Verdict::Red);
    assert_eq!(report.findings[0].code, CODE_CRATE_WITHOUT_CATALOG_ROW);
}

#[test]
fn frozen_baseline_is_exactly_the_live_gap_set() {
    // Neither stale (listing a crate that now has a row, or no longer exists) nor
    // over-broad. Burn-down must shrink the baseline in the same change.
    let root = repo_root();
    let observed = collect(&root);
    let baseline = load_baseline(&root);

    let live_gaps: BTreeSet<String> = observed
        .crates
        .keys()
        .filter(|n| !observed.catalog_rows.contains(*n))
        .cloned()
        .collect();

    assert_eq!(
        live_gaps, baseline.uncatalogued,
        "the frozen baseline must equal the live uncatalogued set exactly"
    );
}

#[test]
fn collector_sees_the_real_corpus() {
    // FALSE-GREEN FLOOR: prove the collection actually walked the tree. Without this,
    // a broken collector reports zero crates, zero findings, and a clean pass.
    let root = repo_root();
    let observed = collect(&root);
    assert!(
        observed.crates.len() > 800,
        "collected only {} crates — the collector is broken",
        observed.crates.len()
    );
    assert!(
        observed.catalog_rows.len() > 700,
        "collected only {} catalog rows — the catalog scan is broken",
        observed.catalog_rows.len()
    );
}

#[test]
fn package_name_parser_ignores_non_package_tables() {
    // The narrow-parse invariant: only `[package] name`, never a key of the same
    // name in another table. Getting this wrong would invent phantom crates.
    let manifest = "\
[package]
name = \"real-crate\"

[dependencies]
some-dep = { path = \"../x\" }

[lib]
name = \"real_crate_lib\"
";
    assert_eq!(package_name(manifest), Some("real-crate".to_owned()));

    let no_package = "[dependencies]\nname = \"not-a-package\"\n";
    assert_eq!(package_name(no_package), None);
}
