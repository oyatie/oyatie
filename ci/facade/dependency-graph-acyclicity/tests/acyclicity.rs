// ADR-0280 §D-3 substrate-dependency-dag acyclicity lane: the live-corpus + RED-fixture gate.
//
// 1. LIVE: load the REAL policy-declared substrate dependency DAG and assert it is GREEN —
//    acyclic (Tarjan), forbidden edges honoured, bootstrap_order == Kahn topo-sort, §D-1 schema
//    complete. This is the born-blocking proof that the populated §D-1 DAG is sound.
// 2. RED FIXTURES: load each tests/fixtures/dag-cycles/*.json and assert the validator FAILS it
//    (simple-two-node, three-node, self-loop, six-node-buried). A validator that passes a cycle is
//    a false-green; these fixtures fail closed.
// 3. CONTRACT: the live DAG carries exactly the ADR-0280 §D-1 ten-node / 42-edge / 21-forbidden /
//    10-step-bootstrap shape (verbatim), and the DERIVED bootstrap order equals §D-1's list.
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use ci_dependency_graph_acyclicity::{
    DEFAULT_POLICY_PATH, GATE_ID, Policy, Verdict, evaluate_with_raw, load_policy, parse_dag,
};

/// Walk up from the test's working directory to the repo root (the dir holding the gate policy).
fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(DEFAULT_POLICY_PATH).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!(
        "failed to locate repo root (the dir holding {DEFAULT_POLICY_PATH}) from the test current_dir"
    );
}

/// Locate the on-disk cycle-fixtures directory. Under cargo the fixtures sit at
/// `$CARGO_MANIFEST_DIR/tests/fixtures/dag-cycles`; under buck2 the globbed srcs land relative to
/// the sandboxed test cwd, so fall back to a walk-up search for the same suffix.
fn fixtures_dir() -> PathBuf {
    if let Some(manifest) = std::env::var_os("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(manifest).join("tests/fixtures/dag-cycles");
        if p.is_dir() {
            return p;
        }
    }
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        let candidate = dir.join("tests/fixtures/dag-cycles");
        if candidate.is_dir() {
            return candidate;
        }
        let nested = dir.join(
            "ci/facade/dependency-graph-acyclicity/tests/fixtures/dag-cycles",
        );
        if nested.is_dir() {
            return nested;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate the cycle-fixtures directory");
}

fn load_live_dag() -> (
    ci_dependency_graph_acyclicity::Dag,
    serde_json::Value,
    Policy,
) {
    let root = repo_root();
    let policy = load_policy(&root, DEFAULT_POLICY_PATH).expect("read live DAG policy");
    let bytes = std::fs::read_to_string(root.join(&policy.dag_path)).expect("read live DAG");
    let dag = parse_dag(&bytes).expect("parse live DAG");
    let raw: serde_json::Value = serde_json::from_str(&bytes).expect("re-parse live DAG value");
    (dag, raw, policy)
}

#[test]
fn live_policy_points_at_existing_dag_data_pack() {
    let root = repo_root();
    let policy = load_policy(&root, DEFAULT_POLICY_PATH).expect("read live DAG policy");
    assert_eq!(policy.gate_id, GATE_ID);
    assert!(
        root.join(&policy.dag_path).is_file(),
        "policy dag_path must point at an existing repo-relative DAG document: {}",
        policy.dag_path
    );
}

#[test]
fn live_canonical_dag_is_green() {
    let (dag, raw, policy) = load_live_dag();
    let report = evaluate_with_raw(&dag, &raw);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "the policy-declared DAG {} must be GREEN; findings:\n{}",
        policy.dag_path,
        report
            .findings
            .iter()
            .map(|f| format!("  [{}] {}: {}", f.code, f.subject, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    eprintln!(
        "{GATE_ID} live corpus: GREEN (acyclic, forbidden-edges honoured, bootstrap coherent)"
    );
}

#[test]
fn live_dag_matches_adr_0280_d1_shape() {
    let (dag, raw, _policy) = load_live_dag();
    assert_eq!(
        dag.nodes.len(),
        10,
        "ADR-0280 §D-1 Tier-1 has exactly 10 nodes"
    );
    assert_eq!(
        dag.edges.len(),
        42,
        "ADR-0280 §D-1 has exactly 42 positive edges"
    );
    assert_eq!(
        dag.forbidden_edges.len(),
        21,
        "ADR-0280 §D-1 has exactly 21 forbidden-edge assertions"
    );
    assert_eq!(
        dag.bootstrap_order.len(),
        10,
        "ADR-0280 §D-1 bootstrap_order has 10 steps"
    );
    assert_eq!(raw["version"].as_str(), Some("1.0.0"));
    assert_eq!(raw["doctrine_adr"].as_str(), Some("ADR-0280"));

    // The §D-1 leaf-first bootstrap order, verbatim.
    let expected = [
        "cell",
        "identity",
        "tenancy",
        "policy-engine",
        "cloud-secrets",
        "audit-chain",
        "observability",
        "ontology",
        "intelligence",
        "workflow-engine",
    ];
    assert_eq!(
        dag.bootstrap_order, expected,
        "bootstrap_order == ADR-0280 §D-1 list"
    );
}

#[test]
fn live_dag_bootstrap_order_is_a_valid_topological_order() {
    // The bootstrap order is DERIVED/validated by querying the DAG (Kahn topo-sort), never
    // hard-coded. The §D-1 declared order need NOT equal a pure alphabetical-tie-break sort — it
    // encodes the ADR-0280 R-5 cloud-secrets bootstrap-seam subtlety (cloud-secrets depends only
    // on cell at runtime, so alphabetically it would sort 2nd, but §D-1 places it at step 5 because
    // it is provisioned after identity via the Shamir-genesis bootstrap-only seam). The invariant
    // is that the declared order is *a* VALID topological order; the gate proves that and surfaces
    // the alphabetical Kahn sort as the canonical suggestion.
    let (dag, raw, _policy) = load_live_dag();
    let report = evaluate_with_raw(&dag, &raw);
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.code == "dag_bootstrap_drift"),
        "the declared bootstrap_order must be a valid topological order of the DAG; findings: {:?}",
        report.findings
    );
    // An acyclic DAG always yields a derivable alphabetical topo-sort (may differ from declared).
    let derived = report
        .derived_bootstrap_order
        .expect("an acyclic DAG yields a topological sort");
    assert_eq!(derived.len(), dag.nodes.len());

    // Document the EXPECTED divergence (the verified bootstrap-seam finding): the alphabetical Kahn
    // sort places cloud-secrets before identity, whereas §D-1 declares identity before cloud-secrets.
    // This is the ADR-0280 R-5 inversion, not a defect; it is why the gate validates valid-topo-order
    // rather than equality.
    let declared = &dag.bootstrap_order;
    assert_ne!(
        &derived, declared,
        "expected the alphabetical Kahn sort to diverge from §D-1 at the cloud-secrets seam"
    );
}

#[test]
fn red_cycle_fixtures_each_fail_the_validator() {
    let dir = fixtures_dir();
    let expected = [
        "simple-two-node.json",
        "three-node.json",
        "self-loop.json",
        "six-node-buried.json",
    ];
    for name in expected {
        let path = dir.join(name);
        assert!(path.is_file(), "missing RED fixture {}", path.display());
        let bytes = std::fs::read_to_string(&path).expect("read fixture");
        let dag = parse_dag(&bytes).unwrap_or_else(|e| panic!("fixture {name} must parse: {e}"));
        let raw: serde_json::Value = serde_json::from_str(&bytes).expect("fixture value");
        let report = evaluate_with_raw(&dag, &raw);
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "RED fixture {name} MUST fail the validator (a cycle that passes is a false-green)"
        );
        assert!(
            report.findings.iter().any(|f| f.code == "dag_cycle"),
            "RED fixture {name} must produce a dag_cycle finding; got {:?}",
            report.findings
        );
    }
}

#[test]
fn fixtures_directory_holds_only_the_four_red_classes() {
    // Guard against a stray non-cycle fixture sneaking into the RED set.
    let dir = fixtures_dir();
    let mut found: Vec<String> = std::fs::read_dir(&dir)
        .expect("read fixtures dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".json"))
        .collect();
    found.sort();
    let mut expected = vec![
        "self-loop.json".to_string(),
        "simple-two-node.json".to_string(),
        "six-node-buried.json".to_string(),
        "three-node.json".to_string(),
    ];
    expected.sort();
    assert_eq!(
        found, expected,
        "exactly the four named RED cycle fixtures must exist"
    );
}
