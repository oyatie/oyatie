#![forbid(unsafe_code)]
//! Authorization policy decisions, sharing gates, export permissions, and
//! audit-bound access checks for tenant-security surfaces.
//!
//! This crate depends only on `oya-office-kernel` at this stage. Policy engines,
//! storage adapters, and API middleware live in later source-shaped layers.

use oya_office_kernel::{
    AuditAction, AuditEvent, AuditEventInput, AuditOutcome, DataClass, ObjectId, RequestContext,
    RequestId, TenantId,
};

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-authz-domain";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "tenant-security";

/// Source-shaped architectural layer represented by this crate.
pub const ARCHITECTURE_LAYER: &str = "domain";

/// Version for the G083 tenant/authz/audit baseline across Drive/API/search/collab/storage.
pub const G083_TENANT_SECURITY_BASELINE_VERSION: &str = "g083-tenant-security-baseline-v1";

/// Product surfaces covered by the G083 tenant/security baseline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TenantSecuritySurface {
    /// Built-in Drive domain metadata, ACL, KMS-shred, lifecycle, quota, and audit.
    DriveDomain,
    /// Drive/API request, route, event-envelope, and boundary-validation contracts.
    DriveApi,
    /// Search/index query and result projection contracts.
    Search,
    /// Collaboration op-log, snapshot, awareness, and load-harness contracts.
    Collaboration,
    /// Object upload/download and storage-key port contracts.
    ObjectStorage,
    /// Tenant onboarding, admin, billing/quota, rate-limit, and abuse controls.
    TenantControlPlane,
}

/// Launch-blocking gate categories for tenant/security baseline evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TenantSecurityBaselineGateKind {
    /// A request context carries request id, tenant id, principal id, and cell id.
    RequestContext,
    /// Request tenant must match resource tenant before ACL/policy evaluation.
    TenantBoundaryPrecheck,
    /// Object-level authorization is required before metadata, content, ops, or automation.
    ObjectAuthorization,
    /// Every allow/deny/security-sensitive decision projects to audit evidence.
    AuditProjection,
    /// Public/API/search outputs redact private storage, KMS, content, and adapter internals.
    Redaction,
    /// KMS-shredded or revoked objects deny content/export/download access.
    KmsShred,
    /// Quota/rate-limit/abuse controls bound tenant operations before release claims.
    QuotaRateLimitAbuse,
    /// Inputs, ranges, operations, webhooks, imports, and storage accesses stay bounded.
    BoundedOperation,
}

/// One static G083 tenant/security launch-blocking gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TenantSecurityBaselineGate {
    kind: TenantSecurityBaselineGateKind,
    evidence: &'static str,
    launch_blocking: bool,
}

impl TenantSecurityBaselineGate {
    /// Returns the gate kind.
    #[must_use]
    pub const fn kind(self) -> TenantSecurityBaselineGateKind {
        self.kind
    }

    /// Returns the current repo evidence anchor.
    #[must_use]
    pub const fn evidence(self) -> &'static str {
        self.evidence
    }

    /// Returns true when missing evidence blocks launch/readiness claims.
    #[must_use]
    pub const fn launch_blocking(self) -> bool {
        self.launch_blocking
    }
}

const G083_TENANT_SECURITY_BASELINE_GATES: [TenantSecurityBaselineGate; 8] = [
    TenantSecurityBaselineGate {
        kind: TenantSecurityBaselineGateKind::RequestContext,
        evidence: "oya-office-kernel::RequestContext carries request id, tenant id, principal id, and cell id through API, worker, and audit paths.",
        launch_blocking: true,
    },
    TenantSecurityBaselineGate {
        kind: TenantSecurityBaselineGateKind::TenantBoundaryPrecheck,
        evidence: "request_tenant_matches_resource() and Drive security tests fail closed before ACL evaluation when request/resource tenants differ.",
        launch_blocking: true,
    },
    TenantSecurityBaselineGate {
        kind: TenantSecurityBaselineGateKind::ObjectAuthorization,
        evidence: "AuthorizationRequest, AclRole, DriveAction, and DriveSecurityPolicy gates protect object metadata, content, sharing, export, and delete actions.",
        launch_blocking: true,
    },
    TenantSecurityBaselineGate {
        kind: TenantSecurityBaselineGateKind::AuditProjection,
        evidence: "AuthorizationDecision::to_audit_event projects request, tenant, actor, action, resource, data class, outcome, and reason.",
        launch_blocking: true,
    },
    TenantSecurityBaselineGate {
        kind: TenantSecurityBaselineGateKind::Redaction,
        evidence: "Drive public views and search projections omit storage pointers, KMS references, object content, and adapter internals.",
        launch_blocking: true,
    },
    TenantSecurityBaselineGate {
        kind: TenantSecurityBaselineGateKind::KmsShred,
        evidence: "Drive KMS-shred lifecycle tests deny content access even for owners after key shred.",
        launch_blocking: true,
    },
    TenantSecurityBaselineGate {
        kind: TenantSecurityBaselineGateKind::QuotaRateLimitAbuse,
        evidence: "Tenant quota, rate-limit, onboarding, billing, and abuse-control contracts bind service eligibility before public traffic.",
        launch_blocking: true,
    },
    TenantSecurityBaselineGate {
        kind: TenantSecurityBaselineGateKind::BoundedOperation,
        evidence: "Drive API page limits, search limits, collab sequence/replay gates, format jobs, and storage byte ranges are bounded before runtime adapters.",
        launch_blocking: true,
    },
];

/// Cross-surface security baseline row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TenantSecuritySurfaceContract {
    surface: TenantSecuritySurface,
    owning_crate: &'static str,
    evidence: &'static str,
    requires_request_context: bool,
    requires_authorization_before_data: bool,
    requires_audit_event: bool,
    requires_redaction_or_bounded_output: bool,
    launch_blocking: bool,
}

impl TenantSecuritySurfaceContract {
    /// Returns the covered surface.
    #[must_use]
    pub const fn surface(self) -> TenantSecuritySurface {
        self.surface
    }

    /// Returns the crate/app that owns the evidence contract.
    #[must_use]
    pub const fn owning_crate(self) -> &'static str {
        self.owning_crate
    }

    /// Returns current evidence.
    #[must_use]
    pub const fn evidence(self) -> &'static str {
        self.evidence
    }

    /// Returns true when request context is required.
    #[must_use]
    pub const fn requires_request_context(self) -> bool {
        self.requires_request_context
    }

    /// Returns true when authorization must happen before returning data or mutating state.
    #[must_use]
    pub const fn requires_authorization_before_data(self) -> bool {
        self.requires_authorization_before_data
    }

    /// Returns true when audit evidence is required.
    #[must_use]
    pub const fn requires_audit_event(self) -> bool {
        self.requires_audit_event
    }

    /// Returns true when output redaction or operation bounds are required.
    #[must_use]
    pub const fn requires_redaction_or_bounded_output(self) -> bool {
        self.requires_redaction_or_bounded_output
    }

    /// Returns true when this surface blocks release claims if missing.
    #[must_use]
    pub const fn launch_blocking(self) -> bool {
        self.launch_blocking
    }
}

const G083_TENANT_SECURITY_SURFACES: [TenantSecuritySurfaceContract; 6] = [
    TenantSecuritySurfaceContract {
        surface: TenantSecuritySurface::DriveDomain,
        owning_crate: "crates/oya-office-drive-domain",
        evidence: "Drive metadata, ACL, KMS-shred, version/trash/quota, and G080 contract gates remain tenant/object scoped and audit-bound.",
        requires_request_context: true,
        requires_authorization_before_data: true,
        requires_audit_event: true,
        requires_redaction_or_bounded_output: true,
        launch_blocking: true,
    },
    TenantSecuritySurfaceContract {
        surface: TenantSecuritySurface::DriveApi,
        owning_crate: "crates/oya-office-drive-api + apps/oya-office-drive-api",
        evidence: "Drive request/route/event contracts carry tenant id, object id, data class, sequence number, and bounded page limits.",
        requires_request_context: true,
        requires_authorization_before_data: true,
        requires_audit_event: true,
        requires_redaction_or_bounded_output: true,
        launch_blocking: true,
    },
    TenantSecuritySurfaceContract {
        surface: TenantSecuritySurface::Search,
        owning_crate: "crates/oya-office-search-kernel",
        evidence: "DriveSearchDocument and DriveSearchQuery are tenant scoped, data-class aware, redacted, and bounded by MAX_LIMIT.",
        requires_request_context: true,
        requires_authorization_before_data: true,
        requires_audit_event: true,
        requires_redaction_or_bounded_output: true,
        launch_blocking: true,
    },
    TenantSecuritySurfaceContract {
        surface: TenantSecuritySurface::Collaboration,
        owning_crate: "crates/oya-office-collab-domain",
        evidence: "Collab operation, snapshot, state-vector, awareness, replay, and load contracts reject tenant/object/sequence mismatches.",
        requires_request_context: true,
        requires_authorization_before_data: true,
        requires_audit_event: true,
        requires_redaction_or_bounded_output: true,
        launch_blocking: true,
    },
    TenantSecuritySurfaceContract {
        surface: TenantSecuritySurface::ObjectStorage,
        owning_crate: "crates/oya-office-storage-kernel",
        evidence: "UploadIntent and DownloadIntent are tenant/object/data-class scoped; storage keys are normalized and byte ranges are bounded.",
        requires_request_context: true,
        requires_authorization_before_data: true,
        requires_audit_event: true,
        requires_redaction_or_bounded_output: true,
        launch_blocking: true,
    },
    TenantSecuritySurfaceContract {
        surface: TenantSecuritySurface::TenantControlPlane,
        owning_crate: "crates/oya-office-tenant-domain",
        evidence: "Tenant onboarding, admin assignments, billing/quota/rate-limit, and abuse controls fail closed across tenant boundaries.",
        requires_request_context: true,
        requires_authorization_before_data: true,
        requires_audit_event: true,
        requires_redaction_or_bounded_output: true,
        launch_blocking: true,
    },
];

/// Returns all launch-blocking G083 tenant/security gates.
#[must_use]
pub const fn g083_tenant_security_baseline_gates() -> &'static [TenantSecurityBaselineGate] {
    G083_TENANT_SECURITY_BASELINE_GATES.as_slice()
}

/// Returns cross-surface G083 tenant/security contracts.
#[must_use]
pub const fn g083_tenant_security_surfaces() -> &'static [TenantSecuritySurfaceContract] {
    G083_TENANT_SECURITY_SURFACES.as_slice()
}

/// Drive-specific action vocabulary for ACL policy decisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveAction {
    /// Read or preview object metadata/content.
    Read,
    /// Modify object metadata/content.
    Write,
    /// Share or revoke access.
    Share,
    /// Export or download object content.
    Export,
    /// Delete, trash, or lifecycle-remove an object.
    Delete,
}

impl DriveAction {
    fn audit_action(self) -> AuditAction {
        match self {
            Self::Read => AuditAction::DriveRead,
            Self::Write => AuditAction::DriveWrite,
            Self::Share => AuditAction::DriveShare,
            Self::Export => AuditAction::DriveExport,
            Self::Delete => AuditAction::DriveDelete,
        }
    }
}

/// Drive ACL role compatible with Workspace-style sharing semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AclRole {
    /// May view metadata/content only.
    Viewer,
    /// May view and comment; comment contracts land later.
    Commenter,
    /// May view and modify content.
    Editor,
    /// Full object owner permissions, including transfer/delete.
    Owner,
}

impl AclRole {
    /// Returns true when this role allows the Drive action.
    #[must_use]
    pub const fn allows_drive_action(self, action: DriveAction) -> bool {
        matches!(
            (self, action),
            (_, DriveAction::Read)
                | (
                    Self::Editor | Self::Owner,
                    DriveAction::Write | DriveAction::Export
                )
                | (Self::Owner, DriveAction::Share | DriveAction::Delete)
        )
    }
}

/// Resource kind known to the early authz contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    /// Drive object resource.
    DriveObject,
    /// Tenant control-plane resource.
    Tenant,
    /// Document editor resource.
    Document,
    /// Spreadsheet editor resource.
    Sheet,
    /// Presentation editor resource.
    Slide,
}

/// Tenant-scoped resource reference for authorization and audit projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceRef {
    tenant_id: TenantId,
    kind: ResourceKind,
    object_id: Option<ObjectId>,
    data_class: DataClass,
}

impl ResourceRef {
    /// Creates a Drive object resource reference.
    #[must_use]
    pub const fn drive_object(
        tenant_id: TenantId,
        object_id: ObjectId,
        data_class: DataClass,
    ) -> Self {
        Self {
            tenant_id,
            kind: ResourceKind::DriveObject,
            object_id: Some(object_id),
            data_class,
        }
    }

    /// Returns the resource tenant.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the resource kind.
    #[must_use]
    pub const fn kind(&self) -> ResourceKind {
        self.kind
    }

    /// Returns the optional object id.
    #[must_use]
    pub const fn object_id(&self) -> Option<&ObjectId> {
        self.object_id.as_ref()
    }

    /// Returns the resource data class.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }
}

/// Authorization request connecting context, resource, and action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizationRequest {
    context: RequestContext,
    resource: ResourceRef,
    action: DriveAction,
}

impl AuthorizationRequest {
    /// Creates an authorization request.
    #[must_use]
    pub const fn new(context: RequestContext, resource: ResourceRef, action: DriveAction) -> Self {
        Self {
            context,
            resource,
            action,
        }
    }

    /// Returns request context.
    #[must_use]
    pub const fn context(&self) -> &RequestContext {
        &self.context
    }

    /// Returns the resource under decision.
    #[must_use]
    pub const fn resource(&self) -> &ResourceRef {
        &self.resource
    }

    /// Returns the requested action.
    #[must_use]
    pub const fn action(&self) -> DriveAction {
        self.action
    }
}

/// Returns true when the request tenant matches the resource tenant.
///
/// This precheck must run before ACL role evaluation or adapter-specific
/// policy decisions. It deliberately lives in the domain layer so API, search,
/// collaboration, and storage adapters share the same fail-closed invariant.
#[must_use]
pub fn request_tenant_matches_resource(request: &AuthorizationRequest) -> bool {
    request.context().tenant_id() == request.resource().tenant_id()
}

/// Authorization decision produced by a policy or ACL evaluator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizationDecision {
    outcome: AuditOutcome,
    policy_id: String,
    reason: Option<String>,
}

impl AuthorizationDecision {
    /// Creates an allow decision.
    #[must_use]
    pub fn allow(policy_id: impl Into<String>) -> Self {
        Self {
            outcome: AuditOutcome::Allowed,
            policy_id: policy_id.into(),
            reason: None,
        }
    }

    /// Creates a deny decision with a human-readable policy reason.
    #[must_use]
    pub fn deny(policy_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            outcome: AuditOutcome::Denied,
            policy_id: policy_id.into(),
            reason: Some(reason.into()),
        }
    }

    /// Returns true when the decision allows the action.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self.outcome, AuditOutcome::Allowed)
    }

    /// Returns true when the decision denies the action.
    #[must_use]
    pub const fn is_denied(&self) -> bool {
        matches!(self.outcome, AuditOutcome::Denied)
    }

    /// Returns the policy identifier that produced the decision.
    #[must_use]
    pub fn policy_id(&self) -> &str {
        self.policy_id.as_str()
    }

    /// Returns the optional decision reason.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    /// Projects the decision into the shared immutable audit event shape.
    #[must_use]
    pub fn to_audit_event(
        &self,
        request: &AuthorizationRequest,
        event_id: RequestId,
    ) -> AuditEvent {
        AuditEvent::new(AuditEventInput {
            event_id,
            request_id: request.context().request_id().clone(),
            tenant_id: request.context().tenant_id().clone(),
            actor: request.context().principal_id().clone(),
            action: request.action().audit_action(),
            resource: request.resource().object_id().cloned(),
            data_class: request.resource().data_class(),
            outcome: self.outcome,
            reason: self.reason.clone(),
        })
    }
}

/// Pure authorization interface for future policy-engine adapters.
pub trait Authorizer {
    /// Returns an authorization decision for the request.
    fn authorize(&self, request: &AuthorizationRequest) -> AuthorizationDecision;
}

/// G076 tenant/security threat-model review identifier.
pub const G076_TENANT_SECURITY_THREAT_MODEL_REVIEW_ID: &str =
    "g076-tenant-security-threat-model-v1";

/// Threat categories that must stay covered before tenant/security launch claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ThreatModelRiskKind {
    /// A request crosses from one tenant boundary into another tenant's resource.
    CrossTenantAccess,
    /// Authenticated users access objects without object-level authorization.
    BrokenObjectLevelAuthorization,
    /// Public responses leak private object properties or adapter internals.
    ObjectPropertyExposure,
    /// Shredded or revoked KMS material is bypassed for content access.
    KmsShredBypass,
    /// An allow/deny decision is missing immutable audit projection.
    AuditGap,
    /// Secrets, payment data, raw provider tokens, or credentials enter product state.
    SecretLeakage,
    /// Automation, export, SDK, webhook, or format actions bypass tenant bounds.
    UnboundedAutomation,
    /// Sharing, admin, billing, or abuse actions grant cross-tenant or elevated rights.
    UnauthorizedPrivilegeChange,
}

impl ThreatModelRiskKind {
    /// Returns the stable threat label used by docs and static review gates.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CrossTenantAccess => "cross-tenant-access",
            Self::BrokenObjectLevelAuthorization => "broken-object-level-authorization",
            Self::ObjectPropertyExposure => "object-property-exposure",
            Self::KmsShredBypass => "kms-shred-bypass",
            Self::AuditGap => "audit-gap",
            Self::SecretLeakage => "secret-leakage",
            Self::UnboundedAutomation => "unbounded-automation",
            Self::UnauthorizedPrivilegeChange => "unauthorized-privilege-change",
        }
    }
}

/// One launch-blocking control in the tenant/security threat model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ThreatModelControl {
    risk: ThreatModelRiskKind,
    required_gate: &'static str,
    baseline_evidence: &'static str,
    launch_blocking: bool,
}

impl ThreatModelControl {
    /// Creates a static threat-model control row.
    #[must_use]
    pub const fn new(
        risk: ThreatModelRiskKind,
        required_gate: &'static str,
        baseline_evidence: &'static str,
        launch_blocking: bool,
    ) -> Self {
        Self {
            risk,
            required_gate,
            baseline_evidence,
            launch_blocking,
        }
    }

    /// Returns the risk covered by this control.
    #[must_use]
    pub const fn risk(self) -> ThreatModelRiskKind {
        self.risk
    }

    /// Returns the gate that must pass before release claims.
    #[must_use]
    pub const fn required_gate(self) -> &'static str {
        self.required_gate
    }

    /// Returns the current baseline evidence anchor.
    #[must_use]
    pub const fn baseline_evidence(self) -> &'static str {
        self.baseline_evidence
    }

    /// Returns true when missing evidence blocks launch/readiness claims.
    #[must_use]
    pub const fn is_launch_blocking(self) -> bool {
        self.launch_blocking
    }
}

const G076_TENANT_SECURITY_CONTROLS: [ThreatModelControl; 8] = [
    ThreatModelControl::new(
        ThreatModelRiskKind::CrossTenantAccess,
        "request tenant must match resource tenant before ACL evaluation",
        "DriveSecurityPolicy::authorize_drive_object cross-tenant denial and G065 tests",
        true,
    ),
    ThreatModelControl::new(
        ThreatModelRiskKind::BrokenObjectLevelAuthorization,
        "Drive action must be authorized through tenant/object ACL or stricter policy",
        "AclRole::allows_drive_action plus Drive ACL owner/editor/viewer tests",
        true,
    ),
    ThreatModelControl::new(
        ThreatModelRiskKind::ObjectPropertyExposure,
        "public metadata views must redact storage pointers, KMS refs, content, and internals",
        "DriveObjectMetadata::public_view and G065 object metadata privacy test",
        true,
    ),
    ThreatModelControl::new(
        ThreatModelRiskKind::KmsShredBypass,
        "content/export actions must deny when the Drive object's KMS key is shredded",
        "DriveSecurityPolicy::authorize_drive_object KMS-shred denial test",
        true,
    ),
    ThreatModelControl::new(
        ThreatModelRiskKind::AuditGap,
        "every allow/deny decision must project request, tenant, actor, action, resource, data class, outcome, and reason",
        "AuthorizationDecision::to_audit_event and Drive security audit tests",
        true,
    ),
    ThreatModelControl::new(
        ThreatModelRiskKind::SecretLeakage,
        "tenant billing/provider references must reject raw secrets, API keys, tokens, passwords, and payment data",
        "BillingReference validation and tenant control-plane contract",
        true,
    ),
    ThreatModelControl::new(
        ThreatModelRiskKind::UnboundedAutomation,
        "SDK, webhook, export, and format actions must be tenant-bound, bounded, and audited",
        "Sheets API contracts, Drive export authorization, and format benchmark no-claim gates",
        true,
    ),
    ThreatModelControl::new(
        ThreatModelRiskKind::UnauthorizedPrivilegeChange,
        "share, admin, billing, and abuse-control mutations require tenant-scoped roles before state changes",
        "TenantAdminAssignment, billing authority, abuse policy, and Drive share ACL contracts",
        true,
    ),
];

/// Code-reviewer threat-model assessment for the tenant/security baseline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ThreatModelReview {
    review_id: &'static str,
    controls: &'static [ThreatModelControl],
    runtime_peer_dependency_allowed: bool,
    production_claim_allowed: bool,
}

impl ThreatModelReview {
    /// Creates a static threat-model review.
    #[must_use]
    pub const fn new(
        review_id: &'static str,
        controls: &'static [ThreatModelControl],
        runtime_peer_dependency_allowed: bool,
        production_claim_allowed: bool,
    ) -> Self {
        Self {
            review_id,
            controls,
            runtime_peer_dependency_allowed,
            production_claim_allowed,
        }
    }

    /// Returns the review identifier.
    #[must_use]
    pub const fn review_id(self) -> &'static str {
        self.review_id
    }

    /// Returns launch-blocking control rows.
    #[must_use]
    pub const fn controls(self) -> &'static [ThreatModelControl] {
        self.controls
    }

    /// Returns true when the review includes the given risk.
    #[must_use]
    pub fn covers_risk(self, risk: ThreatModelRiskKind) -> bool {
        self.controls.iter().any(|control| control.risk() == risk)
    }

    /// Returns true when every current control remains launch-blocking.
    #[must_use]
    pub fn all_controls_block_launch(self) -> bool {
        self.controls
            .iter()
            .all(|control| control.is_launch_blocking())
    }

    /// Returns true when runtime Google Workspace/ONLYOFFICE dependencies are allowed.
    #[must_use]
    pub const fn runtime_peer_dependency_allowed(self) -> bool {
        self.runtime_peer_dependency_allowed
    }

    /// Returns true when this review grants production readiness.
    #[must_use]
    pub const fn production_claim_allowed(self) -> bool {
        self.production_claim_allowed
    }
}

/// Returns the G076 code-reviewer tenant/security threat-model review.
#[must_use]
pub const fn g076_tenant_security_threat_model() -> ThreatModelReview {
    ThreatModelReview::new(
        G076_TENANT_SECURITY_THREAT_MODEL_REVIEW_ID,
        &G076_TENANT_SECURITY_CONTROLS,
        false,
        false,
    )
}

/// G090 tenant/security executable-test evidence marker.
pub const G090_TENANT_SECURITY_TEST_EVIDENCE_VERSION: &str = "g090-tenant-security-tests-v1";

/// One named tenant/security test that must remain executable through Buck2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TenantSecurityTestEvidence {
    buck_target: &'static str,
    crate_path: &'static str,
    test_case: &'static str,
    coverage: &'static str,
    launch_blocking: bool,
    runtime_peer_dependency_allowed: bool,
    production_claim_allowed: bool,
}

impl TenantSecurityTestEvidence {
    /// Creates a static test-evidence row.
    #[must_use]
    pub const fn new(
        buck_target: &'static str,
        crate_path: &'static str,
        test_case: &'static str,
        coverage: &'static str,
        launch_blocking: bool,
    ) -> Self {
        Self {
            buck_target,
            crate_path,
            test_case,
            coverage,
            launch_blocking,
            runtime_peer_dependency_allowed: false,
            production_claim_allowed: false,
        }
    }

    /// Returns the Buck2 test target that executes this case.
    #[must_use]
    pub const fn buck_target(self) -> &'static str {
        self.buck_target
    }

    /// Returns the source file containing the test case.
    #[must_use]
    pub const fn crate_path(self) -> &'static str {
        self.crate_path
    }

    /// Returns the stable test function name.
    #[must_use]
    pub const fn test_case(self) -> &'static str {
        self.test_case
    }

    /// Returns the tenant/security behavior covered by the test.
    #[must_use]
    pub const fn coverage(self) -> &'static str {
        self.coverage
    }

    /// Returns true when this test remains launch-blocking.
    #[must_use]
    pub const fn launch_blocking(self) -> bool {
        self.launch_blocking
    }

    /// Returns true when this row allows runtime Google Workspace/ONLYOFFICE dependencies.
    #[must_use]
    pub const fn runtime_peer_dependency_allowed(self) -> bool {
        self.runtime_peer_dependency_allowed
    }

    /// Returns true when this row grants production readiness.
    #[must_use]
    pub const fn production_claim_allowed(self) -> bool {
        self.production_claim_allowed
    }
}

const G090_TENANT_SECURITY_TESTS: [TenantSecurityTestEvidence; 15] = [
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-authz-domain:test",
        "crates/oya-office-authz-domain/src/lib.rs",
        "viewer_role_can_read_but_not_export",
        "ACL role vocabulary denies export/delete privileges to reader-only principals",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-authz-domain:test",
        "crates/oya-office-authz-domain/src/lib.rs",
        "denied_decision_projects_to_audit_event",
        "denied authorization decisions project tenant, actor, action, and reason into audit evidence",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-authz-domain:test",
        "crates/oya-office-authz-domain/src/lib.rs",
        "g083_tenant_security_baseline_covers_drive_api_search_collab_and_storage",
        "Drive/API/search/collab/storage/tenant-control-plane baseline requires request context, authz before data, audit, redaction, KMS-shred, quota, and bounded-operation gates",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-authz-domain:test",
        "crates/oya-office-authz-domain/src/lib.rs",
        "g083_cross_tenant_precheck_denies_before_acl_and_projects_audit",
        "cross-tenant requests deny before ACL evaluation and still emit audit evidence",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-authz-domain:test",
        "crates/oya-office-authz-domain/src/lib.rs",
        "g076_tenant_security_threat_model_covers_baseline_risks",
        "threat-model controls cover cross-tenant access, object authz, redaction, KMS, audit, secrets, automation, and privilege-change risks",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-authz-domain:test",
        "crates/oya-office-authz-domain/src/lib.rs",
        "g076_threat_model_blocks_launch_claims_without_later_security_evidence",
        "threat-model gates block launch claims until later runtime/browser/audit-pipeline evidence exists",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-tenant-domain:test",
        "crates/oya-office-tenant-domain/src/lib.rs",
        "quota_policy_denies_storage_overage",
        "tenant quota rejects storage overage",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-tenant-domain:test",
        "crates/oya-office-tenant-domain/src/lib.rs",
        "admin_assignment_rejects_cross_tenant_context",
        "tenant admin assignment rejects cross-tenant request context",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-tenant-domain:test",
        "crates/oya-office-tenant-domain/src/lib.rs",
        "billing_reference_rejects_secret_like_values",
        "tenant billing references reject secret-like values",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-tenant-domain:test",
        "crates/oya-office-tenant-domain/src/lib.rs",
        "onboarding_plan_rejects_billing_quota_below_tenant_plan",
        "tenant onboarding blocks billing plans whose quotas cannot cover tenant plan limits",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-tenant-domain:test",
        "crates/oya-office-tenant-domain/src/lib.rs",
        "abuse_policy_suspends_critical_tenant_signal",
        "critical abuse signals suspend tenant traffic",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-drive-domain:test",
        "crates/oya-office-drive-domain/src/lib.rs",
        "cross_tenant_access_is_denied_and_audited",
        "Drive domain denies cross-tenant object access and audits the denial",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-drive-domain:test",
        "crates/oya-office-drive-domain/src/lib.rs",
        "drive_authorization_allows_owner_and_emits_complete_audit_event",
        "Drive owner authorization emits complete audit event fields",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-drive-domain:test",
        "crates/oya-office-drive-domain/src/lib.rs",
        "shredded_kms_key_blocks_content_access_even_for_owner",
        "KMS-shredded Drive content denies even owner access",
        true,
    ),
    TenantSecurityTestEvidence::new(
        "//crates/oya-office-drive-domain:test",
        "crates/oya-office-drive-domain/src/lib.rs",
        "object_metadata_public_view_does_not_expose_storage_pointer_or_kms_ref",
        "public object metadata redacts storage pointer and KMS reference internals",
        true,
    ),
];

/// Returns the G090 tenant/security executable-test evidence rows.
#[must_use]
pub const fn g090_tenant_security_tests() -> &'static [TenantSecurityTestEvidence] {
    G090_TENANT_SECURITY_TESTS.as_slice()
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
mod authz_contract_tests {
    use oya_office_kernel::{
        AuditOutcome, CellId, DataClass, ObjectId, PrincipalId, RequestContext, RequestId, TenantId,
    };

    use super::{
        AclRole, AuthorizationDecision, AuthorizationRequest, DriveAction,
        G083_TENANT_SECURITY_BASELINE_VERSION, G090_TENANT_SECURITY_TEST_EVIDENCE_VERSION,
        ResourceRef, TenantSecurityBaselineGateKind, TenantSecuritySurface, ThreatModelRiskKind,
        g076_tenant_security_threat_model, g083_tenant_security_baseline_gates,
        g083_tenant_security_surfaces, g090_tenant_security_tests, request_tenant_matches_resource,
    };

    #[test]
    fn viewer_role_can_read_but_not_export() {
        assert!(AclRole::Viewer.allows_drive_action(DriveAction::Read));
        assert!(!AclRole::Viewer.allows_drive_action(DriveAction::Export));
        assert!(AclRole::Owner.allows_drive_action(DriveAction::Delete));
    }

    #[test]
    fn denied_decision_projects_to_audit_event() {
        let tenant_id = TenantId::new("tenant-alpha").expect("valid tenant id");
        let actor = PrincipalId::new("user-123").expect("valid principal id");
        let context = RequestContext::new(
            RequestId::new("req-123").expect("valid request id"),
            tenant_id.clone(),
            actor.clone(),
            CellId::new("iad-1").expect("valid cell id"),
        );
        let resource = ResourceRef::drive_object(
            tenant_id,
            ObjectId::new("drive-object-1").expect("valid object id"),
            DataClass::Confidential,
        );
        let request = AuthorizationRequest::new(context, resource, DriveAction::Export);
        let decision =
            AuthorizationDecision::deny("drive-acl", "viewer cannot export confidential object");

        let audit =
            decision.to_audit_event(&request, RequestId::new("evt-1").expect("valid event id"));

        assert_eq!(audit.outcome(), AuditOutcome::Denied);
        assert_eq!(audit.tenant_id().as_str(), "tenant-alpha");
        assert_eq!(audit.actor().as_str(), "user-123");
        assert_eq!(
            audit.reason(),
            Some("viewer cannot export confidential object")
        );
    }

    #[test]
    fn g083_tenant_security_baseline_covers_drive_api_search_collab_and_storage() {
        assert_eq!(
            G083_TENANT_SECURITY_BASELINE_VERSION,
            "g083-tenant-security-baseline-v1"
        );

        let gates = g083_tenant_security_baseline_gates();
        for required in [
            TenantSecurityBaselineGateKind::RequestContext,
            TenantSecurityBaselineGateKind::TenantBoundaryPrecheck,
            TenantSecurityBaselineGateKind::ObjectAuthorization,
            TenantSecurityBaselineGateKind::AuditProjection,
            TenantSecurityBaselineGateKind::Redaction,
            TenantSecurityBaselineGateKind::KmsShred,
            TenantSecurityBaselineGateKind::QuotaRateLimitAbuse,
            TenantSecurityBaselineGateKind::BoundedOperation,
        ] {
            assert!(
                gates
                    .iter()
                    .any(|gate| gate.kind() == required && gate.launch_blocking()),
                "missing launch-blocking G083 gate: {required:?}"
            );
        }

        let surfaces = g083_tenant_security_surfaces();
        for surface in [
            TenantSecuritySurface::DriveDomain,
            TenantSecuritySurface::DriveApi,
            TenantSecuritySurface::Search,
            TenantSecuritySurface::Collaboration,
            TenantSecuritySurface::ObjectStorage,
            TenantSecuritySurface::TenantControlPlane,
        ] {
            assert!(
                surfaces.iter().any(|contract| contract.surface() == surface
                    && contract.launch_blocking()
                    && contract.requires_request_context()
                    && contract.requires_authorization_before_data()
                    && contract.requires_audit_event()
                    && contract.requires_redaction_or_bounded_output()
                    && !contract.owning_crate().is_empty()
                    && !contract.evidence().is_empty()),
                "missing complete G083 surface contract: {surface:?}"
            );
        }
    }

    #[test]
    fn g083_cross_tenant_precheck_denies_before_acl_and_projects_audit() {
        let context = RequestContext::new(
            RequestId::new("req-g083-cross-tenant").expect("valid request id"),
            TenantId::new("tenant-beta").expect("valid tenant id"),
            PrincipalId::new("user-tenant-beta").expect("valid principal id"),
            CellId::new("iad-1").expect("valid cell id"),
        );
        let resource = ResourceRef::drive_object(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("drive-object-alpha").expect("valid object id"),
            DataClass::Confidential,
        );
        let request = AuthorizationRequest::new(context, resource, DriveAction::Read);

        assert!(!request_tenant_matches_resource(&request));
        assert!(
            AclRole::Owner.allows_drive_action(request.action()),
            "tenant precheck must run before an otherwise permissive ACL role"
        );

        let decision = AuthorizationDecision::deny(
            "g083-tenant-boundary",
            "request tenant does not match resource tenant",
        );
        let audit = decision.to_audit_event(
            &request,
            RequestId::new("evt-g083-cross-tenant").expect("valid event id"),
        );

        assert!(decision.is_denied());
        assert_eq!(decision.policy_id(), "g083-tenant-boundary");
        assert_eq!(audit.tenant_id().as_str(), "tenant-beta");
        assert_eq!(audit.actor().as_str(), "user-tenant-beta");
        assert_eq!(audit.action(), oya_office_kernel::AuditAction::DriveRead);
        assert_eq!(
            audit.resource().map(oya_office_kernel::ObjectId::as_str),
            Some("drive-object-alpha")
        );
        assert_eq!(audit.data_class(), DataClass::Confidential);
        assert_eq!(audit.outcome(), AuditOutcome::Denied);
        assert_eq!(
            audit.reason(),
            Some("request tenant does not match resource tenant")
        );
    }

    #[test]
    fn g076_tenant_security_threat_model_covers_baseline_risks() {
        let review = g076_tenant_security_threat_model();
        let risks = [
            ThreatModelRiskKind::CrossTenantAccess,
            ThreatModelRiskKind::BrokenObjectLevelAuthorization,
            ThreatModelRiskKind::ObjectPropertyExposure,
            ThreatModelRiskKind::KmsShredBypass,
            ThreatModelRiskKind::AuditGap,
            ThreatModelRiskKind::SecretLeakage,
            ThreatModelRiskKind::UnboundedAutomation,
            ThreatModelRiskKind::UnauthorizedPrivilegeChange,
        ];

        assert_eq!(review.review_id(), "g076-tenant-security-threat-model-v1");
        assert_eq!(review.controls().len(), risks.len());
        assert!(risks.iter().all(|risk| review.covers_risk(*risk)));
        assert!(review.all_controls_block_launch());
        assert!(!review.runtime_peer_dependency_allowed());
        assert!(!review.production_claim_allowed());
        assert!(review.controls().iter().all(|control| {
            !control.risk().as_str().is_empty()
                && !control.required_gate().is_empty()
                && !control.baseline_evidence().is_empty()
        }));
    }

    #[test]
    fn g076_threat_model_blocks_launch_claims_without_later_security_evidence() {
        let review = g076_tenant_security_threat_model();

        assert!(review.covers_risk(ThreatModelRiskKind::CrossTenantAccess));
        assert!(review.covers_risk(ThreatModelRiskKind::UnboundedAutomation));
        assert!(review.covers_risk(ThreatModelRiskKind::UnauthorizedPrivilegeChange));
        assert!(review.controls().iter().all(|control| {
            control.is_launch_blocking()
                && !control.required_gate().is_empty()
                && !control
                    .baseline_evidence()
                    .contains("Google Workspace runtime")
                && !control.baseline_evidence().contains("ONLYOFFICE runtime")
        }));
    }

    #[test]
    fn g090_tenant_security_tests_exist() {
        assert_eq!(
            G090_TENANT_SECURITY_TEST_EVIDENCE_VERSION,
            "g090-tenant-security-tests-v1"
        );

        let evidence = g090_tenant_security_tests();
        assert!(evidence.len() >= 15);

        for required_target in [
            "//crates/oya-office-authz-domain:test",
            "//crates/oya-office-tenant-domain:test",
            "//crates/oya-office-drive-domain:test",
        ] {
            assert!(
                evidence
                    .iter()
                    .any(|row| row.buck_target() == required_target),
                "missing tenant/security Buck2 target: {required_target}"
            );
        }

        for required_case in [
            "g083_cross_tenant_precheck_denies_before_acl_and_projects_audit",
            "admin_assignment_rejects_cross_tenant_context",
            "billing_reference_rejects_secret_like_values",
            "cross_tenant_access_is_denied_and_audited",
            "shredded_kms_key_blocks_content_access_even_for_owner",
        ] {
            assert!(
                evidence.iter().any(|row| row.test_case() == required_case
                    && row.launch_blocking()
                    && !row.crate_path().is_empty()
                    && !row.coverage().is_empty()),
                "missing launch-blocking tenant/security test evidence: {required_case}"
            );
        }

        assert!(
            evidence.iter().all(|row| {
                row.launch_blocking()
                    && !row.runtime_peer_dependency_allowed()
                    && !row.production_claim_allowed()
            }),
            "G090 rows must stay launch-blocking and cannot claim peer runtime or production readiness"
        );
    }
}
