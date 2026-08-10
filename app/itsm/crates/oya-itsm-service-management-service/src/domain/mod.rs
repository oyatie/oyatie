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
pub struct TicketId(String);

impl TicketId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("ticket_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RequesterId(String);

impl RequesterId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("requester_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ChangeId(String);

impl ChangeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("change_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CiId(String);

impl CiId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("ci_id", &value)?;
        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Priority {
    P0,
    P1,
    P2,
    P3,
    P4,
}

impl Priority {
    pub const fn is_major(&self) -> bool {
        matches!(self, Self::P0 | Self::P1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum TicketStatus {
    Draft,
    Open,
    Triaged,
    LinkedToProblem,
    ChangePending,
    Resolved,
    Cancelled,
}

impl TicketStatus {
    pub const fn allows_sla_recompute(&self) -> bool {
        matches!(self, Self::Open | Self::Triaged | Self::LinkedToProblem)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ServiceImpact {
    SingleUser,
    Team,
    Department,
    TenantWide,
    CrossTenant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum DataClass {
    SupportConfidential,
    OperationalTelemetry,
    ChangeEvidence,
    AuditEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AuditEventKind {
    IncidentOpened,
    SlaBreached,
    ProblemLinked,
    ChangeApproved,
    CmdbRelationUpdated,
    MajorIncidentBridgeOpened,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Capability {
    IncidentOpen,
    SlaRecompute,
    ProblemLink,
    ChangeApprove,
    CmdbSync,
    ServiceCatalogPublish,
}

impl Capability {
    pub const fn action_slug(&self) -> &'static str {
        match self {
            Self::IncidentOpen => "itsm.incident.open",
            Self::SlaRecompute => "itsm.sla.recompute",
            Self::ProblemLink => "itsm.problem.link",
            Self::ChangeApprove => "itsm.change.approve",
            Self::CmdbSync => "itsm.cmdb.sync",
            Self::ServiceCatalogPublish => "itsm.service_catalog.publish",
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
        capability: Capability::IncidentOpen,
        command_name: "OpenIncidentCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Usecase,
        data_class: DataClass::SupportConfidential,
        audit_event: AuditEventKind::IncidentOpened,
        idempotency_key: "tenant_id + ticket_id",
    },
    CapabilityContract {
        capability: Capability::SlaRecompute,
        command_name: "RecomputeSlaCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Worker,
        data_class: DataClass::OperationalTelemetry,
        audit_event: AuditEventKind::SlaBreached,
        idempotency_key: "tenant_id + ticket_id + sla_window",
    },
    CapabilityContract {
        capability: Capability::ProblemLink,
        command_name: "LinkProblemCommand",
        result_name: "ProblemLinkReceipt",
        required_layer: ArchitectureLayer::Usecase,
        data_class: DataClass::SupportConfidential,
        audit_event: AuditEventKind::ProblemLinked,
        idempotency_key: "tenant_id + ticket_id + problem_id",
    },
    CapabilityContract {
        capability: Capability::ChangeApprove,
        command_name: "ApproveChangeCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Api,
        data_class: DataClass::ChangeEvidence,
        audit_event: AuditEventKind::ChangeApproved,
        idempotency_key: "tenant_id + ticket_id + change_id",
    },
    CapabilityContract {
        capability: Capability::CmdbSync,
        command_name: "SyncCmdbRelationCommand",
        result_name: "CmdbSyncReceipt",
        required_layer: ArchitectureLayer::Adapter,
        data_class: DataClass::OperationalTelemetry,
        audit_event: AuditEventKind::CmdbRelationUpdated,
        idempotency_key: "tenant_id + ci_id + relation_hash",
    },
    CapabilityContract {
        capability: Capability::ServiceCatalogPublish,
        command_name: "PublishServiceCatalogCommand",
        result_name: "ServiceCatalogReceipt",
        required_layer: ArchitectureLayer::Rest,
        data_class: DataClass::AuditEvidence,
        audit_event: AuditEventKind::MajorIncidentBridgeOpened,
        idempotency_key: "tenant_id + catalog_version",
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
        handler: "ItsmHttpHandler::open_incident",
        contract_path: "contracts/openapi-v1.yaml#/paths/~1v1~1incidents",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Rest,
        protocol: "HTTP",
        handler: "ItsmHttpHandler::recompute_sla",
        contract_path: "contracts/openapi-v1.yaml#/paths/~1v1~1tickets~1{id}~1sla",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Grpc,
        protocol: "gRPC",
        handler: "ItsmGrpcHandler::open_incident",
        contract_path: "contracts/itsm-v1.proto",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "ItsmAsyncApiHandler::incident_opened",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/incident_opened",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "ItsmAsyncApiHandler::sla_breached",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/sla_breached",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "ItsmAsyncApiHandler::change_approved",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/change_approved",
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
        name: "incident-open-policy",
        invariant: "policy_checked",
        expected_evidence: "Cedar decision for itsm.incident.open",
    },
    OperationalCheckpoint {
        name: "sla-clock-monotonic",
        invariant: "sla_monotonic",
        expected_evidence: "elapsed_minutes never decreases",
    },
    OperationalCheckpoint {
        name: "change-freeze-approval",
        invariant: "change_approval_status_gate",
        expected_evidence: "change window and freeze policy decision",
    },
    OperationalCheckpoint {
        name: "cmdb-relation-scope",
        invariant: "tenant_scope_match",
        expected_evidence: "tenant id on CI relation edge",
    },
    OperationalCheckpoint {
        name: "service-catalog-residency",
        invariant: "region_bound",
        expected_evidence: "catalog publish region pack id",
    },
    OperationalCheckpoint {
        name: "audit-chain-emission",
        invariant: "audit_emitted",
        expected_evidence: "EVT-ITSM-*",
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
        name: "incident-policy-is-usecase-owned",
        layer: ArchitectureLayer::Usecase,
        owner_surface: "OpenIncidentInteractor",
        proof: "policy decision is required before IncidentTicket::open",
    },
    ScaffoldAcceptanceRule {
        name: "change-approval-is-api-visible",
        layer: ArchitectureLayer::Api,
        owner_surface: "ItsmHttpHandler::approve_change",
        proof: "REST, gRPC, and AsyncAPI expose the approval checkpoint",
    },
    ScaffoldAcceptanceRule {
        name: "cmdb-sync-is-worker-owned",
        layer: ArchitectureLayer::Worker,
        owner_surface: "CmdbSyncPort",
        proof: "CMDB relation sync stays outside request mutation flow",
    },
    ScaffoldAcceptanceRule {
        name: "sla-breach-is-audit-visible",
        layer: ArchitectureLayer::Api,
        owner_surface: "ItsmAsyncApiHandler::sla_breached",
        proof: "SLA breach events carry tenant-scoped evidence",
    },
];

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ItsmPolicy {
    pub capability: Capability,
    pub required_layer: ArchitectureLayer,
    pub data_class: DataClass,
    pub audit_event: AuditEventKind,
}

impl ItsmPolicy {
    pub fn baseline() -> Vec<Self> {
        vec![
            Self {
                capability: Capability::IncidentOpen,
                required_layer: ArchitectureLayer::Usecase,
                data_class: DataClass::SupportConfidential,
                audit_event: AuditEventKind::IncidentOpened,
            },
            Self {
                capability: Capability::ChangeApprove,
                required_layer: ArchitectureLayer::Api,
                data_class: DataClass::ChangeEvidence,
                audit_event: AuditEventKind::ChangeApproved,
            },
            Self {
                capability: Capability::CmdbSync,
                required_layer: ArchitectureLayer::Worker,
                data_class: DataClass::OperationalTelemetry,
                audit_event: AuditEventKind::CmdbRelationUpdated,
            },
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SlaClock {
    pub tenant_id: TenantId,
    pub ticket_id: TicketId,
    pub elapsed_minutes: u32,
    pub target_minutes: u32,
    pub breached: bool,
}

impl SlaClock {
    pub fn recompute(mut self, additional_minutes: u32) -> Self {
        self.elapsed_minutes += additional_minutes;
        self.breached = self.elapsed_minutes > self.target_minutes;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct IncidentTicket {
    pub tenant_id: TenantId,
    pub ticket_id: TicketId,
    pub requester_id: RequesterId,
    pub title: String,
    pub priority: Priority,
    pub status: TicketStatus,
    pub impact: ServiceImpact,
    pub linked_ci_count: u16,
}

impl IncidentTicket {
    pub fn new(
        tenant_id: TenantId,
        ticket_id: TicketId,
        requester_id: RequesterId,
        title: String,
        priority: Priority,
        status: TicketStatus,
    ) -> Self {
        Self {
            tenant_id,
            ticket_id,
            requester_id,
            title,
            priority,
            status,
            impact: ServiceImpact::SingleUser,
            linked_ci_count: 0,
        }
    }

    pub fn open(mut self) -> ServiceResult<Self> {
        if !matches!(self.status, TicketStatus::Draft) {
            return Err(ServiceError::invariant(
                "incident_open_transition",
                "only draft incidents can be opened",
            ));
        }
        self.status = TicketStatus::Open;
        Ok(self)
    }

    pub fn recompute_sla(mut self) -> ServiceResult<Self> {
        if !self.status.allows_sla_recompute() {
            return Err(ServiceError::invariant(
                "sla_recompute_status_gate",
                "ticket status does not allow SLA recomputation",
            ));
        }
        if self.priority.is_major() {
            self.impact = ServiceImpact::TenantWide;
        }
        Ok(self)
    }

    pub fn approve_change(mut self) -> ServiceResult<Self> {
        if !matches!(self.status, TicketStatus::ChangePending) {
            return Err(ServiceError::invariant(
                "change_approval_status_gate",
                "ticket must be pending change approval",
            ));
        }
        self.status = TicketStatus::Triaged;
        Ok(self)
    }

    pub fn validate(&self) -> ServiceResult<()> {
        validate_identifier("tenant_id", self.tenant_id.as_str())?;
        validate_identifier("ticket_id", self.ticket_id.as_str())?;
        if self.title.trim().is_empty() {
            return Err(ServiceError::missing_field("title"));
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

    pub fn sla_monotonic(statement: impl Into<String>) -> Self {
        Self {
            name: "sla_monotonic",
            layer: ArchitectureLayer::Domain,
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
            "cross-tenant ITSM records are not allowed",
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
