// §2.5#4 cloud-ci-bnf-layer-suffix: born-blocking self-test over TODAY's real corpus.
// Per the firewall doctrine ("a firewall that doesn't block today is the facade we're killing"),
// this runs the producer `--face bnf-layer-suffix` to resolve the live first-party oya-* crate
// names, then asserts the gate FIRES — there are non-canonical trailing segments in the tree
// today (the ~79 BNF-debt crates, baseline-block-on-new, burned down before L1 office). The
// count is MEASURED, not hardcoded. ADR-0083 Tier-3: integration tests assert via unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ci_corpus_census_adapters::{assert_census_matches, independent_oya_prefix_census};
use serde_json::Value;

use ci_crate_layer_suffix::{Verdict, evaluate, evaluate_keyed};

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

fn producer_binary(root: &Path, producer_bin: Option<&str>) -> Result<PathBuf, String> {
    let Some(bin) = producer_bin else {
        return Err(
            "FAIL-CLOSED: missing OYA_CI_PRODUCER_BIN; Cargo fallback is forbidden".to_owned(),
        );
    };
    ci_path_resolver_adapters::resolve_cargo_test_binary(root, std::ffi::OsStr::new(bin))
}

#[test]
fn producer_binary_env_is_required_for_hermetic_gate() {
    let err = producer_binary(Path::new("/repo"), None)
        .expect_err("missing OYA_CI_PRODUCER_BIN must fail closed");
    assert!(err.contains("OYA_CI_PRODUCER_BIN"));
}

/// Run the producer to emit a single face to stdout, HERMETICALLY. The producer binary must be
/// provided by `OYA_CI_PRODUCER_BIN`; missing env fails closed so tests cannot silently fall back to
/// Cargo. The producer reads the materialized scm-facts face (a declared input); it never calls git.
fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json");
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let bin = producer_binary(root, producer_bin.as_deref()).unwrap_or_else(|e| panic!("{e}"));
    let output = Command::new(bin)
        .arg("--repo-root")
        .arg(root)
        .arg("--scm-facts")
        .arg(&scm_facts)
        .arg("--stdout")
        .arg("--face")
        .arg(face)
        .current_dir(root)
        .output()
        .expect("run producer binary");
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}

#[test]
fn bnf_layer_suffix_is_born_blocking_on_the_live_corpus() {
    let root = repo_root();
    let face = run_producer_face(&root, "bnf-layer-suffix");
    let rows = face["rows"].as_array().expect("bnf face rows");
    let face_names: BTreeSet<String> = rows
        .iter()
        .map(|r| r["crate_name"].as_str().expect("crate_name").to_owned())
        .collect();

    // INDEPENDENT DYNAMIC CENSUS (not a hardcoded magnitude floor, and not a bare non-empty
    // check): re-derive the live oya-* crate set via the canonical workspace-member resolver +
    // a from-scratch parse, then assert EXACT set equality against the face. Self-adjusts
    // through future de-brands (the census shrinks in lockstep with the face) while catching
    // scan-root/glob/prefix/parse/truncation/exclusion regressions a magnitude floor cannot: a
    // producer that silently drops even ONE eligible crate is caught as a set difference, not
    // masked by "some debt survived."
    //
    // The census and its assertion live in ci/adapters/corpus-census — SHARED, not inlined. This
    // gate and ci/facade/package-manifest-hygiene each carried a verbatim copy of both, proofs
    // included; two copies of a control is two places for it to drift. The should_panic proofs
    // that make it trustworthy (empty face, near-empty face, exactly-one-missing, plus the
    // fail-closed unreadable/unparseable-manifest fixtures) moved with it and are that crate's
    // own tests.
    let census = independent_oya_prefix_census(&root);
    assert_census_matches(&face_names, &census);

    let findings = evaluate_keyed(&face);
    let unknown_role = findings
        .iter()
        .filter(|f| f.code == "bnf_unknown_role")
        .count();

    eprintln!(
        "BORN-BLOCKING bnf-layer-suffix: oya-* crates={} total_findings={} bnf_unknown_role={}",
        rows.len(),
        findings.len(),
        unknown_role
    );

    assert_eq!(
        evaluate(&face).verdict,
        Verdict::Red,
        "GATE must go RED on today's corpus (non-canonical trailing segments exist)"
    );
    assert!(
        unknown_role > 0,
        "the live corpus must surface at least one non-canonical layer suffix (bnf_unknown_role)"
    );
}
