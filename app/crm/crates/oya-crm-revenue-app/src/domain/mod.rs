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
    Capability::AccountMaster,
    Capability::Opportunity,
    Capability::Quote,
    Capability::Campaign,
    Capability::ServiceCase,
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
    AccountMaster,
    Opportunity,
    Quote,
    Campaign,
    ServiceCase,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    AccountMaster,
    Opportunity,
    Quote,
    Campaign,
    ServiceCase,
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
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        {
            return Err(ServiceError::invariant(
                "tenant_id",
                "tenant id must be ASCII alphanumeric, dash, or underscore",
            ));
        }
        Ok(Self(trimmed.to_string()))
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
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PrincipalId(String);
impl PrincipalId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        bounded_identifier("principal_id", value.into()).map(Self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RequestId(String);
impl RequestId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        bounded_identifier("request_id", value.into()).map(Self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ResourceId(String);
impl ResourceId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        bounded_identifier("resource_id", value.into()).map(Self)
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UsecaseActor {
    pub tenant_id: TenantId,
    pub principal_id: PrincipalId,
    pub request_id: RequestId,
    pub idempotency_key: IdempotencyKey,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ServiceCommand {
    Submit {
        capability: Capability,
        resource_id: ResourceId,
    },
    Reconcile {
        capability: Capability,
        resource_id: ResourceId,
    },
    ApplyGovernanceHold {
        capability: Capability,
        resource_id: ResourceId,
        reason: String,
    },
    ExportEvidence {
        capability: Capability,
        resource_id: ResourceId,
    },
}

impl ServiceCommand {
    pub fn capability(&self) -> Capability {
        match self {
            Self::Submit { capability, .. }
            | Self::Reconcile { capability, .. }
            | Self::ApplyGovernanceHold { capability, .. }
            | Self::ExportEvidence { capability, .. } => *capability,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ServiceEvent {
    CommandAccepted {
        capability: Capability,
        tenant_id: TenantId,
    },
    ReconciliationQueued {
        capability: Capability,
        tenant_id: TenantId,
    },
    GovernanceHoldApplied {
        capability: Capability,
        tenant_id: TenantId,
    },
    EvidenceExportQueued {
        capability: Capability,
        tenant_id: TenantId,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceInvariant {
    TenantScoped,
    IdempotentCommand,
    PolicyBeforeMutation,
    AuditEveryStateMutation,
    DataResidencyBound,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompliancePack {
    Sox404,
    Soc2,
    Iso27001,
    Gdpr,
    Lgpd,
    KrPipa,
    JurisdictionalTax,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DataBoundary {
    CustomerMasterRecord,
    RevenuePipelineRecord,
    CommercialQuoteRecord,
    CampaignEngagementRecord,
    ServiceCaseRecord,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityTier {
    Regulated,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapabilityDescriptor {
    pub capability: Capability,
    pub bounded_context: BoundedContext,
    pub invariant: ServiceInvariant,
    pub tier: CapabilityTier,
    pub data_boundary: DataBoundary,
    pub required_packs: Vec<CompliancePack>,
}

impl CapabilityDescriptor {
    pub fn descriptors() -> Vec<Self> {
        vec![
            Self {
                capability: Capability::AccountMaster,
                bounded_context: BoundedContext::AccountMaster,
                invariant: ServiceInvariant::TenantScoped,
                tier: CapabilityTier::Regulated,
                data_boundary: DataBoundary::CustomerMasterRecord,
                required_packs: vec![
                    CompliancePack::Soc2,
                    CompliancePack::Iso27001,
                    CompliancePack::Gdpr,
                    CompliancePack::Lgpd,
                    CompliancePack::KrPipa,
                ],
            },
            Self {
                capability: Capability::Opportunity,
                bounded_context: BoundedContext::Opportunity,
                invariant: ServiceInvariant::TenantScoped,
                tier: CapabilityTier::Regulated,
                data_boundary: DataBoundary::RevenuePipelineRecord,
                required_packs: vec![
                    CompliancePack::Sox404,
                    CompliancePack::Soc2,
                    CompliancePack::JurisdictionalTax,
                ],
            },
            Self {
                capability: Capability::Quote,
                bounded_context: BoundedContext::Quote,
                invariant: ServiceInvariant::TenantScoped,
                tier: CapabilityTier::Regulated,
                data_boundary: DataBoundary::CommercialQuoteRecord,
                required_packs: vec![
                    CompliancePack::Sox404,
                    CompliancePack::Soc2,
                    CompliancePack::JurisdictionalTax,
                ],
            },
            Self {
                capability: Capability::Campaign,
                bounded_context: BoundedContext::Campaign,
                invariant: ServiceInvariant::TenantScoped,
                tier: CapabilityTier::Regulated,
                data_boundary: DataBoundary::CampaignEngagementRecord,
                required_packs: vec![
                    CompliancePack::Soc2,
                    CompliancePack::Gdpr,
                    CompliancePack::Lgpd,
                    CompliancePack::KrPipa,
                ],
            },
            Self {
                capability: Capability::ServiceCase,
                bounded_context: BoundedContext::ServiceCase,
                invariant: ServiceInvariant::TenantScoped,
                tier: CapabilityTier::Regulated,
                data_boundary: DataBoundary::ServiceCaseRecord,
                required_packs: vec![
                    CompliancePack::Soc2,
                    CompliancePack::Iso27001,
                    CompliancePack::Gdpr,
                    CompliancePack::Lgpd,
                    CompliancePack::KrPipa,
                ],
            },
        ]
    }
}

impl Capability {
    pub fn bounded_context(self) -> BoundedContext {
        match self {
            Self::AccountMaster => BoundedContext::AccountMaster,
            Self::Opportunity => BoundedContext::Opportunity,
            Self::Quote => BoundedContext::Quote,
            Self::Campaign => BoundedContext::Campaign,
            Self::ServiceCase => BoundedContext::ServiceCase,
        }
    }
}
