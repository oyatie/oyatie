//! Live-tree caller for the competitive-benchmark gate (ADR-0062, restated live by ADR-0709
//! as the "ADR-62 residual").
//!
//! The kernel is pure; this walks the REAL PRD corpus and hands it observations as DATA.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use check_benchmark::{Prd, ViolationKind, check};
use serde_json::Value;

const POLICY_PATH: &str = "governance/check/benchmark/benchmark-policy.json";

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

fn load_policy(root: &Path) -> Value {
    let raw = std::fs::read_to_string(root.join(POLICY_PATH)).expect("read policy");
    serde_json::from_str(&raw).expect("parse policy")
}

/// Every tracked PRD markdown under both scopes, filtered to real PRDs.
///
/// `docs/prds/*.md` also contains `INDEX.md` (`doc_class: Index`), which the doctrine does not
/// govern -- it is a directory of PRDs, not a PRD. Filtering on the frontmatter `doc_class`
/// rather than excluding the filename by name is deliberate: it is the same signal a new index
/// or template file would carry, so the exclusion does not need a per-file edit to stay correct.
fn live_prds(root: &Path) -> Vec<Prd> {
    let mut paths = Vec::new();
    collect(&root.join("docs/prds"), &mut paths);
    collect(&root.join("docs/products"), &mut paths);
    paths.sort();
    paths.dedup();

    let mut prds = Vec::new();
    for path in paths {
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        let doc_class = content
            .lines()
            .find(|l| l.starts_with("doc_class:"))
            .map(|l| l.trim_start_matches("doc_class:").trim())
            .unwrap_or("");
        if doc_class != "PRD" && doc_class != "ProductRequirements" {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .expect("path under root")
            .to_str()
            .expect("utf8 path")
            .replace('\\', "/");
        prds.push(Prd { path: rel, content });
    }
    prds
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        panic!(
            "read_dir {}: expected PRD scope root to exist",
            dir.display()
        );
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".md") && (n.starts_with("PRD") || dir.ends_with("prds")))
        {
            out.push(path);
        }
    }
}

#[test]
fn live_corpus_matches_the_frozen_violation_set() {
    let root = repo_root();
    let policy = load_policy(&root);

    let prds = live_prds(&root);
    let min_expected = policy["min_expected_prds"]
        .as_u64()
        .expect("min_expected_prds") as usize;
    assert!(
        prds.len() >= min_expected,
        "observed {} PRDs, below the floor of {min_expected} -- the walk collapsed or a scope \
         root moved",
        prds.len()
    );

    let competitors: Vec<&str> = policy["known_competitors"]
        .as_array()
        .expect("known_competitors")
        .iter()
        .map(|v| v.as_str().expect("competitor token is a string"))
        .collect();

    let report = check(&prds, &competitors).expect("no empty/duplicate paths in a real walk");

    let live: BTreeSet<String> = report
        .violations
        .iter()
        .map(|v| format!("{}::{:?}", v.path, v.kind))
        .collect();

    let frozen: BTreeSet<String> = policy["frozen_violations"]
        .as_array()
        .expect("frozen_violations")
        .iter()
        .map(|v| v.as_str().expect("frozen entry is a string").to_owned())
        .collect();

    let new: Vec<&String> = live.difference(&frozen).collect();
    assert!(
        new.is_empty(),
        "{} PRD(s) missing a real Competitive Benchmark section, not in {POLICY_PATH}: {:#?}\n\
         Each is either genuinely new debt (add it to frozen_violations in this change) or a \
         section this change added (nothing to do -- it will disappear from `new` on its own).",
        new.len(),
        new,
    );

    let repaired: Vec<&String> = frozen.difference(&live).collect();
    assert!(
        repaired.is_empty(),
        "{} frozen violation(s) no longer produced: {:#?}\n\
         If the PRD was repaired, delete it from frozen_violations in {POLICY_PATH} in this \
         same change. If the corpus walk narrowed instead, that is the regression this \
         assertion exists to catch.",
        repaired.len(),
        repaired,
    );
}

#[test]
fn heading_match_is_case_insensitive() {
    // The doctrine's canonical text (ADR-0062 archived) spells the heading in TITLE case;
    // several real PRDs follow it verbatim. Regression guard for the false-negative this
    // change fixed: a title-case heading with a real digit must be OK, not SectionMissing.
    let prd = Prd {
        path: "synthetic.md".to_owned(),
        content: "# X\n\n## Competitive Benchmark\n\nBeats the incumbent by 3x on latency.\n"
            .to_owned(),
    };
    let report = check(&[prd], &[]).expect("single prd, no dup");
    assert!(
        report.violations.is_empty(),
        "title-case heading with a real digit must not be SectionMissing: {:?}",
        report.violations
    );
}

#[test]
fn a_missing_section_is_still_caught() {
    let prd = Prd {
        path: "synthetic.md".to_owned(),
        content: "# X\n\nNo benchmark section at all.\n".to_owned(),
    };
    let report = check(&[prd], &[]).expect("single prd, no dup");
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].kind, ViolationKind::SectionMissing);
}
