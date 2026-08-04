#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_cloud_ci_multi_region_disposition_app::{Verdict, evaluate, evaluate_keyed};
use serde_json::json;

#[test]
fn green_active_active_manifest_and_doc_agree() {
    let input = json!({"rows": [{
        "service_id": "workflow",
        "manifest_path": "specs/microservices/workflow.json",
        "manifest_present": true,
        "manifest_disposition": "active_active",
        "manifest_disposition_valid": true,
        "doc_path": "oya/workflow/multi-region.md",
        "doc_present": true,
        "doc_disposition": "active_active",
        "doc_required_sections": {"disposition_statement": true, "rationale": true, "rpo_rto_numbers_if_active_passive": true},
        "deployment_shape_source": null,
        "deployment_shape_disposition": null
    }]});

    assert_eq!(evaluate(&input).verdict, Verdict::Green);
    assert!(evaluate_keyed(&input).is_empty());
}

#[test]
fn missing_manifest_field_and_doc_are_red() {
    let input = json!({"rows": [{
        "service_id": "mail",
        "manifest_path": "oya/mail/manifest.json",
        "manifest_present": true,
        "manifest_disposition": null,
        "manifest_disposition_valid": false,
        "doc_present": false
    }]});

    let findings = evaluate_keyed(&input);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "manifest_missing_multi_region_disposition" && f.key == "mail")
    );
    assert!(
        findings
            .iter()
            .any(|f| f.code == "multi_region_doc_missing" && f.key == "mail")
    );
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn invalid_doc_mismatch_rpo_and_deployment_shape_are_red() {
    let input = json!({"rows": [{
        "service_id": "calendar",
        "manifest_present": true,
        "manifest_disposition": "active_passive",
        "manifest_disposition_valid": true,
        "doc_present": true,
        "doc_disposition": "single_region",
        "doc_required_sections": {"disposition_statement": false, "rationale": true, "rpo_rto_numbers_if_active_passive": false},
        "deployment_shape_source": "fixture",
        "deployment_shape_disposition": "active_active"
    }]});

    let codes = evaluate(&input).violations;
    for code in [
        "multi_region_doc_missing_required_section",
        "active_passive_missing_rpo_rto",
        "disposition_doc_mismatch",
        "deployment_shape_mismatch",
    ] {
        assert!(codes.contains(code), "missing {code}: {codes:?}");
    }
}

#[test]
fn invalid_manifest_enum_is_red() {
    let input = json!({"rows": [{
        "service_id": "bad-enum",
        "manifest_present": true,
        "manifest_disposition": "global_magic",
        "manifest_disposition_valid": false,
        "doc_present": true,
        "doc_disposition": "active_active",
        "doc_required_sections": {"disposition_statement": true, "rationale": true}
    }]});

    assert!(
        evaluate(&input)
            .violations
            .contains("manifest_invalid_multi_region_disposition")
    );
}

#[test]
fn missing_rows_fail_closed_instead_of_false_green() {
    let input = json!({"rows": []});
    let findings = evaluate_keyed(&input);

    assert!(findings.iter().any(|finding| {
        finding.code == "manifest_missing_multi_region_disposition"
            && finding.key == "<multi-region-disposition-corpus>"
    }));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}
