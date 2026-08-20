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
        corpus.crate_leaves.len() >= 50,
        "the leaf scan enumerated {} crate leaves across k8s/ and os/; a collapsed denominator makes the classification meaningless",
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
fn live_os_tree_holds_the_frozen_upstream_emit_site_ceiling() {
    let root = repo_root();
    let corpus = ci_k8s_program_docs::load_repository(&root)
        .expect("the live R-DOC corpus must load without malformed or missing inputs");

    assert!(
        corpus.os_rust_files >= 300,
        "the INV-3 scan read {} Rust files under os/; a collapsed denominator makes the ceiling meaningless",
        corpus.os_rust_files
    );
    assert!(
        corpus.upstream_emit_sites == ci_k8s_program_docs::UPSTREAM_EMIT_SITE_CEILING,
        "os/ hand-writes {} upstream-Kubernetes apiVersion emit sites, which does not equal the frozen census of {}",
        corpus.upstream_emit_sites,
        ci_k8s_program_docs::UPSTREAM_EMIT_SITE_CEILING
    );

    // Positive control: this file really does carry the token the naive ratchet would key on...
    let control = std::fs::read_to_string(root.join("os/core/block-domain/src/controller.rs"))
        .expect("read the T-1 control file");
    assert!(
        control
            .lines()
            .filter(|line| line.contains("apiVersion:"))
            .count()
            >= 16,
        "the T-1 control file no longer carries apiVersion lines, so it controls for nothing"
    );
    // ...and none of them is Kubernetes surface, because the API group is the discriminator.
    assert_eq!(
        ci_k8s_program_docs::upstream_emit_sites(&control),
        0,
        "Talos v1alpha1 machine-config surface must never enter the INV-3 count"
    );
}
