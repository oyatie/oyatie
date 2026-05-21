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
pub struct DesignFileId(String);

impl DesignFileId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("design_file_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DesignerId(String);

impl DesignerId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("designer_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VersionId(String);

impl VersionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("version_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CommentThreadId(String);

impl CommentThreadId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("comment_thread_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ArtifactStatus {
    Draft,
    Open,
    ReviewRequested,
    CommentResolution,
    TokenPromoted,
    Exported,
    Archived,
}

impl ArtifactStatus {
    pub const fn allows_comment_resolution(&self) -> bool {
        matches!(
            self,
            Self::Open | Self::ReviewRequested | Self::CommentResolution
        )
    }

    pub const fn allows_token_promotion(&self) -> bool {
        matches!(self, Self::ReviewRequested | Self::CommentResolution)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PermissionScope {
    Viewer,
    Commenter,
    Editor,
    Owner,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum HandoffFormat {
    Png,
    Svg,
    Pdf,
    DesignTokens,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum DataClass {
    CreativeConfidential,
    CollaborationMetadata,
    ExportArtifact,
    AuditEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AuditEventKind {
    FileOpened,
    VersionSaved,
    CommentResolved,
    TokenPromoted,
    HandoffExported,
    PermissionChecked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Capability {
    FileOpen,
    VersionSave,
    CommentResolve,
    TokenPromote,
    HandoffExport,
    PermissionCheck,
}

impl Capability {
    pub const fn action_slug(&self) -> &'static str {
        match self {
            Self::FileOpen => "design.file.open",
            Self::VersionSave => "design.version.save",
            Self::CommentResolve => "design.comment.resolve",
            Self::TokenPromote => "design.token.promote",
            Self::HandoffExport => "design.handoff.export",
            Self::PermissionCheck => "design.permission.check",
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
        capability: Capability::FileOpen,
        command_name: "OpenDesignFileCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Usecase,
        data_class: DataClass::CreativeConfidential,
        audit_event: AuditEventKind::FileOpened,
        idempotency_key: "tenant_id + design_file_id",
    },
    CapabilityContract {
        capability: Capability::VersionSave,
        command_name: "SaveVersionCommand",
        result_name: "VersionReceipt",
        required_layer: ArchitectureLayer::Rest,
        data_class: DataClass::CreativeConfidential,
        audit_event: AuditEventKind::VersionSaved,
        idempotency_key: "tenant_id + design_file_id + version_hash",
    },
    CapabilityContract {
        capability: Capability::CommentResolve,
        command_name: "ResolveCommentCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Rest,
        data_class: DataClass::CollaborationMetadata,
        audit_event: AuditEventKind::CommentResolved,
        idempotency_key: "tenant_id + design_file_id + comment_thread_id",
    },
    CapabilityContract {
        capability: Capability::TokenPromote,
        command_name: "PromoteTokenCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Api,
        data_class: DataClass::AuditEvidence,
        audit_event: AuditEventKind::TokenPromoted,
        idempotency_key: "tenant_id + design_file_id + token_ref",
    },
    CapabilityContract {
        capability: Capability::HandoffExport,
        command_name: "ExportHandoffCommand",
        result_name: "HandoffExportReceipt",
        required_layer: ArchitectureLayer::Adapter,
        data_class: DataClass::ExportArtifact,
        audit_event: AuditEventKind::HandoffExported,
        idempotency_key: "tenant_id + design_file_id + handoff_format",
    },
    CapabilityContract {
        capability: Capability::PermissionCheck,
        command_name: "CheckPermissionCommand",
        result_name: "PermissionDecision",
        required_layer: ArchitectureLayer::Usecase,
        data_class: DataClass::CollaborationMetadata,
        audit_event: AuditEventKind::PermissionChecked,
        idempotency_key: "tenant_id + design_file_id + principal_id",
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
        handler: "DesignHttpHandler::open_design_file",
        contract_path: "contracts/openapi-v1.yaml#/paths/~1v1~1design-files",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Rest,
        protocol: "HTTP",
        handler: "DesignHttpHandler::resolve_comment",
        contract_path: "contracts/openapi-v1.yaml#/paths/~1v1~1design-files~1{id}~1comments",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Grpc,
        protocol: "gRPC",
        handler: "DesignGrpcHandler::open_design_file",
        contract_path: "contracts/design-collaboration-v1.proto",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "DesignAsyncApiHandler::file_opened",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/file_opened",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "DesignAsyncApiHandler::comment_resolved",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/comment_resolved",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "DesignAsyncApiHandler::token_promoted",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/token_promoted",
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
        name: "file-open-policy",
        invariant: "policy_checked",
        expected_evidence: "Cedar decision for design.file.open",
    },
    OperationalCheckpoint {
        name: "version-append-only",
        invariant: "version_monotonic",
        expected_evidence: "version_number increments by append",
    },
    OperationalCheckpoint {
        name: "comment-resolution-status",
        invariant: "comment_resolution_status_gate",
        expected_evidence: "ArtifactStatus allows comment resolution",
    },
    OperationalCheckpoint {
        name: "token-promotion-review",
        invariant: "token_promotion_status_gate",
        expected_evidence: "review requested or comment resolution status",
    },
    OperationalCheckpoint {
        name: "handoff-export-residency",
        invariant: "region_bound",
        expected_evidence: "asset export region pack id",
    },
    OperationalCheckpoint {
        name: "audit-chain-emission",
        invariant: "audit_emitted",
        expected_evidence: "EVT-DESIGN-COLLABORATION-*",
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
        name: "file-open-is-usecase-owned",
        layer: ArchitectureLayer::Usecase,
        owner_surface: "OpenDesignFileInteractor",
        proof: "policy decision is required before DesignArtifact mutation",
    },
    ScaffoldAcceptanceRule {
        name: "version-history-is-domain-owned",
        layer: ArchitectureLayer::Domain,
        owner_surface: "VersionHistory::append",
        proof: "version numbers advance monotonically inside domain state",
    },
    ScaffoldAcceptanceRule {
        name: "comment-resolution-is-api-visible",
        layer: ArchitectureLayer::Api,
        owner_surface: "DesignHttpHandler::resolve_comment",
        proof: "REST, gRPC, and AsyncAPI expose comment resolution",
    },
    ScaffoldAcceptanceRule {
        name: "design-token-promotion-is-review-gated",
        layer: ArchitectureLayer::Usecase,
        owner_surface: "PromoteTokenPort",
        proof: "token promotion requires reviewed artifact status",
    },
];

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DesignPolicy {
    pub capability: Capability,
    pub required_layer: ArchitectureLayer,
    pub data_class: DataClass,
    pub audit_event: AuditEventKind,
}

impl DesignPolicy {
    pub fn baseline() -> Vec<Self> {
        vec![
            Self {
                capability: Capability::FileOpen,
                required_layer: ArchitectureLayer::Usecase,
                data_class: DataClass::CreativeConfidential,
                audit_event: AuditEventKind::FileOpened,
            },
            Self {
                capability: Capability::CommentResolve,
                required_layer: ArchitectureLayer::Rest,
                data_class: DataClass::CollaborationMetadata,
                audit_event: AuditEventKind::CommentResolved,
            },
            Self {
                capability: Capability::TokenPromote,
                required_layer: ArchitectureLayer::Api,
                data_class: DataClass::AuditEvidence,
                audit_event: AuditEventKind::TokenPromoted,
            },
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DesignArtifact {
    pub tenant_id: TenantId,
    pub design_file_id: DesignFileId,
    pub owner_id: DesignerId,
    pub title: String,
    pub status: ArtifactStatus,
    pub version_number: u32,
    pub open_comment_count: u16,
}

impl DesignArtifact {
    pub fn new(
        tenant_id: TenantId,
        design_file_id: DesignFileId,
        owner_id: DesignerId,
        title: String,
        status: ArtifactStatus,
    ) -> Self {
        Self {
            tenant_id,
            design_file_id,
            owner_id,
            title,
            status,
            version_number: 0,
            open_comment_count: 0,
        }
    }

    pub fn open(mut self) -> ServiceResult<Self> {
        if !matches!(self.status, ArtifactStatus::Draft) {
            return Err(ServiceError::invariant(
                "design_file_open_transition",
                "only draft design files can be opened",
            ));
        }
        self.status = ArtifactStatus::Open;
        self.version_number = 1;
        Ok(self)
    }

    pub fn resolve_comment(mut self) -> ServiceResult<Self> {
        if !self.status.allows_comment_resolution() {
            return Err(ServiceError::invariant(
                "comment_resolution_status_gate",
                "design file status does not allow comment resolution",
            ));
        }
        self.open_comment_count = self.open_comment_count.saturating_sub(1);
        self.status = ArtifactStatus::CommentResolution;
        Ok(self)
    }

    pub fn promote_token(mut self) -> ServiceResult<Self> {
        if !self.status.allows_token_promotion() {
            return Err(ServiceError::invariant(
                "token_promotion_status_gate",
                "tokens can be promoted only after review",
            ));
        }
        self.status = ArtifactStatus::TokenPromoted;
        Ok(self)
    }

    pub fn validate(&self) -> ServiceResult<()> {
        validate_identifier("tenant_id", self.tenant_id.as_str())?;
        validate_identifier("design_file_id", self.design_file_id.as_str())?;
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

    pub fn version_monotonic(statement: impl Into<String>) -> Self {
        Self {
            name: "version_monotonic",
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
            "cross-tenant design artifacts are not allowed",
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
