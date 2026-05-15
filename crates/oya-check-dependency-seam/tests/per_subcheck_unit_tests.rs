//! Per-sub-check unit tests for the 4 TG2 sub-checks.
//! Resolves CONV-6 from TG2 11-facet debate synthesis (F3 adversarial + F4
//! ergonomic convergence: prior to this file, the canonical-IDs test asserted
//! wiring only; behavior was untested).
//!
//! Each sub-check has 2 tests:
//!   - happy-path  → scan tree shape that the (eventual) armed check would Pass
//!   - failing-path → scan tree shape that the (eventual) armed check would Fail
//!
//! All 4 sub-checks today return NotYetArmed regardless of input (day-1 stubs
//! aligned per CONV-3), so tests assert on FINDINGS content, not status. When
//! the FixupTasks F-LANE-RUST-DEFAULT-ENFORCE / F-NAMING-CONVENTION-ENFORCE /
//! F-LANE-SCORECARD-RENDER / F-LANE-DEBATE-SUBCHECK promote status to Pass/
//! Fail, these tests get extended with status assertions then.
//!
//! Std-only: builds a fresh tmp workspace per test via std::env::temp_dir +
//! std::fs. No tempfile crate dep (matches kernel std-only policy).

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use oya_check_dependency_seam::{
    check_a6_schema_adherence, check_consensus_debate_evidence, check_naming_convention,
    check_rust_default_language, check_scorecard_render, parse_top_level_object,
    render_audit_chain_rows, run_composite, JsonValueKind, SubCheckStatus, WorkspaceContext,
    ALL_FACETS_FROM_SPEC, CHANGE_CLASSES, CHANGE_CLASSES_FROM_SPEC, EVIDENCE_REQUIRED_FACETS,
};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Build a fresh tmp workspace dir unique to this test invocation.
fn make_workspace() -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("oya-seam-test-{}-{}-{}", pid, nanos, n));
    fs::create_dir_all(&dir).expect("mkdir tmp workspace");
    dir
}

fn cleanup(ws: &PathBuf) {
    let _ = fs::remove_dir_all(ws);
}

fn findings_contain(result: &oya_check_dependency_seam::SubCheckResult, needle: &str) -> bool {
    result.findings.iter().any(|f| f.contains(needle))
}

// =============== check_rust_default_language ===============

#[test]
fn rust_default_language_happy_path_no_non_rust_scripts() {
    let ws = make_workspace();
    let scripts = ws.join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(scripts.join("ok.rs"), "fn main() {}").unwrap();
    fs::write(scripts.join("README.md"), "docs").unwrap();

    let result = check_rust_default_language(&WorkspaceContext::new(&ws));
    assert_eq!(result.id, "rust-default-language");
    assert_eq!(result.status, SubCheckStatus::NotYetArmed);
    assert!(
        findings_contain(&result, "non-Rust file count: 0"),
        "happy path should report count: 0, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

#[test]
fn rust_default_language_failing_path_counts_disallowed_extensions() {
    let ws = make_workspace();
    let scripts = ws.join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(scripts.join("setup.sh"), "#!/bin/sh\n").unwrap();
    fs::write(scripts.join("helper.py"), "print('x')").unwrap();
    fs::write(scripts.join("lint.mjs"), "// x").unwrap();
    fs::write(scripts.join("ok.rs"), "fn main() {}").unwrap();

    let result = check_rust_default_language(&WorkspaceContext::new(&ws));
    assert_eq!(result.status, SubCheckStatus::NotYetArmed);
    assert!(
        findings_contain(&result, "non-Rust file count: 3"),
        "failing path should report count: 3, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

#[test]
fn rust_default_language_surfaces_io_error_when_scripts_missing() {
    let ws = make_workspace(); // intentionally NO scripts/ dir
    let result = check_rust_default_language(&WorkspaceContext::new(&ws));
    assert!(
        findings_contain(&result, "read_dir") && findings_contain(&result, "failed"),
        "missing scripts/ should surface read_dir failure (CONV-2 fix), got {:?}",
        result.findings
    );
    cleanup(&ws);
}

// =============== check_naming_convention ===============

#[test]
fn naming_convention_happy_path_kebab_case_files() {
    let ws = make_workspace();
    let home = ws.join("specs/cross-cutting");
    fs::create_dir_all(&home).unwrap();
    fs::write(home.join("oyatie-doctrine-v1.0.0.json"), "{}").unwrap();
    fs::write(home.join("fixuptasks.jsonl"), "").unwrap();

    let result = check_naming_convention(&WorkspaceContext::new(&ws));
    assert_eq!(result.status, SubCheckStatus::NotYetArmed);
    assert!(
        findings_contain(&result, "kebab-case violations: 0"),
        "happy path should report 0 violations, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

#[test]
fn naming_convention_failing_path_detects_uppercase_and_snake_case() {
    let ws = make_workspace();
    let home = ws.join("specs/cross-cutting");
    fs::create_dir_all(&home).unwrap();
    fs::write(home.join("BadCase.json"), "{}").unwrap();
    fs::write(home.join("snake_case.json"), "{}").unwrap();
    fs::write(home.join("ok-kebab.json"), "{}").unwrap();

    let result = check_naming_convention(&WorkspaceContext::new(&ws));
    assert_eq!(result.status, SubCheckStatus::NotYetArmed);
    assert!(
        findings_contain(&result, "kebab-case violations: 2"),
        "failing path should report 2 violations, got {:?}",
        result.findings
    );
    assert!(
        findings_contain(&result, "BadCase.json"),
        "violation list should include BadCase.json"
    );
    assert!(
        findings_contain(&result, "snake_case.json"),
        "violation list should include snake_case.json"
    );
    cleanup(&ws);
}

// =============== check_scorecard_render ===============

#[test]
fn scorecard_render_happy_path_evidence_with_required_keys() {
    let ws = make_workspace();
    let dir = ws.join("evidence/multispectrum");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("renderable.json"),
        r#"{"change_class_id": "CC-7", "facets": {}}"#,
    )
    .unwrap();

    let result = check_scorecard_render(&WorkspaceContext::new(&ws));
    assert_eq!(result.status, SubCheckStatus::NotYetArmed);
    assert!(
        findings_contain(&result, "evidence files scanned: 1; minimum-renderable as scorecard: 1"),
        "happy path should report 1/1, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

#[test]
fn scorecard_render_failing_path_missing_facets_key() {
    let ws = make_workspace();
    let dir = ws.join("evidence/multispectrum");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("incomplete.json"),
        r#"{"change_class_id": "CC-5"}"#,
    )
    .unwrap();

    let result = check_scorecard_render(&WorkspaceContext::new(&ws));
    assert!(
        findings_contain(&result, "evidence files scanned: 1; minimum-renderable as scorecard: 0"),
        "failing path: file missing 'facets' should not be renderable, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

// =============== check_consensus_debate_evidence ===============

#[test]
fn consensus_debate_happy_path_meta_triggered_with_matching_synthesis() {
    let ws = make_workspace();
    let multispectrum = ws.join("evidence/multispectrum");
    let debate = ws.join("evidence/debate");
    fs::create_dir_all(&multispectrum).unwrap();
    fs::create_dir_all(&debate).unwrap();
    fs::write(
        multispectrum.join("CHG-X-r1.json"),
        r#"{"meta_review_triggered": true}"#,
    )
    .unwrap();
    fs::write(debate.join("CHG-X-synthesis.json"), r#"{}"#).unwrap();

    let result = check_consensus_debate_evidence(&WorkspaceContext::new(&ws));
    assert!(
        findings_contain(&result, "meta_review_triggered: 1") && findings_contain(&result, "synthesis files present: 1"),
        "happy path should report balanced meta=1/synthesis=1, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

// =============== JSON parser (CONV-1) ===============

// =============== A6 schema_adherence (v2.3.0 A-family) ===============

#[test]
fn a6_schema_adherence_happy_path_compliant_spec() {
    let ws = make_workspace();
    let home = ws.join("specs/cross-cutting");
    fs::create_dir_all(&home).unwrap();
    fs::write(
        home.join("ok.json"),
        r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"https://docs.oyatie.dev/specs/cross-cutting/ok.json","_meta":{"doc_class":"Spec"}}"#,
    )
    .unwrap();

    let result = check_a6_schema_adherence(&WorkspaceContext::new(&ws));
    assert_eq!(result.id, "a6-schema-adherence");
    assert_eq!(result.status, SubCheckStatus::NotYetArmed);
    assert!(
        findings_contain(&result, "JSON files scanned: 1; ADR-0069 minimum-keys-compliant ($schema+$id+_meta): 1; non-compliant: 0"),
        "happy path should report 1/1 compliant, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

#[test]
fn a6_schema_adherence_failing_path_missing_keys() {
    let ws = make_workspace();
    let home = ws.join("specs/cross-cutting");
    fs::create_dir_all(&home).unwrap();
    fs::write(home.join("no-schema.json"), r#"{"$id":"x","_meta":{}}"#).unwrap();
    fs::write(home.join("no-id.json"), r#"{"$schema":"x","_meta":{}}"#).unwrap();
    fs::write(home.join("no-meta.json"), r#"{"$schema":"x","$id":"y"}"#).unwrap();
    fs::write(
        home.join("compliant.json"),
        r#"{"$schema":"x","$id":"y","_meta":{}}"#,
    )
    .unwrap();

    let result = check_a6_schema_adherence(&WorkspaceContext::new(&ws));
    assert!(
        findings_contain(&result, "JSON files scanned: 4; ADR-0069 minimum-keys-compliant ($schema+$id+_meta): 1; non-compliant: 3"),
        "failing path: 1/4 compliant, got {:?}",
        result.findings
    );
    assert!(findings_contain(&result, "missing $schema"));
    assert!(findings_contain(&result, "missing $id"));
    assert!(findings_contain(&result, "missing _meta"));
    cleanup(&ws);
}

// =============== spec/code parity drift detection (CONV-8) ===============

#[test]
fn change_classes_matches_spec_no_drift() {
    use std::collections::BTreeSet;
    let hand: BTreeSet<&str> = CHANGE_CLASSES.iter().copied().collect();
    let spec: BTreeSet<&str> = CHANGE_CLASSES_FROM_SPEC.iter().copied().collect();
    assert_eq!(
        hand, spec,
        "CHANGE_CLASSES drift between hand-rolled lane constant and \
         specs/cross-cutting/multispectrum-review.json#change_classes. \
         Hand-only: {:?} | Spec-only: {:?}",
        hand.difference(&spec).collect::<Vec<_>>(),
        spec.difference(&hand).collect::<Vec<_>>()
    );
}

#[test]
fn evidence_required_facets_is_subset_of_spec() {
    use std::collections::BTreeSet;
    let hand: BTreeSet<&str> = EVIDENCE_REQUIRED_FACETS.iter().copied().collect();
    let spec: BTreeSet<&str> = ALL_FACETS_FROM_SPEC.iter().copied().collect();
    let extras: Vec<&&str> = hand.difference(&spec).collect();
    assert!(
        extras.is_empty(),
        "EVIDENCE_REQUIRED_FACETS contains keys not present in spec's facets object: {:?}",
        extras
    );
}

#[test]
fn spec_constants_from_spec_non_empty() {
    assert!(
        !CHANGE_CLASSES_FROM_SPEC.is_empty(),
        "build.rs did not populate CHANGE_CLASSES_FROM_SPEC; spec read may have failed"
    );
    assert!(
        !ALL_FACETS_FROM_SPEC.is_empty(),
        "build.rs did not populate ALL_FACETS_FROM_SPEC"
    );
}

// =============== perf-budget regression gate (CONV-7) ===============

#[test]
fn composite_suite_meets_perf_budget_on_synthetic_workload() {
    // CONV-7 (TG2 11-facet debate): F2 hyperscaler + F8 performance flagged
    // no benchmark + no perf-budget regression test. Std-only constraint
    // forbids the criterion crate; this test is a coarse regression gate
    // (1 run, no warmup, no statistical confidence) — full criterion bench
    // deferred to F-SEAM-LANE-BENCHMARK-PERF-BUDGET-FULL when std-only is
    // relaxed for [dev-dependencies].
    //
    // Budget: 500ms for a 30-file synthetic workload (10x today's live count).
    // If this fails on a normal dev machine, a real regression exists.
    let ws = make_workspace();
    // 10 evidence/multispectrum files
    let evidence = ws.join("evidence/multispectrum");
    fs::create_dir_all(&evidence).unwrap();
    for i in 0..10 {
        fs::write(
            evidence.join(format!("ev-{:03}.json", i)),
            r#"{"change_class_id":"CC-7","facets":{},"meta_review_triggered":true}"#,
        )
        .unwrap();
    }
    // 10 evidence/per-change files
    let per_change = ws.join("evidence/per-change");
    fs::create_dir_all(&per_change).unwrap();
    for i in 0..10 {
        fs::write(
            per_change.join(format!("pc-{:03}.json", i)),
            r#"{"change_class_id":"CC-4","facets":{}}"#,
        )
        .unwrap();
    }
    // 10 evidence/debate synthesis files
    let debate = ws.join("evidence/debate");
    fs::create_dir_all(&debate).unwrap();
    for i in 0..10 {
        fs::write(
            debate.join(format!("CHG-{:03}-synthesis.json", i)),
            r#"{"termination_reason":"consensus_reached"}"#,
        )
        .unwrap();
    }
    // Scripts dir + 1 valid Rust file (rust-default-language target)
    let scripts = ws.join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(scripts.join("ok.rs"), "fn main(){}").unwrap();
    // Canonical homes (naming-convention scan targets)
    for home in ["specs/cross-cutting", "registries/cross-cutting", "templates"] {
        fs::create_dir_all(ws.join(home)).unwrap();
    }

    let start = std::time::Instant::now();
    let report = run_composite(&WorkspaceContext::new(&ws));
    let elapsed = start.elapsed();

    // Sanity: composite ran all 11 sub-checks (10 TG2 + A6 v2.3.0).
    assert_eq!(report.sub_checks.len(), 11);

    let budget_ms = 500u128;
    let actual_ms = elapsed.as_millis();
    assert!(
        actual_ms <= budget_ms,
        "composite suite exceeded perf budget: {}ms > {}ms on 30-file synthetic workload",
        actual_ms,
        budget_ms
    );
    cleanup(&ws);
}

// =============== render_audit_chain_rows (CONV-9) ===============

#[test]
fn audit_chain_rows_one_per_sub_check_with_required_keys() {
    let ws = make_workspace();
    let report = run_composite(&WorkspaceContext::new(&ws));
    let rows = render_audit_chain_rows(&report, "CHG-TEST-X", "session-y", 1700000000);
    assert_eq!(rows.len(), report.sub_checks.len(),
        "one audit-chain row per sub-check");
    for (idx, row) in rows.iter().enumerate() {
        let sub_check_id = report.sub_checks[idx].id;
        assert!(row.starts_with("{"), "row {} must be JSON object", idx);
        assert!(row.contains("\"event_type\":\"seam_lane_subcheck_run\""), "missing event_type in row {}", idx);
        assert!(row.contains("\"change_id\":\"CHG-TEST-X\""), "missing change_id");
        assert!(row.contains("\"session_id\":\"session-y\""), "missing session_id");
        assert!(row.contains("\"timestamp_unix\":1700000000"), "missing timestamp_unix");
        assert!(row.contains(&format!("\"sub_check_id\":\"{}\"", sub_check_id)),
            "row {} missing sub_check_id {}", idx, sub_check_id);
        assert!(row.contains("\"status\":\""), "missing status");
        assert!(row.contains("\"findings_count\":"), "missing findings_count");
    }
    cleanup(&ws);
}

#[test]
fn audit_chain_row_escapes_double_quote_in_findings() {
    use oya_check_dependency_seam::{CompositeReport, Severity, SubCheckResult};
    let report = CompositeReport {
        sub_checks: vec![SubCheckResult {
            id: "test-x",
            status: SubCheckStatus::Pass,
            findings: vec!["a \"quoted\" finding".into()],
            severity_day_1: Severity::ReportOnly,
        }],
    };
    let rows = render_audit_chain_rows(&report, "C", "S", 0);
    assert_eq!(rows.len(), 1);
    assert!(rows[0].contains("\\\"quoted\\\""),
        "double-quote in finding must be escaped, got: {}", rows[0]);
}

// =============== JSON parser (CONV-1) ===============

#[test]
fn parse_top_level_object_classifies_kinds() {
    let raw = r#"{"a": true, "b": false, "c": null, "d": 42, "e": "str", "f": {}, "g": [1,2]}"#;
    let m = parse_top_level_object(raw);
    assert_eq!(m.get("a"), Some(&JsonValueKind::BoolTrue));
    assert_eq!(m.get("b"), Some(&JsonValueKind::BoolFalse));
    assert_eq!(m.get("c"), Some(&JsonValueKind::Null));
    assert_eq!(m.get("d"), Some(&JsonValueKind::Number));
    assert_eq!(m.get("e"), Some(&JsonValueKind::String));
    assert_eq!(m.get("f"), Some(&JsonValueKind::Object));
    assert_eq!(m.get("g"), Some(&JsonValueKind::Array));
}

#[test]
fn parse_top_level_object_tolerates_whitespace_variants() {
    // F7 finding: substring grep failed on `"meta_review_triggered" :  true`
    // (extra whitespace before colon + multi-space after) — parser must handle.
    let raw1 = r#"{"meta_review_triggered" :  true}"#;
    let raw2 = r#"{"meta_review_triggered":true}"#;
    let raw3 = "{\n  \"meta_review_triggered\":\n    true\n}";
    for raw in [raw1, raw2, raw3] {
        let m = parse_top_level_object(raw);
        assert_eq!(
            m.get("meta_review_triggered"),
            Some(&JsonValueKind::BoolTrue),
            "parser failed on whitespace variant: {:?}",
            raw
        );
    }
}

#[test]
fn parse_top_level_object_rejects_substring_bypass() {
    // F7 finding: substring grep false-positives when literal appears inside
    // nested string value. Parser must NOT confuse this for top-level key.
    let raw = r#"{
      "note": "we discussed meta_review_triggered: true in round 1",
      "change_id": "CC-X"
    }"#;
    let m = parse_top_level_object(raw);
    assert_eq!(m.get("meta_review_triggered"), None,
        "embedded string occurrence MUST NOT be classified as top-level key");
    assert_eq!(m.get("note"), Some(&JsonValueKind::String));
    assert_eq!(m.get("change_id"), Some(&JsonValueKind::String));
}

#[test]
fn parse_top_level_object_skips_nested_object_contents_correctly() {
    let raw = r#"{
      "facets": {"F1_linus": {"considered": true}, "F2": {"considered": false}},
      "change_class_id": "CC-7"
    }"#;
    let m = parse_top_level_object(raw);
    assert_eq!(m.get("facets"), Some(&JsonValueKind::Object));
    assert_eq!(m.get("change_class_id"), Some(&JsonValueKind::String));
    // Nested keys must NOT leak to top-level map.
    assert_eq!(m.get("F1_linus"), None);
    assert_eq!(m.get("considered"), None);
}

#[test]
fn parse_top_level_object_returns_empty_on_malformed_input() {
    assert!(parse_top_level_object("not json").is_empty());
    assert!(parse_top_level_object("").is_empty());
    assert!(parse_top_level_object("[1, 2, 3]").is_empty()); // array, not object
}

#[test]
fn consensus_debate_failing_path_meta_triggered_no_synthesis() {
    let ws = make_workspace();
    let multispectrum = ws.join("evidence/multispectrum");
    let debate = ws.join("evidence/debate");
    fs::create_dir_all(&multispectrum).unwrap();
    fs::create_dir_all(&debate).unwrap();
    fs::write(
        multispectrum.join("CHG-Y-r1.json"),
        r#"{"meta_review_triggered": true}"#,
    )
    .unwrap();
    // Intentionally NO synthesis file in debate/

    let result = check_consensus_debate_evidence(&WorkspaceContext::new(&ws));
    assert!(
        findings_contain(&result, "meta_review_triggered: 1") && findings_contain(&result, "synthesis files present: 0"),
        "failing path: meta=1, synthesis=0 imbalance, got {:?}",
        result.findings
    );
    cleanup(&ws);
}
