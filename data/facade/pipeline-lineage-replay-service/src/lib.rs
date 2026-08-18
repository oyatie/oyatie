#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod usecase;

pub use adapter::{
    asyncapi::{
        DataPipelineAsyncApiHandler, DeadLetterReplayApprovedEvent, IngestRunStartedEvent,
        LineageCapturedEvent,
    },
    grpc::{DataPipelineGrpcHandler, PipelineGrpcRequest, PipelineGrpcResponse},
    http::{DataPipelineHttpHandler, HttpMethod, RouteDescriptor},
};
pub use config::{RuntimeProfile, ServiceConfig};
pub use domain::{
    ArchitectureLayer, AuditEventKind, Capability, DataClass, DeadLetterBatchId, DomainInvariant,
    IngestRun, LineageRecord, PipelineId, PipelinePolicy, PipelineStatus, QualityBand, SourceId,
    TenantId, TransformId,
};
pub use error::{ServiceError, ServiceResult};
pub use usecase::{
    ApproveDeadLetterReplay, ApproveDeadLetterReplayCommand, DataPipelinePorts,
    DataPipelineService, RecordLineage, RecordLineageCommand, StartIngestRun,
    StartIngestRunCommand,
};

pub const MICROSERVICE: &str = "data-pipeline";
pub const BOUNDED_CONTEXT: &str = "lineage-replay";
pub const PRIMARY_CAPABILITY: &str = "lineage-first-ingest";
pub const PRIMARY_ADR: &str = "ADR-0105";
pub const USECASE_RENAME_ADR: &str = "ADR-0106";
pub const OPENAPI_CONTRACT: &str = "data/data-pipeline/contracts/openapi-v1.yaml";
pub const GRPC_CONTRACT: &str = "data/data-pipeline/contracts/data-pipeline-v1.proto";
pub const ASYNCAPI_CONTRACT: &str = "data/data-pipeline/contracts/asyncapi-v1.yaml";

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ServiceDescriptor {
    pub microservice: &'static str,
    pub bounded_context: &'static str,
    pub primary_capability: &'static str,
    pub architecture_layers: Vec<&'static str>,
    pub contract_paths: Vec<&'static str>,
}

impl ServiceDescriptor {
    pub fn layer_count(&self) -> usize {
        self.architecture_layers.len()
    }

    pub fn contract_count(&self) -> usize {
        self.contract_paths.len()
    }

    pub fn includes_layer(&self, layer: ArchitectureLayer) -> bool {
        self.architecture_layers.contains(&layer.slug())
    }
}

pub fn descriptor() -> ServiceDescriptor {
    ServiceDescriptor {
        microservice: MICROSERVICE,
        bounded_context: BOUNDED_CONTEXT,
        primary_capability: PRIMARY_CAPABILITY,
        architecture_layers: ArchitectureLayer::all()
            .iter()
            .map(ArchitectureLayer::slug)
            .collect(),
        contract_paths: vec![OPENAPI_CONTRACT, GRPC_CONTRACT, ASYNCAPI_CONTRACT],
    }
}

pub fn default_ingest_run() -> IngestRun {
    IngestRun::new(
        TenantId::new("tenant-demo"),
        PipelineId::new("pipeline-demo"),
        SourceId::new("source-demo"),
        "crm-account-sync".to_owned(),
        PipelineStatus::Draft,
    )
}

pub fn default_http_routes() -> Vec<RouteDescriptor> {
    DataPipelineHttpHandler::routes()
}

pub fn default_domain_invariants() -> Vec<DomainInvariant> {
    vec![
        DomainInvariant::tenant_scoped("pipeline runs remain tenant scoped"),
        DomainInvariant::policy_checked("dead-letter replay requires approval"),
        DomainInvariant::audit_emitted("lineage capture emits immutable audit evidence"),
        DomainInvariant::data_classified("ingest records declare source data class"),
        DomainInvariant::region_bound("source bindings honor residency pack boundaries"),
        DomainInvariant::quality_gated("quality bands gate promotion and replay"),
    ]
}

pub fn validate_scaffold() -> ServiceResult<()> {
    let descriptor = descriptor();
    if descriptor.layer_count() != 12 {
        return Err(ServiceError::InvariantViolation {
            invariant: "adr_0105_layer_count",
            details: format!("expected 12 layers, found {}", descriptor.layer_count()),
        });
    }
    if descriptor.contract_count() != 3 {
        return Err(ServiceError::InvariantViolation {
            invariant: "contract_surface_count",
            details: format!(
                "expected 3 contracts, found {}",
                descriptor.contract_count()
            ),
        });
    }
    Ok(())
}
