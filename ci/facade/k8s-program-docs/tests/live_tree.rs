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

/// The leaf census against the real tree. The denominator is enumerated, so this asserts it is not
/// vacuous before trusting the equality the evaluation checks.
#[test]
fn live_seam_leaf_census_is_enumerated_not_declared() {
    let root = repo_root();
    let corpus = ci_k8s_program_docs::load_repository(&root)
        .expect("the live R-DOC corpus must load without malformed or missing inputs");

    assert!(
        corpus.crate_leaves.len() >= 18,
        "the leaf scan enumerated {} crate leaves under k8s/; a collapsed denominator makes the classification meaningless",
        corpus.crate_leaves.len()
    );
    assert_eq!(
        corpus.declared_leaves.total_leaves,
        corpus.crate_leaves.len(),
        "the declared leaf census does not equal the tree at HEAD"
    );
    assert_eq!(
        corpus.declared_leaves.rows.len(),
        corpus.crate_leaves.len(),
        "declared leaf rows do not equal the enumerated crate leaves"
    );
}

/// INV-3 against the real tree, with the positive control the negative claim needs (trap T-2).
#[test]
fn retired_os_tree_has_zero_upstream_emit_sites() {
    let root = repo_root();
    let corpus = ci_k8s_program_docs::load_repository(&root)
        .expect("the live R-DOC corpus must load without malformed or missing inputs");

    assert!(
        corpus.os_rust_files == 0,
        "retired os/ unexpectedly contributed {} Rust files",
        corpus.os_rust_files,
    );
    assert!(
        corpus.upstream_emit_sites == ci_k8s_program_docs::UPSTREAM_EMIT_SITE_CEILING,
        "retired os/ contributes {} upstream-Kubernetes apiVersion emit sites, not {}",
        corpus.upstream_emit_sites,
        ci_k8s_program_docs::UPSTREAM_EMIT_SITE_CEILING
    );

    assert!(!root.join("os").exists(), "retired os/ root reappeared");
}
