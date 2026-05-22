#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_warehouse_tenant_olap_service::adapter::asyncapi::{
    DataWarehouseAsyncApiHandler, MaterializationRefreshedEvent,
};
use oya_data_warehouse_tenant_olap_service::adapter::http::{
    DataWarehouseHttpHandler, RegisterDatasetHttpRequest,
};
use oya_data_warehouse_tenant_olap_service::adapter::memory::InMemoryDataWarehousePorts;
use oya_data_warehouse_tenant_olap_service::{
    ArchitectureLayer, AuditEventKind, DataWarehouseService, DatasetId, ServiceConfig, TenantId,
    default_domain_invariants, descriptor, validate_scaffold,
};

#[test]
fn descriptor_declares_thirteen_layers_and_three_contracts() {
    let descriptor = descriptor();
    assert_eq!(descriptor.layer_count(), 13);
    assert_eq!(descriptor.contract_count(), 3);
    assert!(descriptor.includes_layer(ArchitectureLayer::Usecase));
    assert!(descriptor.includes_layer(ArchitectureLayer::Api));
}

#[test]
fn scaffold_validation_accepts_default_contract_shape() {
    validate_scaffold().expect("scaffold should validate");
}

#[test]
fn config_default_is_valid_for_local_runtime() {
    let config = ServiceConfig::default();
    config.validate().expect("default config should validate");
    assert_eq!(config.service_name, "data-warehouse");
}

#[test]
fn domain_invariants_cover_policy_audit_region_and_freshness() {
    let invariants = default_domain_invariants();
    assert_eq!(invariants.len(), 6);
    assert!(
        invariants
            .iter()
            .any(|invariant| invariant.name == "policy_checked")
    );
    assert!(
        invariants
            .iter()
            .any(|invariant| invariant.name == "audit_emitted")
    );
    assert!(
        invariants
            .iter()
            .any(|invariant| invariant.name == "freshness_bounded")
    );
}

#[test]
fn http_handler_registers_dataset_through_usecase_port() {
    let ports = InMemoryDataWarehousePorts::new();
    let mut service = DataWarehouseService::new(ports);
    let receipt = DataWarehouseHttpHandler::register_dataset(
        &mut service,
        RegisterDatasetHttpRequest {
            tenant_id: "tenant-demo".to_owned(),
            dataset_id: "dataset-2026".to_owned(),
            name: "finance_mart".to_owned(),
        },
    )
    .expect("register dataset should succeed");

    assert_eq!(receipt.tenant_id.as_str(), "tenant-demo");
    assert_eq!(receipt.dataset_id.as_str(), "dataset-2026");
}

#[test]
fn asyncapi_handler_serializes_materialization_refreshed_event() {
    let message = DataWarehouseAsyncApiHandler::materialization_refreshed(
        "data-warehouse",
        MaterializationRefreshedEvent {
            tenant_id: TenantId::new("tenant-demo"),
            dataset_id: DatasetId::new("dataset-2026"),
            audit_event: AuditEventKind::MaterializationRefreshed,
        },
    )
    .expect("event should serialize");

    assert_eq!(message.topic, "data-warehouse.materialization.refreshed");
    assert!(message.payload_json.contains("MaterializationRefreshed"));
}

#[test]
fn invalid_identifier_is_rejected() {
    let error = TenantId::parse("bad tenant id").expect_err("spaces are invalid");
    assert!(error.to_string().contains("tenant_id"));
}
