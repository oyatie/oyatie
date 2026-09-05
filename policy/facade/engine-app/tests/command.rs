#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod support;

use policy_engine_app::{CommandOutput, qualify_json};

#[test]
fn check_and_prepare_emit_reports_and_unsigned_qualified_candidates() {
    let source = serde_json::to_vec(&support::project()).unwrap();
    let report: serde_json::Value =
        serde_json::from_str(&qualify_json(&source, CommandOutput::Report).unwrap()).unwrap();
    assert_eq!(report["passed_cases"], 2);
    let bundle: policy_pdp_kernel::PolicyBundle =
        serde_json::from_str(&qualify_json(&source, CommandOutput::UnsignedBundle).unwrap())
            .unwrap();
    assert_eq!(report["policy_version"], bundle.version.as_str());
    policy_pdp_cedar::validate_bundle(&bundle).unwrap();
    let fixture = include_bytes!("fixtures/read-access.json");
    assert!(qualify_json(fixture, CommandOutput::Report).is_ok());
}

#[test]
fn invalid_json_and_failed_cases_do_not_emit_an_artifact() {
    assert!(qualify_json(b"not JSON", CommandOutput::UnsignedBundle).is_err());
    let mut source = support::project();
    source.cases[0].expected.obligations.clear();
    assert!(
        qualify_json(
            &serde_json::to_vec(&source).unwrap(),
            CommandOutput::UnsignedBundle
        )
        .is_err()
    );
}
