// ADR-0546 canonical-json: born-blocking self-test over TODAY's real governed JSON corpus. The test
// loads the bundled policy, collects every governed *.json under specs/ (minus the .generated.json
// and specs/fixtures/ exclusions), canonicalizes each committed file, and asserts the live corpus is
// GREEN at ZERO baseline: every governed file already equals its canonical re-serialization, so any
// NEW non-canonical governed json is born-blocking. The per-code RED/GREEN fixtures live in the lib
// unittest and prove every violation class fails closed without a filesystem.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use ci_canonical_json::{GATE_ID, POLICY_PATH, Verdict, collect_observed, evaluate, load_policy};

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

fn load_repo_policy(root: &Path) -> serde_json::Value {
    load_policy(root, POLICY_PATH).expect("load bundled canonical-json policy")
}

#[test]
fn policy_gate_id_matches_the_crate_contract() {
    let root = repo_root();
    let policy = load_repo_policy(&root);
    assert_eq!(policy["gate_id"].as_str(), Some(GATE_ID));
}

#[test]
fn live_governed_corpus_is_canonical_at_zero_baseline() {
    let root = repo_root();
    let policy = load_repo_policy(&root);
    let observed = collect_observed(&root, &policy).expect("collect governed corpus");

    // Sanity: the walk found a meaningful corpus (specs/ carries hundreds of tracked json).
    assert!(
        observed.files.len() >= 100,
        "expected the governed specs corpus to carry >=100 files; got {}",
        observed.files.len()
    );

    let report = evaluate(&policy, &observed);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "the governed JSON corpus must be canonical at zero baseline; non-canonical files:\n{}",
        report
            .findings
            .iter()
            .map(|f| format!("  {} {}: {}", f.code, f.key, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );

    eprintln!(
        "CANONICAL-JSON live corpus: governed_files={} findings=0 (born-blocking green, zero baseline)",
        observed.files.len()
    );
}

#[test]
fn exclusions_keep_generated_faces_and_fixtures_out_of_scope() {
    let root = repo_root();
    let policy = load_repo_policy(&root);
    let observed = collect_observed(&root, &policy).expect("collect governed corpus");
    for file in &observed.files {
        assert!(
            !file.path.ends_with(".generated.json"),
            "generated faces must be excluded (owned by freshness): {}",
            file.path
        );
        assert!(
            !file.path.starts_with("specs/fixtures/"),
            "specs/fixtures are owned by their consuming gates and must be excluded: {}",
            file.path
        );
        assert!(
            file.path.starts_with("specs/") || file.path.starts_with("governance/"),
            "only governed roots are collected: {}",
            file.path
        );
    }
}

#[test]
fn root_hub_pointers_is_canonical_the_friction_exemplar() {
    // FRIC-1781130000's exemplar file: after this PR's fix it must be canonical (literal UTF-8), so
    // a future lane re-encoding it (either direction) is born-blocking.
    let root = repo_root();
    let policy = load_repo_policy(&root);
    let observed = collect_observed(&root, &policy).expect("collect governed corpus");
    let report = evaluate(&policy, &observed);
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.key == "specs/root-hub-pointers.json"),
        "the FRIC-1781130000 exemplar must be canonical after this PR"
    );
}
