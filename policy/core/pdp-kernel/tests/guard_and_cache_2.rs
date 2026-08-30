// ADR-0083 Tier 3: integration tests legitimately use `.unwrap()` / `.expect()` / `panic!()`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use policy_pdp_kernel::*;
use shared_platform_contracts_kernel::pdp::*;

mod guard_and_cache_fixtures;

use guard_and_cache_fixtures::*;

#[test]
fn tenant_policies_round_trip_through_serde_deterministically() {
    let bundle = PolicyBundle {
        version: PolicyVersion::new("psv-000001").unwrap(),
        schema_src: "schema".to_owned(),
        policies_src: "policies".to_owned(),
        tenant_policies: BTreeMap::from([
            ("globex".to_owned(), "// globex overlay".to_owned()),
            ("acme".to_owned(), "// acme overlay".to_owned()),
        ]),
        templates: vec![],
        template_links: vec![],
        action_map: BTreeMap::new(),
    };
    let json = serde_json::to_string(&bundle).unwrap();
    let back: PolicyBundle = serde_json::from_str(&json).unwrap();
    assert_eq!(back, bundle);
    // BTreeMap keeps overlays in a deterministic (sorted) order.
    let keys: Vec<&String> = back.tenant_policies.keys().collect();
    assert_eq!(keys, vec!["acme", "globex"]);
}

#[test]
fn unknown_bundle_field_is_rejected_closed_schema() {
    let mut value: serde_json::Value =
        serde_json::from_str(&seed_bundle_json_without_overlays()).unwrap();
    value["smuggled"] = serde_json::json!("x");
    assert!(
        serde_json::from_value::<PolicyBundle>(value).is_err(),
        "deny_unknown_fields must still reject unknown fields after the overlay addition"
    );
}

#[test]
fn pdp_error_messages_are_legible() {
    let e = PdpError::StalePolicyVersion {
        required: PolicyVersion::new("psv-2").unwrap(),
        loaded: PolicyVersion::new("psv-1").unwrap(),
    };
    assert_eq!(
        e.to_string(),
        "policy bundle too stale: caller pinned psv-2 but loaded version is psv-1"
    );
}

#[test]
fn decision_authz_request_projects_target_tenant_into_pdp_request() {
    let request = decision_authz_request("acme", "globex");

    let pdp_request = request
        .to_authorization_request("req-pdp-1", Some(PolicyVersion::new("psv-9").unwrap()))
        .unwrap();

    assert_eq!(pdp_request.tenant_id, "globex");
    assert_eq!(pdp_request.principal.entity_type, "OyaPlatform::Principal");
    assert_eq!(
        pdp_request.principal.entity_id,
        serde_json::json!(["acme", "control-plane"]).to_string()
    );
    assert_eq!(
        pdp_request.resource.entity_type,
        "OyaPlatform::TenantResource"
    );
    assert_eq!(
        pdp_request.context.get("caller_tenant"),
        Some(&serde_json::json!("acme"))
    );
    assert_eq!(
        pdp_request.context.get("caller_id"),
        Some(&serde_json::json!("control-plane"))
    );
    assert_eq!(
        pdp_request.context.get("target_tenant"),
        Some(&serde_json::json!("globex"))
    );
    assert_eq!(
        pdp_request.context.get("target_subject_id"),
        Some(&serde_json::json!("wl-secrets-sync"))
    );
}

#[test]
fn projection_refuses_empty_fields_before_pdp_request() {
    let fault = decision_authz_request("", "acme")
        .to_authorization_request("req-pdp-1", None)
        .unwrap_err();
    assert_eq!(
        fault,
        DecisionAuthzError::MissingValue {
            field: "caller_tenant"
        }
    );

    let fault = DecisionAuthzRequest {
        action: "",
        ..decision_authz_request("acme", "acme")
    }
    .to_authorization_request("req-pdp-1", None)
    .unwrap_err();
    assert_eq!(fault, DecisionAuthzError::MissingValue { field: "action" });
}

#[test]
fn projection_refuses_whitespace_only_trusted_fields() {
    let fault = DecisionAuthzRequest {
        caller_id: "   ",
        ..decision_authz_request("acme", "acme")
    }
    .to_authorization_request("req-pdp-1", None)
    .unwrap_err();
    assert_eq!(
        fault,
        DecisionAuthzError::MissingValue { field: "caller_id" }
    );

    let fault = DecisionAuthzRequest {
        target_subject_id: "\t",
        ..decision_authz_request("acme", "acme")
    }
    .to_authorization_request("req-pdp-1", None)
    .unwrap_err();
    assert_eq!(
        fault,
        DecisionAuthzError::MissingValue {
            field: "target_subject_id"
        }
    );
}

#[test]
fn fail_closed_authorizer_refuses_even_same_tenant_without_pdp() {
    let authorizer = FailClosedDecisionAuthorizer::new();

    let fault = authorizer
        .decide(&decision_authz_request("acme", "acme"))
        .unwrap_err();
    assert!(matches!(fault, DecisionAuthzError::PdpRefused { .. }));
    assert!(fault.to_string().contains("no PDP-backed"));

    let fault = authorizer
        .decide(&decision_authz_request("acme", "globex"))
        .unwrap_err();
    assert!(matches!(fault, DecisionAuthzError::PdpRefused { .. }));
}

#[test]
fn fail_closed_authorizer_faults_on_empty_trusted_tenants() {
    let authorizer = FailClosedDecisionAuthorizer::new();

    let fault = authorizer
        .decide(&decision_authz_request("", "acme"))
        .unwrap_err();
    assert!(fault.to_string().contains("caller_tenant"));

    let fault = authorizer
        .decide(&decision_authz_request("acme", ""))
        .unwrap_err();
    assert!(fault.to_string().contains("target_tenant"));
}
