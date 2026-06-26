#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_crm_revenue_app::adapter::AdapterRegistry;
use oya_crm_revenue_app::config::ServiceConfig;
use oya_crm_revenue_app::domain::{
    BoundedContext, Capability, CapabilityDescriptor, CapabilityTier, CompliancePack, DataBoundary,
    IdempotencyKey, ServiceInvariant, TenantId,
};
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
fn scaffold_declares_adr_0105_layers() {
    assert_eq!(scaffold().layers.len(), 12);
}

#[test]
fn scaffold_declares_domain_capabilities() {
    let scaffold = scaffold();
    assert!(scaffold.capabilities.contains(&Capability::AccountMaster));
    assert!(scaffold.capabilities.contains(&Capability::Opportunity));
}

#[test]
fn capability_descriptors_preserve_resource_model_boundaries() {
    let descriptors = CapabilityDescriptor::descriptors();
    let expected = [
        (
            Capability::AccountMaster,
            BoundedContext::AccountMaster,
            CapabilityTier::Regulated,
            DataBoundary::CustomerMasterRecord,
            &[
                CompliancePack::Soc2,
                CompliancePack::Iso27001,
                CompliancePack::Gdpr,
                CompliancePack::Lgpd,
                CompliancePack::KrPipa,
            ][..],
        ),
        (
            Capability::Opportunity,
            BoundedContext::Opportunity,
            CapabilityTier::Regulated,
            DataBoundary::RevenuePipelineRecord,
            &[
                CompliancePack::Sox404,
                CompliancePack::Soc2,
                CompliancePack::JurisdictionalTax,
            ],
        ),
        (
            Capability::Quote,
            BoundedContext::Quote,
            CapabilityTier::Regulated,
            DataBoundary::CommercialQuoteRecord,
            &[
                CompliancePack::Sox404,
                CompliancePack::Soc2,
                CompliancePack::JurisdictionalTax,
            ],
        ),
        (
            Capability::Campaign,
            BoundedContext::Campaign,
            CapabilityTier::Regulated,
            DataBoundary::CampaignEngagementRecord,
            &[
                CompliancePack::Soc2,
                CompliancePack::Gdpr,
                CompliancePack::Lgpd,
                CompliancePack::KrPipa,
            ],
        ),
        (
            Capability::ServiceCase,
            BoundedContext::ServiceCase,
            CapabilityTier::Regulated,
            DataBoundary::ServiceCaseRecord,
            &[
                CompliancePack::Soc2,
                CompliancePack::Iso27001,
                CompliancePack::Gdpr,
                CompliancePack::Lgpd,
                CompliancePack::KrPipa,
            ],
        ),
    ];

    assert_eq!(descriptors.len(), expected.len());
    assert_eq!(
        scaffold().capabilities,
        expected.map(|(capability, _, _, _, _)| capability)
    );
    for (descriptor, (capability, bounded_context, tier, data_boundary, packs)) in
        descriptors.iter().zip(expected)
    {
        assert_eq!(descriptor.capability, capability);
        assert_eq!(descriptor.bounded_context, bounded_context);
        assert_eq!(descriptor.invariant, ServiceInvariant::TenantScoped);
        assert_eq!(descriptor.tier, tier);
        assert_eq!(descriptor.data_boundary, data_boundary);
        assert_eq!(descriptor.required_packs, packs);
    }
}

#[test]
fn serialized_capability_descriptors_expose_metadata_fields() {
    let serialized = serde_json::to_value(CapabilityDescriptor::descriptors()).unwrap();
    let descriptors = serialized
        .as_array()
        .expect("descriptors serialize as array");
    let expected = [
        (
            "account-master",
            "regulated",
            "customer-master-record",
            &["soc2", "iso27001", "gdpr", "lgpd", "kr-pipa"][..],
        ),
        (
            "opportunity",
            "regulated",
            "revenue-pipeline-record",
            &["sox404", "soc2", "jurisdictional-tax"],
        ),
        (
            "quote",
            "regulated",
            "commercial-quote-record",
            &["sox404", "soc2", "jurisdictional-tax"],
        ),
        (
            "campaign",
            "regulated",
            "campaign-engagement-record",
            &["soc2", "gdpr", "lgpd", "kr-pipa"],
        ),
        (
            "service-case",
            "regulated",
            "service-case-record",
            &["soc2", "iso27001", "gdpr", "lgpd", "kr-pipa"],
        ),
    ];

    assert_eq!(descriptors.len(), expected.len());
    for (descriptor, (capability, tier, data_boundary, required_packs)) in
        descriptors.iter().zip(expected)
    {
        assert_eq!(descriptor["capability"], capability);
        assert_eq!(descriptor["tier"], tier);
        assert_eq!(descriptor["data_boundary"], data_boundary);
        let packs: Vec<_> = descriptor["required_packs"]
            .as_array()
            .expect("required_packs serializes as array")
            .iter()
            .map(|pack| pack.as_str().expect("pack serializes as string"))
            .collect();
        assert_eq!(packs, required_packs);
    }
}

#[test]
fn capability_descriptor_metadata_has_no_missing_or_default_values() {
    let descriptors = serde_json::to_value(CapabilityDescriptor::descriptors()).unwrap();
    let descriptors = descriptors
        .as_array()
        .expect("descriptors serialize as array");
    let placeholders = ["", "unknown", "default", "unspecified"];

    for descriptor in descriptors {
        for field in ["tier", "data_boundary"] {
            let value = descriptor[field].as_str().expect("metadata field exists");
            assert!(
                !placeholders.contains(&value),
                "{field} must not use placeholder metadata"
            );
        }

        let packs = descriptor["required_packs"]
            .as_array()
            .expect("required_packs field exists");
        assert!(!packs.is_empty(), "required_packs must not be empty");
        for pack in packs {
            let value = pack.as_str().expect("pack serializes as string");
            assert!(
                !placeholders.contains(&value),
                "required_packs must not use placeholder metadata"
            );
        }
    }
}

#[test]
fn config_default_validates_with_named_cli_args() {
    ServiceConfig::local_default("tenant-alpha", 9080)
        .validate()
        .expect("default config validates");
}

#[test]
fn adapter_registry_contains_three_contract_surfaces() {
    let registry = AdapterRegistry::scaffolded();
    registry.validate().expect("registry validates");
    assert!(registry.http_routes.len() >= 5);
    assert!(registry.grpc_methods.len() >= 4);
    assert!(registry.asyncapi_channels.len() >= 5);
}

#[test]
fn tenant_id_rejects_empty_value() {
    assert!(TenantId::new("   ").is_err());
}

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
    fn ensure_authorized(
        &self,
        _p: &VerifiedPrincipal,
        _r: &CrmResource,
    ) -> Result<(), CrmAuthorizationError> {
        Ok(())
    }
}

fn provider() -> CrmAuthzProvider {
    let verifier =
        ConfiguredBearerPrincipalVerifier::new("bearer-secret-abc", "svc-crm", "tenant-alpha")
            .unwrap();
    CrmAuthzProvider::new(Arc::new(verifier), Arc::new(AllowAuthorizer))
}

fn forged_request(body_tenant: &str) -> HttpRequest {
    HttpRequest {
        tenant_id: body_tenant.to_string(),
        principal_id: "attacker-claims-anything".to_string(),
        request_id: "req-1".to_string(),
        idempotency_key: "crm-key-0001".to_string(),
        body: serde_json::json!({"resource": "x"}),
    }
}

#[test]
fn forged_crm_mutation_without_credential_is_unauthenticated() {
    // RED: no bearer credential but the body claims a victim tenant → 401.
    let cred = CallerCredential {
        authorization: None,
    };
    let err = HttpHandler::handle(
        &provider(),
        &cred,
        Capability::AccountMaster,
        forged_request("tenant-victim"),
    )
    .unwrap_err();
    assert_eq!(err.kind(), ServiceErrorKind::Unauthenticated);
    assert_eq!(err.http_status(), 401);
}

// HIGH (cross-model + in-house): the gate must validate the REAL request body,
// not a free-floating claim. A valid bearer for tenant-alpha with the body
// forging `tenant_id = "tenant-victim"` must NEVER resolve the resource tenant to
// the victim. The resource scope is structurally the VERIFIED tenant (alpha).
// This fails against the pre-fix code (the body tenant was never bound to the
// gate and the handler ignored `_request`, so a buggy/forgeable edge that left
// `request.tenant_id = "victim"` downstream would mutate the victim's records).
#[test]
fn forged_body_tenant_never_becomes_the_resource_tenant() {
    let cred = CallerCredential {
        authorization: Some("Bearer bearer-secret-abc".into()),
    };
    let scope = HttpHandler::resolve_scope(
        &provider(),
        &cred,
        Capability::Opportunity,
        &forged_request("tenant-victim"),
    )
    .expect("valid bearer authorizes");
    // The resolved resource/scope tenant is the VERIFIED tenant, never the forged body tenant.
    assert_eq!(scope.tenant_id(), "tenant-alpha");
    assert_ne!(scope.tenant_id(), "tenant-victim");
    // And the verified actor is the bound principal, never the body-claimed one.
    assert_eq!(scope.principal_id(), "svc-crm");
}

#[test]
fn pdp_authorized_request_reaches_business_handler() {
    // GREEN: valid bearer + PDP grant → passes the gate, reaches the scaffolded
    // business handler (ContractStub). The body identity grants nothing even
    // though the body still carries a (now non-authoritative) tenant.
    let cred = CallerCredential {
        authorization: Some("Bearer bearer-secret-abc".into()),
    };
    let err = HttpHandler::handle(
        &provider(),
        &cred,
        Capability::Quote,
        forged_request("tenant-victim"),
    )
    .unwrap_err();
    assert_eq!(err.kind(), ServiceErrorKind::ContractStub);
}
