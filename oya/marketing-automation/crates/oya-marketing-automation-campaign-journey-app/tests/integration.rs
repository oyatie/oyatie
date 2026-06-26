#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_marketing_automation_campaign_journey_app::adapter::AdapterRegistry;
use oya_marketing_automation_campaign_journey_app::config::ServiceConfig;
use oya_marketing_automation_campaign_journey_app::domain::{
    BoundedContext, Capability, CapabilityDescriptor, CapabilityTier, CompliancePack, DataBoundary,
    IdempotencyKey, TenantId,
};
use oya_marketing_automation_campaign_journey_app::{public_api_surface, scaffold};

#[test]
fn scaffold_declares_expected_contracts() {
    let scaffold = scaffold();
    assert_eq!(scaffold.microservice, "marketing-automation");
    assert_eq!(scaffold.contracts.openapi, "contracts/openapi-v1.yaml");
    assert_eq!(scaffold.contracts.asyncapi, "contracts/asyncapi-v1.yaml");
    assert_eq!(
        scaffold.contracts.grpc,
        "contracts/marketing-automation-v1.proto"
    );
}

#[test]
fn scaffold_declares_adr_0105_layers() {
    let scaffold = scaffold();
    assert_eq!(scaffold.layers.len(), 12);
}

#[test]
fn scaffold_declares_marketing_capabilities() {
    let scaffold = scaffold();
    assert!(scaffold.capabilities.contains(&Capability::JourneyExecute));
    assert!(
        scaffold
            .capabilities
            .contains(&Capability::SuppressionEnforce)
    );
    assert!(
        scaffold
            .capabilities
            .contains(&Capability::AttributionRollup)
    );
}

#[test]
fn capability_descriptors_cover_marketing_resource_boundaries() {
    let descriptors = CapabilityDescriptor::descriptors();
    let expected = [
        (
            Capability::JourneyExecute,
            BoundedContext::Journey,
            CapabilityTier::Regulated,
            DataBoundary::TenantOnly,
            &[
                CompliancePack::Soc2,
                CompliancePack::Gdpr,
                CompliancePack::CanSpam,
                CompliancePack::Casl,
            ][..],
        ),
        (
            Capability::SegmentSync,
            BoundedContext::Segment,
            CapabilityTier::Core,
            DataBoundary::TenantAndAudienceProvider,
            &[CompliancePack::Soc2, CompliancePack::Iso27001],
        ),
        (
            Capability::SuppressionEnforce,
            BoundedContext::ConsentAudience,
            CapabilityTier::Regulated,
            DataBoundary::ConsentLedgerProjection,
            &[
                CompliancePack::Gdpr,
                CompliancePack::KrPipa,
                CompliancePack::Cpra,
                CompliancePack::CanSpam,
            ],
        ),
        (
            Capability::AttributionRollup,
            BoundedContext::Attribution,
            CapabilityTier::Core,
            DataBoundary::AggregatedAttribution,
            &[CompliancePack::Soc2, CompliancePack::Lgpd],
        ),
        (
            Capability::ConsentExport,
            BoundedContext::ConsentAudience,
            CapabilityTier::Regulated,
            DataBoundary::ConsentLedgerProjection,
            &[
                CompliancePack::Gdpr,
                CompliancePack::KrPipa,
                CompliancePack::Cpra,
            ],
        ),
        (
            Capability::MarketplaceAudienceLicense,
            BoundedContext::ConsentAudience,
            CapabilityTier::MarketplaceLinked,
            DataBoundary::MarketplaceDealSetProjection,
            &[CompliancePack::Soc2, CompliancePack::Gdpr],
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
        assert_eq!(descriptor.tier, tier);
        assert_eq!(descriptor.data_boundary, data_boundary);
        assert_eq!(descriptor.required_packs, packs);
    }
}

#[test]
fn config_default_validates_with_named_cli_args() {
    let config = ServiceConfig::local_default("tenant-alpha", 9080);
    config.validate().expect("default config validates");
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
    assert!(IdempotencyKey::new("marketing-key-0001").is_ok());
}

#[test]
#[ignore = "implementation packet will bind OpenAPI request fixtures"]
fn openapi_launch_journey_fixture_round_trips() {}

#[test]
#[ignore = "implementation packet will bind proto-generated request fixtures"]
fn grpc_launch_journey_fixture_round_trips() {}

#[test]
#[ignore = "implementation packet will bind AsyncAPI event fixtures"]
fn asyncapi_journey_event_fixture_round_trips() {}

#[test]
#[ignore = "implementation packet will bind Cedar policy fixtures"]
fn cedar_policy_denies_cross_tenant_journey_launch() {}

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
