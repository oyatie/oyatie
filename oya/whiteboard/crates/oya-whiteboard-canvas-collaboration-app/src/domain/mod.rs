use crate::error::{Result, ServiceError};
use crate::MICROSERVICE;
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
    Layer::Graphql,
    Layer::Worker,
    Layer::Cli,
    Layer::Sdk,
    Layer::Api,
];

pub const CAPABILITIES: &[Capability] = &[
    Capability::BoardOpen,
    Capability::CanvasOpAppend,
    Capability::ExportRender,
    Capability::HistorySnapshot,
    Capability::PresenceSync,
    Capability::TemplateMarketplaceInstall,
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
    Graphql,
    Worker,
    Cli,
    Sdk,
    Api,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoundedContext {
    Canvas,
    BoardSession,
    StickyNote,
    Template,
    Export,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    BoardOpen,
    CanvasOpAppend,
    ExportRender,
    HistorySnapshot,
    PresenceSync,
    TemplateMarketplaceInstall,
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
pub struct BoardId(String);

impl BoardId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        bounded_identifier("board_id", value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CanvasOpId(String);

impl CanvasOpId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        bounded_identifier("canvas_op_id", value.into()).map(Self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TemplateId(String);

impl TemplateId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        bounded_identifier("template_id", value.into()).map(Self)
    }
}

fn bounded_identifier(field: &'static str, value: String) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ServiceError::invariant(field, "identifier must not be empty"));
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
pub enum BoardState {
    Created,
    Open,
    Collaborating,
    Exporting,
    Archived,
    Quarantined,
    Deleted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollaborativeBoard {
    pub id: BoardId,
    pub latest_canvas_op_id: CanvasOpId,
    pub template_id: TemplateId,
    pub state: BoardState,
    pub version: u64,
}

impl CollaborativeBoard {
    pub fn new(id: BoardId, latest_canvas_op_id: CanvasOpId, template_id: TemplateId) -> Self {
        Self {
            id,
            latest_canvas_op_id,
            template_id,
            state: BoardState::Created,
            version: 1,
        }
    }

    pub fn open(&mut self) -> Result<()> {
        if !matches!(self.state, BoardState::Created | BoardState::Archived) {
            return Err(ServiceError::invariant(
                "board_state",
                "only created or archived boards can be opened",
            ));
        }
        self.state = BoardState::Open;
        self.version += 1;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum WhiteboardCommand {
    OpenBoard {
        board_id: BoardId,
    },
    AppendCanvasOp {
        board_id: BoardId,
        canvas_op_id: CanvasOpId,
    },
    RenderExport {
        board_id: BoardId,
    },
    SnapshotHistory {
        board_id: BoardId,
    },
    SyncPresence {
        board_id: BoardId,
    },
    InstallTemplate {
        board_id: BoardId,
        template_id: TemplateId,
    },
}

impl WhiteboardCommand {
    pub fn capability(&self) -> Capability {
        match self {
            Self::OpenBoard { .. } => Capability::BoardOpen,
            Self::AppendCanvasOp { .. } => Capability::CanvasOpAppend,
            Self::RenderExport { .. } => Capability::ExportRender,
            Self::SnapshotHistory { .. } => Capability::HistorySnapshot,
            Self::SyncPresence { .. } => Capability::PresenceSync,
            Self::InstallTemplate { .. } => Capability::TemplateMarketplaceInstall,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum WhiteboardEvent {
    BoardOpened {
        board_id: BoardId,
        tenant_id: TenantId,
    },
    CanvasOpAppendQueued {
        board_id: BoardId,
        tenant_id: TenantId,
    },
    ExportRenderQueued {
        board_id: BoardId,
        tenant_id: TenantId,
    },
    HistorySnapshotQueued {
        board_id: BoardId,
        tenant_id: TenantId,
    },
    PresenceSyncRequested {
        board_id: BoardId,
        tenant_id: TenantId,
    },
    TemplateInstallQueued {
        template_id: TemplateId,
        tenant_id: TenantId,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WhiteboardInvariant {
    TenantScoped,
    CanvasOpAppendOrdered,
    CrdtMergeAudited,
    ExportProvenancePreserved,
    PresenceNeverCrossesTenant,
    AuditEveryStateMutation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompliancePack {
    Soc2,
    Iso27001,
    Gdpr,
    KrPipa,
    Education,
    PublicSector,
    Hipaa,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DataBoundary {
    TenantOnly,
    BoardHistoryProjection,
    RealtimePresence,
    ExportArtifact,
    TemplateMarketplaceProjection,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityTier {
    Core,
    Realtime,
    Export,
    MarketplaceLinked,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapabilityDescriptor {
    pub capability: Capability,
    pub bounded_context: BoundedContext,
    pub tier: CapabilityTier,
    pub data_boundary: DataBoundary,
    pub required_packs: Vec<CompliancePack>,
}

impl CapabilityDescriptor {
    pub fn descriptors() -> Vec<Self> {
        vec![
            Self {
                capability: Capability::BoardOpen,
                bounded_context: BoundedContext::BoardSession,
                tier: CapabilityTier::Core,
                data_boundary: DataBoundary::TenantOnly,
                required_packs: vec![CompliancePack::Soc2, CompliancePack::Gdpr],
            },
            Self {
                capability: Capability::CanvasOpAppend,
                bounded_context: BoundedContext::Canvas,
                tier: CapabilityTier::Realtime,
                data_boundary: DataBoundary::BoardHistoryProjection,
                required_packs: vec![CompliancePack::Soc2, CompliancePack::Iso27001],
            },
            Self {
                capability: Capability::ExportRender,
                bounded_context: BoundedContext::Export,
                tier: CapabilityTier::Export,
                data_boundary: DataBoundary::ExportArtifact,
                required_packs: vec![CompliancePack::Gdpr, CompliancePack::KrPipa],
            },
            Self {
                capability: Capability::PresenceSync,
                bounded_context: BoundedContext::BoardSession,
                tier: CapabilityTier::Realtime,
                data_boundary: DataBoundary::RealtimePresence,
                required_packs: vec![CompliancePack::Soc2],
            },
            Self {
                capability: Capability::TemplateMarketplaceInstall,
                bounded_context: BoundedContext::Template,
                tier: CapabilityTier::MarketplaceLinked,
                data_boundary: DataBoundary::TemplateMarketplaceProjection,
                required_packs: vec![CompliancePack::Education, CompliancePack::PublicSector],
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
            layer: Layer::Graphql,
            owner: MICROSERVICE,
            responsibility: "future graph projection boundary",
            allowed_dependencies: vec!["api", "sdk"],
            forbidden_dependencies: vec!["write-side-domain-mutation"],
            verification_hint: "graph projections remain read-model oriented",
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
    pub invariant: WhiteboardInvariant,
    pub event_type: String,
}

impl AuditEnvelope {
    pub fn new(
        tenant_id: TenantId,
        request_id: RequestId,
        capability: Capability,
        invariant: WhiteboardInvariant,
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
