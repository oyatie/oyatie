// Proves the sharded contract-slice-policy migration (the "every slice PR edits
// one shared file" merge-conflict class-fix): the fragment-loaded slice set is
// byte-faithful to the committed generated aggregate, and each fail-closed
// fragment-loader defect (non-JSON fragment, duplicate slice_id) is a distinct
// Finding. A per-fragment unknown-key typo is proven to still fail closed via
// the SAME existing evaluator check once the fragments are aggregated — no new
// code needed for that case, so it is proven here rather than re-implemented.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

use ci_contract_slice_conformance::{
    aggregate_policy, evaluate_configured, load_slice_fragments, render_policy_json,
};
use serde_json::{Value, json};

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current dir");
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
    root.join("ci/facade/contract-slice-conformance")
}

fn committed_policy(root: &Path) -> Value {
    let path = gate_dir(root).join("contract-slice-policy.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// The byte-faithful migration proof: materializing the committed `slices/`
/// fragments reproduces the committed `contract-slice-policy.json` bytes
/// exactly. This is the SAME rendering function
/// `oya-cloud-ci-materialize-contract-slice-policy` uses to write the file, so
/// this test is the gate half of the repo's `check_equals_fix` doctrine — the
/// fixer can never disagree with it.
#[test]
fn fragments_materialize_byte_identical_to_the_committed_aggregate() {
    let root = repo_root();
    let load = load_slice_fragments(&gate_dir(&root).join("slices"));
    assert!(
        load.findings.is_empty(),
        "the committed fragments must load cleanly: {:?}",
        load.findings
    );
    let aggregated = aggregate_policy(&load);
    let rendered = render_policy_json(&aggregated);

    let committed_path = gate_dir(&root).join("contract-slice-policy.json");
    let committed_bytes = fs::read_to_string(&committed_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", committed_path.display()));
    assert_eq!(
        rendered, committed_bytes,
        "materializing slices/*.json must reproduce contract-slice-policy.json byte-for-byte; run \
         `cargo run --bin oya-cloud-ci-materialize-contract-slice-policy` (or the buck2 bin target) \
         to resettle it"
    );

    // Same proof at the rule-set level (defense in depth: a value-equality check
    // that is independent of the chosen serialization, so a future formatter
    // change can't silently mask a real slice-set drift).
    assert_eq!(
        aggregated,
        committed_policy(&root),
        "the fragment-loaded rule set must be identical to the committed policy (same slices, same rules)"
    );
}

/// Every declared slice must have exactly one fragment file.
#[test]
fn every_committed_policy_slice_has_a_matching_fragment() {
    let root = repo_root();
    let policy = committed_policy(&root);
    let load = load_slice_fragments(&gate_dir(&root).join("slices"));
    assert!(load.findings.is_empty(), "{:?}", load.findings);
    let fragment_ids: std::collections::BTreeSet<&str> = load
        .slices
        .iter()
        .filter_map(|slice| slice["slice_id"].as_str())
        .collect();
    for slice in policy["slices"].as_array().expect("slices array") {
        let id = slice["slice_id"].as_str().expect("slice_id");
        assert!(
            fragment_ids.contains(id),
            "slice {id} has no matching slices/{id}.json fragment"
        );
    }
    assert_eq!(
        fragment_ids.len(),
        policy["slices"].as_array().expect("slices").len(),
        "fragment count must match the committed policy's slice count"
    );
}

fn write_fragment(dir: &Path, file_name: &str, content: &Value) {
    fs::write(
        dir.join(file_name),
        serde_json::to_string_pretty(content).expect("serialize fixture fragment"),
    )
    .unwrap_or_else(|e| panic!("write {file_name}: {e}"));
}

/// A fragment file that is not valid JSON must fail closed with a distinct,
/// keyed Finding rather than panicking or silently being skipped.
#[test]
fn non_json_fragment_is_red() {
    let dir = std::env::temp_dir().join(format!(
        "contract-slice-fragments-nonjson-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create temp fragments dir");
    fs::write(dir.join("broken.json"), "{ not json").expect("write broken fragment");

    let load = load_slice_fragments(&dir);
    fs::remove_dir_all(&dir).ok();

    assert!(
        load.findings
            .iter()
            .any(|f| f.code == "contract_slice_fragment_parse_error" && f.key == "broken.json"),
        "a non-JSON fragment must be a distinct, keyed finding: {:?}",
        load.findings
    );
    assert!(
        load.slices.is_empty(),
        "a broken fragment must not silently appear as a slice"
    );
}

/// A fragment that parses but has no string `slice_id` must also fail closed —
/// there is no meaningful key to aggregate it under.
#[test]
fn fragment_missing_slice_id_is_red() {
    let dir = std::env::temp_dir().join(format!(
        "contract-slice-fragments-noid-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create temp fragments dir");
    write_fragment(&dir, "noid.json", &json!({ "spec_path": "specs/x.json" }));

    let load = load_slice_fragments(&dir);
    fs::remove_dir_all(&dir).ok();

    assert!(
        load.findings
            .iter()
            .any(|f| f.code == "contract_slice_fragment_parse_error" && f.key == "noid.json"),
        "a fragment without a string slice_id must fail closed: {:?}",
        load.findings
    );
}

/// Two fragments declaring the same `slice_id` (regardless of filename) must
/// both be rejected, not silently resolved by picking one.
#[test]
fn duplicate_slice_id_across_fragments_is_red() {
    let dir = std::env::temp_dir().join(format!(
        "contract-slice-fragments-dup-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create temp fragments dir");
    let slice =
        json!({ "slice_id": "dup-slice", "spec_path": "specs/x.json", "required_fields": [] });
    write_fragment(&dir, "a-dup-slice.json", &slice);
    write_fragment(&dir, "b-dup-slice.json", &slice);

    let load = load_slice_fragments(&dir);
    fs::remove_dir_all(&dir).ok();

    assert!(
        load.findings
            .iter()
            .any(|f| f.code == "contract_slice_fragment_duplicate_slice_id" && f.key == "dup-slice"),
        "a slice_id collision across two fragment files must be rejected: {:?}",
        load.findings
    );
    assert!(
        !load.slices.iter().any(|s| s["slice_id"] == "dup-slice"),
        "neither half of a colliding pair should be silently admitted"
    );
}

/// A per-fragment key typo must still fail closed once fragments are
/// aggregated and evaluated — the SAME existing evaluator check
/// (`contract_slice_unknown_policy_key`) that already guards a hand-authored
/// policy, proven here to fire unchanged through the new fragment-loading path.
#[test]
fn unknown_key_in_a_fragment_still_fails_closed_through_the_evaluator() {
    let dir = std::env::temp_dir().join(format!(
        "contract-slice-fragments-typo-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create temp fragments dir");
    write_fragment(
        &dir,
        "typo.json",
        &json!({
            "slice_id": "typo",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_field": ["slice_id"] // typo: should be required_fields
        }),
    );

    let load = load_slice_fragments(&dir);
    fs::remove_dir_all(&dir).ok();
    assert!(
        load.findings.is_empty(),
        "a shape typo is not a load-time defect: {:?}",
        load.findings
    );

    let policy = aggregate_policy(&load);
    let corpus = std::collections::BTreeMap::from([(
        "fixtures/exemplar-slice.json".to_owned(),
        json!({ "slice_id": "exemplar" }),
    )]);
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report
            .violations
            .contains("contract_slice_unknown_policy_key"),
        "a typo'd fragment key must still be rejected once aggregated: {:?}",
        report.findings
    );
}
