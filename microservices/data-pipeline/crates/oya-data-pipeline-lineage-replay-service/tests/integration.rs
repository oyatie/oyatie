#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_pipeline_lineage_replay_service::adapter::asyncapi::{
    DataPipelineAsyncApiHandler, IngestRunStartedEvent,
};
use oya_data_pipeline_lineage_replay_service::adapter::http::{
    DataPipelineHttpHandler, StartIngestRunHttpRequest,
};
use oya_data_pipeline_lineage_replay_service::adapter::memory::InMemoryDataPipelinePorts;
use oya_data_pipeline_lineage_replay_service::{
    ArchitectureLayer, AuditEventKind, DataPipelineService, PipelineId, ServiceConfig, TenantId,
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
    assert_eq!(config.service_name, "data-pipeline");
}

#[test]
fn domain_invariants_cover_policy_audit_region_and_quality() {
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
            .any(|invariant| invariant.name == "quality_gated")
    );
}

#[test]
fn http_handler_starts_ingest_run_through_usecase_port() {
    let ports = InMemoryDataPipelinePorts::new();
    let mut service = DataPipelineService::new(ports);
    let receipt = DataPipelineHttpHandler::start_ingest_run(
        &mut service,
        StartIngestRunHttpRequest {
            tenant_id: "tenant-demo".to_owned(),
            pipeline_id: "pipeline-2026".to_owned(),
            source_id: "source-crm".to_owned(),
            name: "crm-account-sync".to_owned(),
        },
    )
    .expect("start ingest run should succeed");

    assert_eq!(receipt.tenant_id.as_str(), "tenant-demo");
    assert_eq!(receipt.pipeline_id.as_str(), "pipeline-2026");
}

#[test]
fn asyncapi_handler_serializes_ingest_run_started_event() {
    let message = DataPipelineAsyncApiHandler::ingest_run_started(
        "data-pipeline",
        IngestRunStartedEvent {
            tenant_id: TenantId::new("tenant-demo"),
            pipeline_id: PipelineId::new("pipeline-2026"),
            audit_event: AuditEventKind::IngestRunStarted,
        },
    )
    .expect("event should serialize");

    assert_eq!(message.topic, "data-pipeline.ingest_run.started");
    assert!(message.payload_json.contains("IngestRunStarted"));
}

#[test]
fn invalid_identifier_is_rejected() {
    let error = TenantId::parse("bad tenant id").expect_err("spaces are invalid");
    assert!(error.to_string().contains("tenant_id"));
}
