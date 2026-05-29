use crate::error::{ServiceError, ServiceResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ArchitectureLayer {
    Kernel,
    Domain,
    Usecase,
    App,
    Adapter,
    Infrastructure,
    Cli,
    Rest,
    Grpc,
    Graphql,
    Worker,
    Sdk,
    Api,
}

impl ArchitectureLayer {
    pub const fn all() -> [Self; 13] {
        [
            Self::Kernel,
            Self::Domain,
            Self::Usecase,
            Self::App,
            Self::Adapter,
            Self::Infrastructure,
            Self::Cli,
            Self::Rest,
            Self::Grpc,
            Self::Graphql,
            Self::Worker,
            Self::Sdk,
            Self::Api,
        ]
    }

    pub const fn slug(&self) -> &'static str {
        match self {
            Self::Kernel => "kernel",
            Self::Domain => "domain",
            Self::Usecase => "usecase",
            Self::App => "app",
            Self::Adapter => "adapter",
            Self::Infrastructure => "infrastructure",
            Self::Cli => "cli",
            Self::Rest => "rest",
            Self::Grpc => "grpc",
            Self::Graphql => "graphql",
            Self::Worker => "worker",
            Self::Sdk => "sdk",
            Self::Api => "api",
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TenantId(String);

impl TenantId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("tenant_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PipelineId(String);

impl PipelineId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("pipeline_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("source_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TransformId(String);

impl TransformId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("transform_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DeadLetterBatchId(String);

impl DeadLetterBatchId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("dead_letter_batch_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PipelineStatus {
    Draft,
    Running,
    LineageCaptured,
    QualityGated,
    ReplayPending,
    ReplayApproved,
    Failed,
}

impl PipelineStatus {
    pub const fn allows_lineage_capture(&self) -> bool {
        matches!(self, Self::Running | Self::LineageCaptured)
    }

    pub const fn allows_replay_approval(&self) -> bool {
        matches!(self, Self::QualityGated | Self::ReplayPending)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum QualityBand {
    Green,
    Yellow,
    Red,
    Quarantined,
}

impl QualityBand {
    pub const fn allows_promotion(&self) -> bool {
        matches!(self, Self::Green | Self::Yellow)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum DataClass {
    SourceOperational,
    TransformIntermediate,
    QualityEvidence,
    AuditEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AuditEventKind {
    IngestRunStarted,
    LineageCaptured,
    QualityThresholdEvaluated,
    DeadLetterReplayApproved,
    SchemaDriftQuarantined,
    WatermarkAdvanced,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Capability {
    IngestStart,
    LineageRecord,
    QualityEvaluate,
    DeadLetterReplayApprove,
    SchemaDriftQuarantine,
    WatermarkAdvance,
}

impl Capability {
    pub const fn action_slug(&self) -> &'static str {
        match self {
            Self::IngestStart => "pipeline.ingest.start",
            Self::LineageRecord => "pipeline.lineage.record",
            Self::QualityEvaluate => "pipeline.quality.evaluate",
            Self::DeadLetterReplayApprove => "pipeline.dead_letter.replay.approve",
            Self::SchemaDriftQuarantine => "pipeline.schema_drift.quarantine",
            Self::WatermarkAdvance => "pipeline.watermark.advance",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CapabilityContract {
    pub capability: Capability,
    pub command_name: &'static str,
    pub result_name: &'static str,
    pub required_layer: ArchitectureLayer,
    pub data_class: DataClass,
    pub audit_event: AuditEventKind,
    pub idempotency_key: &'static str,
}

pub const CAPABILITY_CONTRACTS: &[CapabilityContract] = &[
    CapabilityContract {
        capability: Capability::IngestStart,
        command_name: "StartIngestRunCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Usecase,
        data_class: DataClass::SourceOperational,
        audit_event: AuditEventKind::IngestRunStarted,
        idempotency_key: "tenant_id + pipeline_id",
    },
    CapabilityContract {
        capability: Capability::LineageRecord,
        command_name: "RecordLineageCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Worker,
        data_class: DataClass::TransformIntermediate,
        audit_event: AuditEventKind::LineageCaptured,
        idempotency_key: "tenant_id + pipeline_id + transform_id",
    },
    CapabilityContract {
        capability: Capability::QualityEvaluate,
        command_name: "EvaluateQualityCommand",
        result_name: "QualityEvaluationReceipt",
        required_layer: ArchitectureLayer::Worker,
        data_class: DataClass::QualityEvidence,
        audit_event: AuditEventKind::QualityThresholdEvaluated,
        idempotency_key: "tenant_id + pipeline_id + quality_window",
    },
    CapabilityContract {
        capability: Capability::DeadLetterReplayApprove,
        command_name: "ApproveDeadLetterReplayCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Api,
        data_class: DataClass::AuditEvidence,
        audit_event: AuditEventKind::DeadLetterReplayApproved,
        idempotency_key: "tenant_id + pipeline_id + dead_letter_batch_id",
    },
    CapabilityContract {
        capability: Capability::SchemaDriftQuarantine,
        command_name: "QuarantineSchemaDriftCommand",
        result_name: "SchemaDriftReceipt",
        required_layer: ArchitectureLayer::Adapter,
        data_class: DataClass::QualityEvidence,
        audit_event: AuditEventKind::SchemaDriftQuarantined,
        idempotency_key: "tenant_id + source_id + schema_hash",
    },
    CapabilityContract {
        capability: Capability::WatermarkAdvance,
        command_name: "AdvanceWatermarkCommand",
        result_name: "WatermarkReceipt",
        required_layer: ArchitectureLayer::Worker,
        data_class: DataClass::SourceOperational,
        audit_event: AuditEventKind::WatermarkAdvanced,
        idempotency_key: "tenant_id + pipeline_id + watermark",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AdapterSurface {
    pub layer: ArchitectureLayer,
    pub protocol: &'static str,
    pub handler: &'static str,
    pub contract_path: &'static str,
}

pub const ADAPTER_SURFACES: &[AdapterSurface] = &[
    AdapterSurface {
        layer: ArchitectureLayer::Rest,
        protocol: "HTTP",
        handler: "DataPipelineHttpHandler::start_ingest_run",
        contract_path: "contracts/openapi-v1.yaml#/paths/~1v1~1ingest-runs",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Rest,
        protocol: "HTTP",
        handler: "DataPipelineHttpHandler::record_lineage",
        contract_path: "contracts/openapi-v1.yaml#/paths/~1v1~1pipelines~1{id}~1lineage",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Grpc,
        protocol: "gRPC",
        handler: "DataPipelineGrpcHandler::start_ingest_run",
        contract_path: "contracts/data-pipeline-v1.proto",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "DataPipelineAsyncApiHandler::ingest_run_started",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/ingest_run_started",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "DataPipelineAsyncApiHandler::lineage_captured",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/lineage_captured",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "DataPipelineAsyncApiHandler::dead_letter_replay_approved",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/dead_letter_replay_approved",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OperationalCheckpoint {
    pub name: &'static str,
    pub invariant: &'static str,
    pub expected_evidence: &'static str,
}

pub const OPERATIONAL_CHECKPOINTS: &[OperationalCheckpoint] = &[
    OperationalCheckpoint {
        name: "ingest-start-policy",
        invariant: "policy_checked",
        expected_evidence: "Cedar decision for pipeline.ingest.start",
    },
    OperationalCheckpoint {
        name: "lineage-identity-complete",
        invariant: "lineage_identity",
        expected_evidence: "tenant_id + pipeline_id + source_id + transform_id",
    },
    OperationalCheckpoint {
        name: "quality-band-promotion",
        invariant: "quality_gated",
        expected_evidence: "QualityBand allows promotion before replay",
    },
    OperationalCheckpoint {
        name: "dead-letter-replay-approval",
        invariant: "dead_letter_replay_status_gate",
        expected_evidence: "PipelineStatus::ReplayPending before approval",
    },
    OperationalCheckpoint {
        name: "source-binding-residency",
        invariant: "region_bound",
        expected_evidence: "source residency pack id",
    },
    OperationalCheckpoint {
        name: "audit-chain-emission",
        invariant: "audit_emitted",
        expected_evidence: "EVT-DATA-PIPELINE-*",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ScaffoldAcceptanceRule {
    pub name: &'static str,
    pub layer: ArchitectureLayer,
    pub owner_surface: &'static str,
    pub proof: &'static str,
}

pub const SCAFFOLD_ACCEPTANCE_RULES: &[ScaffoldAcceptanceRule] = &[
    ScaffoldAcceptanceRule {
        name: "ingest-start-is-usecase-owned",
        layer: ArchitectureLayer::Usecase,
        owner_surface: "StartIngestInteractor",
        proof: "policy decision is required before PipelineDefinition::start_ingest",
    },
    ScaffoldAcceptanceRule {
        name: "lineage-capture-is-domain-owned",
        layer: ArchitectureLayer::Domain,
        owner_surface: "LineageRecord::validate",
        proof: "lineage identity is complete before adapter publication",
    },
    ScaffoldAcceptanceRule {
        name: "quality-gate-is-worker-visible",
        layer: ArchitectureLayer::Worker,
        owner_surface: "QualityGatePort",
        proof: "pipeline quality gates are evaluated before replay approval",
    },
    ScaffoldAcceptanceRule {
        name: "dead-letter-replay-is-api-visible",
        layer: ArchitectureLayer::Api,
        owner_surface: "DataPipelineHttpHandler::approve_replay",
        proof: "REST, gRPC, and AsyncAPI expose replay approval",
    },
];

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PipelinePolicy {
    pub capability: Capability,
    pub required_layer: ArchitectureLayer,
    pub data_class: DataClass,
    pub audit_event: AuditEventKind,
}

impl PipelinePolicy {
    pub fn baseline() -> Vec<Self> {
        vec![
            Self {
                capability: Capability::IngestStart,
                required_layer: ArchitectureLayer::Usecase,
                data_class: DataClass::SourceOperational,
                audit_event: AuditEventKind::IngestRunStarted,
            },
            Self {
                capability: Capability::LineageRecord,
                required_layer: ArchitectureLayer::Worker,
                data_class: DataClass::TransformIntermediate,
                audit_event: AuditEventKind::LineageCaptured,
            },
            Self {
                capability: Capability::DeadLetterReplayApprove,
                required_layer: ArchitectureLayer::Api,
                data_class: DataClass::AuditEvidence,
                audit_event: AuditEventKind::DeadLetterReplayApproved,
            },
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LineageRecord {
    pub tenant_id: TenantId,
    pub pipeline_id: PipelineId,
    pub source_id: SourceId,
    pub transform_id: TransformId,
}

impl LineageRecord {
    pub fn validate(&self) -> ServiceResult<()> {
        validate_identifier("tenant_id", self.tenant_id.as_str())?;
        validate_identifier("pipeline_id", self.pipeline_id.as_str())?;
        validate_identifier("source_id", self.source_id.as_str())?;
        validate_identifier("transform_id", self.transform_id.as_str())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct IngestRun {
    pub tenant_id: TenantId,
    pub pipeline_id: PipelineId,
    pub source_id: SourceId,
    pub name: String,
    pub status: PipelineStatus,
    pub quality_band: QualityBand,
    pub lineage_record_count: u32,
}

impl IngestRun {
    pub fn new(
        tenant_id: TenantId,
        pipeline_id: PipelineId,
        source_id: SourceId,
        name: String,
        status: PipelineStatus,
    ) -> Self {
        Self {
            tenant_id,
            pipeline_id,
            source_id,
            name,
            status,
            quality_band: QualityBand::Yellow,
            lineage_record_count: 0,
        }
    }

    pub fn start(mut self) -> ServiceResult<Self> {
        if !matches!(self.status, PipelineStatus::Draft) {
            return Err(ServiceError::invariant(
                "ingest_start_transition",
                "only draft pipeline runs can start",
            ));
        }
        self.status = PipelineStatus::Running;
        Ok(self)
    }

    pub fn record_lineage(mut self) -> ServiceResult<Self> {
        if !self.status.allows_lineage_capture() {
            return Err(ServiceError::invariant(
                "lineage_capture_status_gate",
                "pipeline status does not allow lineage capture",
            ));
        }
        self.lineage_record_count += 1;
        self.status = PipelineStatus::LineageCaptured;
        Ok(self)
    }

    pub fn approve_replay(mut self) -> ServiceResult<Self> {
        if !self.status.allows_replay_approval() {
            return Err(ServiceError::invariant(
                "dead_letter_replay_status_gate",
                "pipeline status does not allow replay approval",
            ));
        }
        if !self.quality_band.allows_promotion() {
            return Err(ServiceError::invariant(
                "quality_gate_replay_approval",
                "red or quarantined quality bands cannot replay",
            ));
        }
        self.status = PipelineStatus::ReplayApproved;
        Ok(self)
    }

    pub fn validate(&self) -> ServiceResult<()> {
        validate_identifier("tenant_id", self.tenant_id.as_str())?;
        validate_identifier("pipeline_id", self.pipeline_id.as_str())?;
        validate_identifier("source_id", self.source_id.as_str())?;
        if self.name.trim().is_empty() {
            return Err(ServiceError::missing_field("name"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DomainInvariant {
    pub name: &'static str,
    pub layer: ArchitectureLayer,
    pub statement: String,
}

impl DomainInvariant {
    pub fn tenant_scoped(statement: impl Into<String>) -> Self {
        Self {
            name: "tenant_scoped",
            layer: ArchitectureLayer::Kernel,
            statement: statement.into(),
        }
    }

    pub fn policy_checked(statement: impl Into<String>) -> Self {
        Self {
            name: "policy_checked",
            layer: ArchitectureLayer::Usecase,
            statement: statement.into(),
        }
    }

    pub fn audit_emitted(statement: impl Into<String>) -> Self {
        Self {
            name: "audit_emitted",
            layer: ArchitectureLayer::Api,
            statement: statement.into(),
        }
    }

    pub fn data_classified(statement: impl Into<String>) -> Self {
        Self {
            name: "data_classified",
            layer: ArchitectureLayer::Domain,
            statement: statement.into(),
        }
    }

    pub fn region_bound(statement: impl Into<String>) -> Self {
        Self {
            name: "region_bound",
            layer: ArchitectureLayer::Infrastructure,
            statement: statement.into(),
        }
    }

    pub fn quality_gated(statement: impl Into<String>) -> Self {
        Self {
            name: "quality_gated",
            layer: ArchitectureLayer::Worker,
            statement: statement.into(),
        }
    }
}

pub fn ensure_same_tenant(left: &TenantId, right: &TenantId) -> ServiceResult<()> {
    if left == right {
        Ok(())
    } else {
        Err(ServiceError::invariant(
            "tenant_scope_match",
            "cross-tenant pipeline records are not allowed",
        ))
    }
}

pub fn validate_identifier(field: &'static str, value: &str) -> ServiceResult<()> {
    let valid = !value.trim().is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));
    if valid {
        Ok(())
    } else {
        Err(ServiceError::invalid_identifier(field, value))
    }
}
