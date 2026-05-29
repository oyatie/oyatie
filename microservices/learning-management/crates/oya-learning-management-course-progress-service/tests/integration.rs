#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_learning_management_course_progress_service::adapter::asyncapi::{
    EnrollmentOpenedEvent, LearningAsyncApiHandler,
};
use oya_learning_management_course_progress_service::adapter::http::{
    LearningHttpHandler, OpenEnrollmentHttpRequest,
};
use oya_learning_management_course_progress_service::adapter::memory::InMemoryLearningPorts;
use oya_learning_management_course_progress_service::{
    ArchitectureLayer, AuditEventKind, EnrollmentId, LearningManagementService, ServiceConfig,
    TenantId, default_domain_invariants, descriptor, validate_scaffold,
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
    assert_eq!(config.service_name, "learning-management");
}

#[test]
fn domain_invariants_cover_policy_audit_region_and_progress() {
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
            .any(|invariant| invariant.name == "progress_monotonic")
    );
}

#[test]
fn http_handler_opens_enrollment_through_usecase_port() {
    let ports = InMemoryLearningPorts::new();
    let mut service = LearningManagementService::new(ports);
    let receipt = LearningHttpHandler::open_enrollment(
        &mut service,
        OpenEnrollmentHttpRequest {
            tenant_id: "tenant-demo".to_owned(),
            enrollment_id: "enrollment-2026".to_owned(),
            course_id: "course-security".to_owned(),
            title: "Security foundations".to_owned(),
        },
    )
    .expect("open enrollment should succeed");

    assert_eq!(receipt.tenant_id.as_str(), "tenant-demo");
    assert_eq!(receipt.enrollment_id.as_str(), "enrollment-2026");
}

#[test]
fn asyncapi_handler_serializes_enrollment_opened_event() {
    let message = LearningAsyncApiHandler::enrollment_opened(
        "learning-management",
        EnrollmentOpenedEvent {
            tenant_id: TenantId::new("tenant-demo"),
            enrollment_id: EnrollmentId::new("enrollment-2026"),
            audit_event: AuditEventKind::EnrollmentOpened,
        },
    )
    .expect("event should serialize");

    assert_eq!(message.topic, "learning-management.enrollment.opened");
    assert!(message.payload_json.contains("EnrollmentOpened"));
}

#[test]
fn invalid_identifier_is_rejected() {
    let error = TenantId::parse("bad tenant id").expect_err("spaces are invalid");
    assert!(error.to_string().contains("tenant_id"));
}
