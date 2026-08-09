#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let mut directory = std::env::current_dir().expect("read current directory");
    for _ in 0..16 {
        if directory
            .join("docs/programs/k8s-port/wave-registry.rdoc")
            .is_file()
        {
            return directory;
        }
        if !directory.pop() {
            break;
        }
    }
    panic!("failed to locate the repository root from the test working directory");
}

#[test]
fn live_k8s_program_document_corpus_is_green_and_nonempty() {
    let root = repo_root();
    let corpus = ci_k8s_program_docs::load_repository(&root)
        .expect("the live R-DOC corpus must load without malformed or missing inputs");
    let report = ci_k8s_program_docs::evaluate(&corpus);

    assert!(
        report.counters.scanned_population >= 6,
        "the live scan must include four program indexes and both governing ADRs"
    );
    assert_eq!(
        report.counters.finding_count, 0,
        "live R-DOC findings: {:?}",
        report.findings
    );
    assert!(report.is_green());
}

/// The corpus-token check is only worth its green if the walk actually REACHED both neutral roots.
/// A unit test cannot answer that — it hands the scan its own inputs — so the reach is asserted
/// against the live tree, once, here. This is the assertion that a narrowed scan cannot survive.
#[test]
fn the_neutral_scan_reaches_both_neutral_roots_on_the_live_tree() {
    let root = repo_root();
    let corpus = ci_k8s_program_docs::load_repository(&root).expect("the live R-DOC corpus loads");

    for prefix in ["specs/port-rules/", "build/port-engine/"] {
        assert!(
            corpus
                .neutral_artifacts
                .iter()
                .any(|artifact| artifact.path.starts_with(prefix)),
            "the neutral scan reached no file under {prefix}; scanned: {:?}",
            corpus
                .neutral_artifacts
                .iter()
                .map(|artifact| artifact.path.as_str())
                .collect::<Vec<_>>()
        );
    }
}
