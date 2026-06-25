#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_performance_management_review_calibration_service::adapter::asyncapi::{
    PerformanceAsyncApiHandler, ReviewCycleOpenedEvent,
};
use oya_performance_management_review_calibration_service::adapter::http::{
    OpenReviewCycleHttpRequest, PerformanceHttpHandler,
};
use oya_performance_management_review_calibration_service::adapter::memory::InMemoryPerformancePorts;
use oya_performance_management_review_calibration_service::{
    ArchitectureLayer, AuditEventKind, PerformanceManagementService, ReviewCycleId, ServiceConfig,
    TenantId, default_domain_invariants, descriptor, validate_scaffold,
};

#[test]
fn descriptor_declares_twelve_layers_and_three_contracts() {
    let descriptor = descriptor();
    assert_eq!(descriptor.layer_count(), 12);
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
    assert_eq!(config.service_name, "performance-management");
}

#[test]
fn domain_invariants_cover_policy_audit_and_region_boundaries() {
    let invariants = default_domain_invariants();
    assert_eq!(invariants.len(), 5);
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
            .any(|invariant| invariant.name == "region_bound")
    );
}

#[test]
fn http_handler_opens_review_cycle_through_usecase_port() {
    let ports = InMemoryPerformancePorts::new();
    let mut service = PerformanceManagementService::new(ports);
    let receipt = PerformanceHttpHandler::open_review_cycle(
        &mut service,
        OpenReviewCycleHttpRequest {
            tenant_id: "tenant-demo".to_owned(),
            review_cycle_id: "cycle-2026".to_owned(),
            title: "FY26 review".to_owned(),
        },
    )
    .expect("open review cycle should succeed");

    assert_eq!(receipt.tenant_id.as_str(), "tenant-demo");
    assert_eq!(receipt.review_cycle_id.as_str(), "cycle-2026");
}

#[test]
fn asyncapi_handler_serializes_review_cycle_opened_event() {
    let message = PerformanceAsyncApiHandler::review_cycle_opened(
        "performance-management",
        ReviewCycleOpenedEvent {
            tenant_id: TenantId::new("tenant-demo"),
            review_cycle_id: ReviewCycleId::new("cycle-2026"),
            audit_event: AuditEventKind::ReviewCycleOpened,
        },
    )
    .expect("event should serialize");

    assert_eq!(message.topic, "performance-management.review_cycle.opened");
    assert!(message.payload_json.contains("ReviewCycleOpened"));
}

#[test]
fn invalid_identifier_is_rejected() {
    let error = TenantId::parse("bad tenant id").expect_err("spaces are invalid");
    assert!(error.to_string().contains("tenant_id"));
}
