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
    Capability::JourneyExecute,
    Capability::SegmentSync,
    Capability::SuppressionEnforce,
    Capability::AttributionRollup,
    Capability::ConsentExport,
    Capability::MarketplaceAudienceLicense,
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
    Segment,
    Campaign,
    Journey,
    ConsentAudience,
    Attribution,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    JourneyExecute,
    SegmentSync,
    SuppressionEnforce,
    AttributionRollup,
    ConsentExport,
    MarketplaceAudienceLicense,
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
pub struct CampaignJourneyId(String);

impl CampaignJourneyId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        bounded_identifier("campaign_journey_id", value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SegmentId(String);

impl SegmentId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        bounded_identifier("segment_id", value.into()).map(Self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ConsentLedgerRef(String);

impl ConsentLedgerRef {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        bounded_identifier("consent_ledger_ref", value.into()).map(Self)
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
pub enum JourneyState {
    Draft,
    AwaitingApproval,
    Scheduled,
    Running,
    PausedForConsent,
    Completed,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CampaignJourney {
    pub id: CampaignJourneyId,
    pub segment_id: SegmentId,
    pub consent_ref: ConsentLedgerRef,
    pub state: JourneyState,
    pub version: u64,
}

impl CampaignJourney {
    pub fn new(
        id: CampaignJourneyId,
        segment_id: SegmentId,
        consent_ref: ConsentLedgerRef,
    ) -> Self {
        Self {
            id,
            segment_id,
            consent_ref,
            state: JourneyState::Draft,
            version: 1,
        }
    }

    pub fn schedule(&mut self) -> Result<()> {
        if !matches!(
            self.state,
            JourneyState::Draft | JourneyState::AwaitingApproval
        ) {
            return Err(ServiceError::invariant(
                "journey_state",
                "only draft or approval-ready journeys can be scheduled",
            ));
        }
        self.state = JourneyState::Scheduled;
        self.version += 1;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum MarketingAutomationCommand {
    LaunchJourney {
        journey_id: CampaignJourneyId,
        segment_id: SegmentId,
        consent_ref: ConsentLedgerRef,
    },
    EnforceSuppression {
        segment_id: SegmentId,
        consent_ref: ConsentLedgerRef,
    },
    SyncSegment {
        segment_id: SegmentId,
    },
    RollupAttribution {
        journey_id: CampaignJourneyId,
    },
    ExportConsent {
        consent_ref: ConsentLedgerRef,
    },
    LicenseMarketplaceAudience {
        segment_id: SegmentId,
    },
}

impl MarketingAutomationCommand {
    pub fn capability(&self) -> Capability {
        match self {
            Self::LaunchJourney { .. } => Capability::JourneyExecute,
            Self::EnforceSuppression { .. } => Capability::SuppressionEnforce,
            Self::SyncSegment { .. } => Capability::SegmentSync,
            Self::RollupAttribution { .. } => Capability::AttributionRollup,
            Self::ExportConsent { .. } => Capability::ConsentExport,
            Self::LicenseMarketplaceAudience { .. } => Capability::MarketplaceAudienceLicense,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum MarketingAutomationEvent {
    JourneyLaunchAccepted {
        journey_id: CampaignJourneyId,
        tenant_id: TenantId,
    },
    SuppressionApplied {
        segment_id: SegmentId,
        tenant_id: TenantId,
    },
    SegmentSyncRequested {
        segment_id: SegmentId,
        tenant_id: TenantId,
    },
    AttributionRollupQueued {
        journey_id: CampaignJourneyId,
        tenant_id: TenantId,
    },
    ConsentExportQueued {
        consent_ref: ConsentLedgerRef,
        tenant_id: TenantId,
    },
    MarketplaceAudienceLicenseHeld {
        segment_id: SegmentId,
        tenant_id: TenantId,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarketingAutomationInvariant {
    TenantScoped,
    ConsentBeforeActivation,
    SuppressionBeforeSend,
    AttributionNeverCrossesTenant,
    MarketplaceAudienceLicenseRequired,
    AuditEveryStateMutation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompliancePack {
    Soc2,
    Iso27001,
    Gdpr,
    Lgpd,
    KrPipa,
    Cpra,
    CanSpam,
    Casl,
    Hipaa,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DataBoundary {
    TenantOnly,
    TenantAndAudienceProvider,
    AggregatedAttribution,
    ConsentLedgerProjection,
    MarketplaceDealSetProjection,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityTier {
    Core,
    Regulated,
    MarketplaceLinked,
    Experimental,
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
                capability: Capability::JourneyExecute,
                bounded_context: BoundedContext::Journey,
                tier: CapabilityTier::Regulated,
                data_boundary: DataBoundary::TenantOnly,
                required_packs: vec![
                    CompliancePack::Soc2,
                    CompliancePack::Gdpr,
                    CompliancePack::CanSpam,
                    CompliancePack::Casl,
                ],
            },
            Self {
                capability: Capability::SegmentSync,
                bounded_context: BoundedContext::Segment,
                tier: CapabilityTier::Core,
                data_boundary: DataBoundary::TenantAndAudienceProvider,
                required_packs: vec![CompliancePack::Soc2, CompliancePack::Iso27001],
            },
            Self {
                capability: Capability::SuppressionEnforce,
                bounded_context: BoundedContext::ConsentAudience,
                tier: CapabilityTier::Regulated,
                data_boundary: DataBoundary::ConsentLedgerProjection,
                required_packs: vec![
                    CompliancePack::Gdpr,
                    CompliancePack::KrPipa,
                    CompliancePack::Cpra,
                    CompliancePack::CanSpam,
                ],
            },
            Self {
                capability: Capability::AttributionRollup,
                bounded_context: BoundedContext::Attribution,
                tier: CapabilityTier::Core,
                data_boundary: DataBoundary::AggregatedAttribution,
                required_packs: vec![CompliancePack::Soc2, CompliancePack::Lgpd],
            },
            Self {
                capability: Capability::MarketplaceAudienceLicense,
                bounded_context: BoundedContext::ConsentAudience,
                tier: CapabilityTier::MarketplaceLinked,
                data_boundary: DataBoundary::MarketplaceDealSetProjection,
                required_packs: vec![CompliancePack::Soc2, CompliancePack::Gdpr],
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
    pub invariant: MarketingAutomationInvariant,
    pub event_type: String,
}

impl AuditEnvelope {
    pub fn new(
        tenant_id: TenantId,
        request_id: RequestId,
        capability: Capability,
        invariant: MarketingAutomationInvariant,
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
