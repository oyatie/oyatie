// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! F3 adversarial fixtures for the `change-class-declared` and
//! `multispectrum-evidence-attached` sub-checks. Crafted evidence JSON
//! files in tempdir; assert the lane catches the violation classes.

use std::fs;
use std::path::{Path, PathBuf};

use oya_check_dependency_seam::{
    SubCheckStatus, WorkspaceContext, check_change_class_declared,
    check_multispectrum_evidence_attached, extract_string_field,
};

use std::sync::atomic::{AtomicU64, Ordering};
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn make_tmp_workspace() -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let seq = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("oya-evidence-test-{}-{}-{}", pid, stamp, seq));
    fs::create_dir_all(base.join("evidence/multispectrum")).unwrap();
    base
}

fn write_evidence(root: &Path, name: &str, body: &str) {
    fs::write(
        root.join("evidence/multispectrum")
            .join(format!("{}.json", name)),
        body,
    )
    .unwrap();
}

// ---- change-class-declared ----

#[test]
fn change_class_declared_passes_when_canonical_present() {
    let root = make_tmp_workspace();
    write_evidence(
        &root,
        "ok",
        r#"{ "change_class_id": "CC-1_kernel_public_api" }"#,
    );
    let result = check_change_class_declared(&WorkspaceContext::new(&root));
    assert_eq!(result.status, SubCheckStatus::Pass, "{:?}", result.findings);
}

#[test]
fn change_class_declared_fails_when_non_canonical() {
    let root = make_tmp_workspace();
    write_evidence(
        &root,
        "bad",
        r#"{ "change_class_id": "CC-99_made_up_class" }"#,
    );
    let result = check_change_class_declared(&WorkspaceContext::new(&root));
    assert_eq!(result.status, SubCheckStatus::Fail);
    let joined = result.findings.join("\n");
    assert!(joined.contains("non-canonical"));
    assert!(joined.contains("CC-99_made_up_class"));
}

#[test]
fn change_class_declared_fails_when_field_missing() {
    let root = make_tmp_workspace();
    write_evidence(&root, "missing", r#"{ "change_id": "X-123" }"#);
    let result = check_change_class_declared(&WorkspaceContext::new(&root));
    assert_eq!(result.status, SubCheckStatus::Fail);
    let joined = result.findings.join("\n");
    assert!(joined.contains("change_class_id missing"));
}

#[test]
fn change_class_declared_reports_each_violation() {
    let root = make_tmp_workspace();
    write_evidence(
        &root,
        "a-ok",
        r#"{ "change_class_id": "CC-2_adapter_or_infrastructure" }"#,
    );
    write_evidence(&root, "b-bad", r#"{ "change_class_id": "CC-bogus" }"#);
    write_evidence(&root, "c-missing", r#"{ }"#);
    let result = check_change_class_declared(&WorkspaceContext::new(&root));
    assert_eq!(result.status, SubCheckStatus::Fail);
    let joined = result.findings.join("\n");
    // a-ok is canonical so it doesn't appear in violations
    assert!(joined.contains("b-bad"));
    assert!(joined.contains("c-missing"));
}

// ---- multispectrum-evidence-attached ----

fn full_evidence_body(change_class: &str) -> String {
    // v2.0.0 of multispectrum-review.json requires F1..F9 facets present.
    // M1/M2 are optional meta-facets per the spec; not included here.
    format!(
        r#"{{
  "change_class_id": "{}",
  "git_sha": "abc1234",
  "freshness_unix": 1700000001,
  "facets": {{
    "F1_linus": {{}},
    "F2_hyperscaler": {{}},
    "F3_adversarial": {{}},
    "F4_ergonomic": {{}},
    "F5_quality": {{}},
    "F6_alternatives": {{}},
    "F7_security": {{}},
    "F8_performance": {{}},
    "F9_compliance": {{}}
  }}
}}
"#,
        change_class
    )
}

#[test]
fn evidence_attached_passes_full_shape() {
    let root = make_tmp_workspace();
    write_evidence(&root, "full", &full_evidence_body("CC-1_kernel_public_api"));
    let result = check_multispectrum_evidence_attached(&WorkspaceContext::new(&root));
    assert_eq!(result.status, SubCheckStatus::Pass, "{:?}", result.findings);
}

#[test]
fn evidence_attached_fails_when_top_key_missing() {
    let root = make_tmp_workspace();
    let body = r#"{ "git_sha": "abc1234", "freshness_unix": 1, "facets": { "F1_linus": {}, "F2_hyperscaler": {}, "F3_adversarial": {}, "F4_ergonomic": {}, "F5_quality": {}, "F6_alternatives": {}, "F7_security": {} } }"#;
    write_evidence(&root, "missing-class", body);
    let result = check_multispectrum_evidence_attached(&WorkspaceContext::new(&root));
    assert_eq!(result.status, SubCheckStatus::Fail);
    let joined = result.findings.join("\n");
    assert!(joined.contains("change_class_id"));
}

#[test]
fn evidence_attached_fails_when_facet_missing() {
    let root = make_tmp_workspace();
    // Missing F7_security AND F8_performance AND F9_compliance in facets block
    // (v2.0.0 added F8 + F9 as required).
    let body = r#"{
        "change_class_id": "CC-1_kernel_public_api",
        "git_sha": "abc",
        "freshness_unix": 1,
        "facets": {
          "F1_linus": {}, "F2_hyperscaler": {}, "F3_adversarial": {},
          "F4_ergonomic": {}, "F5_quality": {}, "F6_alternatives": {}
        }
    }"#;
    write_evidence(&root, "missing-facet", body);
    let result = check_multispectrum_evidence_attached(&WorkspaceContext::new(&root));
    assert_eq!(result.status, SubCheckStatus::Fail);
    let joined = result.findings.join("\n");
    assert!(joined.contains("facets.F7_security"));
    assert!(joined.contains("facets.F8_performance"));
    assert!(joined.contains("facets.F9_compliance"));
}

// F3 adversarial v2.0.0: evidence with all 9 required facets but missing
// F8 alone is still detected.
#[test]
fn evidence_attached_fails_when_only_f8_missing() {
    let root = make_tmp_workspace();
    let body = r#"{
        "change_class_id": "CC-1_kernel_public_api",
        "git_sha": "abc",
        "freshness_unix": 1,
        "facets": {
          "F1_linus": {}, "F2_hyperscaler": {}, "F3_adversarial": {},
          "F4_ergonomic": {}, "F5_quality": {}, "F6_alternatives": {},
          "F7_security": {}, "F9_compliance": {}
        }
    }"#;
    write_evidence(&root, "missing-f8", body);
    let result = check_multispectrum_evidence_attached(&WorkspaceContext::new(&root));
    assert_eq!(result.status, SubCheckStatus::Fail);
    let joined = result.findings.join("\n");
    assert!(joined.contains("facets.F8_performance"));
    assert!(!joined.contains("facets.F7_security"));
}

#[test]
fn evidence_attached_not_yet_armed_when_dir_empty() {
    let root = make_tmp_workspace();
    let result = check_multispectrum_evidence_attached(&WorkspaceContext::new(&root));
    assert_eq!(result.status, SubCheckStatus::NotYetArmed);
}

// ---- extract_string_field ----

#[test]
fn extract_string_field_returns_first_occurrence_value() {
    let raw = r#"{ "other": "x", "target": "found", "second": "ignored" }"#;
    assert_eq!(
        extract_string_field(raw, "target").as_deref(),
        Some("found")
    );
}

#[test]
fn extract_string_field_returns_none_when_absent() {
    let raw = r#"{ "other": "x" }"#;
    assert_eq!(extract_string_field(raw, "missing"), None);
}

#[test]
fn extract_string_field_handles_whitespace_around_colon() {
    let raw = r#"{ "target"   :    "value-with-spaces"   }"#;
    assert_eq!(
        extract_string_field(raw, "target").as_deref(),
        Some("value-with-spaces")
    );
}
