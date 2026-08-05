#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::{env, fs};

use oya_plant_maintenance_work_order_app::adapter::{
    AdapterRegistry,
    asyncapi::AsyncApiHandler,
    grpc::GrpcHandler,
    http::{HttpHandler, HttpRequest},
};
use oya_plant_maintenance_work_order_app::config::ServiceConfig;
use oya_plant_maintenance_work_order_app::domain::{Capability, IdempotencyKey, TenantId};
use oya_plant_maintenance_work_order_app::{public_api_surface, scaffold};

fn read_fixture(manifest_relative: &str, repo_relative: &str) -> String {
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let candidate = std::path::Path::new(&manifest_dir).join(manifest_relative);
        if candidate.is_file() {
            return fs::read_to_string(&candidate)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", candidate.display()));
        }
    }

    let mut search_root =
        env::current_dir().unwrap_or_else(|error| panic!("failed to read current dir: {error}"));
    loop {
        let candidate = search_root.join(repo_relative);
        if candidate.is_file() {
            return fs::read_to_string(&candidate)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", candidate.display()));
        }
        if !search_root.pop() {
            break;
        }
    }

    panic!("fixture not found: {manifest_relative} or {repo_relative}");
}

fn assert_fixture_contains(fixture: &str, needle: &str, fixture_name: &str) {
    assert!(
        fixture.contains(needle),
        "{fixture_name} missing metadata fixture `{needle}`"
    );
}

fn assert_occurs_before(fixture: &str, first: &str, second: &str, fixture_name: &str) {
    let first_index = fixture
        .find(first)
        .unwrap_or_else(|| panic!("{fixture_name} missing `{first}`"));
    let second_index = fixture
        .find(second)
        .unwrap_or_else(|| panic!("{fixture_name} missing `{second}`"));
    assert!(
        first_index < second_index,
        "{fixture_name} should keep `{first}` before `{second}`"
    );
}

#[test]
fn scaffold_declares_expected_contracts() {
    let scaffold = scaffold();
    assert_eq!(scaffold.microservice, "plant-maintenance");
    assert_eq!(scaffold.contracts.openapi, "contracts/openapi-v1.yaml");
    assert_eq!(scaffold.contracts.asyncapi, "contracts/asyncapi-v1.yaml");
    assert_eq!(
        scaffold.contracts.grpc,
        "contracts/plant-maintenance-v1.proto"
    );
}

#[test]
fn scaffold_declares_adr_0105_layers() {
    assert_eq!(scaffold().layers.len(), 13);
}

#[test]
fn scaffold_declares_domain_capabilities() {
    let scaffold = scaffold();
    assert!(scaffold.capabilities.contains(&Capability::EquipmentMaster));
    assert!(scaffold.capabilities.contains(&Capability::WorkOrder));
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
    assert!(IdempotencyKey::new("plant-maintenance-key-0001").is_ok());
}

#[test]
fn openapi_command_fixture_round_trips() {
    let openapi_contract = read_fixture(
        "../../../contracts/openapi-v1.yaml",
        "oya/plant-maintenance/contracts/openapi-v1.yaml",
    );
    let response = HttpHandler::handle(HttpRequest {
        tenant_id: "tenant-alpha".to_owned(),
        principal_id: "principal-maintenance-planner".to_owned(),
        request_id: "request-plant-maintenance-openapi-fixture".to_owned(),
        idempotency_key: "plant-maintenance-openapi-fixture-0001".to_owned(),
        body: serde_json::json!({"metadata_fixture_only": true}),
    });
    assert!(
        response.is_err(),
        "OpenAPI fixture is metadata-only; HTTP handler must remain a contract stub"
    );

    let missing_routes: Vec<_> = HttpHandler::routes()
        .iter()
        .filter(|route| !openapi_contract.contains(route.path))
        .map(|route| route.path)
        .collect();
    assert!(
        missing_routes.is_empty(),
        "OpenAPI fixture missing app route metadata: {missing_routes:?}"
    );

    for schema in [
        "EquipmentMasterCommand",
        "MaintenancePlanCommand",
        "WorkOrderCommand",
    ] {
        assert_fixture_contains(&openapi_contract, schema, "OpenAPI contract");
    }
    assert_fixture_contains(
        &openapi_contract,
        "required: [tenant_id, principal_id, idempotency_key, payload, compliance_packs]",
        "OpenAPI contract",
    );
}

#[test]
fn grpc_command_fixture_round_trips() {
    let grpc_contract = read_fixture(
        "../../../contracts/plant-maintenance-v1.proto",
        "oya/plant-maintenance/contracts/plant-maintenance-v1.proto",
    );
    let missing_methods: Vec<_> = GrpcHandler::methods()
        .iter()
        .filter(|method| {
            let needle = format!("method: {}; contract_stub_only: true", method.method);
            !grpc_contract.contains(&needle)
        })
        .map(|method| method.method)
        .collect();
    assert!(
        missing_methods.is_empty(),
        "gRPC proto fixture missing app method metadata: {missing_methods:?}"
    );

    for message in [
        "message EquipmentMasterCommand",
        "message MaintenancePlanCommand",
        "message WorkOrderCommand",
    ] {
        assert_fixture_contains(&grpc_contract, message, "gRPC proto contract");
    }
}

#[test]
fn asyncapi_event_fixture_round_trips() {
    let asyncapi_contract = read_fixture(
        "../../../contracts/asyncapi-v1.yaml",
        "oya/plant-maintenance/contracts/asyncapi-v1.yaml",
    );
    let missing_channels: Vec<_> = AsyncApiHandler::channels()
        .iter()
        .filter(|channel| !asyncapi_contract.contains(channel.channel))
        .map(|channel| channel.channel)
        .collect();
    assert!(
        missing_channels.is_empty(),
        "AsyncAPI fixture missing app channel metadata: {missing_channels:?}"
    );

    for event in [
        "EquipmentMasterChanged",
        "MaintenancePlanChanged",
        "WorkOrderChanged",
    ] {
        assert_fixture_contains(&asyncapi_contract, event, "AsyncAPI contract");
    }
}

#[test]
fn cedar_policy_denies_cross_tenant_command() {
    let work_order_policy = read_fixture(
        "../../../policy/work-order-authorization.cedar",
        "oya/plant-maintenance/policy/work-order-authorization.cedar",
    );
    assert_fixture_contains(
        &work_order_policy,
        "forbid (principal, action, resource);",
        "work-order Cedar policy",
    );
    assert_fixture_contains(
        &work_order_policy,
        "principal.tenant_id == resource.tenant_id",
        "work-order Cedar policy",
    );
    assert_fixture_contains(
        &work_order_policy,
        "context.marketplace_settlement_ref != \"\"",
        "work-order Cedar policy",
    );
    assert_occurs_before(
        &work_order_policy,
        "forbid (principal, action, resource);",
        "permit (",
        "work-order Cedar policy",
    );
}

#[test]
fn repository_port_enforces_idempotency() {
    let usecase_source = read_fixture(
        "../src/usecase/mod.rs",
        "oya/plant-maintenance/crates/oya-plant-maintenance-work-order-app/src/usecase/mod.rs",
    );
    assert_fixture_contains(
        &usecase_source,
        "fn reserve_idempotency_key(&self, envelope: &CommandEnvelope) -> Result<()>;",
        "RepositoryPort source contract",
    );
    assert_occurs_before(
        &usecase_source,
        "self.policy.authorize(&envelope)?;",
        "self.repository.reserve_idempotency_key(&envelope)?;",
        "ServiceInteractor source contract",
    );
    assert_occurs_before(
        &usecase_source,
        "self.repository.reserve_idempotency_key(&envelope)?;",
        "self.audit.append(&event)?;",
        "ServiceInteractor source contract",
    );
    assert_occurs_before(
        &usecase_source,
        "self.repository.reserve_idempotency_key(&envelope)?;",
        "self.repository.persist_command_receipt(&receipt)?;",
        "ServiceInteractor source contract",
    );
}

#[test]
fn public_surface_names_required_handlers() {
    let surface = public_api_surface();
    assert!(surface.iter().any(|name| name.contains("HttpHandler")));
    assert!(surface.iter().any(|name| name.contains("GrpcHandler")));
    assert!(surface.iter().any(|name| name.contains("AsyncApiHandler")));
}
