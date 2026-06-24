#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_crm_revenue_app::adapter::AdapterRegistry;
use oya_crm_revenue_app::config::ServiceConfig;
use oya_crm_revenue_app::domain::{Capability, IdempotencyKey, TenantId};
use oya_crm_revenue_app::{public_api_surface, scaffold};

#[test]
fn scaffold_declares_expected_contracts() {
    let scaffold = scaffold();
    assert_eq!(scaffold.microservice, "crm");
    assert_eq!(scaffold.contracts.openapi, "contracts/openapi-v1.yaml");
    assert_eq!(scaffold.contracts.asyncapi, "contracts/asyncapi-v1.yaml");
    assert_eq!(scaffold.contracts.grpc, "contracts/crm-v1.proto");
}

#[test]
fn scaffold_declares_adr_0105_layers() { assert_eq!(scaffold().layers.len(), 13); }

#[test]
fn scaffold_declares_domain_capabilities() {
    let scaffold = scaffold();
    assert!(scaffold.capabilities.contains(&Capability::AccountMaster));
    assert!(scaffold.capabilities.contains(&Capability::Opportunity));
}

#[test]
fn config_default_validates_with_named_cli_args() { ServiceConfig::local_default("tenant-alpha", 9080).validate().expect("default config validates"); }

#[test]
fn adapter_registry_contains_three_contract_surfaces() {
    let registry = AdapterRegistry::scaffolded();
    registry.validate().expect("registry validates");
    assert!(registry.http_routes.len() >= 5);
    assert!(registry.grpc_methods.len() >= 4);
    assert!(registry.asyncapi_channels.len() >= 5);
}

#[test]
fn tenant_id_rejects_empty_value() { assert!(TenantId::new("   ").is_err()); }

#[test]
fn idempotency_key_requires_stable_length() {
    assert!(IdempotencyKey::new("short").is_err());
    assert!(IdempotencyKey::new("crm-key-0001").is_ok());
}

#[test]
#[ignore = "implementation packet will bind OpenAPI request fixtures"]
fn openapi_command_fixture_round_trips() {}

#[test]
#[ignore = "implementation packet will bind proto-generated request fixtures"]
fn grpc_command_fixture_round_trips() {}

#[test]
#[ignore = "implementation packet will bind AsyncAPI event fixtures"]
fn asyncapi_event_fixture_round_trips() {}

#[test]
#[ignore = "implementation packet will bind Cedar policy fixtures"]
fn cedar_policy_denies_cross_tenant_command() {}

#[test]
#[ignore = "implementation packet will bind repository adapter fixtures"]
fn repository_port_enforces_idempotency() {}

#[test]
fn public_surface_names_required_handlers() {
    let surface = public_api_surface();
    assert!(surface.iter().any(|name| name.contains("HttpHandler")));
    assert!(surface.iter().any(|name| name.contains("GrpcHandler")));
    assert!(surface.iter().any(|name| name.contains("AsyncApiHandler")));
}

// ---------------------------------------------------------------------------
// AUTH-005 (ADR-0603): caller-supplied identity at the CRM adapters is
// non-authoritative. A forged-identity mutation is rejected (401/403); a
// PDP-authorized request reaches the (scaffolded) business handler.
// ---------------------------------------------------------------------------

use oya_crm_revenue_app::adapter::http::{HttpHandler, HttpRequest};
use oya_crm_revenue_app::authz::{
    CallerCredential, ConfiguredBearerPrincipalVerifier, CrmAuthorizationError, CrmAuthorizer,
    CrmAuthzProvider, CrmResource, VerifiedPrincipal,
};
use oya_crm_revenue_app::error::ServiceErrorKind;
use std::sync::Arc;

struct AllowAuthorizer;
impl CrmAuthorizer for AllowAuthorizer {
    fn ensure_authorized(&self, _p: &VerifiedPrincipal, _r: &CrmResource) -> Result<(), CrmAuthorizationError> { Ok(()) }
}

fn provider() -> CrmAuthzProvider {
    let verifier = ConfiguredBearerPrincipalVerifier::new("bearer-secret-abc", "svc-crm", "tenant-alpha").unwrap();
    CrmAuthzProvider::new(Arc::new(verifier), Arc::new(AllowAuthorizer))
}

fn forged_request(claimed_tenant: &str) -> HttpRequest {
    HttpRequest {
        tenant_id: claimed_tenant.to_string(),
        principal_id: "attacker-claims-anything".to_string(),
        request_id: "req-1".to_string(),
        idempotency_key: "crm-key-0001".to_string(),
        body: serde_json::json!({"resource": "x"}),
    }
}

#[test]
fn forged_crm_mutation_without_credential_is_unauthorized() {
    // RED: no bearer credential but the body claims a victim tenant.
    let cred = CallerCredential { authorization: None, claimed_principal_id: "attacker".into(), claimed_tenant_id: "tenant-victim".into() };
    let err = HttpHandler::handle(&provider(), &cred, Capability::AccountMaster, forged_request("tenant-victim")).unwrap_err();
    assert_eq!(err.kind(), ServiceErrorKind::Authorization);
}

#[test]
fn forged_cross_tenant_body_claim_is_unauthorized() {
    // RED: valid bearer (binds tenant-alpha) but body forges tenant-victim → denied.
    let cred = CallerCredential { authorization: Some("Bearer bearer-secret-abc".into()), claimed_principal_id: "x".into(), claimed_tenant_id: "tenant-victim".into() };
    let err = HttpHandler::handle(&provider(), &cred, Capability::Opportunity, forged_request("tenant-victim")).unwrap_err();
    assert_eq!(err.kind(), ServiceErrorKind::Authorization);
}

#[test]
fn pdp_authorized_request_reaches_business_handler() {
    // GREEN: valid bearer + matching/blank tenant + blank principal claim + PDP
    // grant → passes the gate, reaches the scaffolded business handler
    // (ContractStub, not Authorization). The body identity grants nothing.
    let cred = CallerCredential { authorization: Some("Bearer bearer-secret-abc".into()), claimed_principal_id: String::new(), claimed_tenant_id: "tenant-alpha".into() };
    let err = HttpHandler::handle(&provider(), &cred, Capability::Quote, forged_request("tenant-alpha")).unwrap_err();
    assert_eq!(err.kind(), ServiceErrorKind::ContractStub);
}
