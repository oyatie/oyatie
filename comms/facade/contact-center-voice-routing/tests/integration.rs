#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use comms_contact_center_voice_routing::adapter::AdapterRegistry;
use comms_contact_center_voice_routing::config::ServiceConfig;
use comms_contact_center_voice_routing::domain::{Capability, IdempotencyKey, TenantId};
use comms_contact_center_voice_routing::{public_api_surface, scaffold};

#[test]
fn scaffold_declares_expected_contracts() {
    let scaffold = scaffold();
    assert_eq!(scaffold.microservice, "contact-center");
    assert_eq!(scaffold.contracts.openapi, "contracts/openapi-v1.yaml");
    assert_eq!(scaffold.contracts.asyncapi, "contracts/asyncapi-v1.yaml");
    assert_eq!(scaffold.contracts.grpc, "contracts/contact-center-v1.proto");
}

#[test]
fn scaffold_declares_adr_0105_layers() {
    let scaffold = scaffold();
    assert_eq!(scaffold.layers.len(), 12);
}

#[test]
fn scaffold_declares_contact_center_capabilities() {
    let scaffold = scaffold();
    assert!(scaffold.capabilities.contains(&Capability::VoiceRoute));
    assert!(
        scaffold
            .capabilities
            .contains(&Capability::RecordingConsent)
    );
    assert!(
        scaffold
            .capabilities
            .contains(&Capability::EmergencyCallerBypass)
    );
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
    assert!(IdempotencyKey::new("contact-key-0001").is_ok());
}

#[test]
#[ignore = "implementation packet will bind OpenAPI request fixtures"]
fn openapi_voice_route_fixture_round_trips() {}

#[test]
#[ignore = "implementation packet will bind proto-generated request fixtures"]
fn grpc_voice_route_fixture_round_trips() {}

#[test]
#[ignore = "implementation packet will bind AsyncAPI event fixtures"]
fn asyncapi_voice_route_event_fixture_round_trips() {}

#[test]
#[ignore = "implementation packet will bind Cedar policy fixtures"]
fn cedar_policy_denies_cross_tenant_voice_route() {}

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
