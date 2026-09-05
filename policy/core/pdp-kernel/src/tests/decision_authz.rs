use shared_platform_contracts_kernel::pdp::PolicyVersion;

use crate::{
    DecisionAuthorizer, DecisionAuthzError, DecisionAuthzRequest, FailClosedDecisionAuthorizer,
};

fn decision_authz_request<'a>(
    caller_tenant: &'a str,
    target_tenant: &'a str,
) -> DecisionAuthzRequest<'a> {
    DecisionAuthzRequest {
        caller_tenant,
        caller_id: "control-plane",
        target_tenant,
        target_subject_id: "wl-secrets-sync",
        action: "tenant-rbac.policy.admission",
        resource_type: "OyaPlatform::TenantResource",
        resource_id: "tenant-rbac/policy-admissions/pa-1",
    }
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
