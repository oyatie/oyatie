#![forbid(unsafe_code)]
//! Tenant model, quota policy, rate-limit policy, and region/cell allowlist
//! contracts for public-SaaS isolation.
//!
//! This crate depends only on `oya-office-kernel` at this stage. Provider
//! adapters, databases, and runtime enforcement live in later source-shaped
//! layers.

use oya_office_kernel::{CellId, PrincipalId, RequestContext, TenantId};

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-tenant-domain";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "tenant-security";

/// Source-shaped architectural layer represented by this crate.
pub const ARCHITECTURE_LAYER: &str = "domain";

/// Validation error for tenant policy contracts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TenantPolicyError {
    message: String,
}

impl TenantPolicyError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the validation error message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl core::fmt::Display for TenantPolicyError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for TenantPolicyError {}

/// Tenant lifecycle state used by the public-SaaS control plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TenantStatus {
    /// Tenant is provisioned but not yet active.
    Provisioning,
    /// Tenant can serve user traffic.
    Active,
    /// Tenant is temporarily blocked from user traffic.
    Suspended,
    /// Tenant is scheduled for deletion or export-retention workflow.
    Deleting,
}

/// Commercial/operational plan limits for a tenant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TenantPlan {
    name: String,
    storage_bytes_limit: u64,
    operations_per_window_limit: u64,
}

impl TenantPlan {
    /// Creates a validated tenant plan.
    pub fn new(
        name: impl Into<String>,
        storage_bytes_limit: u64,
        operations_per_window_limit: u64,
    ) -> Result<Self, TenantPolicyError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(TenantPolicyError::new("tenant plan name must not be empty"));
        }
        if storage_bytes_limit == 0 {
            return Err(TenantPolicyError::new(
                "tenant storage limit must be non-zero",
            ));
        }
        if operations_per_window_limit == 0 {
            return Err(TenantPolicyError::new(
                "tenant operations-per-window limit must be non-zero",
            ));
        }
        Ok(Self {
            name,
            storage_bytes_limit,
            operations_per_window_limit,
        })
    }

    /// Returns the stable tenant plan name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the storage quota in bytes.
    #[must_use]
    pub const fn storage_bytes_limit(&self) -> u64 {
        self.storage_bytes_limit
    }

    /// Returns the per-window operation quota.
    #[must_use]
    pub const fn operations_per_window_limit(&self) -> u64 {
        self.operations_per_window_limit
    }
}

/// Region/cell routing policy for tenant isolation and future multi-region cells.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionPolicy {
    primary_cell: CellId,
    allowed_cells: Vec<CellId>,
}

impl RegionPolicy {
    /// Creates a region policy and guarantees the primary cell is in the allowlist.
    pub fn new(
        primary_cell: CellId,
        mut allowed_cells: Vec<CellId>,
    ) -> Result<Self, TenantPolicyError> {
        if allowed_cells.is_empty() {
            return Err(TenantPolicyError::new(
                "tenant region policy requires at least one cell",
            ));
        }
        if !allowed_cells.iter().any(|cell| cell == &primary_cell) {
            allowed_cells.push(primary_cell.clone());
        }
        Ok(Self {
            primary_cell,
            allowed_cells,
        })
    }

    /// Returns true when the tenant may serve traffic in the cell.
    #[must_use]
    pub fn allows_cell(&self, cell_id: &CellId) -> bool {
        self.allowed_cells.iter().any(|allowed| allowed == cell_id)
    }

    /// Returns the tenant primary cell.
    #[must_use]
    pub const fn primary_cell(&self) -> &CellId {
        &self.primary_cell
    }

    /// Returns all currently allowed cells.
    #[must_use]
    pub fn allowed_cells(&self) -> &[CellId] {
        self.allowed_cells.as_slice()
    }
}

/// Tenant aggregate root for early public-SaaS control-plane contracts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tenant {
    tenant_id: TenantId,
    status: TenantStatus,
    plan: TenantPlan,
    region_policy: RegionPolicy,
}

impl Tenant {
    /// Creates a tenant aggregate from validated subcontracts.
    #[must_use]
    pub const fn new(
        tenant_id: TenantId,
        status: TenantStatus,
        plan: TenantPlan,
        region_policy: RegionPolicy,
    ) -> Self {
        Self {
            tenant_id,
            status,
            plan,
            region_policy,
        }
    }

    /// Returns the tenant identifier.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the tenant lifecycle status.
    #[must_use]
    pub const fn status(&self) -> TenantStatus {
        self.status
    }

    /// Returns the tenant plan.
    #[must_use]
    pub const fn plan(&self) -> &TenantPlan {
        &self.plan
    }

    /// Returns the tenant region policy.
    #[must_use]
    pub const fn region_policy(&self) -> &RegionPolicy {
        &self.region_policy
    }
}

/// Reason a quota or rate-limit decision denied a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QuotaDenyReason {
    /// Storage would exceed the tenant byte limit.
    StorageBytesExceeded,
    /// Operation count would exceed the current request window limit.
    OperationWindowExceeded,
}

/// Result of quota or rate-limit evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct QuotaDecision {
    reason: Option<QuotaDenyReason>,
}

impl QuotaDecision {
    /// Creates an allow decision.
    #[must_use]
    pub const fn allow() -> Self {
        Self { reason: None }
    }

    /// Creates a deny decision.
    #[must_use]
    pub const fn deny(reason: QuotaDenyReason) -> Self {
        Self {
            reason: Some(reason),
        }
    }

    /// Returns true when the request is allowed.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        self.reason.is_none()
    }

    /// Returns true when the request is denied.
    #[must_use]
    pub const fn is_denied(&self) -> bool {
        self.reason.is_some()
    }

    /// Returns the optional deny reason.
    #[must_use]
    pub const fn reason(&self) -> Option<QuotaDenyReason> {
        self.reason
    }
}

/// Current quota usage snapshot for one tenant and accounting window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct QuotaUsage {
    storage_bytes: u64,
    operations_in_window: u64,
}

impl QuotaUsage {
    /// Creates a quota usage snapshot.
    #[must_use]
    pub const fn new(storage_bytes: u64, operations_in_window: u64) -> Self {
        Self {
            storage_bytes,
            operations_in_window,
        }
    }

    /// Returns current storage bytes.
    #[must_use]
    pub const fn storage_bytes(&self) -> u64 {
        self.storage_bytes
    }

    /// Returns current operation count in the active window.
    #[must_use]
    pub const fn operations_in_window(&self) -> u64 {
        self.operations_in_window
    }
}

/// Tenant quota policy for storage and operation windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct QuotaPolicy {
    storage_bytes_limit: u64,
    operations_per_window_limit: u64,
}

impl QuotaPolicy {
    /// Creates a quota policy with non-zero limits.
    pub fn new(
        storage_bytes_limit: u64,
        operations_per_window_limit: u64,
    ) -> Result<Self, TenantPolicyError> {
        if storage_bytes_limit == 0 {
            return Err(TenantPolicyError::new(
                "storage quota limit must be non-zero",
            ));
        }
        if operations_per_window_limit == 0 {
            return Err(TenantPolicyError::new(
                "operation quota limit must be non-zero",
            ));
        }
        Ok(Self {
            storage_bytes_limit,
            operations_per_window_limit,
        })
    }

    /// Evaluates whether additional storage bytes fit within the tenant quota.
    #[must_use]
    pub fn evaluate_storage_bytes(
        &self,
        usage: &QuotaUsage,
        requested_bytes: u64,
    ) -> QuotaDecision {
        if usage.storage_bytes().saturating_add(requested_bytes) > self.storage_bytes_limit {
            QuotaDecision::deny(QuotaDenyReason::StorageBytesExceeded)
        } else {
            QuotaDecision::allow()
        }
    }

    /// Evaluates whether additional operations fit within the tenant quota window.
    #[must_use]
    pub fn evaluate_operations(
        &self,
        usage: &QuotaUsage,
        requested_operations: u64,
    ) -> QuotaDecision {
        if usage
            .operations_in_window()
            .saturating_add(requested_operations)
            > self.operations_per_window_limit
        {
            QuotaDecision::deny(QuotaDenyReason::OperationWindowExceeded)
        } else {
            QuotaDecision::allow()
        }
    }

    /// Returns the configured storage byte limit.
    #[must_use]
    pub const fn storage_bytes_limit(&self) -> u64 {
        self.storage_bytes_limit
    }

    /// Returns the configured operation window limit.
    #[must_use]
    pub const fn operations_per_window_limit(&self) -> u64 {
        self.operations_per_window_limit
    }
}

/// Noisy-neighbor rate-limit policy for one tenant request window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RateLimitPolicy {
    max_requests_per_window: u64,
}

impl RateLimitPolicy {
    /// Creates a rate-limit policy with a non-zero request window limit.
    pub fn new(max_requests_per_window: u64) -> Result<Self, TenantPolicyError> {
        if max_requests_per_window == 0 {
            return Err(TenantPolicyError::new("rate-limit window must be non-zero"));
        }
        Ok(Self {
            max_requests_per_window,
        })
    }

    /// Evaluates the current request count against the configured window.
    #[must_use]
    pub const fn evaluate_window(&self, requests_in_window: u64) -> QuotaDecision {
        if requests_in_window > self.max_requests_per_window {
            QuotaDecision::deny(QuotaDenyReason::OperationWindowExceeded)
        } else {
            QuotaDecision::allow()
        }
    }

    /// Returns the configured maximum request count per window.
    #[must_use]
    pub const fn max_requests_per_window(&self) -> u64 {
        self.max_requests_per_window
    }
}

/// Provider-neutral billing reference that must not contain raw secrets.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BillingReference(String);

impl BillingReference {
    /// Creates a safe billing reference for plan or customer identifiers.
    pub fn new(value: impl Into<String>) -> Result<Self, TenantPolicyError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(TenantPolicyError::new(
                "billing reference must not be empty",
            ));
        }
        if trimmed.len() > 128 {
            return Err(TenantPolicyError::new(
                "billing reference must be 128 characters or fewer",
            ));
        }
        if !trimmed.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        }) {
            return Err(TenantPolicyError::new(
                "billing reference contains an unsupported character",
            ));
        }

        let lowercase = trimmed.to_ascii_lowercase();
        let secret_markers = [
            "sk_",
            "secret",
            "password",
            "passwd",
            "token",
            "api_key",
            "apikey",
            "card_number",
            "cvv",
        ];
        if secret_markers
            .iter()
            .any(|marker| lowercase.contains(marker))
        {
            return Err(TenantPolicyError::new(
                "billing reference must not contain raw secrets or payment data",
            ));
        }

        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the validated billing reference.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Product billing tier associated with tenant entitlements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BillingTier {
    /// Time-limited onboarding or evaluation tier.
    Trial,
    /// Team/business tenant tier.
    Business,
    /// Enterprise tenant tier with stricter admin and audit controls.
    Enterprise,
}

/// Billing account lifecycle state used by onboarding readiness gates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BillingAccountStatus {
    /// Account is in a trial period and may serve traffic within quota.
    Trial,
    /// Account is active and may serve traffic within quota.
    Active,
    /// Account has a billing issue and should be blocked from new onboarding.
    PastDue,
    /// Account is suspended by billing or risk controls.
    Suspended,
    /// Account is cancelled and not eligible for serving traffic.
    Cancelled,
}

impl BillingAccountStatus {
    /// Returns true when this state may serve tenant traffic.
    #[must_use]
    pub const fn is_service_eligible(self) -> bool {
        matches!(self, Self::Trial | Self::Active)
    }
}

/// Billing plan contract that binds commercial tier to quota and rate-limit controls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TenantBillingPlan {
    tier: BillingTier,
    plan_reference: BillingReference,
    quota_policy: QuotaPolicy,
    rate_limit_policy: RateLimitPolicy,
}

impl TenantBillingPlan {
    /// Creates a validated, provider-neutral billing plan contract.
    pub fn new(
        tier: BillingTier,
        plan_reference: BillingReference,
        quota_policy: QuotaPolicy,
        rate_limit_policy: RateLimitPolicy,
    ) -> Result<Self, TenantPolicyError> {
        if quota_policy.storage_bytes_limit() == 0 {
            return Err(TenantPolicyError::new(
                "billing plan storage quota must be non-zero",
            ));
        }
        if quota_policy.operations_per_window_limit() == 0 {
            return Err(TenantPolicyError::new(
                "billing plan operation quota must be non-zero",
            ));
        }
        if rate_limit_policy.max_requests_per_window() == 0 {
            return Err(TenantPolicyError::new(
                "billing plan rate limit must be non-zero",
            ));
        }
        Ok(Self {
            tier,
            plan_reference,
            quota_policy,
            rate_limit_policy,
        })
    }

    /// Returns the product billing tier.
    #[must_use]
    pub const fn tier(&self) -> BillingTier {
        self.tier
    }

    /// Returns the provider-neutral billing plan reference.
    #[must_use]
    pub const fn plan_reference(&self) -> &BillingReference {
        &self.plan_reference
    }

    /// Returns the quota policy tied to the billing plan.
    #[must_use]
    pub const fn quota_policy(&self) -> &QuotaPolicy {
        &self.quota_policy
    }

    /// Returns the rate-limit policy tied to the billing plan.
    #[must_use]
    pub const fn rate_limit_policy(&self) -> &RateLimitPolicy {
        &self.rate_limit_policy
    }
}

/// Tenant billing account contract used by onboarding and quota enforcement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TenantBillingAccount {
    tenant_id: TenantId,
    status: BillingAccountStatus,
    customer_reference: BillingReference,
    plan: TenantBillingPlan,
}

impl TenantBillingAccount {
    /// Creates a tenant-scoped billing account without storing raw payment data.
    pub fn new(
        tenant_id: TenantId,
        status: BillingAccountStatus,
        customer_reference: BillingReference,
        plan: TenantBillingPlan,
    ) -> Result<Self, TenantPolicyError> {
        if !status.is_service_eligible() && matches!(status, BillingAccountStatus::Cancelled) {
            return Err(TenantPolicyError::new(
                "cancelled billing accounts cannot be used for tenant controls",
            ));
        }
        Ok(Self {
            tenant_id,
            status,
            customer_reference,
            plan,
        })
    }

    /// Returns the billing account tenant.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the billing lifecycle status.
    #[must_use]
    pub const fn status(&self) -> BillingAccountStatus {
        self.status
    }

    /// Returns true when billing state allows tenant service.
    #[must_use]
    pub const fn is_service_eligible(&self) -> bool {
        self.status.is_service_eligible()
    }

    /// Returns the safe customer reference.
    #[must_use]
    pub const fn customer_reference(&self) -> &BillingReference {
        &self.customer_reference
    }

    /// Returns the active billing plan.
    #[must_use]
    pub const fn plan(&self) -> &TenantBillingPlan {
        &self.plan
    }
}

/// Tenant administration roles for public-SaaS control-plane actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TenantAdminRole {
    /// Full tenant owner; required for initial onboarding.
    Owner,
    /// Billing manager for plan and payment-state workflows.
    BillingAdmin,
    /// Security manager for abuse, access, and audit workflows.
    SecurityAdmin,
    /// Workspace manager for normal suite administration.
    WorkspaceAdmin,
    /// Scoped support role that must not complete onboarding.
    SupportAdmin,
}

impl TenantAdminRole {
    /// Returns true when the role may own initial tenant onboarding.
    #[must_use]
    pub const fn can_complete_onboarding(self) -> bool {
        matches!(self, Self::Owner)
    }

    /// Returns true when the role may manage billing contracts.
    #[must_use]
    pub const fn can_manage_billing(self) -> bool {
        matches!(self, Self::Owner | Self::BillingAdmin)
    }

    /// Returns true when the role may manage abuse controls.
    #[must_use]
    pub const fn can_manage_abuse_controls(self) -> bool {
        matches!(self, Self::Owner | Self::SecurityAdmin)
    }
}

/// Tenant-scoped admin assignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TenantAdminAssignment {
    tenant_id: TenantId,
    principal_id: PrincipalId,
    role: TenantAdminRole,
}

impl TenantAdminAssignment {
    /// Creates an admin assignment from already validated tenant and principal IDs.
    pub fn new(
        tenant_id: TenantId,
        principal_id: PrincipalId,
        role: TenantAdminRole,
    ) -> Result<Self, TenantPolicyError> {
        Ok(Self {
            tenant_id,
            principal_id,
            role,
        })
    }

    /// Creates an assignment only when the actor request context is in the target tenant.
    pub fn from_request_context(
        context: &RequestContext,
        tenant_id: TenantId,
        principal_id: PrincipalId,
        role: TenantAdminRole,
    ) -> Result<Self, TenantPolicyError> {
        if context.tenant_id() != &tenant_id {
            return Err(TenantPolicyError::new(
                "admin assignment request tenant must match target tenant",
            ));
        }
        Self::new(tenant_id, principal_id, role)
    }

    /// Returns the tenant boundary for the admin assignment.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the assigned principal.
    #[must_use]
    pub const fn principal_id(&self) -> &PrincipalId {
        &self.principal_id
    }

    /// Returns the assigned tenant admin role.
    #[must_use]
    pub const fn role(&self) -> TenantAdminRole {
        self.role
    }
}

/// Abuse signal category for tenant control-plane decisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AbuseSignalKind {
    /// Credential stuffing or account-takeover pattern.
    CredentialAttack,
    /// Suspicious sharing or spam invitation behavior.
    SuspiciousSharing,
    /// Excessive import/export or conversion activity.
    FormatConversionFlood,
    /// Bulk storage exfiltration or scraping pattern.
    StorageExfiltration,
    /// Attempts to evade quota or rate-limit controls.
    QuotaEvasion,
}

/// Severity assigned by abuse detection or policy rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AbuseSeverity {
    /// Low-risk signal for observation.
    Low,
    /// Medium-risk signal that may trigger throttling at volume.
    Medium,
    /// High-risk signal that requires review or throttling.
    High,
    /// Critical-risk signal that must fail closed.
    Critical,
}

/// Tenant-scoped abuse signal consumed by the abuse control policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TenantAbuseSignal {
    tenant_id: TenantId,
    kind: AbuseSignalKind,
    severity: AbuseSeverity,
    observed_count: u64,
}

impl TenantAbuseSignal {
    /// Creates a tenant abuse signal with a non-zero observation count.
    pub fn new(
        tenant_id: TenantId,
        kind: AbuseSignalKind,
        severity: AbuseSeverity,
        observed_count: u64,
    ) -> Result<Self, TenantPolicyError> {
        if observed_count == 0 {
            return Err(TenantPolicyError::new(
                "abuse signal observation count must be non-zero",
            ));
        }
        Ok(Self {
            tenant_id,
            kind,
            severity,
            observed_count,
        })
    }

    /// Returns the tenant associated with the signal.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the signal category.
    #[must_use]
    pub const fn kind(&self) -> AbuseSignalKind {
        self.kind
    }

    /// Returns the signal severity.
    #[must_use]
    pub const fn severity(&self) -> AbuseSeverity {
        self.severity
    }

    /// Returns the observed count in the policy window.
    #[must_use]
    pub const fn observed_count(&self) -> u64 {
        self.observed_count
    }
}

/// Abuse control action selected by tenant policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AbuseControlAction {
    /// Permit traffic while continuing normal observation.
    Allow,
    /// Throttle tenant traffic to protect shared cells.
    ThrottleTenant,
    /// Require manual security review before normal serving continues.
    RequireReview,
    /// Suspend tenant traffic because the signal is critical.
    SuspendTenant,
}

/// Abuse policy decision with user-safe reason text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbuseControlDecision {
    action: AbuseControlAction,
    reason: String,
}

impl AbuseControlDecision {
    fn new(action: AbuseControlAction, reason: impl Into<String>) -> Self {
        Self {
            action,
            reason: reason.into(),
        }
    }

    /// Returns the selected abuse action.
    #[must_use]
    pub const fn action(&self) -> AbuseControlAction {
        self.action
    }

    /// Returns true when the action denies or restricts normal traffic.
    #[must_use]
    pub const fn is_denied(&self) -> bool {
        !matches!(self.action, AbuseControlAction::Allow)
    }

    /// Returns user-safe reason text for audit and support workflows.
    #[must_use]
    pub fn reason(&self) -> &str {
        self.reason.as_str()
    }
}

/// Tenant abuse control thresholds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TenantAbusePolicy {
    high_severity_review_threshold: u64,
    max_events_per_window: u64,
}

impl TenantAbusePolicy {
    /// Creates abuse thresholds with non-zero limits.
    pub fn new(
        high_severity_review_threshold: u64,
        max_events_per_window: u64,
    ) -> Result<Self, TenantPolicyError> {
        if high_severity_review_threshold == 0 {
            return Err(TenantPolicyError::new(
                "abuse high-severity review threshold must be non-zero",
            ));
        }
        if max_events_per_window == 0 {
            return Err(TenantPolicyError::new(
                "abuse max-events-per-window threshold must be non-zero",
            ));
        }
        Ok(Self {
            high_severity_review_threshold,
            max_events_per_window,
        })
    }

    /// Evaluates an abuse signal and fails closed for critical signals.
    #[must_use]
    pub fn evaluate(&self, signal: &TenantAbuseSignal) -> AbuseControlDecision {
        if matches!(signal.severity(), AbuseSeverity::Critical) {
            return AbuseControlDecision::new(
                AbuseControlAction::SuspendTenant,
                "critical tenant abuse signal",
            );
        }
        if signal.observed_count() > self.max_events_per_window {
            return AbuseControlDecision::new(
                AbuseControlAction::ThrottleTenant,
                "tenant abuse event window exceeded",
            );
        }
        if matches!(signal.severity(), AbuseSeverity::High)
            && signal.observed_count() >= self.high_severity_review_threshold
        {
            return AbuseControlDecision::new(
                AbuseControlAction::RequireReview,
                "high-severity tenant abuse review threshold reached",
            );
        }
        AbuseControlDecision::new(AbuseControlAction::Allow, "tenant abuse signal allowed")
    }

    /// Returns high-severity review threshold.
    #[must_use]
    pub const fn high_severity_review_threshold(&self) -> u64 {
        self.high_severity_review_threshold
    }

    /// Returns max events per policy window.
    #[must_use]
    pub const fn max_events_per_window(&self) -> u64 {
        self.max_events_per_window
    }
}

/// Reason a tenant onboarding plan is not ready.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TenantOnboardingBlocker {
    /// Initial admin is not an owner.
    MissingOwnerAdmin,
    /// Billing state is not eligible to serve traffic.
    BillingNotServiceEligible,
    /// Region policy does not allow the primary cell.
    PrimaryCellNotAllowed,
    /// Billing quota is smaller than the tenant plan.
    BillingQuotaTooSmall,
}

/// Tenant onboarding readiness decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TenantOnboardingReadiness {
    /// Tenant has the required admin, billing, quota, region, and abuse controls.
    Ready,
    /// Tenant is blocked by a named control gate.
    Blocked(TenantOnboardingBlocker),
}

/// Complete tenant onboarding contract for admin, billing, quota, and abuse gates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TenantOnboardingPlan {
    tenant_id: TenantId,
    tenant_plan: TenantPlan,
    region_policy: RegionPolicy,
    owner_admin: TenantAdminAssignment,
    billing_account: TenantBillingAccount,
    abuse_policy: TenantAbusePolicy,
}

impl TenantOnboardingPlan {
    /// Creates a tenant onboarding plan after validating cross-contract tenant scope.
    pub fn new(
        tenant_id: TenantId,
        tenant_plan: TenantPlan,
        region_policy: RegionPolicy,
        owner_admin: TenantAdminAssignment,
        billing_account: TenantBillingAccount,
        abuse_policy: TenantAbusePolicy,
    ) -> Result<Self, TenantPolicyError> {
        if owner_admin.tenant_id() != &tenant_id {
            return Err(TenantPolicyError::new(
                "tenant onboarding owner admin must match tenant",
            ));
        }
        if billing_account.tenant_id() != &tenant_id {
            return Err(TenantPolicyError::new(
                "tenant onboarding billing account must match tenant",
            ));
        }
        let plan = Self {
            tenant_id,
            tenant_plan,
            region_policy,
            owner_admin,
            billing_account,
            abuse_policy,
        };
        match plan.evaluate_readiness() {
            TenantOnboardingReadiness::Ready => Ok(plan),
            TenantOnboardingReadiness::Blocked(blocker) => {
                Err(TenantPolicyError::new(blocker.error_message()))
            }
        }
    }

    /// Returns the tenant being onboarded.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the commercial tenant plan.
    #[must_use]
    pub const fn tenant_plan(&self) -> &TenantPlan {
        &self.tenant_plan
    }

    /// Returns the tenant region/cell policy.
    #[must_use]
    pub const fn region_policy(&self) -> &RegionPolicy {
        &self.region_policy
    }

    /// Returns the primary cell for the tenant.
    #[must_use]
    pub const fn primary_cell(&self) -> &CellId {
        self.region_policy.primary_cell()
    }

    /// Returns the initial owner admin assignment.
    #[must_use]
    pub const fn owner_admin(&self) -> &TenantAdminAssignment {
        &self.owner_admin
    }

    /// Returns the billing account contract.
    #[must_use]
    pub const fn billing_account(&self) -> &TenantBillingAccount {
        &self.billing_account
    }

    /// Returns the abuse policy contract.
    #[must_use]
    pub const fn abuse_policy(&self) -> &TenantAbusePolicy {
        &self.abuse_policy
    }

    /// Evaluates sequence-aware onboarding readiness.
    #[must_use]
    pub fn evaluate_readiness(&self) -> TenantOnboardingReadiness {
        if !self.owner_admin.role().can_complete_onboarding() {
            return TenantOnboardingReadiness::Blocked(TenantOnboardingBlocker::MissingOwnerAdmin);
        }
        if !self.billing_account.is_service_eligible() {
            return TenantOnboardingReadiness::Blocked(
                TenantOnboardingBlocker::BillingNotServiceEligible,
            );
        }
        if !self
            .region_policy
            .allows_cell(self.region_policy.primary_cell())
        {
            return TenantOnboardingReadiness::Blocked(
                TenantOnboardingBlocker::PrimaryCellNotAllowed,
            );
        }
        let billing_quota = self.billing_account.plan().quota_policy();
        if billing_quota.storage_bytes_limit() < self.tenant_plan.storage_bytes_limit()
            || billing_quota.operations_per_window_limit()
                < self.tenant_plan.operations_per_window_limit()
        {
            return TenantOnboardingReadiness::Blocked(
                TenantOnboardingBlocker::BillingQuotaTooSmall,
            );
        }
        TenantOnboardingReadiness::Ready
    }
}

impl TenantOnboardingBlocker {
    fn error_message(self) -> &'static str {
        match self {
            Self::MissingOwnerAdmin => "tenant onboarding requires an owner admin",
            Self::BillingNotServiceEligible => {
                "tenant onboarding billing status must be trial or active"
            }
            Self::PrimaryCellNotAllowed => {
                "tenant onboarding region policy must allow the primary cell"
            }
            Self::BillingQuotaTooSmall => {
                "tenant onboarding billing quota must cover the tenant plan"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ARCHITECTURE_LAYER, CRATE_NAME, VERTICAL_SLICE};

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!ARCHITECTURE_LAYER.is_empty());
        assert!(!CRATE_NAME.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }
}

#[cfg(test)]
mod tenant_contract_tests {
    use oya_office_kernel::{CellId, PrincipalId, RequestContext, RequestId, TenantId};

    use super::{
        AbuseControlAction, AbuseSeverity, AbuseSignalKind, BillingAccountStatus, BillingReference,
        BillingTier, QuotaDenyReason, QuotaPolicy, QuotaUsage, RateLimitPolicy, RegionPolicy,
        TenantAbusePolicy, TenantAbuseSignal, TenantAdminAssignment, TenantAdminRole,
        TenantBillingAccount, TenantBillingPlan, TenantOnboardingPlan, TenantOnboardingReadiness,
        TenantPlan,
    };

    #[test]
    fn quota_policy_denies_storage_overage() {
        let policy = QuotaPolicy::new(100, 10).expect("valid quota policy");
        let usage = QuotaUsage::new(100, 3);

        let decision = policy.evaluate_storage_bytes(&usage, 1);

        assert!(decision.is_denied());
        assert_eq!(
            decision.reason(),
            Some(QuotaDenyReason::StorageBytesExceeded)
        );
    }

    #[test]
    fn rate_limit_policy_denies_noisy_neighbor_window() {
        let policy = RateLimitPolicy::new(10).expect("valid rate limit policy");

        let decision = policy.evaluate_window(11);

        assert!(decision.is_denied());
        assert_eq!(
            decision.reason(),
            Some(QuotaDenyReason::OperationWindowExceeded)
        );
    }

    #[test]
    fn region_policy_preserves_tenant_cell_allowlist() {
        let tenant_id = TenantId::new("tenant-alpha").expect("valid tenant id");
        let primary = CellId::new("iad-1").expect("valid cell id");
        let policy =
            RegionPolicy::new(primary.clone(), vec![primary.clone()]).expect("valid region policy");
        let plan = TenantPlan::new("public-saas-standard", 1024, 100).expect("valid tenant plan");

        assert!(policy.allows_cell(&primary));
        assert_eq!(plan.name(), "public-saas-standard");
        assert_eq!(tenant_id.as_str(), "tenant-alpha");
    }

    #[test]
    fn onboarding_plan_requires_admin_billing_quota_and_abuse_controls() {
        let tenant_id = TenantId::new("tenant-alpha").expect("valid tenant");
        let primary = CellId::new("iad-1").expect("valid cell");
        let creator = PrincipalId::new("principal-owner").expect("valid principal");
        let plan = TenantPlan::new("public-saas-standard", 1_000_000, 1_000).expect("valid plan");
        let region_policy =
            RegionPolicy::new(primary.clone(), vec![primary.clone()]).expect("valid region");
        let owner =
            TenantAdminAssignment::new(tenant_id.clone(), creator.clone(), TenantAdminRole::Owner)
                .expect("valid admin owner");
        let billing_plan = TenantBillingPlan::new(
            BillingTier::Business,
            BillingReference::new("billing-plan-business").expect("safe billing reference"),
            QuotaPolicy::new(1_000_000, 1_000).expect("valid quota"),
            RateLimitPolicy::new(10_000).expect("valid rate limit"),
        )
        .expect("valid billing plan");
        let billing_account = TenantBillingAccount::new(
            tenant_id.clone(),
            BillingAccountStatus::Active,
            BillingReference::new("customer-alpha").expect("safe customer reference"),
            billing_plan,
        )
        .expect("valid billing account");
        let abuse_policy = TenantAbusePolicy::new(100, 1_000).expect("valid abuse policy");

        let onboarding = TenantOnboardingPlan::new(
            tenant_id.clone(),
            plan,
            region_policy,
            owner,
            billing_account,
            abuse_policy,
        )
        .expect("complete onboarding plan");

        let readiness = onboarding.evaluate_readiness();

        assert_eq!(readiness, TenantOnboardingReadiness::Ready);
        assert_eq!(onboarding.tenant_id(), &tenant_id);
        assert_eq!(onboarding.primary_cell(), &primary);
    }

    #[test]
    fn admin_assignment_rejects_cross_tenant_context() {
        let tenant_id = TenantId::new("tenant-alpha").expect("valid tenant");
        let other_tenant = TenantId::new("tenant-beta").expect("valid tenant");
        let actor = PrincipalId::new("principal-owner").expect("valid principal");
        let request = RequestContext::new(
            RequestId::new("req-admin").expect("valid request"),
            other_tenant,
            actor.clone(),
            CellId::new("iad-1").expect("valid cell"),
        );

        let result = TenantAdminAssignment::from_request_context(
            &request,
            tenant_id,
            actor,
            TenantAdminRole::BillingAdmin,
        );

        assert!(result.is_err());
    }

    #[test]
    fn billing_reference_rejects_secret_like_values() {
        let result = BillingReference::new("sk_live_not-a-contract-reference");

        assert!(result.is_err());
    }

    #[test]
    fn onboarding_plan_rejects_billing_quota_below_tenant_plan() {
        let tenant_id = TenantId::new("tenant-alpha").expect("valid tenant");
        let primary = CellId::new("iad-1").expect("valid cell");
        let owner = TenantAdminAssignment::new(
            tenant_id.clone(),
            PrincipalId::new("principal-owner").expect("valid principal"),
            TenantAdminRole::Owner,
        )
        .expect("valid admin owner");
        let undersized_billing_plan = TenantBillingPlan::new(
            BillingTier::Business,
            BillingReference::new("billing-plan-business").expect("safe billing reference"),
            QuotaPolicy::new(512, 10).expect("valid quota"),
            RateLimitPolicy::new(100).expect("valid rate limit"),
        )
        .expect("valid billing plan");
        let billing_account = TenantBillingAccount::new(
            tenant_id.clone(),
            BillingAccountStatus::Active,
            BillingReference::new("customer-alpha").expect("safe customer reference"),
            undersized_billing_plan,
        )
        .expect("valid billing account");

        let result = TenantOnboardingPlan::new(
            tenant_id,
            TenantPlan::new("public-saas-standard", 1_000_000, 1_000).expect("valid plan"),
            RegionPolicy::new(primary.clone(), vec![primary]).expect("valid region"),
            owner,
            billing_account,
            TenantAbusePolicy::new(100, 1_000).expect("valid abuse policy"),
        );

        assert!(result.is_err());
        assert!(
            result
                .expect_err("undersized billing quota must fail")
                .message()
                .contains("billing quota")
        );
    }

    #[test]
    fn abuse_policy_suspends_critical_tenant_signal() {
        let tenant_id = TenantId::new("tenant-alpha").expect("valid tenant");
        let signal = TenantAbuseSignal::new(
            tenant_id,
            AbuseSignalKind::CredentialAttack,
            AbuseSeverity::Critical,
            1,
        )
        .expect("valid signal");
        let policy = TenantAbusePolicy::new(100, 1_000).expect("valid abuse policy");

        let decision = policy.evaluate(&signal);

        assert_eq!(decision.action(), AbuseControlAction::SuspendTenant);
        assert!(decision.is_denied());
    }
}
