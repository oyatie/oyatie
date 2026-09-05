#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use policy_cli::{CommandOutput, qualify_json};

const FIXTURE: &[u8] = include_bytes!("fixtures/read-access.json");

#[test]
fn check_and_prepare_emit_reports_and_unsigned_qualified_candidates() {
    let report: serde_json::Value =
        serde_json::from_str(&qualify_json(FIXTURE, CommandOutput::Report).unwrap()).unwrap();
    assert_eq!(report["passed_cases"], 2);
    let bundle: policy_pdp_kernel::PolicyBundle =
        serde_json::from_str(&qualify_json(FIXTURE, CommandOutput::UnsignedBundle).unwrap())
            .unwrap();
    assert_eq!(report["policy_version"], bundle.version.as_str());
    policy_pdp_cedar::validate_bundle(&bundle).unwrap();
}

#[test]
fn invalid_json_and_failed_cases_do_not_emit_an_artifact() {
    assert!(qualify_json(b"not JSON", CommandOutput::UnsignedBundle).is_err());
    let mut project: serde_json::Value = serde_json::from_slice(FIXTURE).unwrap();
    project["cases"][0]["expected"]["obligations"] = serde_json::json!([]);
    assert!(
        qualify_json(
            &serde_json::to_vec(&project).unwrap(),
            CommandOutput::UnsignedBundle
        )
        .is_err()
    );
}
