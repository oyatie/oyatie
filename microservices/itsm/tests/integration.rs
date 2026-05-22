#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_itsm_service_management_service::adapter::asyncapi::{
    IncidentOpenedEvent, ItsmAsyncApiHandler,
};
use oya_itsm_service_management_service::adapter::http::{
    ItsmHttpHandler, OpenIncidentHttpRequest,
};
use oya_itsm_service_management_service::adapter::memory::InMemoryItsmPorts;
use oya_itsm_service_management_service::{
    ArchitectureLayer, AuditEventKind, ItsmService, Priority, ServiceConfig, TenantId, TicketId,
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
    assert_eq!(config.service_name, "itsm");
}

#[test]
fn domain_invariants_cover_policy_audit_region_and_sla() {
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
            .any(|invariant| invariant.name == "sla_monotonic")
    );
}

#[test]
fn http_handler_opens_incident_through_usecase_port() {
    let ports = InMemoryItsmPorts::new();
    let mut service = ItsmService::new(ports);
    let receipt = ItsmHttpHandler::open_incident(
        &mut service,
        OpenIncidentHttpRequest {
            tenant_id: "tenant-demo".to_owned(),
            ticket_id: "ticket-2026".to_owned(),
            requester_id: "requester-1".to_owned(),
            title: "VPN access degraded".to_owned(),
            priority: Priority::P2,
        },
    )
    .expect("open incident should succeed");

    assert_eq!(receipt.tenant_id.as_str(), "tenant-demo");
    assert_eq!(receipt.ticket_id.as_str(), "ticket-2026");
}

#[test]
fn asyncapi_handler_serializes_incident_opened_event() {
    let message = ItsmAsyncApiHandler::incident_opened(
        "itsm",
        IncidentOpenedEvent {
            tenant_id: TenantId::new("tenant-demo"),
            ticket_id: TicketId::new("ticket-2026"),
            audit_event: AuditEventKind::IncidentOpened,
        },
    )
    .expect("event should serialize");

    assert_eq!(message.topic, "itsm.incident.opened");
    assert!(message.payload_json.contains("IncidentOpened"));
}

#[test]
fn invalid_identifier_is_rejected() {
    let error = TenantId::parse("bad tenant id").expect_err("spaces are invalid");
    assert!(error.to_string().contains("tenant_id"));
}
