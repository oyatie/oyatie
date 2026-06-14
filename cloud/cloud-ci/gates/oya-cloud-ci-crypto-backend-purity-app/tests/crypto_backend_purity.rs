// ADR-0506 cloud-ci-crypto-backend-purity: born-blocking self-test over TODAY's real workspace
// activation graph plus RED/GREEN fixtures. It asserts:
//   * the LIVE workspace is born-blocking GREEN — `cargo tree -i ring --target all` shows zero
//     ACTIVATED dependents today (the zero-ring invariant holds), so any NEW activation fails
//     closed on arrival. This live leg also proves the load-bearing distinction: the Cargo.lock
//     retains an unactivated optional-dep `ring` phantom, yet the gate is GREEN because the
//     feature-resolved activation view prunes it (ADR-0506).
//   * a RED fixture (a synthetic cargo-tree activation view with a real activated ring node) makes
//     the gate FAIL — proving it genuinely catches activated ring, not an always-pass stub.
//   * the committed policy gate_id matches the crate contract.
// Pure-unit RED/GREEN fixtures live in src/lib.rs (the evaluator); these integration tests exercise
// the live collector (the only I/O) end-to-end.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use oya_cloud_ci_crypto_backend_purity_app::{
    Verdict, collect_activated_backends, evaluate, evaluate_keyed, parse_cargo_tree_dependents,
};
use serde_json::{Value, json};

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`). Verbatim from the sibling gates so collection runs from the
/// resolved repo root, not the `cargo test` CWD.
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

fn gate_dir(root: &Path) -> PathBuf {
    root.join("cloud/cloud-ci/gates/oya-cloud-ci-crypto-backend-purity-app")
}

fn load_json(path: &Path) -> Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

#[test]
fn live_workspace_is_born_blocking_green_zero_ring_activation() {
    // GREEN, end-to-end against the LIVE worktree: run the real `cargo tree -i ring --target all`
    // collector and assert zero activated ring. This is the live-tree PASS the gate must produce.
    let root = repo_root();
    let policy = load_json(&gate_dir(&root).join("crypto-backend-purity-policy.json"));

    let observed = collect_activated_backends(&root, &policy)
        .expect("read-only cargo-tree collection should succeed on the live tree");

    // The package census floor must be met (a broken CWD / cargo invocation would otherwise be a
    // silent false-green).
    let census = observed["workspace_packages_found"].as_u64().expect("census");
    let floor = policy["min_expected_workspace_packages"]
        .as_u64()
        .expect("policy floor");
    assert!(
        census >= floor,
        "the live workspace should carry at least the policy package floor ({floor}); got {census}"
    );

    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "crypto-backend-purity is born-blocking green on the live tree (zero ring activation); got {} finding(s):\n{}",
        findings.len(),
        findings
            .iter()
            .map(|f| format!("  {} {}: {}", f.code, f.key, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);

    // Prove the distinction is real and not a coincidence of an empty graph: the live Cargo.lock
    // DOES contain a `ring` stanza (the unactivated optional-dep phantom), yet the gate is GREEN.
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).expect("read Cargo.lock");
    assert!(
        lock.contains("name = \"ring\""),
        "the live Cargo.lock is expected to retain the unactivated optional-dep ring phantom; if this changed (the reqwest->hyper true-purge landed), update this assertion and the friction"
    );
    eprintln!(
        "CRYPTO-BACKEND-PURITY live: packages={census} activated-ring=0 (born-blocking green; Cargo.lock ring phantom present but unactivated — ADR-0506)"
    );
}

#[test]
fn policy_gate_id_matches_the_crate_contract() {
    let root = repo_root();
    let policy = load_json(&gate_dir(&root).join("crypto-backend-purity-policy.json"));
    assert_eq!(
        policy["gate_id"].as_str(),
        Some("cloud-ci-crypto-backend-purity")
    );
}

#[test]
fn red_fixture_activated_ring_makes_the_gate_fail() {
    // RED, hermetic (no network / no live build): a synthetic activation view modeling exactly
    // what `cargo tree -i ring --target all` prints when a crate ACTIVATES ring. The collector
    // would produce this same `observed` shape; feeding it the parsed real-cargo-tree lines proves
    // the gate FAILS on activated ring (the Torvalds bar: not an always-pass stub).
    let root = repo_root();
    let policy = load_json(&gate_dir(&root).join("crypto-backend-purity-policy.json"));

    // The exact text `cargo tree -i ring --target all` emits for an activated ring, run through the
    // real parser the collector uses.
    let cargo_tree_stdout = "\
ring v0.17.14
├── rustls v0.23.40
│   └── some-tls-adapter v0.1.0 (/repo/crates/some-tls-adapter)
│       └── some-workspace-app v0.1.0 (/repo/crates/some-workspace-app)
└── sqlx-core v0.8.0
    └── some-db-adapter v0.1.0 (/repo/crates/some-db-adapter)
";
    let activated = parse_cargo_tree_dependents(cargo_tree_stdout);
    assert!(
        !activated.is_empty(),
        "the RED fixture must parse to a non-empty activated set: {activated:?}"
    );

    let observed = json!({
        "workspace_packages_found": 200,
        "backends": [
            { "crate": "ring", "activated_dependents": activated }
        ]
    });

    let findings = evaluate_keyed(&policy, &observed);
    let ring_finding = findings
        .iter()
        .find(|f| f.code == "CBP-FORBIDDEN-BACKEND-ACTIVATED" && f.key == "ring")
        .unwrap_or_else(|| panic!("activated ring must produce a finding: {findings:#?}"));
    assert!(
        ring_finding.detail.contains("aws-lc-rs"),
        "the remediation must name the mandated replacement: {ring_finding:?}"
    );
    assert!(
        ring_finding.detail.contains("some-workspace-app")
            || ring_finding.detail.contains("some-db-adapter"),
        "the remediation must surface the real activator(s): {ring_finding:?}"
    );
    assert_eq!(
        evaluate(&policy, &observed).verdict,
        Verdict::Red,
        "an activated ring must be RED"
    );
}

#[test]
fn red_fixture_lock_superset_phantom_alone_is_green() {
    // The crux: the SAME committed policy, fed an observation whose activation view is EMPTY (what
    // `cargo tree -i ring --target all` prints — "nothing to print" — even though Cargo.lock lists
    // ring as an unactivated optional-dep phantom), is GREEN. This proves the gate does NOT
    // false-RED on the documented lock-superset phantom (ADR-0506).
    let root = repo_root();
    let policy = load_json(&gate_dir(&root).join("crypto-backend-purity-policy.json"));

    // "nothing to print" → the parser yields zero activated dependents.
    let activated = parse_cargo_tree_dependents("warning: nothing to print.\n\nhint: --target all\n");
    assert!(activated.is_empty(), "the phantom view parses to zero activated dependents");

    let observed = json!({
        "workspace_packages_found": 200,
        "backends": [
            { "crate": "ring", "activated_dependents": activated }
        ]
    });
    assert_eq!(
        evaluate(&policy, &observed).verdict,
        Verdict::Green,
        "an unactivated lock-superset phantom must be GREEN (no false-RED): {:#?}",
        evaluate_keyed(&policy, &observed)
    );
}
