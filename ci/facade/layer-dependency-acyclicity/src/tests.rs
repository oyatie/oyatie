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
        "capability_roots": [],
        "capability_registry_path": "governance/capability-registry.json",
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
    crates: &[(&str, &str)],              // (crate_dir, owning_service)
    tiers: &[(&str, &str, Option<&str>)], // (service, tier, stratum)
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
            .collect::<Vec<_>>(),
        // Registry facts that make `policy()`'s unclassified roots legitimate META dirs, so R6b
        // stays quiet and only the rule under test can fire. R6b's own tests override these.
        "registry_capabilities": [],
        "registry_meta_dirs": ["libs", "tools", "cloud/cloud-ci"]
    })
}

#[test]
fn lateral_product_to_product_is_green() {
    let obs = corpus(
        &[
            ("oya/a/crates/oya-a", "oya/a"),
            ("oya/b/crates/oya-b", "oya/b"),
        ],
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
        &[
            ("cloud/s", "substrate", Some("S0")),
            ("oya/p", "product", None),
        ],
        &[("cloud/s/crates/oya-s", "oya/p/crates/oya-p")],
    );
    let report = evaluate(&policy(), &baseline(&[]), &obs);
    assert_eq!(
        report.verdict,
        Verdict::Red,
        "a NEW substrate->product edge regresses"
    );
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
        &[
            ("cloud/s", "substrate", Some("S0")),
            ("oya/p", "product", None),
        ],
        &[("cloud/s/crates/oya-s", "oya/p/crates/oya-p")],
    );
    let report = evaluate(
        &policy(),
        &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]),
        &obs,
    );
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "a BASELINED violation is advisory-only"
    );
    let f = report
        .findings
        .iter()
        .find(|f| f.code == "TDA-SUBSTRATE-UPWARD")
        .unwrap();
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
        &[
            ("cloud/s", "substrate", Some("S0")),
            ("oya/p", "product", None),
        ],
        &[], // the inverting edge has been removed
    );
    let report = evaluate(
        &policy(),
        &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]),
        &obs,
    );
    assert_eq!(report.verdict, Verdict::Green);
    assert_eq!(report.regressions, 0);
    assert_eq!(
        report.burned_down, 1,
        "the fixed baselined violation counts as burned down"
    );
}

#[test]
fn product_service_cell_cross_is_blocked_both_directions() {
    // product -> service-cell.
    let obs1 = corpus(
        &[
            ("oya/p/crates/oya-p", "oya/p"),
            ("oya/c/crates/oya-c", "oya/c"),
        ],
        &[("oya/p", "product", None), ("oya/c", "service-cell", None)],
        &[("oya/p/crates/oya-p", "oya/c/crates/oya-c")],
    );
    let r1 = evaluate(&policy(), &baseline(&[]), &obs1);
    assert!(
        r1.findings
            .iter()
            .any(|f| f.code == "TDA-PRODUCT-CELL-CROSS")
    );
    assert_eq!(r1.verdict, Verdict::Red);

    // service-cell -> product.
    let obs2 = corpus(
        &[
            ("oya/c/crates/oya-c", "oya/c"),
            ("oya/p/crates/oya-p", "oya/p"),
        ],
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
        &[
            ("cloud/a/crates/oya-a", "cloud/a"),
            ("cloud/b/crates/oya-b", "cloud/b"),
        ],
        &[
            ("cloud/a", "substrate", Some("S0")),
            ("cloud/b", "substrate", Some("S1")),
        ],
        &[("cloud/a/crates/oya-a", "cloud/b/crates/oya-b")],
    );
    let report = evaluate(&policy(), &baseline(&[]), &obs);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "TDA-S-RANK-INVERSION")
    );
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
            ("cloud/hi/crates/oya-hi", "cloud/mid/crates/oya-mid"), // S2 -> S1 ok
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
        &[
            ("cloud/a/crates/oya-a", "cloud/a"),
            ("cloud/fd/crates/oya-fd", "cloud/fd"),
        ],
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
        &[
            ("cloud/s/crates/oya-s-a", "cloud/s"),
            ("cloud/s/crates/oya-s-b", "cloud/s"),
        ],
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
        &[
            ("oya/a/crates/oya-a", "oya/a"),
            ("oya/b/crates/oya-b", "oya/b"),
        ],
        &[("oya/a", "product", None), ("oya/b", "product", None)],
        &[
            ("oya/a/crates/oya-a", "oya/b/crates/oya-b"),
            ("oya/b/crates/oya-b", "oya/a/crates/oya-a"),
        ],
    );
    let report = evaluate(&policy(), &baseline(&[]), &obs);
    assert!(report.findings.iter().any(|f| f.code == "TDA-CYCLE"));
    assert_eq!(
        report.verdict,
        Verdict::Red,
        "a cycle is always a regression"
    );
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
        !report
            .findings
            .iter()
            .any(|f| f.code == "TDA-STALE-BASELINE"),
        "broken scans should report the scan root cause, not phantom stale rows: {:?}",
        report.findings
    );
    assert_eq!(
        report.burned_down, 0,
        "broken scans must not report fake burn-down"
    );
    assert_eq!(report.verdict, Verdict::Red);
}

#[test]
fn malformed_policy_fails_closed() {
    let bad = json!({ "gate_id": GATE_ID }); // missing required arrays
    let report = evaluate(&bad, &baseline(&[]), &corpus(&[], &[], &[]));
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "TDA-POLICY-MALFORMED")
    );
    assert_eq!(report.verdict, Verdict::Red);
}

#[test]
fn malformed_baseline_fails_closed() {
    let bad_baseline = json!({ "gate_id": GATE_ID }); // missing `violations`
    let report = evaluate(&policy(), &bad_baseline, &corpus(&[], &[], &[]));
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "TDA-BASELINE-MALFORMED")
    );
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
        &[
            ("cloud/s", "substrate", Some("S0")),
            ("oya/p", "product", None),
        ],
        &[("cloud/s/crates/oya-s", "oya/p/crates/oya-p")],
    );
    let report = evaluate(&pol, &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]), &obs);
    assert_eq!(
        report.verdict,
        Verdict::Red,
        "blocking mode blocks even baselined debt"
    );
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
    assert!(
        sccs.iter()
            .any(|c| c == &vec!["b".to_string(), "c".to_string()])
    );
}

#[test]
fn parse_baseline_round_trips() {
    let doc = baseline(&[
        ("TDA-CYCLE", "a -> b,c"),
        ("TDA-SUBSTRATE-UPWARD", "x -> y"),
    ]);
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
    let report = evaluate(
        &policy(),
        &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]),
        &obs,
    );
    let f = report
        .findings
        .iter()
        .find(|f| f.code == "TDA-STALE-BASELINE")
        .expect("a stale-baseline finding");
    assert_eq!(f.status, Status::Regression);
    assert_eq!(f.subject, subject);
    assert_eq!(
        report.verdict,
        Verdict::Red,
        "a phantom baseline row must fail the gate"
    );
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
    let report = evaluate(
        &policy(),
        &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]),
        &obs,
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "TDA-STALE-BASELINE")
    );
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
        &[
            ("cloud/s", "substrate", Some("S0")),
            ("oya/p", "product", None),
        ],
        &[], // edge removed, both crates remain
    );
    let report = evaluate(
        &policy(),
        &baseline(&[("TDA-SUBSTRATE-UPWARD", subject)]),
        &obs,
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.code == "TDA-STALE-BASELINE"),
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
    let none: Vec<String> = Vec::new();
    assert_eq!(
        owning_service("cloud/cloud-iam/crates/x", &roots, &none),
        Some("cloud/cloud-iam".to_string())
    );
    assert_eq!(owning_service("messaging/core/domain", &roots, &none), None);

    // Repointing the policy at a different root set must change the projection. Under a hardcode
    // this assertion fails.
    let repointed = vec!["messaging".to_string()];
    assert_eq!(
        owning_service("messaging/core/domain", &repointed, &none),
        Some("messaging/core".to_string())
    );
    assert_eq!(
        owning_service("cloud/cloud-iam/crates/x", &repointed, &none),
        None
    );

    // Degenerate shapes must not panic or invent a service.
    assert_eq!(owning_service("cloud", &roots, &none), None);
    assert_eq!(owning_service("cloud/", &roots, &none), None);
    assert_eq!(owning_service("", &roots, &none), None);
}

#[test]
fn capability_root_projects_to_the_root_not_a_two_component_prefix() {
    // Why capability roots cannot simply join `service_roots`: under the service shape the tier
    // unit is `<root>/<svc>`, which for an ADR-0562 capability tree names `iam/adapters` — a FACE,
    // not a tier-bearing unit, and a key no `service_tiers` entry can ever match. The crate would
    // land back in the unclassified bucket with the policy claiming it was classified.
    let service_roots = vec!["cloud".to_string(), "oya".to_string()];
    let capability_roots = vec!["iam".to_string()];

    assert_eq!(
        owning_service("iam/adapters/cloud-oci", &service_roots, &capability_roots),
        Some("iam".to_string()),
        "a capability crate's tier unit is the capability root itself"
    );
    // The same path under the SERVICE shape yields the face — the wrong unit.
    let as_service_root = vec!["iam".to_string()];
    assert_eq!(
        owning_service("iam/adapters/cloud-oci", &as_service_root, &Vec::new()),
        Some("iam/adapters".to_string()),
        "the service shape names the ADR-0562 face, which is why option 1 does not fit"
    );
    // Service roots keep their 2-component projection when both lists are populated.
    assert_eq!(
        owning_service(
            "cloud/cloud-iam/crates/x",
            &service_roots,
            &capability_roots
        ),
        Some("cloud/cloud-iam".to_string())
    );
    // A root in neither list stays unclassified.
    assert_eq!(
        owning_service("libs/oya-x", &service_roots, &capability_roots),
        None
    );
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
        !report
            .findings
            .iter()
            .any(|f| f.code == "TDA-UNDECLARED-ROOT"),
        "declared roots must not fire; got {:?}",
        report.findings
    );
}

#[test]
fn unclassified_root_silently_exempts_an_otherwise_red_edge() {
    // THE PROOF PAIR for the `unclassified_roots` silent-exemption defect. ONE edge, evaluated
    // twice; the ONLY difference is whether the `from` crate's root carries a tier.
    //
    // A substrate -> product edge is an unambiguous R1 violation. Placed under a root that
    // `owning_service` does not classify, `tier_of` yields None and the edge is SKIPPED at the
    // `let (Some(src), Some(dst)) = ...` guard — the gate reports GREEN on a violating edge.
    // Declaring that root in `unclassified_roots` silences R6 (TDA-UNDECLARED-ROOT) while leaving
    // the skip intact, which is what makes the R6 remedy a permanent silent exemption rather than
    // a fix.
    let edge = (
        "iam/core/identity-domain",
        "oya/community/crates/oya-community-post-store-api",
    );
    let unclassified = {
        let mut c = corpus(
            &[(edge.0, ""), (edge.1, "oya/community")],
            &[("oya/community", "product", None)],
            &[edge],
        );
        // `corpus` writes "" for an unowned crate; model the real collector's null instead.
        c["crates"][0]["service"] = Value::Null;
        c
    };

    // (1) `iam` declared in NEITHER list: R6 fires — but note WHAT it reports. The violating EDGE
    // is still absent from the findings; R6 reports only that the root is undeclared.
    let undeclared = evaluate(&policy(), &baseline(&[]), &unclassified);
    assert_eq!(
        undeclared
            .findings
            .iter()
            .map(|f| f.code.as_str())
            .collect::<Vec<_>>(),
        vec!["TDA-UNDECLARED-ROOT"],
        "an undeclared root reports itself and nothing about the edge it hides"
    );

    // (2) Apply R6's OWN prescribed remedy — declare `iam` in `unclassified_roots`. The gate goes
    // fully GREEN while the tier comparison is still skipped: the violating edge vanishes from the
    // report entirely. THIS is the defect — the remedy is a permanent silent exemption, not a fix.
    // (Modelled here with `iam` presented as a legitimate meta dir, which is exactly the state the
    // pre-R6b gate was in for all 27 declared roots: no rule distinguished a capability from a
    // meta tree, so every declaration bought this silence.)
    let mut declared_policy = policy();
    declared_policy["unclassified_roots"] = json!(["libs", "tools", "cloud/cloud-ci", "iam"]);
    let mut as_meta = unclassified.clone();
    as_meta["registry_meta_dirs"] = json!(["libs", "tools", "cloud/cloud-ci", "iam"]);
    let green = evaluate(&declared_policy, &baseline(&[]), &as_meta);
    assert_eq!(
        green.verdict,
        Verdict::Green,
        "declaring the root swallows the violating edge; findings: {:?}",
        green.findings
    );
    assert!(
        green.findings.is_empty(),
        "no finding at all survives the declaration: {:?}",
        green.findings
    );

    // (2b) R6b closes that door for a CAPABILITY: the same declaration now REDs on the exemption
    // itself. Note what it does NOT do — the edge is still not compared, so R6b is a report that
    // enforcement is missing, not a substitute for it. Only step (3) actually evaluates the edge.
    let mut as_capability = unclassified.clone();
    as_capability["registry_capabilities"] = json!(["iam"]);
    as_capability["registry_meta_dirs"] = json!(["libs", "tools", "cloud/cloud-ci"]);
    let r6b = evaluate(&declared_policy, &baseline(&[]), &as_capability);
    assert_eq!(
        r6b.findings
            .iter()
            .map(|f| f.code.as_str())
            .collect::<Vec<_>>(),
        vec!["TDA-UNCLASSIFIED-ROOT-NOT-META"],
        "R6b names the exemption; the hidden edge is still not compared"
    );

    // RED: the SAME edge, with `iam` tier-classified as a substrate -> R1 fires.
    let classified = corpus(
        &[(edge.0, "iam"), (edge.1, "oya/community")],
        &[
            ("iam", "substrate", Some("S1")),
            ("oya/community", "product", None),
        ],
        &[edge],
    );
    let red = evaluate(&policy(), &baseline(&[]), &classified);
    assert_eq!(
        red.verdict,
        Verdict::Red,
        "the identical edge MUST be RED once its root carries a tier; findings: {:?}",
        red.findings
    );
    let f = red
        .findings
        .iter()
        .find(|f| f.code == "TDA-SUBSTRATE-UPWARD")
        .expect("the substrate-upward violation the unclassified variant hid");
    assert_eq!(f.status, Status::Regression);
    assert_eq!(f.subject, format!("{} -> {}", edge.0, edge.1));
}

/// A corpus carrying only registry facts — R6b/R6c are evaluated over POLICY data, so they need no
/// crates. `crate_count` clears the false-green floor.
fn root_rules_corpus(capabilities: &[&str], meta_dirs: &[&str]) -> Value {
    json!({
        "crate_count": 900,
        "crates": [],
        "service_tiers": {},
        "edges": [],
        "registry_capabilities": capabilities,
        "registry_meta_dirs": meta_dirs
    })
}

#[test]
fn r6b_reds_a_registered_capability_declared_unclassified() {
    // The recurrence-prevention half. Declaring a capability root in `unclassified_roots` is what
    // converts the TDA-UNDECLARED-ROOT remedy into a permanent silent exemption; R6b makes that
    // declaration itself the violation, closed against the ADR-0562 registry.
    let mut policy = policy();
    policy["unclassified_roots"] = json!(["libs", "iam"]);

    // RED: `iam` is a registered capability, so `unclassified` is the wrong class for it.
    let observed = root_rules_corpus(&["iam", "cell"], &["libs", "os"]);
    let report = evaluate(&policy, &baseline(&[]), &observed);
    let f: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "TDA-UNCLASSIFIED-ROOT-NOT-META")
        .collect();
    assert_eq!(f.len(), 1, "exactly `iam` fires; got {:?}", report.findings);
    assert_eq!(f[0].subject, "iam");
    assert_eq!(f[0].status, Status::Regression);
    assert!(
        f[0].detail.contains("capability_roots"),
        "the remedy must name the tier-enforced destination; got {}",
        f[0].detail
    );
    assert_eq!(report.verdict, Verdict::Red);

    // GREEN: the same root moved to `capability_roots` (the fix), with a resolved tier.
    let mut fixed = policy.clone();
    fixed["unclassified_roots"] = json!(["libs"]);
    fixed["capability_roots"] = json!(["iam"]);
    let mut observed = root_rules_corpus(&["iam", "cell"], &["libs", "os"]);
    observed["service_tiers"]["iam"] = json!({"tier": "substrate", "stratum": "S1"});
    let report = evaluate(&fixed, &baseline(&[]), &observed);
    assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

#[test]
fn r6b_reds_an_unclassified_root_that_is_not_registered_at_all() {
    // The other half of the closed allowlist: a root that is neither a registered capability nor a
    // registry meta_directory has no reviewable basis for its exemption. Without this, a new root
    // could be added to `unclassified_roots` without ever touching the closed registry and R6b
    // would stay quiet — the same hole one level up.
    let mut policy = policy();
    policy["unclassified_roots"] = json!(["scratch"]);
    let observed = root_rules_corpus(&["iam"], &["os"]);
    let report = evaluate(&policy, &baseline(&[]), &observed);
    let f = report
        .findings
        .iter()
        .find(|f| f.code == "TDA-UNCLASSIFIED-ROOT-NOT-META")
        .expect("an unregistered unclassified root must fire");
    assert_eq!(f.subject, "scratch");
    assert!(
        f.detail.contains("meta_directories"),
        "the remedy must point at the closed registry; got {}",
        f.detail
    );
}

#[test]
fn r6b_is_quiet_for_a_registry_declared_meta_dir() {
    // `os/` IS a registry meta_directory — the one legitimate unclassified root in the live policy.
    let mut policy = policy();
    policy["unclassified_roots"] = json!(["os"]);
    let observed = root_rules_corpus(&["iam"], &["kernel", "os", "base"]);
    let report = evaluate(&policy, &baseline(&[]), &observed);
    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

#[test]
fn r6b_fails_closed_when_registry_facts_are_missing() {
    // If the registry cannot be read, the safe default is "no root is provably meta" (report them
    // all), never "every root is exempt". The opposite default would silently restore the hole
    // whenever the registry path breaks.
    let mut policy = policy();
    policy["unclassified_roots"] = json!(["libs", "iam"]);
    let observed = json!({
        "crate_count": 900, "crates": [], "service_tiers": {}, "edges": []
    });
    let report = evaluate(&policy, &baseline(&[]), &observed);
    assert_eq!(
        report
            .findings
            .iter()
            .filter(|f| f.code == "TDA-UNCLASSIFIED-ROOT-NOT-META")
            .count(),
        2,
        "absent registry facts must report every unclassified root; got {:?}",
        report.findings
    );
}

#[test]
fn r6c_reds_a_capability_root_whose_tier_never_resolved() {
    // The anti-relapse guard: `capability_roots` must not become the NEW silent exemption. A root
    // declared there but carrying no tier compares exactly as much as an unclassified one — zero —
    // while the policy asserts it is enforced.
    let mut policy = policy();
    policy["capability_roots"] = json!(["marketplace"]);
    let observed = root_rules_corpus(&["marketplace"], &["libs", "tools", "cloud/cloud-ci", "os"]);
    let report = evaluate(&policy, &baseline(&[]), &observed);
    let f = report
        .findings
        .iter()
        .find(|f| f.code == "TDA-CAPABILITY-TIER-UNRESOLVED")
        .expect("an unresolved capability tier must fire");
    assert_eq!(f.subject, "marketplace");
    assert_eq!(f.status, Status::Regression);
    assert_eq!(report.verdict, Verdict::Red);

    // GREEN once the tier resolves.
    let mut observed = observed;
    observed["service_tiers"]["marketplace"] = json!({"tier": "substrate", "stratum": "S0"});
    let report = evaluate(&policy, &baseline(&[]), &observed);
    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

#[test]
fn r6c_is_not_baselineable() {
    // Unlike R6b (which carries the 21 known exemptions as advisory debt), an unresolved capability
    // tier must never be parked in the baseline: it means the gate is claiming enforcement it is
    // not performing, which is the defect itself rather than a symptom of it.
    let mut policy = policy();
    policy["capability_roots"] = json!(["marketplace"]);
    let observed = root_rules_corpus(&["marketplace"], &["libs", "tools", "cloud/cloud-ci", "os"]);
    let report = evaluate(
        &policy,
        &baseline(&[("TDA-CAPABILITY-TIER-UNRESOLVED", "marketplace")]),
        &observed,
    );
    assert_eq!(
        report.verdict,
        Verdict::Red,
        "a baseline row must NOT excuse an unresolved capability tier; {:?}",
        report.findings
    );
}

#[test]
fn a_root_declared_in_both_lists_fails_closed() {
    // `capability_roots` and `unclassified_roots` are contradictory claims about the same root.
    // Silently preferring one would make the policy's plain reading wrong.
    let mut policy = policy();
    policy["capability_roots"] = json!(["iam"]);
    policy["unclassified_roots"] = json!(["libs", "iam"]);
    let report = evaluate(
        &policy,
        &baseline(&[]),
        &root_rules_corpus(&["iam"], &["os"]),
    );
    let f = report
        .findings
        .iter()
        .find(|f| f.code == "TDA-POLICY-MALFORMED")
        .expect("a contradictory root declaration must fail closed");
    assert!(f.detail.contains("iam"), "{}", f.detail);
}

#[test]
fn root_subject_baseline_rows_are_not_phantoms() {
    // Interaction guard between the R6b baseline rows and the B3 liveness backstop. A root subject
    // (`iam`) is never a crate dir, so without the ROOT_SUBJECT_CODES skip every one of the 21
    // committed root rows would fire TDA-STALE-BASELINE — a false RED on the live tree.
    let mut policy = policy();
    policy["unclassified_roots"] = json!(["libs", "iam"]);
    let observed = root_rules_corpus(&["iam"], &["libs", "os"]);
    let report = evaluate(
        &policy,
        &baseline(&[("TDA-UNCLASSIFIED-ROOT-NOT-META", "iam")]),
        &observed,
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.code == "TDA-STALE-BASELINE"),
        "a root subject must not be mistaken for a vanished crate dir; {:?}",
        report.findings
    );
    assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
    assert_eq!(report.baselined, 1);
}

#[test]
fn r6c_reds_a_capability_root_with_an_unrankable_stratum() {
    // CRITICAL-1. R6c tested that a tier was PRESENT, not that it was USABLE. `forward-declared` is
    // absent from `stratum_rank_order`, so `classify_edge`'s R4 arm looks it up, gets None, and the
    // `(Some, Some)` arm never matches — the root compares NOTHING. Meanwhile R6b is quiet (the root
    // left `unclassified_roots`) and R6c was quiet (a record exists).
    //
    // This survives the move to DECLARED tiers, and must: `forward-declared` is exactly as
    // unenforced whether a registry entry declared it or a projection derived it. Four capabilities
    // (ci/billing/storage/flags) have only forward-declared evidence to draw on today, so this is
    // the rule that keeps "just declare it" from becoming the next silent exemption.
    let mut policy = policy();
    policy["capability_roots"] = json!(["storage"]);
    let mut observed = root_rules_corpus(&["storage"], &["libs", "tools", "cloud/cloud-ci", "os"]);
    observed["service_tiers"] = json!({
        "storage": {"tier": "substrate", "stratum": "forward-declared"},
        "network": {"tier": "substrate", "stratum": "S1"}
    });

    let report = evaluate(&policy, &baseline(&[]), &observed);
    let f = report
        .findings
        .iter()
        .find(|f| f.code == "TDA-CAPABILITY-TIER-UNRESOLVED")
        .expect("a PRESENT-but-UNRANKABLE capability tier must fire R6c");
    assert_eq!(f.subject, "storage");
    assert_eq!(f.status, Status::Regression);
    assert!(
        f.detail.contains("forward-declared") && f.detail.contains("no ADR-0280 rank"),
        "the detail must name the cause, not the generic no-unanimity case; got {}",
        f.detail
    );
    assert_eq!(report.verdict, Verdict::Red);

    // The paired proof that this is a FALSE GREEN and not a cosmetic gap: an S0-ward edge out of the
    // forward-declared root is a real ADR-0280 inversion and R4 does not flag it.
    let mut with_edge = observed.clone();
    with_edge["crates"] = json!([
        {"dir": "storage/core/domain", "service": "storage"},
        {"dir": "network/core/residency", "service": "network"}
    ]);
    with_edge["edges"] = json!([{"from": "storage/core/domain", "to": "network/core/residency"}]);
    let report = evaluate(&policy, &baseline(&[]), &with_edge);
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.code == "TDA-S-RANK-INVERSION"),
        "an unrankable stratum compares nothing — that is the defect R6c must name; {:?}",
        report.findings
    );

    // GREEN once the stratum is RANKABLE.
    let mut ranked = observed;
    ranked["service_tiers"]["storage"] = json!({"tier": "substrate", "stratum": "S2"});
    let report = evaluate(&policy, &baseline(&[]), &ranked);
    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

#[test]
fn r6c_reds_a_capability_root_whose_class_no_rule_acts_on() {
    // The same hole one step up from CRITICAL-1, found while re-reading this predicate: it only
    // checked the STRATUM, so a class outside the rules' match arms passed. `classify_edge` matches
    // substrate/product/service-cell and falls through on anything else — `reserved` is a legal
    // tier_field_coverage enum value, and a typo like "susbtrate" lands in the same place. Either
    // one is a crate-bearing capability tree comparing NOTHING while the policy claims enforcement,
    // which is the exemption this whole rule exists to deny.
    for class in ["reserved", "susbtrate"] {
        let mut policy = policy();
        policy["capability_roots"] = json!(["capx"]);
        let mut observed = root_rules_corpus(&["capx"], &["libs", "tools", "cloud/cloud-ci", "os"]);
        observed["service_tiers"] = json!({ "capx": {"tier": class} });

        let report = evaluate(&policy, &baseline(&[]), &observed);
        let f = report
            .findings
            .iter()
            .find(|f| f.code == "TDA-CAPABILITY-TIER-UNRESOLVED")
            .unwrap_or_else(|| panic!("class `{class}` enforces nothing and must RED"));
        assert_eq!(f.status, Status::Regression);
        assert!(f.detail.contains(class), "{}", f.detail);
    }

    // A class the rules DO act on needs no stratum (R2/R3 compare classes alone).
    let mut policy = policy();
    policy["capability_roots"] = json!(["capx"]);
    let mut observed = root_rules_corpus(&["capx"], &["libs", "tools", "cloud/cloud-ci", "os"]);
    observed["service_tiers"] = json!({ "capx": {"tier": "product"} });
    assert!(
        evaluate(&policy, &baseline(&[]), &observed)
            .findings
            .is_empty()
    );
}

#[test]
fn root_rules_run_even_when_the_scan_is_broken() {
    // MEDIUM-5. R6b/R6c are evaluated over POLICY data, exactly as their comment says — so they must
    // not sit behind the `scan_is_broken` guard R6 needs. They did, which meant a scan broken enough
    // to trip the false-green floor silenced the two rules that do not depend on the scan at all.
    let mut policy = policy();
    policy["min_expected_crates"] = json!(700);
    policy["unclassified_roots"] = json!(["iam"]);
    policy["capability_roots"] = json!(["marketplace"]);
    let mut observed = root_rules_corpus(&["iam", "marketplace"], &["os"]);
    observed["crate_count"] = json!(0); // a broken scan

    let report = evaluate(&policy, &baseline(&[]), &observed);
    let codes: BTreeSet<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert!(codes.contains("TDA-EMPTY-SCAN"), "{:?}", report.findings);
    assert!(
        codes.contains("TDA-UNCLASSIFIED-ROOT-NOT-META")
            && codes.contains("TDA-CAPABILITY-TIER-UNRESOLVED"),
        "a broken scan must not silence the POLICY-only rules; got {codes:?}"
    );
}

#[test]
fn a_root_baseline_row_naming_an_undeclared_root_is_stale() {
    // MEDIUM-6. The B3 phantom-row class, reopened one level up: root rows had NO staleness detector
    // at all, and they are now 21 of the 38 committed rows. R6b re-derives its subjects from
    // `unclassified_roots` and R6c from `capability_roots`, so a row naming a root in neither can
    // never be re-derived — permanently inert, and it inflates `burned_down` forever.
    let mut policy = policy();
    policy["unclassified_roots"] = json!(["libs"]);
    let observed = root_rules_corpus(&["iam"], &["libs"]);

    let report = evaluate(
        &policy,
        &baseline(&[("TDA-UNCLASSIFIED-ROOT-NOT-META", "vanished-root")]),
        &observed,
    );
    let f = report
        .findings
        .iter()
        .find(|f| f.code == "TDA-STALE-BASELINE")
        .expect("a root row that no rule can re-derive must fire the liveness backstop");
    assert_eq!(f.subject, "vanished-root");
    assert_eq!(report.verdict, Verdict::Red);

    // A row whose root is still DECLARED is live debt, not a phantom — even while R6b reports it.
    let report = evaluate(
        &policy,
        &baseline(&[("TDA-UNCLASSIFIED-ROOT-NOT-META", "libs")]),
        &root_rules_corpus(&["iam"], &["os"]),
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.code == "TDA-STALE-BASELINE"),
        "a declared root's row is live debt, not stale; {:?}",
        report.findings
    );
}

#[test]
fn emit_baseline_carries_live_root_rows_forward_and_mints_none() {
    // HIGH-4. `--emit-baseline` is the remedy printed in TDA-STALE-BASELINE's detail AND in the
    // baseline `_comment`'s regenerate instruction. It dropped every ROOT row, so an operator
    // following the documented remedy turned 21 advisory rows into 21 blocking regressions.
    let mut policy = policy();
    policy["unclassified_roots"] = json!(["kept", "burned"]);
    let mut observed = root_rules_corpus(&["kept", "burned"], &[]);
    // `burned` moved to capability_roots, so R6b no longer reports it: its committed row is spent.
    policy["capability_roots"] = json!(["burned"]);
    policy["unclassified_roots"] = json!(["kept"]);
    observed["service_tiers"] = json!({"burned": {"tier": "substrate", "stratum": "S1"}});

    let committed = json!({
        "_comment": "hand-written prose that a re-emit must not destroy",
        "frozen_at_ref": "origin/dev abc123",
        "gate_id": GATE_ID,
        "violations": [
            {"code": "TDA-UNCLASSIFIED-ROOT-NOT-META", "subject": "kept"},
            {"code": "TDA-UNCLASSIFIED-ROOT-NOT-META", "subject": "burned"}
        ]
    });
    let report = evaluate(&policy, &committed, &observed);
    let doc = emit_baseline_doc(&report, &committed);
    let rows: Vec<(&str, &str)> = doc["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .map(|v| {
            (
                v["code"].as_str().expect("code"),
                v["subject"].as_str().expect("subject"),
            )
        })
        .collect();

    assert_eq!(
        rows,
        vec![("TDA-UNCLASSIFIED-ROOT-NOT-META", "kept")],
        "the still-live root row survives; the burned-down one drops"
    );
    assert_eq!(
        doc["_comment"], committed["_comment"],
        "prose is carried forward"
    );
    assert_eq!(doc["frozen_at_ref"], committed["frozen_at_ref"]);

    // And a root finding that was never committed can NEVER be laundered in by re-emitting.
    let mut policy = policy;
    policy["unclassified_roots"] = json!(["kept", "brand-new"]);
    let observed = root_rules_corpus(&["kept", "brand-new"], &[]);
    let report = evaluate(&policy, &committed, &observed);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.subject == "brand-new" && f.status == Status::Regression),
        "the new exemption is a live regression; {:?}",
        report.findings
    );
    let doc = emit_baseline_doc(&report, &committed);
    assert!(
        !doc["violations"]
            .as_array()
            .expect("violations")
            .iter()
            .any(|v| v["subject"] == "brand-new"),
        "re-emitting must never MINT a structural row: {doc:#}"
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
        (
            json!({"gate_id": "wrong"}),
            json!({"violations": []}),
            json!({}),
        ),
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
