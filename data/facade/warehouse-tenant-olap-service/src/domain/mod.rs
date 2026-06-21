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
    Worker,
    Sdk,
    Api,
}

impl ArchitectureLayer {
    pub const fn all() -> [Self; 12] {
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
pub struct DatasetId(String);

impl DatasetId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("dataset_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MaterializationId(String);

impl MaterializationId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("materialization_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum FreshnessTier {
    Realtime,
    FifteenMinute,
    Hourly,
    Daily,
}

impl FreshnessTier {
    pub const fn max_lag_minutes(&self) -> u32 {
        match self {
            Self::Realtime => 1,
            Self::FifteenMinute => 15,
            Self::Hourly => 60,
            Self::Daily => 1440,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum WarehouseStatus {
    Draft,
    Registered,
    Materializing,
    Serving,
    Shared,
    Quarantined,
}

impl WarehouseStatus {
    pub const fn allows_refresh(&self) -> bool {
        matches!(self, Self::Registered | Self::Materializing | Self::Serving)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum QueryClass {
    Interactive,
    Scheduled,
    Export,
    Materialization,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum DataClass {
    TenantOperational,
    TenantAnalytical,
    ExternalShare,
    AuditEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AuditEventKind {
    DatasetRegistered,
    MaterializationRefreshed,
    FreshnessBreached,
    DatasetShared,
    QueryAdmitted,
    LineageCaptured,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Capability {
    DatasetRegister,
    MaterializationRefresh,
    FreshnessEvaluate,
    DatasetShare,
    QueryAdmit,
    LineageExport,
}

impl Capability {
    pub const fn action_slug(&self) -> &'static str {
        match self {
            Self::DatasetRegister => "warehouse.dataset.register",
            Self::MaterializationRefresh => "warehouse.materialization.refresh",
            Self::FreshnessEvaluate => "warehouse.freshness.evaluate",
            Self::DatasetShare => "warehouse.dataset.share",
            Self::QueryAdmit => "warehouse.query.admit",
            Self::LineageExport => "warehouse.lineage.export",
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
        capability: Capability::DatasetRegister,
        command_name: "RegisterDatasetCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Usecase,
        data_class: DataClass::TenantAnalytical,
        audit_event: AuditEventKind::DatasetRegistered,
        idempotency_key: "tenant_id + dataset_id",
    },
    CapabilityContract {
        capability: Capability::MaterializationRefresh,
        command_name: "RefreshMaterializationCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Worker,
        data_class: DataClass::TenantOperational,
        audit_event: AuditEventKind::MaterializationRefreshed,
        idempotency_key: "tenant_id + dataset_id + materialization_id",
    },
    CapabilityContract {
        capability: Capability::FreshnessEvaluate,
        command_name: "EvaluateFreshnessCommand",
        result_name: "FreshnessEvaluationReceipt",
        required_layer: ArchitectureLayer::Worker,
        data_class: DataClass::TenantOperational,
        audit_event: AuditEventKind::FreshnessBreached,
        idempotency_key: "tenant_id + dataset_id + freshness_window",
    },
    CapabilityContract {
        capability: Capability::DatasetShare,
        command_name: "ShareDatasetCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Api,
        data_class: DataClass::ExternalShare,
        audit_event: AuditEventKind::DatasetShared,
        idempotency_key: "tenant_id + dataset_id + consumer_tenant_id",
    },
    CapabilityContract {
        capability: Capability::QueryAdmit,
        command_name: "AdmitQueryCommand",
        result_name: "QueryAdmissionReceipt",
        required_layer: ArchitectureLayer::Rest,
        data_class: DataClass::TenantAnalytical,
        audit_event: AuditEventKind::QueryAdmitted,
        idempotency_key: "tenant_id + dataset_id + query_hash",
    },
    CapabilityContract {
        capability: Capability::LineageExport,
        command_name: "ExportLineageCommand",
        result_name: "LineageExportReceipt",
        required_layer: ArchitectureLayer::Adapter,
        data_class: DataClass::AuditEvidence,
        audit_event: AuditEventKind::LineageCaptured,
        idempotency_key: "tenant_id + dataset_id + lineage_window",
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
        handler: "DataWarehouseHttpHandler::register_dataset",
        contract_path: "contracts/openapi-v1.yaml#/paths/~1v1~1datasets",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Rest,
        protocol: "HTTP",
        handler: "DataWarehouseHttpHandler::refresh_materialization",
        contract_path: "contracts/openapi-v1.yaml#/paths/~1v1~1datasets~1{id}~1materializations",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Grpc,
        protocol: "gRPC",
        handler: "DataWarehouseGrpcHandler::register_dataset",
        contract_path: "contracts/data-warehouse-v1.proto",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "DataWarehouseAsyncApiHandler::materialization_refreshed",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/materialization_refreshed",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "DataWarehouseAsyncApiHandler::freshness_breached",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/freshness_breached",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "DataWarehouseAsyncApiHandler::dataset_shared",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/dataset_shared",
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
        name: "dataset-register-policy",
        invariant: "policy_checked",
        expected_evidence: "Cedar decision for warehouse.dataset.register",
    },
    OperationalCheckpoint {
        name: "freshness-tier-bound",
        invariant: "freshness_bounded",
        expected_evidence: "FreshnessTier::max_lag_minutes not widened",
    },
    OperationalCheckpoint {
        name: "materialization-refresh-status",
        invariant: "materialization_refresh_status_gate",
        expected_evidence: "WarehouseStatus allows refresh",
    },
    OperationalCheckpoint {
        name: "dataset-share-grant",
        invariant: "dataset_share_status_gate",
        expected_evidence: "serving dataset plus external grant",
    },
    OperationalCheckpoint {
        name: "lineage-self-edge",
        invariant: "lineage_self_edge",
        expected_evidence: "from_dataset_id != to_dataset_id",
    },
    OperationalCheckpoint {
        name: "audit-chain-emission",
        invariant: "audit_emitted",
        expected_evidence: "EVT-DATA-WAREHOUSE-*",
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
        name: "dataset-registration-is-usecase-owned",
        layer: ArchitectureLayer::Usecase,
        owner_surface: "RegisterDatasetInteractor",
        proof: "policy decision is required before WarehouseDataset registration",
    },
    ScaffoldAcceptanceRule {
        name: "materialization-refresh-is-worker-owned",
        layer: ArchitectureLayer::Worker,
        owner_surface: "RefreshMaterializationPort",
        proof: "refresh orchestration stays outside synchronous request flow",
    },
    ScaffoldAcceptanceRule {
        name: "lineage-link-is-domain-validated",
        layer: ArchitectureLayer::Domain,
        owner_surface: "LineageEdge::validate",
        proof: "self edges are rejected before adapter publication",
    },
    ScaffoldAcceptanceRule {
        name: "external-share-is-api-visible",
        layer: ArchitectureLayer::Api,
        owner_surface: "DataWarehouseHttpHandler::share_dataset",
        proof: "REST, gRPC, and AsyncAPI expose the share checkpoint",
    },
];

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WarehousePolicy {
    pub capability: Capability,
    pub required_layer: ArchitectureLayer,
    pub data_class: DataClass,
    pub audit_event: AuditEventKind,
}

impl WarehousePolicy {
    pub fn baseline() -> Vec<Self> {
        vec![
            Self {
                capability: Capability::DatasetRegister,
                required_layer: ArchitectureLayer::Usecase,
                data_class: DataClass::TenantAnalytical,
                audit_event: AuditEventKind::DatasetRegistered,
            },
            Self {
                capability: Capability::MaterializationRefresh,
                required_layer: ArchitectureLayer::Worker,
                data_class: DataClass::TenantOperational,
                audit_event: AuditEventKind::MaterializationRefreshed,
            },
            Self {
                capability: Capability::DatasetShare,
                required_layer: ArchitectureLayer::Api,
                data_class: DataClass::ExternalShare,
                audit_event: AuditEventKind::DatasetShared,
            },
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct QueryWorkload {
    pub tenant_id: TenantId,
    pub dataset_id: DatasetId,
    pub query_class: QueryClass,
    pub estimated_scan_mb: u32,
}

impl QueryWorkload {
    pub fn validate(&self) -> ServiceResult<()> {
        if self.estimated_scan_mb == 0 {
            return Err(ServiceError::invariant(
                "query_estimate_required",
                "query admission requires a nonzero scan estimate",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LineageEdge {
    pub from_dataset_id: DatasetId,
    pub to_dataset_id: DatasetId,
    pub transform_ref: String,
}

impl LineageEdge {
    pub fn validate(&self) -> ServiceResult<()> {
        if self.from_dataset_id == self.to_dataset_id {
            return Err(ServiceError::invariant(
                "lineage_self_edge",
                "lineage edges cannot point to the same dataset",
            ));
        }
        if self.transform_ref.trim().is_empty() {
            return Err(ServiceError::missing_field("transform_ref"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WarehouseNamespace {
    pub tenant_id: TenantId,
    pub dataset_id: DatasetId,
    pub name: String,
    pub freshness_tier: FreshnessTier,
    pub status: WarehouseStatus,
    pub materialization_count: u16,
}

impl WarehouseNamespace {
    pub fn new(
        tenant_id: TenantId,
        dataset_id: DatasetId,
        name: String,
        freshness_tier: FreshnessTier,
        status: WarehouseStatus,
    ) -> Self {
        Self {
            tenant_id,
            dataset_id,
            name,
            freshness_tier,
            status,
            materialization_count: 0,
        }
    }

    pub fn register(mut self) -> ServiceResult<Self> {
        if !matches!(self.status, WarehouseStatus::Draft) {
            return Err(ServiceError::invariant(
                "dataset_register_transition",
                "only draft namespaces can be registered",
            ));
        }
        self.status = WarehouseStatus::Registered;
        Ok(self)
    }

    pub fn refresh_materialization(mut self) -> ServiceResult<Self> {
        if !self.status.allows_refresh() {
            return Err(ServiceError::invariant(
                "materialization_refresh_status_gate",
                "namespace status does not allow refresh",
            ));
        }
        self.materialization_count += 1;
        self.status = WarehouseStatus::Serving;
        Ok(self)
    }

    pub fn share(mut self) -> ServiceResult<Self> {
        if !matches!(self.status, WarehouseStatus::Serving) {
            return Err(ServiceError::invariant(
                "dataset_share_status_gate",
                "only serving datasets can be shared",
            ));
        }
        self.status = WarehouseStatus::Shared;
        Ok(self)
    }

    pub fn validate(&self) -> ServiceResult<()> {
        validate_identifier("tenant_id", self.tenant_id.as_str())?;
        validate_identifier("dataset_id", self.dataset_id.as_str())?;
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

    pub fn freshness_bounded(statement: impl Into<String>) -> Self {
        Self {
            name: "freshness_bounded",
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
            "cross-tenant warehouse records are not allowed",
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
