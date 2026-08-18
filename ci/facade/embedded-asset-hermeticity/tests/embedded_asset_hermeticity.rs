// ADR-0545 embedded-asset hermeticity: born-blocking self-test over TODAY's real Rust + BUCK corpus.
// The test collects every include_str!/include_bytes! string-literal site, binds it to the BUCK
// targets that compile its file, resolves the include-relative sandbox path, and asserts:
//   * embedded_asset_unmapped_include and embedded_asset_policy_gate_id_mismatch are EMPTY on the
//     live corpus (born-blocking) — any new occurrence fails closed. The webhook cedar adapter
//     (FRIC-1781131000's second instance) was fixed as a prerequisite, so unmapped is empty today.
//   * each skip_* bucket equals the committed shrink-only baseline EXACTLY (set equality) and stays
//     under an independent reviewed ceiling (FRIC-1781112000 anti-laundering): a NEW skip key is
//     born-blocking (not in the baseline), a fixed one shrinks the baseline in the SAME PR.
// RED fixtures prove the unmapped class fails closed without a filesystem; a GREEN sibling proves the
// corrected mapping passes (the FRIC-1781131000 shape, pre-fix RED / post-fix GREEN).
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ci_embedded_asset_hermeticity::{Finding, Verdict, collect_observed, evaluate, evaluate_keyed};
use serde_json::{Value, json};

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
    root.join("ci/facade/embedded-asset-hermeticity")
}

fn load_json(path: &Path) -> Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn keys_for(findings: &BTreeSet<Finding>, code: &str) -> BTreeSet<String> {
    findings
        .iter()
        .filter(|finding| finding.code == code)
        .map(|finding| finding.key.clone())
        .collect()
}

fn baseline_keys(baseline: &Value, code: &str) -> BTreeSet<String> {
    baseline["codes"][code]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// The skip codes the live-repo test baselines (shrink-only). Mirrors `SKIP_CODES` in the crate; an
/// independent literal list here is the review-visible contract (a drift between them is caught by
/// the crate's `violation_and_skip_codes_are_disjoint_and_emittable` unit test).
const BASELINED_SKIP_CODES: [&str; 5] = [
    "skip_non_literal_argument",
    "skip_absolute_literal",
    "skip_build_output_path",
    "skip_no_owning_target",
    "skip_buck_unparseable",
];

#[test]
fn live_corpus_is_hermetic_and_skips_match_baseline() {
    let root = repo_root();
    let policy = load_json(&gate_dir(&root).join("embedded-asset-hermeticity-policy.json"));
    let baseline = load_json(&gate_dir(&root).join("embedded-asset-hermeticity-baseline.json"));

    let observed = collect_observed(&root, &policy)
        .expect("read-only include scan should not need temp files or cleanup");
    let site_count = observed["sites"].as_array().expect("sites").len();
    let floor = baseline["_provenance"]["sites_floor"]
        .as_u64()
        .expect("baseline _provenance.sites_floor") as usize;
    assert!(
        site_count >= floor,
        "the live corpus should carry at least the committed site floor; got {site_count} < {floor}"
    );

    let findings = evaluate_keyed(&policy, &observed);

    // 1. Born-blocking-empty codes: NO key allowed on the live corpus. The webhook cedar adapter was
    //    fixed as a prerequisite, so unmapped is empty today; any new unmapped include fails closed.
    for code in [
        "embedded_asset_unmapped_include",
        "embedded_asset_policy_gate_id_mismatch",
    ] {
        let keys = keys_for(&findings, code);
        assert!(
            keys.is_empty(),
            "{code} is born-blocking empty on the live corpus; got {keys:?}"
        );
    }

    // 2. Shrink-only skip codes: measured legacy skip set must equal the committed baseline EXACTLY.
    for code in BASELINED_SKIP_CODES {
        let measured = keys_for(&findings, code);
        let frozen = baseline_keys(&baseline, code);
        assert_eq!(
            measured, frozen,
            "{code}: measured skip set must equal the committed baseline EXACTLY; a new key means a \
             born-blocking new skip (resolve it or add the mapping), a removed key means a fixed site \
             (shrink the baseline in this PR)"
        );
    }

    // 3. Independent reviewed ceilings (NOT derived from any generated artifact; only ever edited
    //    DOWN) — a growth tripwire on top of set-equality (FRIC-1781112000 anti-laundering).
    for code in BASELINED_SKIP_CODES {
        let measured = keys_for(&findings, code).len();
        let ceiling = baseline["_provenance"]["ceilings"][code]
            .as_u64()
            .unwrap_or(0) as usize;
        assert!(
            measured <= ceiling,
            "{code} skip debt grew past the reviewed ceiling ({measured} > {ceiling}); new skip debt \
             is born-blocking — resolve the site, do not raise the ceiling"
        );
    }

    eprintln!(
        "EMBEDDED-ASSET-HERMETICITY live corpus: sites={site_count} unmapped={} non_literal={} \
         absolute={} build_output={} no_owning_target={} buck_unparseable={}",
        keys_for(&findings, "embedded_asset_unmapped_include").len(),
        keys_for(&findings, "skip_non_literal_argument").len(),
        keys_for(&findings, "skip_absolute_literal").len(),
        keys_for(&findings, "skip_build_output_path").len(),
        keys_for(&findings, "skip_no_owning_target").len(),
        keys_for(&findings, "skip_buck_unparseable").len(),
    );

    // The live corpus is GREEN (no blocking findings) after the prerequisite webhook fix.
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);
}

#[test]
fn policy_gate_id_matches_the_crate_contract() {
    let root = repo_root();
    let policy = load_json(&gate_dir(&root).join("embedded-asset-hermeticity-policy.json"));
    assert_eq!(
        policy["gate_id"].as_str(),
        Some("cloud-ci-embedded-asset-hermeticity")
    );
}

// ---------------------------------------------------------------------------
// RED / GREEN fixtures: the FRIC-1781131000 class fails closed without a filesystem.
// ---------------------------------------------------------------------------

fn fixture_policy() -> Value {
    json!({
        "gate_id": "cloud-ci-embedded-asset-hermeticity",
        "scan_roots": ["x"],
        "embedded_extensions": ["cedar", "json", "txt"],
        "out_of_scope_path_prefixes": ["../../../out/"]
    })
}

/// The FRIC-1781131000 shape as observed rows: an unmapped include resolving to a sandbox path the
/// target's destinations do NOT contain (the original defect) vs the corrected sibling (mapped).
#[test]
fn red_unmapped_include_fails_closed_green_when_mapped() {
    // Pre-fix: include resolves to a sandbox path NOT in destinations -> unmapped -> RED.
    let red = json!({ "sites": [
        {
            "key": "adapter::src/lib.rs::../../../policy/x.cedar",
            "status": "unmapped",
            "detail": "resolves to oya/svc/policy/x.cedar not in destinations"
        }
    ]});
    let report = evaluate(&fixture_policy(), &red);
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .violations
            .contains("embedded_asset_unmapped_include")
    );

    // Post-fix: the corrected mapping makes the resolved path a destination member -> resolved -> GREEN.
    let green = json!({ "sites": [
        { "key": "adapter::src/lib.rs::../../../policy/x.cedar", "status": "resolved" }
    ]});
    assert_eq!(evaluate(&fixture_policy(), &green).verdict, Verdict::Green);
}

#[test]
fn red_include_bytes_unmapped_fails_closed() {
    let observed = json!({ "sites": [
        { "key": "bin::src/main.rs::data/blob.bin", "status": "unmapped", "detail": "not mapped" }
    ]});
    let findings = evaluate_keyed(&fixture_policy(), &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "embedded_asset_unmapped_include"
                && f.key == "bin::src/main.rs::data/blob.bin")
    );
}

#[test]
fn skips_are_surfaced_but_never_red() {
    let observed = json!({ "sites": [
        { "key": "a.rs:1", "status": "skip_non_literal_argument" },
        { "key": "b.rs:2", "status": "skip_build_output_path" },
        { "key": "c.rs:3", "status": "skip_no_owning_target" }
    ]});
    let report = evaluate(&fixture_policy(), &observed);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "skips never flip the verdict"
    );
    assert_eq!(
        evaluate_keyed(&fixture_policy(), &observed).len(),
        3,
        "but every skip is surfaced"
    );
}
