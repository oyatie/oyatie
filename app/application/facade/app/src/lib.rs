//! Foundation application slice composing the W-Foundation kernels.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod product_catalog;
pub use product_catalog::{ProductCatalogError, ProductEntry, ProductId, ProductMetadata};

use std::{collections::BTreeMap, fmt, sync::Arc};

pub use audit_chain_domain::{AuditChain, AuditEvent, Plane};
use cell_regional_pack::{RegionalPack, RegionalPackError};
use cell_routing::{CellBinding, CellBindingCreate, CellError, CellRouter, CellTier};
pub use data_ontology_domain::PropertyTier;
use data_ontology_domain::{ObjectEntity, ObjectGraphError, ObjectProperty};
use iam_identity_domain::{IdentityError, IdpBinding, Token, User, issue_token};
pub use iam_policy_cedar_domain::{
    AuthorizationDecision, PolicyEffect, PolicyRuleInput, PolicyScope, PolicyVersion,
};
use iam_policy_cedar_domain::{AuthorizationQuery, AuthorizationSubject, PolicyError, PolicySet};
use intelligence_adapter_kernel::{
    AdapterError, CostCeiling, InvocationPolicy, ProviderAuth, ProviderCallReceipt, ProviderId,
    ProviderMode, ProviderProfile, ProviderRoute, ProviderRoutePreference, ProviderRouteRequest,
    SubscriptionBindingRegistry, resolve_route,
};
pub use intelligence_bypass_domain::{AutonomyBreakGlass, AutonomyBreakGlassInput};
use intelligence_bypass_domain::{BypassError, BypassLedger, BypassLedgerRecord};
use intelligence_capability_domain::CapabilityError;
pub use intelligence_capability_domain::{
    AutonomyTier, Capability, CapabilityAction, CapabilityCostProfile, CapabilityMcpContract,
    CapabilityRegistry,
};
use intelligence_evidence_domain::EvidenceError;
pub use intelligence_evidence_domain::{EvidenceChain, EvidenceKind, EvidenceRecord};
pub use intelligence_mcp_gateway_domain::{
    DISCOVER_SCOPE, McpAccessTokenClaims, McpGatewayDescriptor, McpPrompt, McpRateLimitPolicy,
    McpTool, scope_for_tool_name,
};
use intelligence_mcp_gateway_domain::{
    McpGatewayError, McpPrincipal, McpRateLimiter, McpTenantEndpoint, authorize_tool_call,
    project_capability_tool, validate_access_token,
};
use intelligence_policy_domain::{
    AutonomyCapReason, AutonomyCapSource, AutonomyDecision, AutonomyVerdict, TenantPolicy,
};
pub use intelligence_run_domain::{Run, RunDisposition, RunState};
use intelligence_run_domain::{RunError, RunLedger, RunStart};
pub use intelligence_step_domain::{Step, StepDisposition, StepKind, StepState};
use intelligence_step_domain::{StepError, StepLedger, StepStart};
use messaging_domain::{EventingError, Outbox, OutboxRecord};
pub use network_residency::ResidencyClass;
use network_residency::{
    RegionRef, RegionRefCreate, infer_region_jurisdiction_label, parse_residency_class_label,
};
use observability_domain::{
    CAPABILITY_INVOCATION_OPERATION_NAME, CapabilityInvocationTraceContext,
    CapabilityInvocationTraceObserver, CapabilityInvocationTraceSpan, FOUNDRY_PROVIDER_NAME,
    InvocationTraceResult, NoopCapabilityInvocationTraceObserver,
    telemetry_data_classifications_label,
};
use check_cost_budget::{
    BudgetCeiling, BudgetError, BudgetLedger, BudgetScope, BudgetSnapshot, BudgetWarning,
};
pub use data_boundary_kernel::{
    AgeBand, ConsentScope, DataClass, PrivacyDataClass, Purpose, SubjectClass,
    privacy_data_classes_from,
};
use data_boundary_kernel::{
    Classified, DataClassification, DataUseAttributes, DataUseDenialReason, OperationalDataClass,
    evaluate_data_use,
};
use check_eval_domain::EvalError;
pub use check_eval_domain::{
    AdversarialKind, EvalCaseInput, EvalGate, EvalMetric, EvalRunInput, EvalSetInput,
    REQUIRED_LINGUISTIC_COHORT_LOCALES,
};
use secrets_domain::SecretRef;
pub use tenancy_domain::Tenant;
use tenancy_domain::TenantError;

const FOUNDATION_LOCAL_PROVIDER_ID: &str = "foundation-local";
const FOUNDATION_LOCAL_MODEL_REF: &str = "foundation-app";
const FOUNDATION_LOCAL_SECRET_REF_NAME: &str = "foundation-local-provider";
const FOUNDATION_LOCAL_PROVIDER_P95_LATENCY_MS: u32 = 1;
const FOUNDATION_LOCAL_PROVIDER_ATTEMPT: u32 = 1;

fn audit_classifications() -> [DataClassification; 1] {
    [DataClassification::from(OperationalDataClass::Audit)]
}

// Audit-chain storage remains hash-compatible with legacy `DataClass` payloads;
// foundation call sites express audit markers through `DataClassification` so
// new code does not construct operational markers as privacy data classes.
fn internal_audit_classifications() -> [DataClassification; 2] {
    [
        DataClassification::from(DataClass::InternalOnly),
        DataClassification::from(OperationalDataClass::Audit),
    ]
}

// Audit, evidence, run, step, telemetry, and MCP records still persist the
// shared `DataClass` vocabulary. Enforcement reads the typed privacy classes
// from `Capability`; these helpers make each record-facing projection explicit.
fn capability_record_classifications(capability: &Capability) -> Vec<DataClassification> {
    capability
        .touched_privacy_data_classes()
        .iter()
        .copied()
        .map(DataClassification::from)
        .collect()
}

fn capability_record_data_class_labels(capability: &Capability) -> String {
    capability
        .touched_privacy_data_classes()
        .iter()
        .map(|data_class| data_class.label())
        .collect::<Vec<_>>()
        .join(",")
}

fn behavioral_audit_classifications() -> [DataClassification; 2] {
    [
        DataClassification::from(DataClass::BehavioralTenantProduct),
        DataClassification::from(OperationalDataClass::Audit),
    ]
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRegistration {
    pub tenant_id: String,
    pub legal_name: String,
    pub home_region: String,
    pub residency_class: String,
    pub regulatory_packs: Vec<String>,
    pub autonomy_ceiling: AutonomyTier,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityRegistration {
    pub tenant_id: String,
    pub user_id: String,
    pub primary_identifier: String,
    pub display_name: String,
    pub roles: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub purpose: Purpose,
    pub ttl_seconds: u64,
    pub issued_at_epoch_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionalPackRegistration {
    pub pack_id: String,
    pub region: String,
    pub residency_class: String,
    pub controls: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectPropertyInput {
    pub name: String,
    pub value: String,
    pub tier: PropertyTier,
    pub privacy_data_class: PrivacyDataClass, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectEntityUpsert {
    pub tenant_id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub properties: Vec<ObjectPropertyInput>,
}

impl ObjectPropertyInput {
    pub fn new(
        name: String,
        value: String,
        tier: PropertyTier,
        privacy_data_class: PrivacyDataClass,
    ) -> Self {
        Self {
            name,
            value,
            tier,
            privacy_data_class,
        }
    }

    /// Compatibility constructor for legacy callers that still supply raw
    /// `DataClass` labels; fails closed for operational and subject markers.
    pub fn try_from_legacy_data_class(
        name: String,
        value: String,
        tier: PropertyTier,
        data_class: DataClass,
    ) -> Result<Self, FoundationError> {
        let privacy_data_class =
            PrivacyDataClass::try_from(data_class).map_err(|_| FoundationError::InvalidInput)?;
        Ok(Self::new(name, value, tier, privacy_data_class))
    }

    /// Legacy object-property compatibility label for call sites that still
    /// traffic in raw `DataClass` payloads. The source of truth is
    /// `privacy_data_class`, and construction fails closed for operational and
    /// subject markers.
    pub fn legacy_data_class(&self) -> DataClass {
        self.privacy_data_class.data_class()
    }

    #[deprecated(
        note = "use privacy_data_class for canonical typed access or legacy_data_class for the compatibility projection"
    )]
    pub fn data_class(&self) -> DataClass {
        self.legacy_data_class()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxPublish {
    pub tenant_id: String,
    pub topic: String,
    pub idempotency_key: String,
    pub payload_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub attributes: Vec<(String, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityRegistration {
    pub capability_id: String,
    pub namespace: String,
    pub action: CapabilityAction, // data_class: INTERNAL_ONLY
    pub required_tier: AutonomyTier,
    pub touched_privacy_data_classes: Vec<PrivacyDataClass>,
    pub evidence_topic: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCapabilityGrant {
    pub tenant_id: String,
    pub capability_id: String,
    pub mcp_visible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpDiscoveryRequest {
    pub tenant_id: String,
    pub access_token: McpAccessTokenClaims,
    pub now_epoch_seconds: u64,
    pub tld: String,
    pub authorization_server: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpToolCallRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub tool_name: String,
    pub access_token: McpAccessTokenClaims,
    pub tld: String,
    pub authorization_server: String,
    pub purpose: Purpose,
    pub subject_class: SubjectClass, // data_class: INTERNAL_ONLY
    pub budget_window_id: String,
    pub projected_cost_micros: u64,
    pub started_at_epoch_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostBudgetRegistration {
    pub tenant_id: String,
    pub capability_id: Option<String>,
    pub window_id: String,
    pub monthly_limit_micros: u64,
    pub per_invocation_limit_micros: u64,
    pub warning_threshold_percent: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInvocationRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub capability_id: String,
    pub purpose: Purpose,
    pub subject_class: SubjectClass, // data_class: INTERNAL_ONLY
    pub budget_window_id: String,
    pub projected_cost_micros: u64,
    pub started_at_epoch_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInvocationPrincipal {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub user_id: String,                // data_class: INTERNAL_ONLY
    pub autonomy_ceiling: AutonomyTier, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvocationReceipt {
    pub tenant_id: String,
    pub user_id: String,
    pub capability_id: String,
    pub evidence_event_hash: String,
    pub cost_reservation_id: Option<String>,
    pub cost_budget_warning: Option<BudgetWarning>,
    pub run_id: Option<String>,
    pub foundry_step_id: Option<String>,
    pub foundry_evidence_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FoundationError {
    TenantAlreadyExists,
    TenantNotFound,
    UserNotFound,
    CapabilityNotFound,
    CapabilityAlreadyExists,
    CapabilityNotLicensed,
    CapabilityEvalGateNotReady,
    McpAccessDenied,
    McpRateLimited,
    CellBindingImmutable,
    RegionalPackAlreadyExists,
    PolicyVersionAlreadyExists,
    DataUseNotAllowed,
    OutboxRecordNotFound,
    TokenTtlTooLong,
    InvalidInput,
    AutonomyCeilingExceeded,
    CapabilityInvocationUnauthorized,
    CostBudgetNotConfigured,
    CostBudgetExceeded,
    /// ADR-0083 amendment 2026-05-15: `AuditChain::append_classifications`
    /// returns `Result<&AuditEvent, AuditChainError>` — Tier 1 fallible.
    /// The variants of `AuditChainError` (`EmptyTenantId`,
    /// `TenantShardMismatch`, etc.) propagate to this app boundary so callers
    /// can pattern-match the failure mode rather than seeing a silent panic.
    AuditChainAppendFailed(audit_chain_domain::AuditChainError),
}

impl From<audit_chain_domain::AuditChainError> for FoundationError {
    fn from(error: audit_chain_domain::AuditChainError) -> Self {
        Self::AuditChainAppendFailed(error)
    }
}

struct DeniedInvocationRecord<'a> {
    request: &'a CapabilityInvocationRequest,
    tenant: &'a Tenant,
    capability: &'a Capability,
    disposition: RunDisposition,
    evidence_kind: EvidenceKind,
    reason: &'static str,
    audit_event_hash: String,
    extra_fields: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InvocationSettlementStatus {
    NotApplicable,
    Completed,
    Failed,
}

impl InvocationSettlementStatus {
    fn as_completion_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    fn as_release_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::Completed => "released",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InvocationDataUseDenial {
    effective_purpose: Purpose,
    denied_data_class: Option<DataClass>,
    reason: &'static str,
}

#[derive(Clone)]
struct FoundationObservability {
    invocation_trace_observer: Arc<dyn CapabilityInvocationTraceObserver>,
}

impl FoundationObservability {
    fn new(observer: impl CapabilityInvocationTraceObserver + 'static) -> Self {
        Self {
            invocation_trace_observer: Arc::new(observer),
        }
    }

    fn start_capability_invocation(
        &self,
        context: &CapabilityInvocationTraceContext,
    ) -> Box<dyn CapabilityInvocationTraceSpan> {
        self.invocation_trace_observer
            .start_capability_invocation(context)
    }
}

impl Default for FoundationObservability {
    fn default() -> Self {
        Self::new(NoopCapabilityInvocationTraceObserver)
    }
}

impl fmt::Debug for FoundationObservability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FoundationObservability")
            .field("invocation_trace_observer", &self.invocation_trace_observer)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct Foundation {
    tenants: BTreeMap<String, Tenant>,
    tenant_policies: BTreeMap<String, TenantPolicy>,
    users: BTreeMap<(String, String), User>,
    capabilities: CapabilityRegistry,
    regional_packs: BTreeMap<String, RegionalPack>,
    object_entities: BTreeMap<(String, String), ObjectEntity>,
    outbox: Outbox,
    consent_scopes: BTreeMap<String, ConsentScope>,
    policies: PolicySet,
    eval_gate: EvalGate,
    cost_budgets: BudgetLedger,
    foundation_bypass_ledger: BypassLedger,
    foundry_runs: RunLedger,
    foundry_steps: StepLedger,
    foundry_evidence: EvidenceChain,
    mcp_rate_limiter: McpRateLimiter,
    cells: CellRouter,
    audit_chain: AuditChain,
    observability: FoundationObservability,
}

impl Default for Foundation {
    fn default() -> Self {
        Self {
            tenants: BTreeMap::new(),
            tenant_policies: BTreeMap::new(),
            users: BTreeMap::new(),
            capabilities: CapabilityRegistry::default(),
            regional_packs: BTreeMap::new(),
            object_entities: BTreeMap::new(),
            outbox: Outbox::default(),
            consent_scopes: BTreeMap::new(),
            policies: PolicySet::default(),
            eval_gate: EvalGate::default(),
            cost_budgets: BudgetLedger::default(),
            foundation_bypass_ledger: BypassLedger::default(),
            foundry_runs: RunLedger::default(),
            foundry_steps: StepLedger::default(),
            foundry_evidence: EvidenceChain::default(),
            mcp_rate_limiter: McpRateLimiter::default(),
            cells: CellRouter::default(),
            audit_chain: AuditChain::multi_tenant_shards(),
            observability: FoundationObservability::default(),
        }
    }
}

impl Foundation {
    pub fn with_invocation_trace_observer(
        mut self,
        observer: impl CapabilityInvocationTraceObserver + 'static,
    ) -> Self {
        self.observability = FoundationObservability::new(observer);
        self
    }

    pub fn register_autonomy_break_glass(
        &mut self,
        input: AutonomyBreakGlassInput,
    ) -> Result<AutonomyBreakGlass, FoundationError> {
        self.require_tenant(&input.tenant_id)?;
        if self.capabilities.get(&input.capability_id).is_none() {
            return Err(FoundationError::CapabilityNotFound);
        }
        let record = input.build().map_err(map_bypass_error)?;
        let mut candidate = self.foundation_bypass_ledger.clone();
        candidate
            .insert_record(BypassLedgerRecord::from(record.clone()))
            .map_err(map_bypass_error)?;
        candidate
            .validate_windows(record.created_at_epoch_days.value)
            .map_err(map_bypass_error)?;
        self.foundation_bypass_ledger = candidate;
        self.audit_chain.append_classifications(
            record.tenant_id.value.clone(),
            "foundry.autonomy.break_glass.approve",
            Plane::Control,
            Purpose::CoreService,
            internal_audit_classifications(),
            "ALLOW",
        )?;
        Ok(record)
    }

    pub fn foundation_bypass_ledger(&self) -> &BypassLedger {
        &self.foundation_bypass_ledger
    }

    pub fn onboard_tenant(
        &mut self,
        registration: TenantRegistration,
    ) -> Result<Tenant, FoundationError> {
        if self.tenants.contains_key(&registration.tenant_id) {
            return Err(FoundationError::TenantAlreadyExists);
        }
        let residency_class = parse_residency_class_label(&registration.residency_class)
            .ok_or(FoundationError::InvalidInput)?;
        let tenant = Tenant::new(
            registration.tenant_id.clone(),
            registration.legal_name,
            registration.home_region,
            residency_class,
            registration.regulatory_packs,
        )
        .map_err(map_tenant_error)?;
        self.tenant_policies.insert(
            tenant.id.clone(),
            TenantPolicy::new(tenant.id.clone(), registration.autonomy_ceiling),
        );
        self.tenants.insert(tenant.id.clone(), tenant.clone());
        self.audit_chain.append_classifications(
            tenant.id.clone(),
            "tenant.create",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(tenant)
    }

    pub fn bind_cell(
        &mut self,
        tenant_id: &str,
        az: impl Into<String>,
        cell_id: impl Into<String>,
    ) -> Result<CellBinding, FoundationError> {
        let tenant = self.require_tenant(tenant_id)?.clone();
        let region = tenant.home_region.value.clone();
        let region_ref = RegionRef::new(RegionRefCreate {
            region_id: region.clone(),
            jurisdiction: infer_region_jurisdiction_label(&region),
            cell_group_ref: format!("cells/{region}"),
        })
        .map_err(|_| FoundationError::InvalidInput)?;
        let cell_id = cell_id.into();
        let binding_input = CellBindingCreate {
            tenant_id: tenant_id.to_string(),
            region: region_ref,
            residency_class: tenant.residency_class.value,
            az: az.into(),
            hsm_partition_ref: format!("hsm/{region}/{cell_id}"),
            cell_id,
            tier: CellTier::Pooled,
        };
        match self.cells.bind(binding_input) {
            Ok(binding) => {
                self.audit_chain.append_classifications(
                    tenant_id,
                    "cloud.cell.bind",
                    Plane::Control,
                    Purpose::CoreService,
                    vec![DataClass::InternalOnly],
                    "ALLOW",
                )?;
                Ok(binding)
            }
            Err(CellError::AlreadyBound) => {
                self.audit_chain.append_classifications(
                    tenant_id,
                    "cloud.cell.bind",
                    Plane::Control,
                    Purpose::CoreService,
                    vec![DataClass::InternalOnly],
                    "DENY",
                )?;
                Err(FoundationError::CellBindingImmutable)
            }
            Err(
                CellError::InvalidTenantId
                | CellError::EmptyAz
                | CellError::EmptyCell
                | CellError::EmptyHsmPartition
                | CellError::AzRegionMismatch
                | CellError::ResidencyRegionMismatch,
            ) => Err(FoundationError::InvalidInput),
        }
    }

    pub fn upsert_identity(
        &mut self,
        registration: IdentityRegistration,
    ) -> Result<User, FoundationError> {
        let tenant = self.require_tenant(&registration.tenant_id)?;
        let region_pack = tenant
            .regulatory_packs
            .value
            .iter()
            .find(|pack| pack.starts_with("oya-pack-"))
            .cloned()
            .unwrap_or_else(|| {
                format!(
                    "oya-pack-{}",
                    tenant
                        .residency_class
                        .value
                        .label()
                        .unwrap_or("global")
                        .replace('_', "-")
                )
            });
        let idp_binding = IdpBinding::new(
            region_pack,
            "idp_foundation_local".to_string(),
            registration.primary_identifier.clone(),
            0,
        )
        .map_err(map_identity_error)?;
        let user = User::new(
            registration.tenant_id.clone(),
            registration.user_id,
            registration.primary_identifier,
            registration.display_name,
            registration.roles,
            idp_binding,
        )
        .map_err(map_identity_error)?;
        self.users.insert(
            (
                registration.tenant_id.clone(),
                user.user_id().as_str().to_string(),
            ),
            user.clone(),
        );
        self.audit_chain.append_classifications(
            registration.tenant_id,
            "identity.user.upsert",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::PiiIdentifying, DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(user)
    }

    pub fn issue_token(&mut self, request: TokenRequest) -> Result<Token, FoundationError> {
        self.require_user(&request.tenant_id, &request.user_id)?;
        match issue_token(
            request.tenant_id.clone(),
            request.user_id,
            request.purpose,
            request.ttl_seconds,
            request.issued_at_epoch_seconds,
        ) {
            Ok(token) => {
                self.audit_chain.append_classifications(
                    token.tenant_id.clone(),
                    "identity.token.issue",
                    Plane::Control,
                    request.purpose,
                    vec![DataClass::PiiIdentifying],
                    "ALLOW",
                )?;
                Ok(token)
            }
            Err(IdentityError::TokenTtlTooLong) => {
                self.audit_chain.append_classifications(
                    request.tenant_id,
                    "identity.token.issue",
                    Plane::Control,
                    request.purpose,
                    vec![DataClass::PiiIdentifying],
                    "DENY",
                )?;
                Err(FoundationError::TokenTtlTooLong)
            }
            Err(_) => Err(FoundationError::InvalidInput),
        }
    }

    pub fn grant_data_use(
        &mut self,
        tenant_id: &str,
        purpose: Purpose,
        data_class: PrivacyDataClass,
    ) -> Result<(), FoundationError> {
        self.grant_privacy_data_use(tenant_id, purpose, data_class)
    }

    /// Compatibility entry point for raw-label callers at import/API seams.
    ///
    /// The canonical grant path takes `PrivacyDataClass`; this path preserves
    /// older raw `DataClass` ingestion while failing closed for operational
    /// markers and subject markers.
    pub fn try_grant_legacy_data_use(
        &mut self,
        tenant_id: &str,
        purpose: Purpose,
        data_class: DataClass,
    ) -> Result<(), FoundationError> {
        let data_class =
            PrivacyDataClass::try_from(data_class).map_err(|_| FoundationError::InvalidInput)?;
        self.grant_privacy_data_use(tenant_id, purpose, data_class)
    }

    pub fn grant_privacy_data_use(
        &mut self,
        tenant_id: &str,
        purpose: Purpose,
        data_class: PrivacyDataClass,
    ) -> Result<(), FoundationError> {
        self.require_tenant(tenant_id)?;
        let current = self.consent_scopes.remove(tenant_id).unwrap_or_default();
        self.consent_scopes.insert(
            tenant_id.to_string(),
            current.allow_privacy_data_class(purpose, data_class),
        );
        let audit_data_class = data_class.data_class();
        self.audit_chain.append_classifications(
            tenant_id,
            "privacy.data-use.grant",
            Plane::Control,
            purpose,
            vec![audit_data_class],
            "ALLOW",
        )?;
        Ok(())
    }

    pub fn publish_policy(
        &mut self,
        version: PolicyVersion,
    ) -> Result<iam_policy_cedar_domain::PublishedPolicy, FoundationError> {
        let scope_tenant_id = match &version.scope {
            PolicyScope::Global => None,
            PolicyScope::Tenant(tenant_id) => Some(tenant_id.clone()),
        };
        if let Some(tenant_id) = &scope_tenant_id {
            self.require_tenant(tenant_id)?;
        }
        let published = self.policies.publish(version).map_err(map_policy_error)?;
        self.audit_chain.append_classifications(
            scope_tenant_id.unwrap_or_else(|| "ten_system".to_string()),
            "cedar.policy.publish",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(published)
    }

    pub fn authorize(
        &mut self,
        request: AuthorizationRequest,
    ) -> Result<AuthorizationDecision, FoundationError> {
        let user = self.require_user(&request.tenant_id, &request.user_id)?;
        let decision = self.policies.authorize(&AuthorizationQuery {
            subject: AuthorizationSubject {
                tenant_id: request.tenant_id.clone(),
                roles: user.roles.value.clone(),
            },
            action: request.action,
            resource: request.resource,
            attributes: request.attributes.into_iter().collect(),
        });
        self.audit_chain.append_classifications(
            request.tenant_id,
            "cedar.policy.authorize",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            if decision.allowed { "ALLOW" } else { "DENY" },
        )?;
        Ok(decision)
    }

    pub fn register_capability(
        &mut self,
        registration: CapabilityRegistration,
    ) -> Result<Capability, FoundationError> {
        self.register_capability_with_cost_profile(
            registration,
            CapabilityCostProfile::foundation_local_default(),
        )
    }

    pub fn register_capability_with_cost_profile(
        &mut self,
        registration: CapabilityRegistration,
        cost_profile: CapabilityCostProfile,
    ) -> Result<Capability, FoundationError> {
        let mcp_contract = CapabilityMcpContract::default_for(
            &registration.capability_id,
            &registration.namespace,
        )
        .map_err(map_capability_error)?;
        self.register_capability_with_cost_profile_and_mcp_contract(
            registration,
            cost_profile,
            mcp_contract,
        )
    }

    pub fn register_capability_with_mcp_contract(
        &mut self,
        registration: CapabilityRegistration,
        mcp_contract: CapabilityMcpContract,
    ) -> Result<Capability, FoundationError> {
        self.register_capability_with_cost_profile_and_mcp_contract(
            registration,
            CapabilityCostProfile::foundation_local_default(),
            mcp_contract,
        )
    }

    pub fn register_capability_with_cost_profile_and_mcp_contract(
        &mut self,
        registration: CapabilityRegistration,
        cost_profile: CapabilityCostProfile,
        mcp_contract: CapabilityMcpContract,
    ) -> Result<Capability, FoundationError> {
        self.eval_gate
            .assert_publish_ready(&registration.capability_id)
            .map_err(map_eval_error)?;
        let mut capability = Capability::new_with_action_and_cost_profile(
            registration.capability_id,
            registration.namespace,
            registration.action,
            registration.required_tier,
            registration.touched_privacy_data_classes,
            registration.evidence_topic,
            cost_profile,
        )
        .map_err(map_capability_error)?;
        capability.mcp_contract = mcp_contract;
        self.capabilities
            .publish(capability.clone())
            .map_err(map_capability_error)?;
        self.audit_chain.append_classifications(
            "ten_system",
            "foundry.capability.publish",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(capability)
    }

    pub fn register_capability_eval_set(
        &mut self,
        eval_set: EvalSetInput,
    ) -> Result<(), FoundationError> {
        let capability_id = eval_set.capability_id.clone();
        self.eval_gate
            .register_eval_set(eval_set)
            .map_err(map_eval_error)?;
        self.audit_chain.append_classifications(
            "ten_system",
            "foundry.eval-set.register",
            Plane::Control,
            Purpose::CoreService,
            audit_classifications(),
            "ALLOW",
        )?;
        self.audit_chain.append_classifications(
            "ten_system",
            format!("foundry.eval-set.ready:{capability_id}"),
            Plane::Control,
            Purpose::CoreService,
            audit_classifications(),
            "ALLOW",
        )?;
        Ok(())
    }

    pub fn record_capability_eval_run(
        &mut self,
        eval_run: EvalRunInput,
    ) -> Result<(), FoundationError> {
        let capability_id = eval_run.capability_id.clone();
        self.eval_gate
            .record_run(eval_run)
            .map_err(map_eval_error)?;
        self.audit_chain.append_classifications(
            "ten_system",
            format!("foundry.eval-run.pass:{capability_id}"),
            Plane::Analytics,
            Purpose::Analytics,
            behavioral_audit_classifications(),
            "ALLOW",
        )?;
        Ok(())
    }

    pub fn grant_capability_to_tenant(
        &mut self,
        grant: TenantCapabilityGrant,
    ) -> Result<(), FoundationError> {
        self.require_tenant(&grant.tenant_id)?;
        self.capabilities
            .grant_to_tenant(
                grant.tenant_id.clone(),
                grant.capability_id.clone(),
                grant.mcp_visible,
            )
            .map_err(map_capability_error)?;
        self.audit_chain.append_classifications(
            grant.tenant_id,
            "foundry.capability.license",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(())
    }

    pub fn discover_tenant_capabilities(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<Capability>, FoundationError> {
        let policy = self
            .tenant_policies
            .get(tenant_id)
            .ok_or(FoundationError::TenantNotFound)?;
        self.capabilities
            .discover_for_tenant(tenant_id, policy.autonomy_ceiling)
            .map_err(map_capability_error)
    }

    pub fn discover_mcp_gateway(
        &mut self,
        request: McpDiscoveryRequest,
    ) -> Result<McpGatewayDescriptor, FoundationError> {
        let tenant = self.require_tenant(&request.tenant_id)?.clone();
        let endpoint = McpTenantEndpoint::new(
            tenant.id.clone(),
            tenant.home_region.value.clone(),
            request.tld,
            request.authorization_server,
        )
        .map_err(map_mcp_error)?;
        let principal =
            self.mcp_principal(&endpoint, request.access_token, request.now_epoch_seconds)?;

        if let Err(error) = McpGatewayDescriptor::new(endpoint.clone(), &principal, &[]) {
            self.audit_chain.append_classifications(
                tenant.id,
                "foundry.mcp.tools.list",
                Plane::Control,
                Purpose::CapabilityInvocation,
                vec![DataClass::InternalOnly],
                "DENY",
            )?;
            return Err(map_mcp_error(error));
        }

        let capabilities = self
            .capabilities
            .discover_for_tenant(&endpoint.tenant_id.value, principal.autonomy_ceiling)
            .map_err(map_capability_error)?;
        let descriptor = McpGatewayDescriptor::new(endpoint, &principal, &capabilities)
            .map_err(map_mcp_error)?;
        self.audit_chain.append_classifications(
            descriptor.endpoint.tenant_id.value.clone(),
            "foundry.mcp.tools.list",
            Plane::Control,
            Purpose::CapabilityInvocation,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(descriptor)
    }

    pub fn invoke_capability_via_mcp(
        &mut self,
        request: McpToolCallRequest,
    ) -> Result<InvocationReceipt, FoundationError> {
        self.require_user(&request.tenant_id, &request.user_id)?;
        let tenant = self.require_tenant(&request.tenant_id)?.clone();
        let endpoint = McpTenantEndpoint::new(
            tenant.id.clone(),
            tenant.home_region.value.clone(),
            request.tld,
            request.authorization_server,
        )
        .map_err(map_mcp_error)?;
        let principal = self.mcp_principal(
            &endpoint,
            request.access_token,
            request.started_at_epoch_seconds,
        )?;
        if endpoint.tenant_id.value != principal.tenant_id.value {
            self.audit_chain.append_classifications(
                tenant.id,
                "foundry.mcp.tool.call",
                Plane::Data,
                request.purpose,
                vec![DataClass::InternalOnly],
                "DENY",
            )?;
            return Err(FoundationError::McpAccessDenied);
        }
        if principal.subject_id.value != request.user_id {
            self.audit_chain.append_classifications(
                tenant.id,
                "foundry.mcp.tool.call",
                Plane::Data,
                request.purpose,
                vec![DataClass::InternalOnly],
                "DENY",
            )?;
            return Err(FoundationError::McpAccessDenied);
        }

        let visible_capability_opt = self
            .capabilities
            .discover_for_tenant(&endpoint.tenant_id.value, principal.autonomy_ceiling)
            .map_err(map_capability_error)?
            .into_iter()
            .find(|capability| capability.id == request.tool_name);
        let visible_capability = match visible_capability_opt {
            Some(capability) => capability,
            None => {
                self.audit_chain.append_classifications(
                    tenant.id.clone(),
                    "foundry.mcp.tool.call",
                    Plane::Data,
                    request.purpose,
                    vec![DataClass::InternalOnly],
                    "DENY",
                )?;
                return Err(FoundationError::McpAccessDenied);
            }
        };
        let tool = project_capability_tool(&visible_capability).map_err(map_mcp_error)?;
        if let Err(error) = authorize_tool_call(&endpoint, &principal, &tool) {
            self.audit_chain.append_classifications(
                tenant.id,
                "foundry.mcp.tool.call",
                Plane::Data,
                request.purpose,
                vec![DataClass::InternalOnly],
                "DENY",
            )?;
            return Err(map_mcp_error(error));
        }

        if let Err(error) = self.mcp_rate_limiter.check_and_record(
            &endpoint.tenant_id.value,
            &tool.name.value,
            request.started_at_epoch_seconds,
        ) {
            self.audit_chain.append_classifications(
                tenant.id,
                "foundry.mcp.rate-limit",
                Plane::Data,
                request.purpose,
                vec![DataClass::InternalOnly, DataClass::BehavioralTenantProduct],
                "DENY",
            )?;
            return Err(map_mcp_error(error));
        }

        self.audit_chain.append_classifications(
            endpoint.tenant_id.value.clone(),
            "foundry.mcp.tool.call",
            Plane::Data,
            request.purpose,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        self.invoke_capability_as_principal(
            CapabilityInvocationPrincipal {
                tenant_id: endpoint.tenant_id.value.clone(),
                user_id: principal.subject_id.value,
                autonomy_ceiling: principal.autonomy_ceiling,
            },
            CapabilityInvocationRequest {
                tenant_id: endpoint.tenant_id.value,
                user_id: request.user_id,
                capability_id: request.tool_name,
                purpose: request.purpose,
                subject_class: request.subject_class,
                budget_window_id: request.budget_window_id,
                projected_cost_micros: request.projected_cost_micros,
                started_at_epoch_seconds: request.started_at_epoch_seconds,
            },
        )
    }

    pub fn configure_mcp_rate_limit(
        &mut self,
        policy: McpRateLimitPolicy,
    ) -> Result<(), FoundationError> {
        self.mcp_rate_limiter.set_policy(policy);
        self.audit_chain.append_classifications(
            "ten_system",
            "foundry.mcp.rate-limit.configure",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(())
    }

    pub fn configure_tenant_cost_budget(
        &mut self,
        registration: CostBudgetRegistration,
    ) -> Result<(), FoundationError> {
        self.require_tenant(&registration.tenant_id)?;
        let ceiling = BudgetCeiling::new(
            registration.monthly_limit_micros,
            registration.per_invocation_limit_micros,
            registration.warning_threshold_percent,
        )
        .map_err(map_budget_error)?;

        if let Some(capability_id) = registration.capability_id {
            if self.capabilities.get(&capability_id).is_none() {
                return Err(FoundationError::CapabilityNotFound);
            }
            let scope = BudgetScope::new(
                registration.tenant_id.clone(),
                capability_id,
                registration.window_id.clone(),
            )
            .map_err(map_budget_error)?;
            self.cost_budgets
                .configure_capability_ceiling(scope, ceiling)
                .map_err(map_budget_error)?;
        } else {
            self.cost_budgets
                .configure_tenant_ceiling(
                    registration.tenant_id.clone(),
                    registration.window_id.clone(),
                    ceiling,
                )
                .map_err(map_budget_error)?;
        }

        self.audit_chain.append_classifications(
            registration.tenant_id,
            "foundry.cost-budget.configure",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(())
    }

    pub fn invoke_capability_as_principal(
        &mut self,
        principal: CapabilityInvocationPrincipal,
        request: CapabilityInvocationRequest,
    ) -> Result<InvocationReceipt, FoundationError> {
        if principal.tenant_id != request.tenant_id || principal.user_id != request.user_id {
            if let (Some(tenant), Some(capability)) = (
                self.tenants.get(&request.tenant_id).cloned(),
                self.capabilities.get(&request.capability_id).cloned(),
            ) {
                let authorization_audit_hash = self
                    .audit_chain
                    .append_classifications(
                        request.tenant_id.clone(),
                        "cedar.policy.authorize",
                        Plane::Control,
                        request.purpose,
                        vec![DataClass::InternalOnly],
                        "DENY",
                    )?
                    .hash
                    .clone();
                let (capability_invoke_audit_hash, topic_audit_hash) =
                    self.append_invocation_denial_audits(&request, &capability)?;
                self.record_denied_invocation(DeniedInvocationRecord {
                    request: &request,
                    tenant: &tenant,
                    capability: &capability,
                    disposition: RunDisposition::FailureAuthorization,
                    evidence_kind: EvidenceKind::CapabilityInvocation,
                    reason: "principal_mismatch",
                    audit_event_hash: topic_audit_hash,
                    extra_fields: BTreeMap::from([
                        (
                            "authorization_audit_event_hash".to_string(),
                            authorization_audit_hash,
                        ),
                        (
                            "capability_invoke_audit_event_hash".to_string(),
                            capability_invoke_audit_hash,
                        ),
                    ]),
                })?;
            }
            return Err(FoundationError::CapabilityInvocationUnauthorized);
        }
        let user = self
            .require_user(&request.tenant_id, &request.user_id)?
            .clone();
        let tenant = self.require_tenant(&request.tenant_id)?.clone();
        let capability = self
            .capabilities
            .get(&request.capability_id)
            .ok_or(FoundationError::CapabilityNotFound)?
            .clone();
        let data_classifications = capability_record_classifications(&capability);
        let privacy_data_classes = capability.touched_privacy_data_classes().to_vec();
        let policy = self
            .tenant_policies
            .get(&request.tenant_id)
            .ok_or(FoundationError::TenantNotFound)?;
        let touched_data_classes = telemetry_data_classifications_label(&data_classifications);
        let cell_id = self
            .cells
            .get(&request.tenant_id)
            .map(|cell_binding| cell_binding.cell_id.value.clone());
        let invocation_span =
            self.observability
                .start_capability_invocation(&CapabilityInvocationTraceContext {
                    service_name: "oya-foundation-app".to_string(),
                    tenant_id: request.tenant_id.clone(),
                    tenant_region: tenant.home_region.value.clone(),
                    cell_id,
                    capability_id: request.capability_id.clone(),
                    data_classes_touched: touched_data_classes,
                    operation_name: CAPABILITY_INVOCATION_OPERATION_NAME.to_string(),
                    provider_name: FOUNDRY_PROVIDER_NAME.to_string(),
                });
        emit_invocation_trace(invocation_span.as_ref(), "started", None);
        if !self
            .capabilities
            .is_licensed_for_tenant(&request.tenant_id, &request.capability_id)
        {
            let license_audit_hash = self
                .audit_chain
                .append_classifications(
                    request.tenant_id.clone(),
                    "foundry.capability.license",
                    Plane::Control,
                    request.purpose,
                    vec![DataClass::InternalOnly],
                    "DENY",
                )?
                .hash
                .clone();
            let (capability_invoke_audit_hash, topic_audit_hash) =
                self.append_invocation_denial_audits(&request, &capability)?;
            self.record_denied_invocation(DeniedInvocationRecord {
                request: &request,
                tenant: &tenant,
                capability: &capability,
                disposition: RunDisposition::FailureLicense,
                evidence_kind: EvidenceKind::CapabilityInvocation,
                reason: "license",
                audit_event_hash: topic_audit_hash,
                extra_fields: BTreeMap::from([
                    (
                        "capability_invoke_audit_event_hash".to_string(),
                        capability_invoke_audit_hash,
                    ),
                    ("license_audit_event_hash".to_string(), license_audit_hash),
                ]),
            })?;
            emit_invocation_trace(invocation_span.as_ref(), "denied", Some("license"));
            return Err(FoundationError::CapabilityNotLicensed);
        }

        let mut autonomy_decision = policy.evaluate_with_context(
            &capability,
            principal.autonomy_ceiling,
            &tenant.regulatory_packs.value,
            request.subject_class,
        );
        let pre_break_glass_autonomy_decision = autonomy_decision.clone();
        let autonomy_break_glass = if autonomy_decision.allowed() {
            None
        } else {
            self.foundation_bypass_ledger
                .active_autonomy_break_glass_for(
                    &request.tenant_id,
                    &request.capability_id,
                    capability.required_tier,
                    epoch_seconds_to_epoch_days(request.started_at_epoch_seconds),
                )
                .cloned()
        };
        if let Some(break_glass) = &autonomy_break_glass {
            apply_autonomy_break_glass(&mut autonomy_decision, break_glass);
        }
        invocation_span
            .record_autonomy_tier(autonomy_tier_label(autonomy_decision.effective_ceiling));
        let authorization_decision = self.policies.authorize(&AuthorizationQuery {
            subject: AuthorizationSubject {
                tenant_id: request.tenant_id.clone(),
                roles: user.roles.value.clone(),
            },
            action: "foundry.capability.invoke".to_string(),
            resource: format!("capability:{}", request.capability_id),
            attributes: invocation_authorization_attributes(
                &request,
                &capability,
                principal.autonomy_ceiling,
                &autonomy_decision,
                autonomy_break_glass.as_ref(),
                if autonomy_break_glass.is_some() {
                    Some(&pre_break_glass_autonomy_decision)
                } else {
                    None
                },
            ),
        });
        let authorization_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "cedar.policy.authorize",
                Plane::Control,
                request.purpose,
                vec![DataClass::InternalOnly],
                if authorization_decision.allowed {
                    "ALLOW"
                } else {
                    "DENY"
                },
            )?
            .hash
            .clone();
        if !authorization_decision.allowed {
            let (capability_invoke_audit_hash, topic_audit_hash) =
                self.append_invocation_denial_audits(&request, &capability)?;
            self.record_denied_invocation(DeniedInvocationRecord {
                request: &request,
                tenant: &tenant,
                capability: &capability,
                disposition: RunDisposition::FailureAuthorization,
                evidence_kind: EvidenceKind::CapabilityInvocation,
                reason: "authorization",
                audit_event_hash: topic_audit_hash,
                extra_fields: BTreeMap::from([
                    (
                        "authorization_audit_event_hash".to_string(),
                        authorization_audit_hash,
                    ),
                    (
                        "authorization_reason".to_string(),
                        authorization_decision.reason,
                    ),
                    (
                        "capability_invoke_audit_event_hash".to_string(),
                        capability_invoke_audit_hash,
                    ),
                ]),
            })?;
            emit_invocation_trace(invocation_span.as_ref(), "denied", Some("authorization"));
            return Err(FoundationError::CapabilityInvocationUnauthorized);
        }

        let autonomy_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.autonomy.decision",
                Plane::Control,
                request.purpose,
                internal_audit_classifications(),
                if autonomy_decision.allowed() {
                    "ALLOW"
                } else {
                    "DENY"
                },
            )?
            .hash
            .clone();
        let break_glass_invoke_audit_hash = match autonomy_break_glass.as_ref() {
            Some(break_glass) => Some(
                self.audit_chain
                    .append_classifications(
                        break_glass.tenant_id.value.clone(),
                        "foundry.autonomy.break_glass.invoke",
                        Plane::Control,
                        request.purpose,
                        internal_audit_classifications(),
                        "ALLOW",
                    )?
                    .hash
                    .clone(),
            ),
            None => None,
        };
        if !autonomy_decision.allowed() {
            let (capability_invoke_audit_hash, topic_audit_hash) =
                self.append_invocation_denial_audits(&request, &capability)?;
            let mut autonomy_fields =
                autonomy_decision_fields(&autonomy_decision, &autonomy_audit_hash);
            autonomy_fields.insert(
                "capability_invoke_audit_event_hash".to_string(),
                capability_invoke_audit_hash,
            );
            self.record_denied_invocation(DeniedInvocationRecord {
                request: &request,
                tenant: &tenant,
                capability: &capability,
                disposition: RunDisposition::FailureAutonomy,
                evidence_kind: EvidenceKind::AutonomyDecision,
                reason: "autonomy",
                audit_event_hash: topic_audit_hash,
                extra_fields: autonomy_fields,
            })?;
            emit_invocation_trace(invocation_span.as_ref(), "denied", Some("autonomy"));
            return Err(FoundationError::AutonomyCeilingExceeded);
        }
        if let Err(denial) = evaluate_invocation_data_use(
            &capability,
            &request,
            self.consent_scopes.get(&request.tenant_id),
        ) {
            let data_use_audit_hash = self
                .audit_chain
                .append_classifications(
                    request.tenant_id.clone(),
                    "privacy.data-use.evaluate",
                    Plane::Control,
                    denial.effective_purpose,
                    capability_record_classifications(&capability),
                    "DENY",
                )?
                .hash
                .clone();
            let (capability_invoke_audit_hash, topic_audit_hash) =
                self.append_invocation_denial_audits(&request, &capability)?;
            let mut data_use_fields = data_use_denial_fields(
                &request,
                &capability,
                &denial,
                capability_invoke_audit_hash,
            );
            data_use_fields.insert("data_use_audit_event_hash".to_string(), data_use_audit_hash);
            self.record_denied_invocation(DeniedInvocationRecord {
                request: &request,
                tenant: &tenant,
                capability: &capability,
                disposition: RunDisposition::FailureClass,
                evidence_kind: EvidenceKind::ConsentCheck,
                reason: "data_boundary",
                audit_event_hash: topic_audit_hash,
                extra_fields: data_use_fields,
            })?;
            emit_invocation_trace(invocation_span.as_ref(), "denied", Some("data_boundary"));
            return Err(FoundationError::DataUseNotAllowed);
        }
        if !capability.allows_projected_invocation_cost(request.projected_cost_micros) {
            let cost_budget_audit_hash = self
                .audit_chain
                .append_classifications(
                    request.tenant_id.clone(),
                    "foundry.cost-budget.reserve",
                    Plane::Control,
                    request.purpose,
                    vec![DataClass::InternalOnly],
                    "DENY",
                )?
                .hash
                .clone();
            let (capability_invoke_audit_hash, topic_audit_hash) =
                self.append_invocation_denial_audits(&request, &capability)?;
            self.record_denied_invocation(DeniedInvocationRecord {
                request: &request,
                tenant: &tenant,
                capability: &capability,
                disposition: RunDisposition::FailureBudget,
                evidence_kind: EvidenceKind::CapabilityInvocation,
                reason: "capability_cost_profile",
                audit_event_hash: topic_audit_hash,
                extra_fields: BTreeMap::from([
                    (
                        "cost_budget_audit_event_hash".to_string(),
                        cost_budget_audit_hash,
                    ),
                    (
                        "capability_invoke_audit_event_hash".to_string(),
                        capability_invoke_audit_hash,
                    ),
                    (
                        "capability_per_invocation_limit_micros".to_string(),
                        capability
                            .cost_profile()
                            .per_invocation_limit_micros
                            .value
                            .to_string(),
                    ),
                    (
                        "projected_cost_micros".to_string(),
                        request.projected_cost_micros.to_string(),
                    ),
                ]),
            })?;
            emit_invocation_trace(
                invocation_span.as_ref(),
                "denied",
                Some("capability_cost_profile"),
            );
            return Err(FoundationError::CostBudgetExceeded);
        }

        let budget_scope = BudgetScope::new(
            request.tenant_id.clone(),
            request.capability_id.clone(),
            request.budget_window_id.clone(),
        )
        .map_err(map_budget_error)?;
        let cost_budget_warning = match self
            .cost_budgets
            .evaluate(&budget_scope, request.projected_cost_micros)
        {
            Ok(decision) => decision.warning.value,
            Err(error) => {
                let cost_budget_audit_hash = self
                    .audit_chain
                    .append_classifications(
                        request.tenant_id.clone(),
                        "foundry.cost-budget.reserve",
                        Plane::Control,
                        request.purpose,
                        vec![DataClass::InternalOnly],
                        "DENY",
                    )?
                    .hash
                    .clone();
                let (capability_invoke_audit_hash, topic_audit_hash) =
                    self.append_invocation_denial_audits(&request, &capability)?;
                self.record_denied_invocation(DeniedInvocationRecord {
                    request: &request,
                    tenant: &tenant,
                    capability: &capability,
                    disposition: RunDisposition::FailureBudget,
                    evidence_kind: EvidenceKind::CapabilityInvocation,
                    reason: "budget",
                    audit_event_hash: topic_audit_hash,
                    extra_fields: BTreeMap::from([
                        (
                            "cost_budget_audit_event_hash".to_string(),
                            cost_budget_audit_hash,
                        ),
                        (
                            "capability_invoke_audit_event_hash".to_string(),
                            capability_invoke_audit_hash,
                        ),
                    ]),
                })?;
                emit_invocation_trace(invocation_span.as_ref(), "denied", Some("budget"));
                return Err(map_budget_error(error));
            }
        };
        let budget_snapshot = match self.cost_budgets.snapshot(&budget_scope) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                let cost_budget_audit_hash = self
                    .audit_chain
                    .append_classifications(
                        request.tenant_id.clone(),
                        "foundry.cost-budget.reserve",
                        Plane::Control,
                        request.purpose,
                        vec![DataClass::InternalOnly],
                        "DENY",
                    )?
                    .hash
                    .clone();
                let (capability_invoke_audit_hash, topic_audit_hash) =
                    self.append_invocation_denial_audits(&request, &capability)?;
                self.record_denied_invocation(DeniedInvocationRecord {
                    request: &request,
                    tenant: &tenant,
                    capability: &capability,
                    disposition: RunDisposition::FailureBudget,
                    evidence_kind: EvidenceKind::CapabilityInvocation,
                    reason: "budget_snapshot",
                    audit_event_hash: topic_audit_hash,
                    extra_fields: BTreeMap::from([
                        (
                            "cost_budget_audit_event_hash".to_string(),
                            cost_budget_audit_hash,
                        ),
                        (
                            "capability_invoke_audit_event_hash".to_string(),
                            capability_invoke_audit_hash,
                        ),
                    ]),
                })?;
                emit_invocation_trace(invocation_span.as_ref(), "denied", Some("budget_snapshot"));
                return Err(map_budget_error(error));
            }
        };
        let provider_route = match Self::resolve_foundation_local_provider_route(
            &tenant,
            &capability,
            &budget_snapshot,
            &request,
            capability.touched_privacy_data_classes(),
        ) {
            Ok(route) => route,
            Err(error) => {
                let provider_route_audit_hash = self
                    .audit_chain
                    .append_classifications(
                        request.tenant_id.clone(),
                        "foundry.provider.route",
                        Plane::Data,
                        request.purpose,
                        internal_audit_classifications(),
                        "DENY",
                    )?
                    .hash
                    .clone();
                let (capability_invoke_audit_hash, topic_audit_hash) =
                    self.append_invocation_denial_audits(&request, &capability)?;
                self.record_denied_invocation(DeniedInvocationRecord {
                    request: &request,
                    tenant: &tenant,
                    capability: &capability,
                    disposition: RunDisposition::FailureProvider,
                    evidence_kind: EvidenceKind::CapabilityInvocation,
                    reason: "provider_route",
                    audit_event_hash: topic_audit_hash,
                    extra_fields: BTreeMap::from([
                        (
                            "provider_route_audit_event_hash".to_string(),
                            provider_route_audit_hash,
                        ),
                        (
                            "capability_invoke_audit_event_hash".to_string(),
                            capability_invoke_audit_hash,
                        ),
                    ]),
                })?;
                emit_invocation_trace(invocation_span.as_ref(), "denied", Some("provider_route"));
                return Err(map_adapter_error(error));
            }
        };
        let provider_id = provider_route
            .primary()
            .map_err(map_adapter_error)?
            .id
            .value
            .value
            .clone();
        let provider_route_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.provider.route",
                Plane::Data,
                request.purpose,
                internal_audit_classifications(),
                "ALLOW",
            )?
            .hash
            .clone();
        let reservation = match self
            .cost_budgets
            .reserve(&budget_scope, request.projected_cost_micros)
        {
            Ok(reservation) => reservation,
            Err(error) => {
                let cost_budget_audit_hash = self
                    .audit_chain
                    .append_classifications(
                        request.tenant_id.clone(),
                        "foundry.cost-budget.reserve",
                        Plane::Control,
                        request.purpose,
                        vec![DataClass::InternalOnly],
                        "DENY",
                    )?
                    .hash
                    .clone();
                let (capability_invoke_audit_hash, topic_audit_hash) =
                    self.append_invocation_denial_audits(&request, &capability)?;
                self.record_denied_invocation(DeniedInvocationRecord {
                    request: &request,
                    tenant: &tenant,
                    capability: &capability,
                    disposition: RunDisposition::FailureBudget,
                    evidence_kind: EvidenceKind::CapabilityInvocation,
                    reason: "budget",
                    audit_event_hash: topic_audit_hash,
                    extra_fields: BTreeMap::from([
                        (
                            "cost_budget_audit_event_hash".to_string(),
                            cost_budget_audit_hash,
                        ),
                        (
                            "capability_invoke_audit_event_hash".to_string(),
                            capability_invoke_audit_hash,
                        ),
                    ]),
                })?;
                emit_invocation_trace(invocation_span.as_ref(), "denied", Some("budget_reserve"));
                return Err(map_budget_error(error));
            }
        };

        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.cost-budget.reserve",
            Plane::Control,
            request.purpose,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        let run = match self.foundry_runs.start(
            RunStart::new(
                request.tenant_id.clone(),
                request.capability_id.clone(),
                request.user_id.clone(),
                autonomy_decision.effective_ceiling,
                privacy_data_classes.clone(),
                tenant.home_region.value.clone(),
                reservation.reservation_id.value.clone(),
                request.started_at_epoch_seconds,
            )
            .map_err(map_run_error)?,
        ) {
            Ok(run) => run,
            Err(error) => {
                let primary_error = map_run_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    Some(&reservation.reservation_id.value),
                    None,
                    RunDisposition::FailureProvider,
                    primary_error,
                )?);
            }
        };
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.run.start",
            Plane::Data,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        let mut autonomy_evidence_fields =
            autonomy_decision_fields(&autonomy_decision, &autonomy_audit_hash);
        append_break_glass_evidence_fields(
            &mut autonomy_evidence_fields,
            autonomy_break_glass.as_ref(),
            if autonomy_break_glass.is_some() {
                Some(&pre_break_glass_autonomy_decision)
            } else {
                None
            },
            break_glass_invoke_audit_hash.as_deref(),
        );
        autonomy_evidence_fields.insert("run_id".to_string(), run.run_id.value.clone());
        autonomy_evidence_fields.insert(
            "evidence_topic".to_string(),
            capability.evidence_topic.value.clone(),
        );
        let autonomy_evidence = match self.foundry_evidence.append(
            request.tenant_id.clone(),
            run.run_id.value.clone(),
            None,
            request.capability_id.clone(),
            EvidenceKind::AutonomyDecision,
            autonomy_evidence_fields,
            privacy_data_classes.clone(),
            request.started_at_epoch_seconds,
        ) {
            Ok(evidence) => evidence,
            Err(error) => {
                let primary_error = map_evidence_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    Some(&reservation.reservation_id.value),
                    Some(&run.run_id.value),
                    RunDisposition::FailureProvider,
                    primary_error,
                )?);
            }
        };
        if let Err(error) = self.outbox.publish(
            request.tenant_id.clone(),
            capability.evidence_topic.value.clone(),
            autonomy_evidence.evidence_id.value.clone(),
            format!("foundry-evidence:{}", autonomy_evidence.evidence_id.value),
        ) {
            let primary_error = map_eventing_error(error);
            return Err(self.settle_failed_invocation(
                &request,
                Some(&reservation.reservation_id.value),
                Some(&run.run_id.value),
                RunDisposition::FailureProvider,
                primary_error,
            )?);
        }
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.evidence.topic.emit",
            Plane::Audit,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.evidence.emit",
            Plane::Audit,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        let step = match self.foundry_steps.start(
            StepStart::new(
                run.run_id.value.clone(),
                StepKind::ProviderCall,
                provider_id.clone(),
                Some(FOUNDATION_LOCAL_MODEL_REF.into()),
                None,
                None,
                privacy_data_classes.clone(),
                request.started_at_epoch_seconds,
            )
            .map_err(map_step_error)?,
        ) {
            Ok(step) => step,
            Err(error) => {
                let primary_error = map_step_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    Some(&reservation.reservation_id.value),
                    Some(&run.run_id.value),
                    RunDisposition::FailureProvider,
                    primary_error,
                )?);
            }
        };
        let completed_step = match self.foundry_steps.complete(
            &step.step_id.value,
            StepDisposition::Succeeded,
            1,
            request.started_at_epoch_seconds.saturating_add(1),
        ) {
            Ok(step) => step,
            Err(error) => {
                let primary_error = map_step_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    Some(&reservation.reservation_id.value),
                    Some(&run.run_id.value),
                    RunDisposition::FailureProvider,
                    primary_error,
                )?);
            }
        };
        let provider_call_idempotency_key = format!(
            "provider-call:{}:{}:{}:{:03}",
            run.run_id.value,
            completed_step.step_id.value,
            provider_id,
            FOUNDATION_LOCAL_PROVIDER_ATTEMPT
        );
        let provider_call_receipt = match ProviderCallReceipt::from_route(
            &provider_route,
            provider_call_idempotency_key,
            FOUNDATION_LOCAL_PROVIDER_ATTEMPT,
            FOUNDATION_LOCAL_MODEL_REF.into(),
            tenant.home_region.value.clone(),
        ) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.audit_chain.append_classifications(
                    request.tenant_id.clone(),
                    "foundry.provider.call",
                    Plane::Data,
                    request.purpose,
                    internal_audit_classifications(),
                    "DENY",
                )?;
                let primary_error = map_adapter_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    Some(&reservation.reservation_id.value),
                    Some(&run.run_id.value),
                    RunDisposition::FailureProvider,
                    primary_error,
                )?);
            }
        };
        let provider_call_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.provider.call",
                Plane::Data,
                request.purpose,
                data_classifications.clone(),
                "ALLOW",
            )?
            .hash
            .clone();
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.step.emit",
            Plane::Data,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        let committed = match self.cost_budgets.commit(&reservation.reservation_id.value) {
            Ok(committed) => committed,
            Err(error) => {
                let primary_error = map_budget_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    Some(&reservation.reservation_id.value),
                    Some(&run.run_id.value),
                    RunDisposition::FailureBudget,
                    primary_error,
                )?);
            }
        };
        let capability_invoke_event_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.capability.invoke",
                Plane::Data,
                request.purpose,
                data_classifications.clone(),
                "ALLOW",
            )?
            .hash
            .clone();
        if let Err(error) = self.foundry_runs.complete(
            &run.run_id.value,
            RunDisposition::Success,
            request.started_at_epoch_seconds.saturating_add(1),
        ) {
            let primary_error = map_run_error(error);
            return Err(self.settle_failed_invocation(
                &request,
                None,
                Some(&run.run_id.value),
                RunDisposition::FailureProvider,
                primary_error,
            )?);
        }
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.run.complete",
            Plane::Data,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        let evidence_event_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                capability.evidence_topic.value.clone(),
                Plane::Audit,
                request.purpose,
                data_classifications.clone(),
                "ALLOW",
            )?
            .hash
            .clone();
        let mut evidence_fields = BTreeMap::new();
        evidence_fields.insert("audit_event_hash".to_string(), evidence_event_hash.clone());
        evidence_fields.insert(
            "capability_invoke_audit_event_hash".to_string(),
            capability_invoke_event_hash,
        );
        evidence_fields.insert(
            "provider_route_audit_event_hash".to_string(),
            provider_route_audit_hash,
        );
        evidence_fields.insert(
            "provider_call_audit_event_hash".to_string(),
            provider_call_audit_hash,
        );
        evidence_fields.insert(
            "cost_reservation_id".to_string(),
            reservation.reservation_id.value.clone(),
        );
        evidence_fields.insert(
            "evidence_topic".to_string(),
            capability.evidence_topic.value.clone(),
        );
        evidence_fields.insert("run_id".to_string(), run.run_id.value.clone());
        evidence_fields.insert("step_id".to_string(), completed_step.step_id.value.clone());
        evidence_fields.insert(
            "provider_id".to_string(),
            provider_call_receipt.provider_id.value.value.clone(),
        );
        evidence_fields.insert(
            "provider_mode".to_string(),
            format!("{:?}", provider_call_receipt.provider_mode.value),
        );
        evidence_fields.insert(
            "provider_call_receipt_id".to_string(),
            provider_call_receipt.receipt_id.value.clone(),
        );
        evidence_fields.insert(
            "provider_call_idempotency_key".to_string(),
            provider_call_receipt.idempotency_key.value.clone(),
        );
        evidence_fields.insert(
            "provider_call_attempt".to_string(),
            provider_call_receipt.attempt.value.to_string(),
        );
        evidence_fields.insert(
            "provider_region".to_string(),
            provider_call_receipt.provider_region.value.clone(),
        );
        evidence_fields.insert(
            "provider_model_ref".to_string(),
            provider_call_receipt.model_ref.value.clone(),
        );
        evidence_fields.insert(
            "provider_projected_cost_micros".to_string(),
            provider_call_receipt
                .projected_cost_micros
                .value
                .to_string(),
        );
        evidence_fields.insert(
            "provider_p95_latency_ms".to_string(),
            provider_call_receipt.p95_latency_ms.value.to_string(),
        );
        let evidence = match self.foundry_evidence.append(
            request.tenant_id.clone(),
            run.run_id.value.clone(),
            Some(completed_step.step_id.value.clone()),
            request.capability_id.clone(),
            EvidenceKind::CapabilityInvocation,
            evidence_fields,
            privacy_data_classes,
            request.started_at_epoch_seconds.saturating_add(1),
        ) {
            Ok(evidence) => evidence,
            Err(error) => {
                let primary_error = map_evidence_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    None,
                    Some(&run.run_id.value),
                    RunDisposition::FailureProvider,
                    primary_error,
                )?);
            }
        };
        if let Err(error) = self.outbox.publish(
            request.tenant_id.clone(),
            capability.evidence_topic.value.clone(),
            evidence.evidence_id.value.clone(),
            format!("foundry-evidence:{}", evidence.evidence_id.value),
        ) {
            let primary_error = map_eventing_error(error);
            return Err(self.settle_failed_invocation(
                &request,
                None,
                Some(&run.run_id.value),
                RunDisposition::FailureProvider,
                primary_error,
            )?);
        }
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.evidence.topic.emit",
            Plane::Audit,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.evidence.emit",
            Plane::Audit,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        emit_invocation_trace(invocation_span.as_ref(), "succeeded", None);
        Ok(InvocationReceipt {
            tenant_id: request.tenant_id,
            user_id: request.user_id,
            capability_id: request.capability_id,
            evidence_event_hash,
            cost_reservation_id: Some(committed.reservation_id.value),
            cost_budget_warning,
            run_id: Some(run.run_id.value),
            foundry_step_id: Some(completed_step.step_id.value),
            foundry_evidence_id: Some(evidence.evidence_id.value),
        })
    }

    fn resolve_foundation_local_provider_route(
        tenant: &Tenant,
        capability: &Capability,
        budget_snapshot: &BudgetSnapshot,
        request: &CapabilityInvocationRequest,
        privacy_data_classes: &[PrivacyDataClass],
    ) -> Result<ProviderRoute, AdapterError> {
        let provider_id = ProviderId::new(FOUNDATION_LOCAL_PROVIDER_ID.into())?;
        let provider_profile = ProviderProfile::new_with_privacy_data_classes(
            provider_id.clone(),
            ProviderMode::Api,
            ProviderAuth::Api {
                secret_ref: SecretRef::new(
                    request.tenant_id.clone(),
                    request.capability_id.clone(),
                    FOUNDATION_LOCAL_SECRET_REF_NAME.into(),
                )
                .map_err(|_| AdapterError::MissingProviderCapability)?,
                billing_account: request.tenant_id.clone(),
            },
            privacy_data_classes.to_vec(),
            vec![tenant.home_region.value.clone()],
            request.projected_cost_micros,
            FOUNDATION_LOCAL_PROVIDER_P95_LATENCY_MS,
        )?;
        let profiles = [provider_profile];
        let subscription_bindings = SubscriptionBindingRegistry::default();
        let provider_preference = capability
            .provider_preference()
            .iter()
            .cloned()
            .map(ProviderId::new)
            .collect::<Result<Vec<_>, _>>()?;
        resolve_route(ProviderRouteRequest {
            capability,
            policy: InvocationPolicy::new_with_privacy_data_classes(
                Classified::new(request.tenant_id.clone(), DataClass::InternalOnly),
                privacy_data_classes.to_vec(),
                Classified::new(tenant.home_region.value.clone(), DataClass::InternalOnly),
                CostCeiling::from_budget_snapshot(budget_snapshot),
                10_000,
            ),
            preference: ProviderRoutePreference::ordered(provider_preference)?,
            profiles: &profiles,
            subscription_bindings: &subscription_bindings,
        })
    }

    // ADR-0083 amendment 2026-05-15: `settle_failed_invocation` returns
    // `Result<FoundationError, FoundationError>` so the 3 internal
    // `append_classifications` sites can propagate `AuditChainError` via `?`.
    // `Ok(primary_error)` carries the original failure for the outer caller to
    // return as `Err(primary_error)`; `Err(audit_chain_error)` supersedes the
    // primary error when the audit chain itself fails — ADR-0083 Tier 1
    // forbids silently dropping `AuditChainError`.
    fn settle_failed_invocation(
        &mut self,
        request: &CapabilityInvocationRequest,
        reservation_id: Option<&str>,
        run_id: Option<&str>,
        disposition: RunDisposition,
        primary_error: FoundationError,
    ) -> Result<FoundationError, FoundationError> {
        let budget_release = if let Some(reservation_id) = reservation_id {
            if self.cost_budgets.release(reservation_id).is_ok() {
                self.audit_chain.append_classifications(
                    request.tenant_id.clone(),
                    "foundry.cost-budget.release",
                    Plane::Control,
                    request.purpose,
                    vec![DataClass::InternalOnly],
                    "ALLOW",
                )?;
                InvocationSettlementStatus::Completed
            } else {
                self.audit_chain.append_classifications(
                    request.tenant_id.clone(),
                    "foundry.cost-budget.release",
                    Plane::Control,
                    request.purpose,
                    vec![DataClass::InternalOnly],
                    "DENY",
                )?;
                InvocationSettlementStatus::Failed
            }
        } else {
            InvocationSettlementStatus::NotApplicable
        };
        let run_completion = if let Some(run_id) = run_id {
            if self
                .foundry_runs
                .complete(
                    run_id,
                    disposition,
                    request.started_at_epoch_seconds.saturating_add(1),
                )
                .is_err()
            {
                self.audit_chain.append_classifications(
                    request.tenant_id.clone(),
                    "foundry.run.complete",
                    Plane::Data,
                    request.purpose,
                    audit_classifications(),
                    "DENY",
                )?;
                InvocationSettlementStatus::Failed
            } else {
                InvocationSettlementStatus::Completed
            }
        } else {
            InvocationSettlementStatus::NotApplicable
        };
        self.record_invocation_compensation(
            request,
            reservation_id,
            run_id,
            disposition,
            &primary_error,
            budget_release,
            run_completion,
        )?;
        Ok(primary_error)
    }

    // ADR-0083 amendment 2026-05-15: `record_invocation_compensation` returns
    // `Result<(), FoundationError>` so the 6 internal `append_classifications`
    // sites can propagate `AuditChainError` via `?`. Caller
    // (`settle_failed_invocation`) re-propagates so the outermost invocation
    // path surfaces audit-chain failure rather than silently swallowing it.
    #[allow(clippy::too_many_arguments)]
    fn record_invocation_compensation(
        &mut self,
        request: &CapabilityInvocationRequest,
        reservation_id: Option<&str>,
        run_id: Option<&str>,
        disposition: RunDisposition,
        primary_error: &FoundationError,
        budget_release: InvocationSettlementStatus,
        run_completion: InvocationSettlementStatus,
    ) -> Result<(), FoundationError> {
        let compensation_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.invocation.compensate",
                Plane::Audit,
                request.purpose,
                audit_classifications(),
                "ALLOW",
            )?
            .hash
            .clone();
        let Some(run_id) = run_id else {
            return Ok(());
        };
        let Some(capability) = self.capabilities.get(&request.capability_id).cloned() else {
            self.audit_chain.append_classifications(
                request.tenant_id.clone(),
                "foundry.invocation.compensate",
                Plane::Audit,
                request.purpose,
                audit_classifications(),
                "DENY",
            )?;
            return Ok(());
        };
        let mut evidence_fields = BTreeMap::from([
            (
                "audit_event_hash".to_string(),
                compensation_audit_hash.clone(),
            ),
            (
                "budget_release".to_string(),
                budget_release.as_release_str().to_string(),
            ),
            ("decision".to_string(), "FAIL".to_string()),
            ("disposition".to_string(), format!("{disposition:?}")),
            (
                "evidence_topic".to_string(),
                capability.evidence_topic.value.clone(),
            ),
            ("primary_error".to_string(), format!("{primary_error:?}")),
            ("reason".to_string(), "invocation_compensation".to_string()),
            (
                "run_completion".to_string(),
                run_completion.as_completion_str().to_string(),
            ),
            ("run_id".to_string(), run_id.to_string()),
        ]);
        if let Some(reservation_id) = reservation_id {
            evidence_fields.insert(
                "cost_reservation_id".to_string(),
                reservation_id.to_string(),
            );
        }
        let evidence = match self.foundry_evidence.append(
            request.tenant_id.clone(),
            run_id.to_string(),
            None,
            request.capability_id.clone(),
            EvidenceKind::CapabilityInvocation,
            evidence_fields,
            capability.touched_privacy_data_classes().to_vec(),
            request.started_at_epoch_seconds.saturating_add(1),
        ) {
            Ok(evidence) => evidence,
            Err(_) => {
                self.audit_chain.append_classifications(
                    request.tenant_id.clone(),
                    "foundry.invocation.compensate",
                    Plane::Audit,
                    request.purpose,
                    audit_classifications(),
                    "DENY",
                )?;
                return Ok(());
            }
        };
        if self
            .outbox
            .publish(
                request.tenant_id.clone(),
                capability.evidence_topic.value,
                evidence.evidence_id.value.clone(),
                format!("foundry-evidence:{}", evidence.evidence_id.value),
            )
            .is_err()
        {
            self.audit_chain.append_classifications(
                request.tenant_id.clone(),
                "foundry.invocation.compensate.outbox",
                Plane::Audit,
                request.purpose,
                audit_classifications(),
                "DENY",
            )?;
            return Ok(());
        }
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.evidence.topic.emit",
            Plane::Audit,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.evidence.emit",
            Plane::Audit,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        Ok(())
    }

    fn append_invocation_denial_audits(
        &mut self,
        request: &CapabilityInvocationRequest,
        capability: &Capability,
    ) -> Result<(String, String), FoundationError> {
        // ADR-0083 amendment 2026-05-15: `append_classifications` is Tier 1
        // fallible; this helper propagates `AuditChainError` to the caller
        // via `FoundationError::AuditChainAppendFailed`.
        let capability_invoke_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.capability.invoke",
                Plane::Data,
                request.purpose,
                capability_record_classifications(capability),
                "DENY",
            )?
            .hash
            .clone();
        let topic_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                capability.evidence_topic.value.clone(),
                Plane::Audit,
                request.purpose,
                capability_record_classifications(capability),
                "DENY",
            )?
            .hash
            .clone();
        Ok((capability_invoke_audit_hash, topic_audit_hash))
    }

    fn record_denied_invocation(
        &mut self,
        denial: DeniedInvocationRecord<'_>,
    ) -> Result<EvidenceRecord, FoundationError> {
        let privacy_data_classes = denial.capability.touched_privacy_data_classes().to_vec();
        let run = self
            .foundry_runs
            .reject(
                RunStart::new(
                    denial.request.tenant_id.clone(),
                    denial.request.capability_id.clone(),
                    denial.request.user_id.clone(),
                    denial.capability.required_tier,
                    privacy_data_classes.clone(),
                    denial.tenant.home_region.value.clone(),
                    format!(
                        "deny:{}:{}:{}",
                        denial.reason,
                        denial.request.capability_id,
                        denial.request.started_at_epoch_seconds
                    ),
                    denial.request.started_at_epoch_seconds,
                )
                .map_err(map_run_error)?,
                denial.disposition,
            )
            .map_err(map_run_error)?;
        let run_reject_audit_hash = self
            .audit_chain
            .append_classifications(
                denial.request.tenant_id.clone(),
                "foundry.run.reject",
                Plane::Data,
                denial.request.purpose,
                audit_classifications(),
                "ALLOW",
            )?
            .hash
            .clone();
        let mut evidence_fields = BTreeMap::from([
            (
                "audit_event_hash".to_string(),
                denial.audit_event_hash.clone(),
            ),
            ("decision".to_string(), "DENY".to_string()),
            (
                "evidence_topic".to_string(),
                denial.capability.evidence_topic.value.clone(),
            ),
            ("reason".to_string(), denial.reason.to_string()),
            ("run_id".to_string(), run.run_id.value.clone()),
            (
                "run_reject_audit_event_hash".to_string(),
                run_reject_audit_hash,
            ),
        ]);
        evidence_fields.extend(denial.extra_fields);
        let evidence = self
            .foundry_evidence
            .append(
                denial.request.tenant_id.clone(),
                run.run_id.value,
                None,
                denial.request.capability_id.clone(),
                denial.evidence_kind,
                evidence_fields,
                privacy_data_classes,
                denial.request.started_at_epoch_seconds,
            )
            .map_err(map_evidence_error)?;
        self.outbox
            .publish(
                denial.request.tenant_id.clone(),
                denial.capability.evidence_topic.value.clone(),
                evidence.evidence_id.value.clone(),
                format!("foundry-evidence:{}", evidence.evidence_id.value),
            )
            .map_err(map_eventing_error)?;
        self.audit_chain.append_classifications(
            denial.request.tenant_id.clone(),
            "foundry.evidence.topic.emit",
            Plane::Audit,
            denial.request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        self.audit_chain.append_classifications(
            denial.request.tenant_id.clone(),
            "foundry.evidence.emit",
            Plane::Audit,
            denial.request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        Ok(evidence)
    }

    pub fn register_regional_pack(
        &mut self,
        registration: RegionalPackRegistration,
    ) -> Result<RegionalPack, FoundationError> {
        if self.regional_packs.contains_key(&registration.pack_id) {
            return Err(FoundationError::RegionalPackAlreadyExists);
        }
        let pack = RegionalPack::new(
            registration.pack_id,
            registration.region,
            registration.residency_class,
            registration.controls,
        )
        .map_err(map_regional_pack_error)?;
        self.regional_packs.insert(pack.id.clone(), pack.clone());
        self.audit_chain.append_classifications(
            "ten_system",
            "regulatory-pack.bind",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(pack)
    }

    pub fn upsert_object_entity(
        &mut self,
        upsert: ObjectEntityUpsert,
    ) -> Result<ObjectEntity, FoundationError> {
        self.require_tenant(&upsert.tenant_id)?;
        let properties = upsert
            .properties
            .into_iter()
            .map(|input| {
                ObjectProperty::new_with_privacy_data_class(
                    input.name,
                    input.value,
                    input.tier,
                    input.privacy_data_class,
                )
            })
            .collect::<Vec<_>>();
        let entity = ObjectEntity::new(
            upsert.tenant_id.clone(),
            upsert.entity_id,
            upsert.entity_type,
            properties,
        )
        .map_err(map_object_graph_error)?;
        self.object_entities.insert(
            (upsert.tenant_id.clone(), entity.id.clone()),
            entity.clone(),
        );
        self.audit_chain.append_classifications(
            upsert.tenant_id,
            "object-graph.entity.upsert",
            Plane::Data,
            Purpose::CoreService,
            entity
                .properties
                .values()
                .map(|property| property.value.data_class.compatibility_data_class())
                .collect::<Vec<_>>(),
            "ALLOW",
        )?;
        Ok(entity)
    }

    pub fn publish_outbox(
        &mut self,
        publish: OutboxPublish,
    ) -> Result<OutboxRecord, FoundationError> {
        self.require_tenant(&publish.tenant_id)?;
        let record = self
            .outbox
            .publish(
                publish.tenant_id.clone(),
                publish.topic,
                publish.idempotency_key,
                publish.payload_ref,
            )
            .map_err(map_eventing_error)?;
        self.audit_chain.append_classifications(
            publish.tenant_id,
            "eventing.outbox.publish",
            Plane::Data,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(record)
    }

    pub fn mark_outbox_published(
        &mut self,
        tenant_id: &str,
        sequence: u64,
    ) -> Result<OutboxRecord, FoundationError> {
        self.require_tenant(tenant_id)?;
        match self.outbox.mark_published(tenant_id, sequence) {
            Ok(record) => {
                self.audit_chain.append_classifications(
                    tenant_id,
                    "eventing.outbox.mark-published",
                    Plane::Data,
                    Purpose::CoreService,
                    vec![DataClass::InternalOnly],
                    "ALLOW",
                )?;
                Ok(record)
            }
            Err(EventingError::OutboxRecordNotFound) => {
                self.audit_chain.append_classifications(
                    tenant_id,
                    "eventing.outbox.mark-published",
                    Plane::Data,
                    Purpose::CoreService,
                    vec![DataClass::InternalOnly],
                    "DENY",
                )?;
                Err(FoundationError::OutboxRecordNotFound)
            }
            Err(error) => Err(map_eventing_error(error)),
        }
    }

    pub fn outbox_records(&self) -> &[OutboxRecord] {
        self.outbox.records()
    }

    pub fn foundry_runs(&self) -> &[Run] {
        self.foundry_runs.runs()
    }

    pub fn foundry_steps(&self) -> &[Step] {
        self.foundry_steps.steps()
    }

    pub fn foundry_evidence_chain(&self) -> &EvidenceChain {
        &self.foundry_evidence
    }

    pub fn audit_chain(&self) -> &AuditChain {
        &self.audit_chain
    }

    fn require_tenant(&self, tenant_id: &str) -> Result<&Tenant, FoundationError> {
        self.tenants
            .get(tenant_id)
            .ok_or(FoundationError::TenantNotFound)
    }

    fn require_user(&self, tenant_id: &str, user_id: &str) -> Result<&User, FoundationError> {
        self.require_tenant(tenant_id)?;
        self.users
            .get(&(tenant_id.to_string(), user_id.to_string()))
            .ok_or(FoundationError::UserNotFound)
    }

    fn mcp_principal(
        &self,
        endpoint: &McpTenantEndpoint,
        access_token: McpAccessTokenClaims,
        now_epoch_seconds: u64,
    ) -> Result<McpPrincipal, FoundationError> {
        let policy = self
            .tenant_policies
            .get(&endpoint.tenant_id.value)
            .ok_or(FoundationError::TenantNotFound)?;
        validate_access_token(
            endpoint,
            access_token,
            now_epoch_seconds,
            policy.autonomy_ceiling,
        )
        .map_err(map_mcp_error)
    }
}

fn map_tenant_error(error: TenantError) -> FoundationError {
    match error {
        TenantError::InvalidTenantId
        | TenantError::EmptyLegalName
        | TenantError::EmptyHomeRegion
        | TenantError::HomeRegionNotAllowedForResidency
        | TenantError::MissingRegionalPack => FoundationError::InvalidInput,
    }
}

fn map_identity_error(error: IdentityError) -> FoundationError {
    match error {
        IdentityError::TokenTtlTooLong => FoundationError::TokenTtlTooLong,
        IdentityError::InvalidTenantId
        | IdentityError::InvalidUserId
        | IdentityError::InvalidRegionPack
        | IdentityError::InvalidIdentityProviderId
        | IdentityError::InvalidServicePrincipalId
        | IdentityError::InvalidCapabilityId
        | IdentityError::EmptyPrimaryIdentifier
        | IdentityError::EmptyExternalSubject
        | IdentityError::TokenTtlZero
        | IdentityError::MissingCredentialScope
        | IdentityError::LongLivedCredentialForbidden => FoundationError::InvalidInput,
    }
}

fn map_capability_error(error: CapabilityError) -> FoundationError {
    match error {
        CapabilityError::InvalidCapabilityId
        | CapabilityError::InvalidTenantId
        | CapabilityError::EmptyNamespace
        | CapabilityError::EmptyEvidenceTopic
        | CapabilityError::MissingDataClasses
        | CapabilityError::NonPrivacyDataClass
        | CapabilityError::InvalidCostProfile
        | CapabilityError::MissingProviderPreference
        | CapabilityError::InvalidProviderPreference
        | CapabilityError::InvalidMcpContract => FoundationError::InvalidInput,
        CapabilityError::DuplicateCapability => FoundationError::CapabilityAlreadyExists,
        CapabilityError::CapabilityNotFound => FoundationError::CapabilityNotFound,
    }
}

fn map_eval_error(error: EvalError) -> FoundationError {
    match error {
        EvalError::EvalSetNotFound
        | EvalError::MissingPassingEvalRun
        | EvalError::UnsignedEvalSet
        | EvalError::MissingAdversarialCoverage
        | EvalError::MissingLinguisticCoverage
        | EvalError::UnsignedEvalRun
        | EvalError::EvalRunVersionMismatch
        | EvalError::EvalRunBelowThreshold => FoundationError::CapabilityEvalGateNotReady,
        EvalError::InvalidCapabilityId
        | EvalError::EmptyVersion
        | EvalError::EmptyCaseId
        | EvalError::EmptyLocale
        | EvalError::EmptyInputRef
        | EvalError::EmptyExpectedRef
        | EvalError::InvalidThreshold
        | EvalError::EmptyEvalSet => FoundationError::InvalidInput,
    }
}

fn map_mcp_error(error: McpGatewayError) -> FoundationError {
    match error {
        McpGatewayError::TenantMismatch | McpGatewayError::MissingScope => {
            FoundationError::McpAccessDenied
        }
        McpGatewayError::TokenAudienceMismatch
        | McpGatewayError::TokenIssuerMismatch
        | McpGatewayError::TokenExpired => FoundationError::McpAccessDenied,
        McpGatewayError::RateLimitExceeded => FoundationError::McpRateLimited,
        McpGatewayError::AutonomyCeilingExceeded => FoundationError::AutonomyCeilingExceeded,
        McpGatewayError::InvalidTenantId
        | McpGatewayError::EmptySubjectId
        | McpGatewayError::EmptyRegion
        | McpGatewayError::EmptyTld
        | McpGatewayError::InvalidHostSegment
        | McpGatewayError::EmptyAuthorizationServer
        | McpGatewayError::InvalidAuthorizationServer
        | McpGatewayError::InvalidRateLimitPolicy
        | McpGatewayError::EmptyToolName
        | McpGatewayError::InvalidToolName
        | McpGatewayError::ToolNameTooLong => FoundationError::InvalidInput,
    }
}

fn map_budget_error(error: BudgetError) -> FoundationError {
    match error {
        BudgetError::MissingBudgetCeiling => FoundationError::CostBudgetNotConfigured,
        BudgetError::PerInvocationLimitExceeded
        | BudgetError::TenantMonthlyLimitExceeded
        | BudgetError::CapabilityMonthlyLimitExceeded => FoundationError::CostBudgetExceeded,
        BudgetError::InvalidTenantId
        | BudgetError::InvalidCapabilityId
        | BudgetError::InvalidWindowId
        | BudgetError::InvalidBudgetCeiling
        | BudgetError::NonPositiveAmount
        | BudgetError::ReservationNotFound
        | BudgetError::ReservationNotPending => FoundationError::InvalidInput,
    }
}

fn map_bypass_error(_error: BypassError) -> FoundationError {
    FoundationError::InvalidInput
}

fn map_run_error(error: RunError) -> FoundationError {
    match error {
        RunError::InvalidTenantId
        | RunError::InvalidCapabilityId
        | RunError::InvalidInitiatorId
        | RunError::InvalidRunHistory
        | RunError::MissingDataClasses
        | RunError::InvalidDataClass
        | RunError::EmptyRegion
        | RunError::EmptyIdempotencyKey
        | RunError::RunNotFound
        | RunError::RunNotRunning => FoundationError::InvalidInput,
    }
}

fn map_step_error(error: StepError) -> FoundationError {
    match error {
        StepError::InvalidRunId
        | StepError::InvalidStepHistory
        | StepError::EmptyProviderKind
        | StepError::EmptyModelRef
        | StepError::MissingDataClasses
        | StepError::InvalidDataClass
        | StepError::StepNotFound
        | StepError::StepNotRunning => FoundationError::InvalidInput,
    }
}

fn map_adapter_error(error: AdapterError) -> FoundationError {
    match error {
        AdapterError::DataClassNotAllowed => FoundationError::DataUseNotAllowed,
        AdapterError::InvalidCostCeiling | AdapterError::NoProviderAvailable => {
            FoundationError::CostBudgetExceeded
        }
        AdapterError::InvalidProviderId
        | AdapterError::InvalidTenantId
        | AdapterError::EmptyProviderAccount
        | AdapterError::EmptyFailoverChain
        | AdapterError::MissingDataClassAllowlist
        | AdapterError::MissingProviderRegion
        | AdapterError::MissingProviderCapability
        | AdapterError::AuthModeMismatch
        | AdapterError::InvalidRequiredRegion
        | AdapterError::EmptyProviderCallIdempotencyKey
        | AdapterError::EmptyProviderModelRef
        | AdapterError::InvalidProviderCallAttempt
        | AdapterError::EmptyProviderRequestId
        | AdapterError::EmptyProviderPromptRef
        | AdapterError::EmptyProviderToolName
        | AdapterError::ProviderAdapterMismatch
        | AdapterError::InvalidProviderEventSequence
        | AdapterError::ProviderRetryableFailure
        | AdapterError::ProviderNonRetryableFailure
        | AdapterError::ProviderCallRegionMismatch
        | AdapterError::InvalidDataClass => FoundationError::InvalidInput,
    }
}

fn map_evidence_error(error: EvidenceError) -> FoundationError {
    match error {
        EvidenceError::InvalidEvidenceId
        | EvidenceError::InvalidTenantId
        | EvidenceError::InvalidRunId
        | EvidenceError::InvalidStepId
        | EvidenceError::InvalidCapabilityId
        | EvidenceError::EmptyFields
        | EvidenceError::MissingDataClasses
        | EvidenceError::InvalidDataClass => FoundationError::InvalidInput,
    }
}

fn map_regional_pack_error(error: RegionalPackError) -> FoundationError {
    match error {
        RegionalPackError::InvalidPackId
        | RegionalPackError::EmptyRegion
        | RegionalPackError::EmptyResidencyClass
        | RegionalPackError::InvalidResidencyClass
        | RegionalPackError::MissingControls => FoundationError::InvalidInput,
    }
}

fn map_object_graph_error(error: ObjectGraphError) -> FoundationError {
    match error {
        ObjectGraphError::InvalidEntityId
        | ObjectGraphError::EmptyEntityType
        | ObjectGraphError::MissingProperties
        | ObjectGraphError::EmptyPropertyName
        | ObjectGraphError::InvalidDataClass => FoundationError::InvalidInput,
    }
}

fn map_eventing_error(error: EventingError) -> FoundationError {
    match error {
        EventingError::EmptyTopic
        | EventingError::EmptyTopicAxis
        | EventingError::EmptyTopicDescription
        | EventingError::InvalidTopicName
        | EventingError::DuplicateTopic
        | EventingError::TopicNotFound
        | EventingError::EmptyIdempotencyKey
        | EventingError::EmptyPayloadRef
        | EventingError::IdempotencyReplayMismatch
        | EventingError::InvalidOutboxHistory => FoundationError::InvalidInput,
        EventingError::OutboxRecordNotFound => FoundationError::OutboxRecordNotFound,
    }
}

fn emit_invocation_trace(
    span: &dyn CapabilityInvocationTraceSpan,
    result: &'static str,
    error_type: Option<&'static str>,
) {
    span.emit_result(InvocationTraceResult { result, error_type });
}

fn autonomy_tier_label(tier: AutonomyTier) -> &'static str {
    match tier {
        AutonomyTier::T1ViewOnly => "T1",
        AutonomyTier::T2Advisory => "T2",
        AutonomyTier::T3ExecuteWithApproval => "T3",
        AutonomyTier::T4AutoExecute => "T4",
    }
}

fn epoch_seconds_to_epoch_days(seconds: u64) -> u64 {
    seconds / 86_400
}

fn apply_autonomy_break_glass(
    autonomy_decision: &mut AutonomyDecision,
    break_glass: &AutonomyBreakGlass,
) {
    autonomy_decision.denial_threshold = break_glass.permitted_tier.value;
    autonomy_decision.effective_ceiling = autonomy_decision.required_tier;
    autonomy_decision.verdict = AutonomyVerdict::Allow;
    autonomy_decision.blocking_cap_source = None;
    autonomy_decision.blocking_cap_reason = None;
    autonomy_decision.lowering_cap_source = AutonomyCapSource::CapabilityRequired;
    autonomy_decision.lowering_cap_reason = AutonomyCapReason::CapabilityRequiredTier;
}

fn autonomy_decision_label(autonomy_decision: &AutonomyDecision) -> &'static str {
    if autonomy_decision.allowed() {
        "ALLOW"
    } else {
        "DENY"
    }
}

fn evaluate_invocation_data_use(
    capability: &Capability,
    request: &CapabilityInvocationRequest,
    consent_scope: Option<&ConsentScope>,
) -> Result<Purpose, InvocationDataUseDenial> {
    let effective_purpose = effective_invocation_purpose(capability, request.purpose)?;
    for privacy_data_class in capability.touched_privacy_data_classes() {
        let data_class = privacy_data_class.data_class();
        if let Err(reason) = evaluate_data_use(DataUseAttributes {
            purpose: effective_purpose,
            data_classification: DataClassification::from(*privacy_data_class),
            subject_class: request.subject_class,
        }) {
            return Err(InvocationDataUseDenial {
                effective_purpose,
                denied_data_class: Some(data_class),
                reason: data_use_denial_reason_label(reason),
            });
        }
        if data_class != DataClass::InternalOnly
            && !consent_scope.is_some_and(|scope| {
                scope.allows_privacy_data_class(effective_purpose, *privacy_data_class)
            })
        {
            return Err(InvocationDataUseDenial {
                effective_purpose,
                denied_data_class: Some(data_class),
                reason: "missing_purpose_bound_data_use_grant",
            });
        }
    }
    Ok(effective_purpose)
}

fn effective_invocation_purpose(
    capability: &Capability,
    requested_purpose: Purpose,
) -> Result<Purpose, InvocationDataUseDenial> {
    if matches!(
        capability.action,
        CapabilityAction::AdsBid | CapabilityAction::AdsBudgetAdjust
    ) {
        if requested_purpose != Purpose::AdsTargeting {
            return Err(InvocationDataUseDenial {
                effective_purpose: Purpose::AdsTargeting,
                denied_data_class: None,
                reason: "underdeclared_ads_purpose",
            });
        }
        return Ok(Purpose::AdsTargeting);
    }
    Ok(requested_purpose)
}

fn data_use_denial_reason_label(reason: DataUseDenialReason) -> &'static str {
    match reason {
        DataUseDenialReason::HardDeniedDataClass => "hard_denied_data_class",
        DataUseDenialReason::MinorSubjectAds => "minor_subject_ads",
    }
}

fn data_use_denial_fields(
    request: &CapabilityInvocationRequest,
    capability: &Capability,
    denial: &InvocationDataUseDenial,
    capability_invoke_audit_hash: String,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "capability_invoke_audit_event_hash".to_string(),
            capability_invoke_audit_hash,
        ),
        (
            "data_use_denial_reason".to_string(),
            denial.reason.to_string(),
        ),
        (
            "consent_result".to_string(),
            if denial.reason == "missing_purpose_bound_data_use_grant" {
                "missing"
            } else {
                "not_evaluated"
            }
            .to_string(),
        ),
        (
            "requested_purpose".to_string(),
            format!("{:?}", request.purpose),
        ),
        (
            "effective_purpose".to_string(),
            format!("{:?}", denial.effective_purpose),
        ),
        (
            "subject_class".to_string(),
            format!("{:?}", request.subject_class),
        ),
        (
            "denied_data_class".to_string(),
            denial
                .denied_data_class
                .map(|data_class| data_class.label().to_string())
                .unwrap_or_else(|| "none".to_string()),
        ),
        (
            "data_classes".to_string(),
            capability_record_data_class_labels(capability),
        ),
    ])
}

fn invocation_authorization_attributes(
    request: &CapabilityInvocationRequest,
    capability: &Capability,
    principal_autonomy_ceiling: AutonomyTier,
    autonomy_decision: &AutonomyDecision,
    break_glass: Option<&AutonomyBreakGlass>,
    pre_break_glass_decision: Option<&AutonomyDecision>,
) -> BTreeMap<String, String> {
    let mut attributes = BTreeMap::from([
        ("tenant_id".to_string(), request.tenant_id.clone()),
        ("purpose".to_string(), format!("{:?}", request.purpose)),
        (
            "subject_class".to_string(),
            format!("{:?}", request.subject_class),
        ),
        (
            "required_tier".to_string(),
            format!("{:?}", capability.required_tier),
        ),
        (
            "principal_autonomy_ceiling".to_string(),
            format!("{principal_autonomy_ceiling:?}"),
        ),
        (
            "tenant_configured_ceiling".to_string(),
            format!("{:?}", autonomy_decision.tenant_configured_ceiling),
        ),
        (
            "principal_ceiling".to_string(),
            format!("{:?}", autonomy_decision.principal_ceiling),
        ),
        (
            "capability_required_cap".to_string(),
            format!("{:?}", autonomy_decision.capability_required_cap),
        ),
        (
            "agentic_ads_cap".to_string(),
            format!("{:?}", autonomy_decision.agentic_ads_cap),
        ),
        (
            "vertical_pack_cap".to_string(),
            format!("{:?}", autonomy_decision.vertical_pack_cap),
        ),
        (
            "subject_class_cap".to_string(),
            format!("{:?}", autonomy_decision.subject_class_cap),
        ),
        (
            "denial_threshold".to_string(),
            format!("{:?}", autonomy_decision.denial_threshold),
        ),
        (
            "effective_ceiling".to_string(),
            format!("{:?}", autonomy_decision.effective_ceiling),
        ),
        (
            "autonomy_verdict".to_string(),
            format!("{:?}", autonomy_decision.verdict),
        ),
        (
            "blocking_cap_source".to_string(),
            autonomy_decision
                .blocking_cap_source
                .map(|source| source.as_str())
                .unwrap_or("none")
                .to_string(),
        ),
        (
            "blocking_cap_reason".to_string(),
            autonomy_decision
                .blocking_cap_reason
                .map(|reason| reason.as_str())
                .unwrap_or("none")
                .to_string(),
        ),
        (
            "lowering_cap_source".to_string(),
            autonomy_decision.lowering_cap_source.as_str().to_string(),
        ),
        (
            "lowering_cap_reason".to_string(),
            autonomy_decision.lowering_cap_reason.as_str().to_string(),
        ),
        (
            "data_classes".to_string(),
            capability_record_data_class_labels(capability),
        ),
    ]);
    append_break_glass_authorization_attributes(
        &mut attributes,
        break_glass,
        pre_break_glass_decision,
    );
    attributes
}

fn append_break_glass_authorization_attributes(
    fields: &mut BTreeMap<String, String>,
    break_glass: Option<&AutonomyBreakGlass>,
    pre_break_glass_decision: Option<&AutonomyDecision>,
) {
    fields.insert(
        "break_glass_applied".to_string(),
        break_glass.is_some().to_string(),
    );
    if let Some(break_glass) = break_glass {
        fields.insert("break_glass_id".to_string(), break_glass.id.value.clone());
        fields.insert(
            "break_glass_requested_tier".to_string(),
            format!("{:?}", break_glass.requested_tier.value),
        );
        fields.insert(
            "break_glass_permitted_tier".to_string(),
            format!("{:?}", break_glass.permitted_tier.value),
        );
        fields.insert(
            "break_glass_approval_quorum".to_string(),
            format!("{:?}", break_glass.approval_quorum.value),
        );
        fields.insert(
            "break_glass_expires_at_epoch_days".to_string(),
            break_glass.expires_at_epoch_days.value.to_string(),
        );
    }
    if let Some(pre_break_glass_decision) = pre_break_glass_decision {
        fields.insert(
            "pre_break_glass_decision".to_string(),
            autonomy_decision_label(pre_break_glass_decision).to_string(),
        );
        fields.insert(
            "pre_break_glass_effective_ceiling".to_string(),
            format!("{:?}", pre_break_glass_decision.effective_ceiling),
        );
        fields.insert(
            "pre_break_glass_denial_threshold".to_string(),
            format!("{:?}", pre_break_glass_decision.denial_threshold),
        );
        fields.insert(
            "pre_break_glass_blocking_cap_source".to_string(),
            pre_break_glass_decision
                .blocking_cap_source
                .map(|source| source.as_str())
                .unwrap_or("none")
                .to_string(),
        );
        fields.insert(
            "pre_break_glass_blocking_cap_reason".to_string(),
            pre_break_glass_decision
                .blocking_cap_reason
                .map(|reason| reason.as_str())
                .unwrap_or("none")
                .to_string(),
        );
    }
}

fn append_break_glass_evidence_fields(
    fields: &mut BTreeMap<String, String>,
    break_glass: Option<&AutonomyBreakGlass>,
    pre_break_glass_decision: Option<&AutonomyDecision>,
    break_glass_invoke_audit_hash: Option<&str>,
) {
    append_break_glass_authorization_attributes(fields, break_glass, pre_break_glass_decision);
    if let Some(break_glass_invoke_audit_hash) = break_glass_invoke_audit_hash {
        fields.insert(
            "break_glass_invoke_audit_event_hash".to_string(),
            break_glass_invoke_audit_hash.to_string(),
        );
    }
}

fn autonomy_decision_fields(
    autonomy_decision: &AutonomyDecision,
    autonomy_audit_hash: &str,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "audit_event_hash".to_string(),
            autonomy_audit_hash.to_string(),
        ),
        (
            "autonomy_audit_event_hash".to_string(),
            autonomy_audit_hash.to_string(),
        ),
        ("tenant_id".to_string(), autonomy_decision.tenant_id.clone()),
        (
            "capability_id".to_string(),
            autonomy_decision.capability_id.clone(),
        ),
        (
            "configured_ceiling".to_string(),
            format!("{:?}", autonomy_decision.configured_ceiling),
        ),
        (
            "tenant_configured_ceiling".to_string(),
            format!("{:?}", autonomy_decision.tenant_configured_ceiling),
        ),
        (
            "principal_ceiling".to_string(),
            format!("{:?}", autonomy_decision.principal_ceiling),
        ),
        (
            "capability_required_cap".to_string(),
            format!("{:?}", autonomy_decision.capability_required_cap),
        ),
        (
            "agentic_ads_cap".to_string(),
            format!("{:?}", autonomy_decision.agentic_ads_cap),
        ),
        (
            "vertical_pack_cap".to_string(),
            format!("{:?}", autonomy_decision.vertical_pack_cap),
        ),
        (
            "subject_class".to_string(),
            format!("{:?}", autonomy_decision.subject_class),
        ),
        (
            "subject_class_cap".to_string(),
            format!("{:?}", autonomy_decision.subject_class_cap),
        ),
        (
            "denial_threshold".to_string(),
            format!("{:?}", autonomy_decision.denial_threshold),
        ),
        (
            "effective_ceiling".to_string(),
            format!("{:?}", autonomy_decision.effective_ceiling),
        ),
        (
            "required_tier".to_string(),
            format!("{:?}", autonomy_decision.required_tier),
        ),
        (
            "decision".to_string(),
            autonomy_decision_label(autonomy_decision).to_string(),
        ),
        (
            "verdict".to_string(),
            format!("{:?}", autonomy_decision.verdict),
        ),
        (
            "blocking_cap_source".to_string(),
            autonomy_decision
                .blocking_cap_source
                .map(|source| source.as_str())
                .unwrap_or("none")
                .to_string(),
        ),
        (
            "blocking_cap_reason".to_string(),
            autonomy_decision
                .blocking_cap_reason
                .map(|reason| reason.as_str())
                .unwrap_or("none")
                .to_string(),
        ),
        (
            "lowering_cap_source".to_string(),
            autonomy_decision.lowering_cap_source.as_str().to_string(),
        ),
        (
            "lowering_cap_reason".to_string(),
            autonomy_decision.lowering_cap_reason.as_str().to_string(),
        ),
    ])
}

fn map_policy_error(error: PolicyError) -> FoundationError {
    match error {
        PolicyError::VersionAlreadyExists => FoundationError::PolicyVersionAlreadyExists,
        PolicyError::InvalidPolicyId
        | PolicyError::InvalidSemver
        | PolicyError::EmptyRules
        | PolicyError::EmptyRuleField
        | PolicyError::SupersedesSelf
        | PolicyError::SupersedesMissing
        | PolicyError::SupersedesScopeMismatch
        | PolicyError::SupersedesNotOlder => FoundationError::InvalidInput,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_preserves_primary_error_and_records_compensation_evidence() {
        let mut foundation = settlement_foundation();
        let request = settlement_request();
        let scope = settlement_scope();
        let reservation = foundation.cost_budgets.reserve(&scope, 10).unwrap();
        let run = foundation
            .foundry_runs
            .start(
                RunStart::new(
                    request.tenant_id.clone(),
                    request.capability_id.clone(),
                    request.user_id.clone(),
                    AutonomyTier::T2Advisory,
                    privacy_data_classes_from(&[DataClass::InternalOnly]).unwrap(),
                    "failover-region".into(),
                    reservation.reservation_id.value.clone(),
                    request.started_at_epoch_seconds,
                )
                .unwrap(),
            )
            .unwrap();

        let error = foundation
            .settle_failed_invocation(
                &request,
                Some(&reservation.reservation_id.value),
                Some(&run.run_id.value),
                RunDisposition::FailureProvider,
                FoundationError::CapabilityInvocationUnauthorized,
            )
            .unwrap();

        assert_eq!(error, FoundationError::CapabilityInvocationUnauthorized);
        let settled_run = foundation.foundry_runs().last().unwrap();
        assert_eq!(settled_run.state.value, RunState::Failed);
        assert_eq!(
            settled_run.disposition.value,
            Some(RunDisposition::FailureProvider)
        );
        assert_eq!(
            foundation
                .cost_budgets
                .release(&reservation.reservation_id.value),
            Err(BudgetError::ReservationNotPending)
        );
        let evidence = foundation
            .foundry_evidence_chain()
            .records()
            .last()
            .unwrap();
        assert_eq!(evidence.kind.value, EvidenceKind::CapabilityInvocation);
        assert_eq!(evidence.step_id.value, None);
        assert_eq!(
            evidence.fields.value.get("reason").map(String::as_str),
            Some("invocation_compensation")
        );
        assert_eq!(
            evidence
                .fields
                .value
                .get("primary_error")
                .map(String::as_str),
            Some("CapabilityInvocationUnauthorized")
        );
        assert_eq!(
            evidence
                .fields
                .value
                .get("budget_release")
                .map(String::as_str),
            Some("released")
        );
        assert_eq!(
            evidence
                .fields
                .value
                .get("run_completion")
                .map(String::as_str),
            Some("completed")
        );
        assert!(foundation.audit_chain().events().iter().any(|event| {
            event.surface == "foundry.invocation.compensate" && event.decision == "ALLOW"
        }));
        let outbox = foundation.outbox_records().last().unwrap();
        assert_eq!(outbox.topic.value, "oya.foundry.capability.invoked");
        assert_eq!(outbox.idempotency_key.value, evidence.evidence_id.value);
    }

    #[test]
    fn settlement_continues_after_budget_release_failure_and_preserves_primary_error() {
        let mut foundation = settlement_foundation();
        let request = settlement_request();
        let run = foundation
            .foundry_runs
            .start(
                RunStart::new(
                    request.tenant_id.clone(),
                    request.capability_id.clone(),
                    request.user_id.clone(),
                    AutonomyTier::T2Advisory,
                    privacy_data_classes_from(&[DataClass::InternalOnly]).unwrap(),
                    "failover-region".into(),
                    "res_missing".into(),
                    request.started_at_epoch_seconds,
                )
                .unwrap(),
            )
            .unwrap();

        let error = foundation
            .settle_failed_invocation(
                &request,
                Some("res_missing"),
                Some(&run.run_id.value),
                RunDisposition::FailureProvider,
                FoundationError::InvalidInput,
            )
            .unwrap();

        assert_eq!(error, FoundationError::InvalidInput);
        let settled_run = foundation.foundry_runs().last().unwrap();
        assert_eq!(settled_run.state.value, RunState::Failed);
        assert_eq!(
            settled_run.disposition.value,
            Some(RunDisposition::FailureProvider)
        );
        assert!(foundation.audit_chain().events().iter().any(|event| {
            event.surface == "foundry.cost-budget.release" && event.decision == "DENY"
        }));
        let evidence = foundation
            .foundry_evidence_chain()
            .records()
            .last()
            .unwrap();
        assert_eq!(
            evidence
                .fields
                .value
                .get("budget_release")
                .map(String::as_str),
            Some("failed")
        );
        assert_eq!(
            evidence
                .fields
                .value
                .get("run_completion")
                .map(String::as_str),
            Some("completed")
        );
    }

    #[test]
    fn settlement_records_run_completion_failure_without_masking_primary_error() {
        let mut foundation = settlement_foundation();
        let request = settlement_request();
        let run = foundation
            .foundry_runs
            .start(
                RunStart::new(
                    request.tenant_id.clone(),
                    request.capability_id.clone(),
                    request.user_id.clone(),
                    AutonomyTier::T2Advisory,
                    privacy_data_classes_from(&[DataClass::InternalOnly]).unwrap(),
                    "failover-region".into(),
                    "res_already_final".into(),
                    request.started_at_epoch_seconds,
                )
                .unwrap(),
            )
            .unwrap();
        foundation
            .foundry_runs
            .complete(
                &run.run_id.value,
                RunDisposition::Success,
                request.started_at_epoch_seconds.saturating_add(1),
            )
            .unwrap();

        let error = foundation
            .settle_failed_invocation(
                &request,
                None,
                Some(&run.run_id.value),
                RunDisposition::FailureProvider,
                FoundationError::CostBudgetExceeded,
            )
            .unwrap();

        assert_eq!(error, FoundationError::CostBudgetExceeded);
        assert!(
            foundation.audit_chain().events().iter().any(|event| {
                event.surface == "foundry.run.complete" && event.decision == "DENY"
            })
        );
        let evidence = foundation
            .foundry_evidence_chain()
            .records()
            .last()
            .unwrap();
        assert_eq!(
            evidence
                .fields
                .value
                .get("budget_release")
                .map(String::as_str),
            Some("not_applicable")
        );
        assert_eq!(
            evidence
                .fields
                .value
                .get("run_completion")
                .map(String::as_str),
            Some("failed")
        );
    }

    #[test]
    fn settlement_audits_no_run_budget_release_without_masking_primary_error() {
        let mut foundation = settlement_foundation();
        let request = settlement_request();
        let scope = settlement_scope();
        let reservation = foundation.cost_budgets.reserve(&scope, 10).unwrap();

        let error = foundation
            .settle_failed_invocation(
                &request,
                Some(&reservation.reservation_id.value),
                None,
                RunDisposition::FailureProvider,
                FoundationError::InvalidInput,
            )
            .unwrap();

        assert_eq!(error, FoundationError::InvalidInput);
        assert_eq!(
            foundation
                .cost_budgets
                .release(&reservation.reservation_id.value),
            Err(BudgetError::ReservationNotPending)
        );
        assert!(foundation.audit_chain().events().iter().any(|event| {
            event.surface == "foundry.cost-budget.release" && event.decision == "ALLOW"
        }));
        assert!(foundation.audit_chain().events().iter().any(|event| {
            event.surface == "foundry.invocation.compensate" && event.decision == "ALLOW"
        }));
        assert!(
            foundation.foundry_evidence_chain().records().is_empty(),
            "no-run settlement cannot append run-scoped evidence"
        );
        assert!(
            foundation.outbox_records().is_empty(),
            "no-run settlement has no evidence record to publish"
        );
    }

    fn settlement_foundation() -> Foundation {
        let mut foundation = Foundation::default();
        foundation
            .capabilities
            .publish(
                Capability::new(
                    "cap.demo.saga".into(),
                    "demo".into(),
                    AutonomyTier::T2Advisory,
                    vec![PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()],
                    "oya.foundry.capability.invoked".into(),
                )
                .unwrap(),
            )
            .unwrap();
        foundation
            .cost_budgets
            .configure_tenant_ceiling(
                "ten_saga".into(),
                "saga-window".into(),
                BudgetCeiling::new(1_000, 100, 80).unwrap(),
            )
            .unwrap();
        foundation
    }

    fn settlement_request() -> CapabilityInvocationRequest {
        CapabilityInvocationRequest {
            tenant_id: "ten_saga".into(),
            user_id: "usr_saga".into(),
            capability_id: "cap.demo.saga".into(),
            purpose: Purpose::CapabilityInvocation,
            subject_class: SubjectClass::Adult,
            budget_window_id: "saga-window".into(),
            projected_cost_micros: 10,
            started_at_epoch_seconds: 1_000,
        }
    }

    fn settlement_scope() -> BudgetScope {
        BudgetScope::new(
            "ten_saga".into(),
            "cap.demo.saga".into(),
            "saga-window".into(),
        )
        .unwrap()
    }
}
