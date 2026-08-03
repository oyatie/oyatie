#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_supply_chain_planning_network_app::adapter::AdapterRegistry;
use oya_supply_chain_planning_network_app::adapter::asyncapi::{AsyncApiHandler, AsyncApiMessage};
use oya_supply_chain_planning_network_app::adapter::grpc::{GrpcHandler, GrpcRequest};
use oya_supply_chain_planning_network_app::adapter::http::{HttpHandler, HttpRequest};
use oya_supply_chain_planning_network_app::config::ServiceConfig;
use oya_supply_chain_planning_network_app::domain::{Capability, IdempotencyKey, TenantId};
use oya_supply_chain_planning_network_app::{public_api_surface, scaffold};

#[test]
fn scaffold_declares_expected_contracts() {
    let scaffold = scaffold();
    assert_eq!(scaffold.microservice, "supply-chain-planning");
    assert_eq!(scaffold.contracts.openapi, "contracts/openapi-v1.yaml");
    assert_eq!(scaffold.contracts.asyncapi, "contracts/asyncapi-v1.yaml");
    assert_eq!(
        scaffold.contracts.grpc,
        "contracts/supply-chain-planning-v1.proto"
    );
}
#[test]
fn scaffold_declares_adr_0105_layers() {
    assert_eq!(scaffold().layers.len(), 13);
}
#[test]
fn scaffold_declares_domain_capabilities() {
    let scaffold = scaffold();
    assert!(scaffold.capabilities.contains(&Capability::DemandPlan));
    assert!(
        scaffold
            .capabilities
            .contains(&Capability::SupplyNetworkPlan)
    );
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
fn network_handlers_remain_contract_stubs_not_runtime_activation() {
    let http_error = HttpHandler::handle(HttpRequest {
        tenant_id: "tenant-alpha".to_string(),
        principal_id: "principal-alpha".to_string(),
        request_id: "request-alpha".to_string(),
        idempotency_key: "supply-chain-planning-key-0001".to_string(),
        body: serde_json::json!({}),
    })
    .expect_err("HTTP handler stays scaffolded");
    assert!(http_error.to_string().contains("ContractStub at http"));

    let grpc_error = GrpcHandler::handle(GrpcRequest {
        tenant_id: "tenant-alpha".to_string(),
        method: "SubmitCommand".to_string(),
        payload_json: serde_json::json!({}),
    })
    .expect_err("gRPC handler stays scaffolded");
    assert!(grpc_error.to_string().contains("ContractStub at grpc"));

    let asyncapi_error = AsyncApiHandler::handle(AsyncApiMessage {
        tenant_id: "tenant-alpha".to_string(),
        message_type: "CommandAccepted".to_string(),
        payload_json: serde_json::json!({}),
    })
    .expect_err("AsyncAPI handler stays scaffolded");
    assert!(
        asyncapi_error
            .to_string()
            .contains("ContractStub at asyncapi")
    );
}
#[test]
fn route_contract_matrix_documents_preview_scaffold_gaps() {
    let routes = HttpHandler::routes();
    let matrix = [
        (
            "demand-plan",
            Some("/v1/supply-chain-planning/demand-plans:publish"),
            "/v1/supply-chain-planning/demand-plan",
            "scaffold action route differs from canonical OpenAPI mutate path",
        ),
        (
            "supply-network-plan",
            Some("/v1/supply-chain-planning/supply-network-plans:reconcile"),
            "/v1/supply-chain-planning/supply-network-plan",
            "scaffold action route differs from canonical OpenAPI mutate path",
        ),
        (
            "available-to-promise",
            Some("/v1/supply-chain-planning/available-to-promise:reserve"),
            "/v1/supply-chain-planning/available-to-promise",
            "scaffold action route adds an action suffix to the canonical OpenAPI mutate path",
        ),
        (
            "replenishment-plan",
            Some("/v1/supply-chain-planning/replenishment-plans:approve"),
            "/v1/supply-chain-planning/replenishment-plan",
            "scaffold action route differs from canonical OpenAPI mutate path",
        ),
        (
            "transportation-plan",
            None,
            "/v1/supply-chain-planning/transportation-plan",
            "contract-only target with no network route scaffold on this card",
        ),
        (
            "planning-scenario",
            Some("/v1/supply-chain-planning/planning-scenarios:simulate"),
            "/v1/supply-chain-planning/planning-scenario",
            "scaffold action route differs from canonical OpenAPI mutate path",
        ),
    ];

    for (capability, scaffold_route, canonical_openapi_path, disposition) in matrix {
        let actual = routes
            .iter()
            .find(|route| route.capability == capability)
            .map(|route| route.path);
        assert_eq!(actual, scaffold_route, "{capability}: {disposition}");
        assert!(
            canonical_openapi_path.starts_with("/v1/supply-chain-planning/"),
            "{capability}: OpenAPI path stays versioned"
        );
        assert!(
            !canonical_openapi_path.contains(':'),
            "{capability}: OpenAPI path remains the canonical mutate path, not an action route"
        );
    }
}
#[test]
fn tenant_id_rejects_empty_value() {
    assert!(TenantId::new("   ").is_err());
}
#[test]
fn idempotency_key_requires_stable_length() {
    assert!(IdempotencyKey::new("short").is_err());
    assert!(IdempotencyKey::new("supply-chain-planning-key-0001").is_ok());
}
#[test]
#[ignore = "split to contract/network adapter fixture work; scaffold classification keeps OpenAPI fixture binding ignored"]
fn openapi_command_fixture_round_trips() {}
#[test]
#[ignore = "split to contract/network adapter fixture work; scaffold classification keeps proto fixture binding ignored"]
fn grpc_command_fixture_round_trips() {}
#[test]
#[ignore = "split to contract/network adapter fixture work; scaffold classification keeps AsyncAPI fixture binding ignored"]
fn asyncapi_event_fixture_round_trips() {}
#[test]
#[ignore = "split to policy work; scaffold classification keeps Cedar fixture binding ignored"]
fn cedar_policy_denies_cross_tenant_command() {}
#[test]
#[ignore = "blocked by metadata-only ceiling until repository/persistence adapter scope is promoted"]
fn repository_port_enforces_idempotency() {}
#[test]
fn public_surface_names_required_handlers() {
    let surface = public_api_surface();
    assert!(surface.iter().any(|name| name.contains("HttpHandler")));
    assert!(surface.iter().any(|name| name.contains("GrpcHandler")));
    assert!(surface.iter().any(|name| name.contains("AsyncApiHandler")));
}
