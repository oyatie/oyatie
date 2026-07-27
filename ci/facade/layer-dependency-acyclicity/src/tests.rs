//! Pure-kernel unit tests (no filesystem): tier-rule classification, S-rank inversion, Tarjan cycle
//! detection, the frozen-baseline split (baselined advisory vs blocking regression), the enforcement
//! flip, the false-green floor, and the cargo/buck dep-string extraction surfaces.
//!
//! ADR-0083 Tier-3: tests use unwrap/expect/panic to assert invariants.

use super::*;

/// A minimal valid policy (born advisory-baseline).
fn policy() -> Value {
    json!({
        "gate_id": GATE_ID,
        "enforcement": "advisory-baseline",
        "crate_root_globs": ["cloud/*/crates/oya-*"],
        "service_roots": ["cloud", "oya"],
        "unclassified_roots": ["libs", "tools", "cloud/cloud-ci"],
        "stratum_rank_order": ["S0", "S1", "S2", "S3", "S4", "S5"],
        "min_expected_crates": 0
    })
}

fn baseline(violations: &[(&str, &str)]) -> Value {
    json!({
        "gate_id": GATE_ID,
        "violations": violations
            .iter()
            .map(|(c, s)| json!({ "code": c, "subject": s }))
            .collect::<Vec<_>>()
    })
}

/// Build an observed corpus: crates with owning service + service tiers + edges.
fn corpus(
    crates: &[(&str, &str)],               // (crate_dir, owning_service)
    tiers: &[(&str, &str, Option<&str>)],  // (service, tier, stratum)
    edges: &[(&str, &str)],
) -> Value {
    let mut service_tiers = serde_json::Map::new();
    for (svc, tier, stratum) in tiers {
        let mut rec = serde_json::Map::new();
        rec.insert("tier".to_owned(), json!(tier));
        if let Some(st) = stratum {
            rec.insert("stratum".to_owned(), json!(st));
        }
        service_tiers.insert((*svc).to_owned(), Value::Object(rec));
    }
    json!({
        "crate_count": crates.len(),
        "edge_count": edges.len(),
        "crates": crates
            .iter()
            .map(|(d, s)| json!({ "dir": d, "service": s }))
            .collect::<Vec<_>>(),
        "service_tiers": Value::Object(service_tiers),
        "edges": edges
            .iter()
            .map(|(f, t)| json!({ "from": f, "to": t }))
            .collect::<Vec<_>>()
    })
}

#[test]
fn lateral_product_to_product_is_green() {
    let obs = corpus(
        &[("oya/a/crates/oya-a", "oya/a"), ("oya/b/crates/oya-b", "oya/b")],
        &[("oya/a", "product", None), ("oya/b", "product", None)],
        &[("oya/a/crates/oya-a", "oya/b/crates/oya-b")],
    );
    let report = evaluate(&policy(), &baseline(&[]), &obs);
    assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
    assert!(report.findings.is_empty());
}

#[test]
fn substrate_depends_on_product_is_a_regression() {
    let obs = corpus(
        &[
            ("cloud/s/crates/oya-s", "cloud/s"),
            ("oya/p/crates/oya-p", "oya/p"),
        ],
        &[("cloud/s", "substrate", Some("S0")), ("oya/p", "product", None)],
        &[("cloud/s/crates/oya-s", "oya/p/crates/oya-p")],
    );
    let report = evaluate(&policy(), &baseline(&[]), &obs);
    assert_eq!(report.verdict, Verdict::Red, "a NEW substrate->product edge regresses");
    let f = report
        .findings
        .iter()
        .find(|f| f.code == "TDA-SUBSTRATE-UPWARD")
        .expect("a substrate-upward finding");
    assert_eq!(f.status, Status::Regression);
    assert_eq!(report.regressions, 1);
}

#[test]
fn substrate_depends_on_product_when_baselined_is_advisory_green() {
    let subject = "cloud/s/crates/oya-s -> oya/p/crates/oya-p";
    let obs = corpus(
        &[
            ("cloud/s/crates/oya-s", "cloud/s"),
            ("oya/p/crates/oya-p", "oya/p"),
        ],
        &[("cloud/s", "substrate", Some("S0")), ("oya/p", "product", None)],
        &[("cloud/s/crates/oya-s", "oya/p/crates/oya-p")],
    );
    let report = evaluate(
        &policy(),
        &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]),
        &obs,
    );
    assert_eq!(report.verdict, Verdict::Green, "a BASELINED violation is advisory-only");
    let f = report.findings.iter().find(|f| f.code == "TDA-SUBSTRATE-UPWARD").unwrap();
    assert_eq!(f.status, Status::Baselined);
    assert_eq!(report.baselined, 1);
    assert_eq!(report.regressions, 0);
}

#[test]
fn burning_down_a_baselined_violation_stays_green() {
    // The edge is fixed (removed); the baseline still lists it. The gate must stay GREEN (allowed to
    // improve) and report the burn-down progress.
    let subject = "cloud/s/crates/oya-s -> oya/p/crates/oya-p";
    let obs = corpus(
        &[
            ("cloud/s/crates/oya-s", "cloud/s"),
            ("oya/p/crates/oya-p", "oya/p"),
        ],
        &[("cloud/s", "substrate", Some("S0")), ("oya/p", "product", None)],
        &[], // the inverting edge has been removed
    );
    let report = evaluate(
        &policy(),
        &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]),
        &obs,
    );
    assert_eq!(report.verdict, Verdict::Green);
    assert_eq!(report.regressions, 0);
    assert_eq!(report.burned_down, 1, "the fixed baselined violation counts as burned down");
}

#[test]
fn product_service_cell_cross_is_blocked_both_directions() {
    // product -> service-cell.
    let obs1 = corpus(
        &[("oya/p/crates/oya-p", "oya/p"), ("oya/c/crates/oya-c", "oya/c")],
        &[("oya/p", "product", None), ("oya/c", "service-cell", None)],
        &[("oya/p/crates/oya-p", "oya/c/crates/oya-c")],
    );
    let r1 = evaluate(&policy(), &baseline(&[]), &obs1);
    assert!(r1.findings.iter().any(|f| f.code == "TDA-PRODUCT-CELL-CROSS"));
    assert_eq!(r1.verdict, Verdict::Red);

    // service-cell -> product.
    let obs2 = corpus(
        &[("oya/c/crates/oya-c", "oya/c"), ("oya/p/crates/oya-p", "oya/p")],
        &[("oya/c", "service-cell", None), ("oya/p", "product", None)],
        &[("oya/c/crates/oya-c", "oya/p/crates/oya-p")],
    );
    let r2 = evaluate(&policy(), &baseline(&[]), &obs2);
    assert!(r2.findings.iter().any(|f| f.code == "TDA-CELL-PRODUCT"));
    assert_eq!(r2.verdict, Verdict::Red);
}

#[test]
fn s_rank_inversion_is_flagged_lower_to_higher() {
    // S0 -> S1 is an inversion (a dep may only point to an equal-or-lower S-rank).
    let obs = corpus(
        &[("cloud/a/crates/oya-a", "cloud/a"), ("cloud/b/crates/oya-b", "cloud/b")],
        &[("cloud/a", "substrate", Some("S0")), ("cloud/b", "substrate", Some("S1"))],
        &[("cloud/a/crates/oya-a", "cloud/b/crates/oya-b")],
    );
    let report = evaluate(&policy(), &baseline(&[]), &obs);
    assert!(report.findings.iter().any(|f| f.code == "TDA-S-RANK-INVERSION"));
    assert_eq!(report.verdict, Verdict::Red);
}

#[test]
fn s_rank_equal_or_lower_is_green() {
    // S2 -> S1 (lower) and S1 -> S1 (equal) are both allowed.
    let obs = corpus(
        &[
            ("cloud/hi/crates/oya-hi", "cloud/hi"),
            ("cloud/mid/crates/oya-mid", "cloud/mid"),
            ("cloud/mid2/crates/oya-mid2", "cloud/mid2"),
        ],
        &[
            ("cloud/hi", "substrate", Some("S2")),
            ("cloud/mid", "substrate", Some("S1")),
            ("cloud/mid2", "substrate", Some("S1")),
        ],
        &[
            ("cloud/hi/crates/oya-hi", "cloud/mid/crates/oya-mid"),    // S2 -> S1 ok
            ("cloud/mid/crates/oya-mid", "cloud/mid2/crates/oya-mid2"), // S1 -> S1 ok
        ],
    );
    let report = evaluate(&policy(), &baseline(&[]), &obs);
    assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
}

#[test]
fn forward_declared_substrate_is_rank_exempt() {
    // forward-declared has no rank; any substrate->forward-declared (or vice versa) edge is exempt.
    let obs = corpus(
        &[("cloud/a/crates/oya-a", "cloud/a"), ("cloud/fd/crates/oya-fd", "cloud/fd")],
        &[
            ("cloud/a", "substrate", Some("S0")),
            ("cloud/fd", "substrate", Some("forward-declared")),
        ],
        &[("cloud/a/crates/oya-a", "cloud/fd/crates/oya-fd")],
    );
    let report = evaluate(&policy(), &baseline(&[]), &obs);
    assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
}

#[test]
fn edge_to_unclassified_crate_is_allowed() {
    // A substrate depending on a libs/ (unclassified) crate is fine — no tier to compare.
    let obs = corpus(
        &[("cloud/s/crates/oya-s", "cloud/s")],
        &[("cloud/s", "substrate", Some("S0"))],
        &[("cloud/s/crates/oya-s", "libs/oya-shared-thing")],
    );
    let report = evaluate(&policy(), &baseline(&[]), &obs);
    assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
}

#[test]
fn intra_service_edge_is_skipped() {
    // Two crates in the SAME substrate service depending on each other is not a cross-tier edge.
    let obs = corpus(
        &[("cloud/s/crates/oya-s-a", "cloud/s"), ("cloud/s/crates/oya-s-b", "cloud/s")],
        &[("cloud/s", "substrate", Some("S0"))],
        &[("cloud/s/crates/oya-s-a", "cloud/s/crates/oya-s-b")],
    );
    let report = evaluate(&policy(), &baseline(&[]), &obs);
    assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
}

#[test]
fn cycle_is_detected_and_always_blocks() {
    // a -> b -> a across two services forms a 2-cycle.
    let obs = corpus(
        &[("oya/a/crates/oya-a", "oya/a"), ("oya/b/crates/oya-b", "oya/b")],
        &[("oya/a", "product", None), ("oya/b", "product", None)],
        &[
            ("oya/a/crates/oya-a", "oya/b/crates/oya-b"),
            ("oya/b/crates/oya-b", "oya/a/crates/oya-a"),
        ],
    );
    let report = evaluate(&policy(), &baseline(&[]), &obs);
    assert!(report.findings.iter().any(|f| f.code == "TDA-CYCLE"));
    assert_eq!(report.verdict, Verdict::Red, "a cycle is always a regression");
}

#[test]
fn self_loop_is_a_cycle() {
    // tarjan returns size-1 SCCs; the self-loop is detected separately by detect_cycles.
    let obs = corpus(
        &[("oya/a/crates/oya-x", "oya/a")],
        &[("oya/a", "product", None)],
        &[("oya/a/crates/oya-x", "oya/a/crates/oya-x")],
    );
    let report = evaluate(&policy(), &baseline(&[]), &obs);
    assert!(report.findings.iter().any(|f| f.code == "TDA-CYCLE"));
}

#[test]
fn empty_scan_below_floor_is_a_regression() {
    let mut pol = policy();
    pol["min_expected_crates"] = json!(100);
    let subject = "cloud/s/crates/oya-s -> oya/p/crates/oya-p";
    let obs = corpus(&[], &[], &[]);
    let report = evaluate(&pol, &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]), &obs);
    assert!(report.findings.iter().any(|f| f.code == "TDA-EMPTY-SCAN"));
    assert!(
        !report.findings.iter().any(|f| f.code == "TDA-STALE-BASELINE"),
        "broken scans should report the scan root cause, not phantom stale rows: {:?}",
        report.findings
    );
    assert_eq!(report.burned_down, 0, "broken scans must not report fake burn-down");
    assert_eq!(report.verdict, Verdict::Red);
}

#[test]
fn malformed_policy_fails_closed() {
    let bad = json!({ "gate_id": GATE_ID }); // missing required arrays
    let report = evaluate(&bad, &baseline(&[]), &corpus(&[], &[], &[]));
    assert!(report.findings.iter().any(|f| f.code == "TDA-POLICY-MALFORMED"));
    assert_eq!(report.verdict, Verdict::Red);
}

#[test]
fn malformed_baseline_fails_closed() {
    let bad_baseline = json!({ "gate_id": GATE_ID }); // missing `violations`
    let report = evaluate(&policy(), &bad_baseline, &corpus(&[], &[], &[]));
    assert!(report.findings.iter().any(|f| f.code == "TDA-BASELINE-MALFORMED"));
    assert_eq!(report.verdict, Verdict::Red);
}

#[test]
fn blocking_enforcement_treats_baselined_as_red() {
    // Post-burn-down flip: with enforcement=blocking, even a baselined violation is RED.
    let subject = "cloud/s/crates/oya-s -> oya/p/crates/oya-p";
    let mut pol = policy();
    pol["enforcement"] = json!("blocking");
    let obs = corpus(
        &[
            ("cloud/s/crates/oya-s", "cloud/s"),
            ("oya/p/crates/oya-p", "oya/p"),
        ],
        &[("cloud/s", "substrate", Some("S0")), ("oya/p", "product", None)],
        &[("cloud/s/crates/oya-s", "oya/p/crates/oya-p")],
    );
    let report = evaluate(&pol, &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]), &obs);
    assert_eq!(report.verdict, Verdict::Red, "blocking mode blocks even baselined debt");
}

#[test]
fn cargo_path_extraction_finds_inline_and_section_deps() {
    let toml = r#"
        [package]
        name = "x"
        [lib]
        path = "src/lib.rs"
        [dependencies]
        oya-a = { path = "../../../../oya/identity/crates/oya-identity-domain" }
        serde_json = { workspace = true }
        [dependencies.oya-b]
        path = "../oya-b"
    "#;
    let vals = extract_cargo_path_values(toml);
    assert!(vals.contains(&"src/lib.rs".to_owned()));
    assert!(vals.contains(&"../../../../oya/identity/crates/oya-identity-domain".to_owned()));
    assert!(vals.contains(&"../oya-b".to_owned()));
}

#[test]
fn cargo_path_extraction_ignores_commented_path() {
    let toml = "# oya-x = { path = \"../should-not-count\" }\noya-y = { path = \"../counts\" }";
    let vals = extract_cargo_path_values(toml);
    assert!(!vals.contains(&"../should-not-count".to_owned()));
    assert!(vals.contains(&"../counts".to_owned()));
}

#[test]
fn buck_first_party_target_extraction() {
    let buck = r#"
        rust_library(
            name = "x",
            deps = [
                "//oya/identity/crates/oya-identity-domain:oya-identity-domain",
                "//libs/oya-shared-thing:oya-shared-thing",
                "third-party//:serde_json",
            ],
            visibility = ["PUBLIC"],
        )
    "#;
    let targets = extract_buck_first_party_targets(buck);
    assert!(targets.contains(&"oya/identity/crates/oya-identity-domain".to_owned()));
    assert!(targets.contains(&"libs/oya-shared-thing".to_owned()));
    // third-party//: must be skipped (not a first-party // target).
    assert!(!targets.iter().any(|t| t.contains("serde_json")));
    assert!(!targets.iter().any(|t| t.starts_with("third-party")));
}

#[test]
fn segment_glob_matching() {
    assert!(segment_matches("oya-*", "oya-cloud-iam-app"));
    assert!(segment_matches("*", "anything"));
    assert!(!segment_matches("oya-*", "third-party"));
    assert!(segment_matches("crates", "crates"));
    assert!(!segment_matches("crates", "src"));
}

#[test]
fn tarjan_finds_buried_cycle() {
    // a->b->c->b: the SCC {b,c}.
    let nodes: BTreeSet<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
    let mut adj: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    adj.entry("a".into()).or_default().insert("b".into());
    adj.entry("b".into()).or_default().insert("c".into());
    adj.entry("c".into()).or_default().insert("b".into());
    let sccs = tarjan_sccs(&nodes, &adj);
    assert!(sccs.iter().any(|c| c == &vec!["b".to_string(), "c".to_string()]));
}

#[test]
fn parse_baseline_round_trips() {
    let doc = baseline(&[("TDA-CYCLE", "a -> b,c"), ("TDA-SUBSTRATE-UPWARD", "x -> y")]);
    let b = parse_baseline(&doc).expect("parse");
    assert!(b.keys.contains("TDA-CYCLE|a -> b,c"));
    assert!(b.keys.contains("TDA-SUBSTRATE-UPWARD|x -> y"));
    assert_eq!(b.keys.len(), 2);
}

#[test]
fn stale_baseline_row_whose_crate_vanished_is_red() {
    // B3 hardening: the baselined inverting edge's `from` crate is NO LONGER in the corpus
    // (moved/renamed by an in-flight strangler). The row is a phantom a subset baseline can never RED
    // on, so the liveness backstop must flag it TDA-STALE-BASELINE and fail the gate.
    let subject = "cloud/gone/crates/oya-gone -> oya/p/crates/oya-p";
    let obs = corpus(
        &[("oya/p/crates/oya-p", "oya/p")], // cloud/gone/... is absent from the live corpus
        &[("oya/p", "product", None)],
        &[],
    );
    let report = evaluate(&policy(), &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]), &obs);
    let f = report
        .findings
        .iter()
        .find(|f| f.code == "TDA-STALE-BASELINE")
        .expect("a stale-baseline finding");
    assert_eq!(f.status, Status::Regression);
    assert_eq!(f.subject, subject);
    assert_eq!(report.verdict, Verdict::Red, "a phantom baseline row must fail the gate");
}

#[test]
fn stale_baseline_detects_a_vanished_to_endpoint() {
    // The `to` endpoint (not just `from`) is checked: if the depended-upon crate vanished, the edge
    // can never exist, so the row is a phantom.
    let subject = "cloud/s/crates/oya-s -> oya/gone/crates/oya-gone";
    let obs = corpus(
        &[("cloud/s/crates/oya-s", "cloud/s")], // oya/gone/... is absent
        &[("cloud/s", "substrate", Some("S0"))],
        &[],
    );
    let report = evaluate(&policy(), &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]), &obs);
    assert!(report.findings.iter().any(|f| f.code == "TDA-STALE-BASELINE"));
    assert_eq!(report.verdict, Verdict::Red);
}

#[test]
fn burned_down_edge_with_live_endpoints_is_not_stale() {
    // Both endpoint crates still exist; only the inverting EDGE was removed (the inversion was fixed).
    // This is a legitimate burn-down, NOT a phantom — the liveness backstop must stay quiet and the
    // gate stays GREEN (the existing subset regression check is unchanged).
    let subject = "cloud/s/crates/oya-s -> oya/p/crates/oya-p";
    let obs = corpus(
        &[
            ("cloud/s/crates/oya-s", "cloud/s"),
            ("oya/p/crates/oya-p", "oya/p"),
        ],
        &[("cloud/s", "substrate", Some("S0")), ("oya/p", "product", None)],
        &[], // edge removed, both crates remain
    );
    let report = evaluate(&policy(), &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]), &obs);
    assert!(
        !report.findings.iter().any(|f| f.code == "TDA-STALE-BASELINE"),
        "both endpoints exist -> burn-down, not a phantom: {:?}",
        report.findings
    );
    assert_eq!(report.verdict, Verdict::Green);
    assert_eq!(report.burned_down, 1);
}

#[test]
fn owning_service_consumes_the_configured_roots_not_a_hardcode() {
    // The regression this guards: the fn took `_service_roots` and hardcoded cloud/oya, so the
    // policy field looked load-bearing and was not.
    let roots = vec!["cloud".to_string(), "oya".to_string()];
    let no_caps = BTreeSet::new();
    assert_eq!(
        owning_service("cloud/cloud-iam/crates/x", &roots, &no_caps),
        Some("cloud/cloud-iam".to_string())
    );
    assert_eq!(owning_service("messaging/core/domain", &roots, &no_caps), None);

    // Repointing the policy at a different root set must change the projection. Under a hardcode
    // this assertion fails.
    let repointed = vec!["messaging".to_string()];
    assert_eq!(
        owning_service("messaging/core/domain", &repointed, &no_caps),
        Some("messaging/core".to_string())
    );
    assert_eq!(
        owning_service("cloud/cloud-iam/crates/x", &repointed, &no_caps),
        None
    );

    // Degenerate shapes must not panic or invent a service.
    assert_eq!(owning_service("cloud", &roots, &no_caps), None);
    assert_eq!(owning_service("cloud/", &roots, &no_caps), None);
    assert_eq!(owning_service("", &roots, &no_caps), None);
}

/// Build a corpus and attach ADR-0563 crate-DIR move pairs.
fn corpus_with_moves(
    crates: &[(&str, &str)],
    tiers: &[(&str, &str, Option<&str>)],
    edges: &[(&str, &str)],
    pairs: &[(&str, &str)],
) -> Value {
    let mut obs = corpus(crates, tiers, edges);
    obs["crate_dir_pairs"] = Value::Array(
        pairs
            .iter()
            .map(|(o, n)| json!({ "old": o, "new": n }))
            .collect(),
    );
    obs
}

#[test]
fn a_moved_crate_keeps_its_baselined_violation_instead_of_reading_as_a_regression() {
    // The defect: baseline subjects are crate DIRS, so after a capability move+rename the SAME
    // violation appears at the NEW dir, misses its OLD-path key, and is reported as a NEW
    // regression — a false RED on a PR that changed no dependency. ADR-0563's relabel exists for
    // exactly this but was never wired to this gate.
    let old_subject = "oya/svc/crates/oya-svc-api -> oya/p/crates/oya-p";
    let obs = corpus_with_moves(
        &[("cap/core/api", "cap"), ("oya/p/crates/oya-p", "oya/p")],
        &[("cap", "substrate", Some("S0")), ("oya/p", "product", None)],
        &[("cap/core/api", "oya/p/crates/oya-p")],
        &[("oya/svc/crates/oya-svc-api", "cap/core/api")],
    );
    let report = evaluate(
        &policy(),
        &baseline(&[("TDA-SUBSTRATE-UPWARD", old_subject)]),
        &obs,
    );
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "a move must not turn accepted debt into a regression"
    );
    assert_eq!(report.regressions, 0);
    assert_eq!(report.baselined, 1);
    assert!(
        !report.findings.iter().any(|f| f.code == "TDA-STALE-BASELINE"),
        "the relabelled row is anchored, so it is not a phantom"
    );
}

#[test]
fn the_relabel_cannot_manufacture_a_false_green() {
    // Guard 1 — EXISTENCE. A pair whose NEW dir is not in the live crate set is a move that did not
    // land. Following it would silently retire a real violation; the honest phantom must survive.
    let obs = corpus_with_moves(
        &[("oya/p/crates/oya-p", "oya/p")],
        &[("oya/p", "product", None)],
        &[],
        &[("oya/gone/crates/x", "cap/core/never-landed")],
    );
    let report = evaluate(
        &policy(),
        &baseline(&[("TDA-SUBSTRATE-UPWARD", "oya/gone/crates/x -> oya/p/crates/oya-p")]),
        &obs,
    );
    assert!(
        report.findings.iter().any(|f| f.code == "TDA-STALE-BASELINE"),
        "an unlanded move must NOT relabel; the phantom row stays reported"
    );

    // Guard 2 — STRICT NO-OP. No pairs (the ordinary no-move PR, and the fail-closed result of a
    // missing or ambiguous manifest) must leave the baseline byte-identical.
    let subject = "cloud/s/crates/oya-s -> oya/p/crates/oya-p";
    let obs = corpus_with_moves(
        &[
            ("cloud/s/crates/oya-s", "cloud/s"),
            ("oya/p/crates/oya-p", "oya/p"),
        ],
        &[("cloud/s", "substrate", Some("S0")), ("oya/p", "product", None)],
        &[("cloud/s/crates/oya-s", "oya/p/crates/oya-p")],
        &[],
    );
    let report = evaluate(&policy(), &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]), &obs);
    assert_eq!(report.verdict, Verdict::Green);
    assert_eq!(report.baselined, 1);
    assert_eq!(report.regressions, 0);
}

#[test]
fn a_capability_root_is_its_own_service_not_its_faces() {
    // ADR-0562 §1/§4: a capability is the ownership unit; core/ports/adapters/facade are sub-folds.
    // Projecting faces as services would duplicate one tier across 3-4 sibling manifests with
    // nothing asserting they agree — and it is why ~412 moved crates were emitted `service: null`.
    let service_roots = vec!["cloud".to_string(), "oya".to_string()];
    let caps: BTreeSet<String> = ["iam".to_string(), "os".to_string()].into_iter().collect();

    // RED before this change: owning_service returned None (root not in service_roots).
    assert_eq!(
        owning_service("iam/core/identity-kernel", &service_roots, &caps),
        Some("iam".to_string()),
        "a capability root owns its crates; the face is not the service"
    );
    // Every face collapses to the same owning service — the point of the change.
    assert_eq!(
        owning_service("iam/adapters/cloud-oci", &service_roots, &caps),
        Some("iam".to_string())
    );
    assert_eq!(
        owning_service("iam/facade/api", &service_roots, &caps),
        Some("iam".to_string())
    );
    // A meta root that owns crates behaves identically (os/ holds 41).
    assert_eq!(
        owning_service("os/core/apid-domain", &service_roots, &caps),
        Some("os".to_string())
    );

    // Service roots keep their two-component projection — no behaviour change there.
    assert_eq!(
        owning_service("cloud/cloud-iam/crates/x", &service_roots, &caps),
        Some("cloud/cloud-iam".to_string())
    );
    // A root in neither set is still unclassified.
    assert_eq!(owning_service("libs/oya-json-kernel", &service_roots, &caps), None);
    // Degenerate shapes stay None even for a declared capability root: the empty-remainder guard
    // runs BEFORE the capability check, so a bare root never invents a service.
    assert_eq!(owning_service("iam", &service_roots, &caps), None);
    assert_eq!(owning_service("iam/", &service_roots, &caps), None);
}

#[test]
fn an_undeclared_crate_root_is_born_blocking() {
    // A crate under a root declared in NEITHER service_roots NOR unclassified_roots is silently
    // unenforced today; R6 makes it RED. `unclassified_roots` was inert config before this.
    let policy = policy();
    let baseline = json!({"violations": []});
    let observed = json!({
        "crate_count": 900,
        "crates": [
            {"dir": "libs/oya-x", "service": null},
            {"dir": "surprise/new-root/crate", "service": null}
        ],
        "service_tiers": {},
        "edges": []
    });
    let report = evaluate(&policy, &baseline, &observed);
    let undeclared: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "TDA-UNDECLARED-ROOT")
        .collect();
    assert_eq!(
        undeclared.len(),
        1,
        "exactly the undeclared root must fire; got {:?}",
        report.findings
    );
    // subject is the ROOT, not the crate dir: one finding per root, stable across crate churn.
    assert_eq!(undeclared[0].subject, "surprise");
    assert!(
        undeclared[0].detail.contains("surprise/new-root/crate"),
        "detail should name an example crate; got {}",
        undeclared[0].detail
    );
    assert!(matches!(undeclared[0].status, Status::Regression));
}

#[test]
fn declared_unclassified_roots_do_not_fire_r6() {
    // libs/ and tools/ are deliberately exempt; making the declaration live must not RED them.
    let policy = policy();
    let baseline = json!({"violations": []});
    let observed = json!({
        "crate_count": 900,
        "crates": [
            {"dir": "libs/oya-x", "service": null},
            {"dir": "tools/oya-y", "service": null},
            {"dir": "cloud/cloud-iam/crates/z", "service": "cloud/cloud-iam"}
        ],
        "service_tiers": {},
        "edges": []
    });
    let report = evaluate(&policy, &baseline, &observed);
    assert!(
        !report.findings.iter().any(|f| f.code == "TDA-UNDECLARED-ROOT"),
        "declared roots must not fire; got {:?}",
        report.findings
    );
}

#[test]
fn evaluator_only_emits_declared_violation_codes() {
    // THE DRIFT GUARD whose absence let a real defect through. Adding TDA-UNDECLARED-ROOT
    // required syncing THREE hand-maintained places: VIOLATION_CODES, the module doc bullet
    // list, and main.rs's --emit-baseline exclusion. Two were updated; the third was missed and
    // only found in review. ~10 sibling gates carry this guard; this one did not.
    let declared: BTreeSet<&str> = VIOLATION_CODES.into_iter().collect();

    // Drive the evaluator through every reachable emission path and assert nothing escapes the
    // declared set.
    let cases = vec![
        // malformed policy
        (json!({"gate_id": "wrong"}), json!({"violations": []}), json!({})),
        // malformed baseline
        (policy(), json!({"violations": "not-an-array"}), json!({})),
        // empty scan + undeclared root + a real edge violation + a stale baseline row
        (
            policy(),
            baseline(&[("TDA-STALE-BASELINE", "ghost/gone")]),
            json!({
                "crate_count": 0,
                "crates": [
                    {"dir": "surprise/root/x", "service": null},
                    {"dir": "cloud/a/crates/oya-a", "service": "cloud/a"},
                    {"dir": "cloud/b/crates/oya-b", "service": "cloud/b"}
                ],
                "service_tiers": {
                    "cloud/a": {"tier": "substrate", "stratum": "S3"},
                    "cloud/b": {"tier": "product"}
                },
                "edges": [
                    {"from": "cloud/a/crates/oya-a", "to": "cloud/b/crates/oya-b"},
                    {"from": "cloud/b/crates/oya-b", "to": "cloud/a/crates/oya-a"}
                ]
            }),
        ),
    ];

    let mut seen: BTreeSet<String> = BTreeSet::new();
    for (p, b, o) in cases {
        for f in evaluate(&p, &b, &o).findings {
            assert!(
                declared.contains(f.code.as_str()),
                "evaluator emitted undeclared code `{}`; add it to VIOLATION_CODES",
                f.code
            );
            seen.insert(f.code.clone());
        }
    }
    // Guard the guard: if these cases stop exercising real paths the test silently weakens.
    assert!(
        seen.len() >= 4,
        "expected the fixtures to exercise several codes, saw {seen:?}"
    );
}
