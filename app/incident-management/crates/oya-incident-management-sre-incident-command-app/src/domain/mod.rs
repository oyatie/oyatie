use crate::MICROSERVICE;
use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};

pub const LAYERS: &[Layer] = &[
    Layer::Kernel,
    Layer::Domain,
    Layer::Usecase,
    Layer::App,
    Layer::Adapter,
    Layer::Infrastructure,
    Layer::Rest,
    Layer::Grpc,
    Layer::Worker,
    Layer::Cli,
    Layer::Sdk,
    Layer::Api,
];

pub const CAPABILITIES: &[Capability] = &[
    Capability::PageDispatch,
    Capability::EscalationEvaluate,
    Capability::IncidentRoomOpen,
    Capability::StakeholderUpdate,
    Capability::StatuspageSync,
    Capability::PostmortemSeal,
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layer {
    Kernel,
    Domain,
    Usecase,
    App,
    Adapter,
    Infrastructure,
    Rest,
    Grpc,
    Worker,
    Cli,
    Sdk,
    Api,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoundedContext {
    OnCallSchedule,
    EscalationPolicy,
    IncidentRoom,
    StatusUpdate,
    Postmortem,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    PageDispatch,
    EscalationEvaluate,
    IncidentRoomOpen,
    StakeholderUpdate,
    StatuspageSync,
    PostmortemSeal,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TenantId(String);

impl TenantId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ServiceError::invariant(
                "tenant_id",
                "tenant id must not be empty",
            ));
        }
        if trimmed.len() > 96 {
            return Err(ServiceError::invariant(
                "tenant_id",
                "tenant id must be at most 96 bytes",
            ));
        }
        if !trimmed
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        {
            return Err(ServiceError::invariant(
                "tenant_id",
                "tenant id must be ASCII alphanumeric, dash, or underscore",
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.len() < 12 {
            return Err(ServiceError::invariant(
                "idempotency_key",
                "idempotency key must be at least 12 characters",
            ));
        }
        if trimmed.len() > 160 {
            return Err(ServiceError::invariant(
                "idempotency_key",
                "idempotency key must be at most 160 characters",
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PrincipalId(String);

impl PrincipalId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ServiceError::invariant(
                "principal_id",
                "principal id must not be empty",
            ));
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RequestId(String);

impl RequestId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ServiceError::invariant(
                "request_id",
                "request id must not be empty",
            ));
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UsecaseActor {
    pub tenant_id: TenantId,
    pub principal_id: PrincipalId,
    pub request_id: RequestId,
    pub idempotency_key: IdempotencyKey,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IncidentId(String);

impl IncidentId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        bounded_identifier("incident_id", value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EscalationPolicyId(String);

impl EscalationPolicyId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        bounded_identifier("escalation_policy_id", value.into()).map(Self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StatusPageRef(String);

impl StatusPageRef {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        bounded_identifier("status_page_ref", value.into()).map(Self)
    }
}

fn bounded_identifier(field: &'static str, value: String) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ServiceError::invariant(
            field,
            "identifier must not be empty",
        ));
    }
    if trimmed.len() > 128 {
        return Err(ServiceError::invariant(
            field,
            "identifier must be at most 128 bytes",
        ));
    }
    Ok(trimmed.to_string())
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IncidentState {
    Detected,
    Paging,
    Acknowledged,
    Mitigating,
    Monitoring,
    Resolved,
    PostmortemSealed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IncidentCommand {
    pub id: IncidentId,
    pub escalation_policy_id: EscalationPolicyId,
    pub status_page_ref: StatusPageRef,
    pub state: IncidentState,
    pub version: u64,
}

impl IncidentCommand {
    pub fn new(
        id: IncidentId,
        escalation_policy_id: EscalationPolicyId,
        status_page_ref: StatusPageRef,
    ) -> Self {
        Self {
            id,
            escalation_policy_id,
            status_page_ref,
            state: IncidentState::Detected,
            version: 1,
        }
    }

    pub fn dispatch_page(&mut self) -> Result<()> {
        if !matches!(self.state, IncidentState::Detected | IncidentState::Paging) {
            return Err(ServiceError::invariant(
                "incident_state",
                "only detected or paging incidents can dispatch a page",
            ));
        }
        self.state = IncidentState::Paging;
        self.version += 1;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum IncidentManagementCommand {
    DispatchPage {
        incident_id: IncidentId,
        escalation_policy_id: EscalationPolicyId,
    },
    EvaluateEscalation {
        incident_id: IncidentId,
        escalation_policy_id: EscalationPolicyId,
    },
    OpenIncidentRoom {
        incident_id: IncidentId,
    },
    SyncStatusPage {
        incident_id: IncidentId,
        status_page_ref: StatusPageRef,
    },
    PublishStakeholderUpdate {
        incident_id: IncidentId,
    },
    SealPostmortem {
        incident_id: IncidentId,
    },
}

impl IncidentManagementCommand {
    pub fn capability(&self) -> Capability {
        match self {
            Self::DispatchPage { .. } => Capability::PageDispatch,
            Self::EvaluateEscalation { .. } => Capability::EscalationEvaluate,
            Self::OpenIncidentRoom { .. } => Capability::IncidentRoomOpen,
            Self::SyncStatusPage { .. } => Capability::StatuspageSync,
            Self::PublishStakeholderUpdate { .. } => Capability::StakeholderUpdate,
            Self::SealPostmortem { .. } => Capability::PostmortemSeal,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum IncidentManagementEvent {
    PageDispatchAccepted {
        incident_id: IncidentId,
        tenant_id: TenantId,
    },
    EscalationEvaluationQueued {
        incident_id: IncidentId,
        tenant_id: TenantId,
    },
    IncidentRoomOpened {
        incident_id: IncidentId,
        tenant_id: TenantId,
    },
    StatusPageSyncQueued {
        incident_id: IncidentId,
        tenant_id: TenantId,
    },
    StakeholderUpdatePublished {
        incident_id: IncidentId,
        tenant_id: TenantId,
    },
    PostmortemSealQueued {
        incident_id: IncidentId,
        tenant_id: TenantId,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IncidentManagementInvariant {
    TenantScoped,
    CriticalIncidentAudited,
    EscalationPolicyBeforePaging,
    IncidentRoomRequiresAuthorizedRole,
    StatusUpdateNeverCrossesTenant,
    AuditEveryStateMutation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompliancePack {
    Soc2,
    Iso27001,
    FedrampHigh,
    KrCsap,
    EuSovereign,
    Dora,
    Gdpr,
    Hipaa,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DataBoundary {
    TenantOnly,
    PagerProviderMetadata,
    StatusPageProjection,
    PostmortemEvidence,
    CrossCellReplayMetadata,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityAvailability {
    AlwaysOn,
    Regulated,
    Collaboration,
    Evidence,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapabilityDescriptor {
    pub capability: Capability,
    pub bounded_context: BoundedContext,
    pub availability: CapabilityAvailability,
    pub data_boundary: DataBoundary,
    pub required_packs: Vec<CompliancePack>,
}

impl CapabilityDescriptor {
    pub fn descriptors() -> Vec<Self> {
        vec![
            Self {
                capability: Capability::PageDispatch,
                bounded_context: BoundedContext::OnCallSchedule,
                availability: CapabilityAvailability::AlwaysOn,
                data_boundary: DataBoundary::PagerProviderMetadata,
                required_packs: vec![CompliancePack::Soc2, CompliancePack::Dora],
            },
            Self {
                capability: Capability::EscalationEvaluate,
                bounded_context: BoundedContext::EscalationPolicy,
                availability: CapabilityAvailability::AlwaysOn,
                data_boundary: DataBoundary::TenantOnly,
                required_packs: vec![CompliancePack::Soc2, CompliancePack::Iso27001],
            },
            Self {
                capability: Capability::IncidentRoomOpen,
                bounded_context: BoundedContext::IncidentRoom,
                availability: CapabilityAvailability::Collaboration,
                data_boundary: DataBoundary::TenantOnly,
                required_packs: vec![CompliancePack::Gdpr, CompliancePack::Hipaa],
            },
            Self {
                capability: Capability::StatuspageSync,
                bounded_context: BoundedContext::StatusUpdate,
                availability: CapabilityAvailability::Regulated,
                data_boundary: DataBoundary::StatusPageProjection,
                required_packs: vec![CompliancePack::FedrampHigh, CompliancePack::EuSovereign],
            },
            Self {
                capability: Capability::PostmortemSeal,
                bounded_context: BoundedContext::Postmortem,
                availability: CapabilityAvailability::Evidence,
                data_boundary: DataBoundary::PostmortemEvidence,
                required_packs: vec![CompliancePack::KrCsap, CompliancePack::Dora],
            },
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LayerContract {
    pub layer: Layer,
    pub owner: &'static str,
    pub responsibility: &'static str,
    pub allowed_dependencies: Vec<&'static str>,
    pub forbidden_dependencies: Vec<&'static str>,
    pub verification_hint: &'static str,
}

pub fn layer_contracts() -> Vec<LayerContract> {
    vec![
        LayerContract {
            layer: Layer::Kernel,
            owner: "platform",
            responsibility: "stable shared primitives and deterministic identifiers",
            allowed_dependencies: vec!["std", "serde"],
            forbidden_dependencies: vec!["transport", "tenant-runtime"],
            verification_hint: "kernel records remain tenant-neutral and transport-free",
        },
        LayerContract {
            layer: Layer::Domain,
            owner: MICROSERVICE,
            responsibility: "bounded-context vocabulary, invariants, and business events",
            allowed_dependencies: vec!["kernel"],
            forbidden_dependencies: vec!["http", "grpc", "asyncapi"],
            verification_hint: "domain commands map to exactly one capability",
        },
        LayerContract {
            layer: Layer::Usecase,
            owner: MICROSERVICE,
            responsibility: "policy-checked interactors and repository/event/audit ports",
            allowed_dependencies: vec!["domain", "kernel"],
            forbidden_dependencies: vec!["wire-protocol", "database-driver"],
            verification_hint: "ports stay trait-shaped and adapter-independent",
        },
        LayerContract {
            layer: Layer::App,
            owner: MICROSERVICE,
            responsibility: "service composition and runtime bootstrap",
            allowed_dependencies: vec!["usecase", "config", "adapter"],
            forbidden_dependencies: vec!["domain-mutation-shortcuts"],
            verification_hint: "startup validates tenant scope before accepting traffic",
        },
        LayerContract {
            layer: Layer::Adapter,
            owner: MICROSERVICE,
            responsibility: "protocol translation into usecase commands",
            allowed_dependencies: vec!["usecase", "domain"],
            forbidden_dependencies: vec!["storage-schema-ownership"],
            verification_hint: "adapters never bypass interactor policy checks",
        },
        LayerContract {
            layer: Layer::Infrastructure,
            owner: "deployment",
            responsibility: "storage, queues, observability, and runtime bindings",
            allowed_dependencies: vec!["adapter", "config"],
            forbidden_dependencies: vec!["domain-rule-authorship"],
            verification_hint: "infrastructure implementations satisfy declared ports",
        },
        LayerContract {
            layer: Layer::Rest,
            owner: MICROSERVICE,
            responsibility: "HTTP route catalog and OpenAPI alignment",
            allowed_dependencies: vec!["adapter", "usecase"],
            forbidden_dependencies: vec!["grpc-only-types"],
            verification_hint: "routes reference the canonical contract path",
        },
        LayerContract {
            layer: Layer::Grpc,
            owner: MICROSERVICE,
            responsibility: "gRPC method catalog and proto alignment",
            allowed_dependencies: vec!["adapter", "usecase"],
            forbidden_dependencies: vec!["rest-only-types"],
            verification_hint: "methods reference the canonical proto package",
        },
        LayerContract {
            layer: Layer::Worker,
            owner: MICROSERVICE,
            responsibility: "background orchestration and retry-safe command handling",
            allowed_dependencies: vec!["usecase", "eventing"],
            forbidden_dependencies: vec!["interactive-session-state"],
            verification_hint: "workers require idempotency keys for mutating actions",
        },
        LayerContract {
            layer: Layer::Cli,
            owner: MICROSERVICE,
            responsibility: "operator entrypoint and local smoke execution",
            allowed_dependencies: vec!["config", "app"],
            forbidden_dependencies: vec!["hidden-default-tenant"],
            verification_hint: "cli requires explicit config, port, and tenant-id",
        },
        LayerContract {
            layer: Layer::Sdk,
            owner: "platform",
            responsibility: "typed client surface for service consumers",
            allowed_dependencies: vec!["api", "contracts"],
            forbidden_dependencies: vec!["server-runtime"],
            verification_hint: "sdk mirrors public command and event names",
        },
        LayerContract {
            layer: Layer::Api,
            owner: MICROSERVICE,
            responsibility: "stable public contract descriptors and compatibility policy",
            allowed_dependencies: vec!["domain", "contracts"],
            forbidden_dependencies: vec!["private-storage-model"],
            verification_hint: "api descriptors remain backward-compatible by default",
        },
    ]
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuditEnvelope {
    pub tenant_id: TenantId,
    pub request_id: RequestId,
    pub capability: Capability,
    pub invariant: IncidentManagementInvariant,
    pub event_type: String,
}

impl AuditEnvelope {
    pub fn new(
        tenant_id: TenantId,
        request_id: RequestId,
        capability: Capability,
        invariant: IncidentManagementInvariant,
        event_type: impl Into<String>,
    ) -> Result<Self> {
        let event_type = event_type.into();
        if event_type.trim().is_empty() {
            return Err(ServiceError::invariant(
                "event_type",
                "audit event type must not be empty",
            ));
        }
        Ok(Self {
            tenant_id,
            request_id,
            capability,
            invariant,
            event_type,
        })
    }
}
