//! # tenancy-tenant-lifecycle-authz-port
//!
//! The authorization DECISION PORT the tenant-lifecycle delivery surface
//! depends on (AUTH-005, ADR-0564 D7). The tenancy facade is a Policy
//! Enforcement Point (PEP): it authenticates the caller, assembles the
//! authorization request, asks this port for a decision, and enforces it.
//!
//! ## Posture (fail-closed, default-deny)
//!
//! - Every decision is `Allow` or `Deny`; there is no "not applicable" — the
//!   absence of an explicit permit IS a deny (deny-by-default).
//! - Any error a backing engine raises is fail-closed: the PEP MUST treat a
//!   [`Result::Err`] as a deny, never as an allow or a bypass.
//! - The verified bearer principal NEVER on its own grants the tenant axis: a
//!   per-tenant action authorizes the caller against the TARGET tenant id, and
//!   the platform-admin axis is a distinct scope from any tenant scope.
//!
//! ## Layering (ADR-0131 / ADR-0562 faces)
//!
//! This is a PORT crate: it depends only on the locked PDP contract family in
//! `oya-shared-platform-contracts-kernel`. It has ZERO dependency on any
//! adapter or facade — the Cedar-backed PDP adapter and the axum facade both
//! depend INWARD on this port. Face-direction review ("would this trait change
//! at W5 cutover?"): no — it models the destination decision surface (caller +
//! action + target tenant in, attributable allow/deny out), not any transient
//! engine detail.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

/// The authenticated caller a PEP presents to the authorizer. Construction is
/// the PEP's job: the caller is materialized ONLY from a verified credential
/// (e.g. a constant-time-checked bearer principal), never from an unverified
/// URL path segment or a self-asserted header alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallerIdentity {
    /// Stable principal id of the verified caller (e.g. `platform-admin`,
    /// or a tenant operator principal).
    pub principal_id: String,
    /// The tenant axis the caller has proven authority over, when the caller
    /// is tenant-scoped. `None` for a platform-scoped caller. A tenant-scoped
    /// caller can never satisfy a platform-admin action, and vice versa.
    pub tenant_scope: Option<String>,
    /// Whether the caller holds the platform-admin scope (the cross-tenant
    /// control-plane axis required to register or enumerate tenants).
    pub platform_admin: bool,
}

/// A fail-closed membership-resolution fault. Any backing-store error/timeout
/// maps to this and the PEP DENIES (the operator gets no proven tenant scope) —
/// a membership-store outage never grants a tenant axis (default-deny).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MembershipFault {
    detail: String,
}

impl MembershipFault {
    /// Construct a fault with a human-facing detail.
    #[must_use]
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    /// Borrow the detail string.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for MembershipFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tenant-membership resolution failed: {}", self.detail)
    }
}

impl std::error::Error for MembershipFault {}

/// SERVER-SIDE tenant-membership resolution PORT (the SECURITY remediation core).
///
/// The tenant-operator bearer is a SHARED credential; on its own it proves only
/// that the caller is *some* operator, NEVER which tenants that operator may act
/// for. A self-attested `x-oya-tenant` header therefore MUST NOT grant a tenant
/// axis (the C7 finding: an operator holding the shared bearer could select ANY
/// victim tenant via the header). This port resolves — from a TRUSTED server-side
/// source keyed on the VERIFIED operator principal — the exact set of tenants the
/// operator is assigned to. The PEP binds the tenant axis ONLY to a tenant in
/// this set; the `x-oya-tenant` header may at most SELECT among assigned tenants,
/// never grant an unassigned one.
///
/// Default-deny: an unknown operator resolves to an EMPTY membership set, so
/// every per-tenant op denies. Any backing-store fault maps to `Err` (the PEP
/// denies) — never an allow. A production adapter is the cloud-iam / OIDC
/// membership store; the in-memory seed adapter lives in the composition root.
pub trait TenantMembershipResolver: Send + Sync {
    /// Resolve the set of tenant ids the verified operator principal is assigned
    /// to. `operator_principal_id` is the VERIFIED operator's stable id (derived
    /// from the credential, never from a self-attested header).
    ///
    /// # Errors
    /// Returns [`MembershipFault`] on any backing-store failure; the PEP denies
    /// (fail-closed — the operator gets no tenant axis).
    fn assigned_tenants(&self, operator_principal_id: &str)
    -> Result<Vec<String>, MembershipFault>;
}

/// The tenancy control-plane actions guarded by this port. Each maps to a
/// stable action slug the backing engine resolves; the slug is the contract,
/// not the Rust variant name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TenantLifecycleAction {
    /// Register a new tenant (`POST /v1/tenants`) — platform-admin scope; the
    /// caller has no prior tenant to be scoped to.
    Register,
    /// List/enumerate all tenants (`GET /v1/tenants`) — platform-admin scope;
    /// the surface discloses every tenant.
    List,
    /// Read one tenant's state (`GET /v1/tenants/{id}`).
    Read,
    /// Provision a tenant (`POST /v1/tenants/{id}/provision`).
    Provision,
    /// Suspend a tenant (`POST /v1/tenants/{id}/suspend`).
    Suspend,
    /// Resume a tenant (`POST /v1/tenants/{id}/resume`).
    Resume,
    /// Retire a tenant (`DELETE /v1/tenants/{id}`) — terminal, irreversible.
    Retire,
}

impl TenantLifecycleAction {
    /// The stable action slug the backing engine resolves. Slugs are
    /// lowercase dotted (`tenancy.<verb>`), matching the locked PDP-contract
    /// slug charset.
    #[must_use]
    pub fn slug(self) -> &'static str {
        match self {
            Self::Register => "tenancy.register",
            Self::List => "tenancy.list",
            Self::Read => "tenancy.read",
            Self::Provision => "tenancy.provision",
            Self::Suspend => "tenancy.suspend",
            Self::Resume => "tenancy.resume",
            Self::Retire => "tenancy.retire",
        }
    }

    /// Whether this action requires the platform-admin (cross-tenant) scope
    /// rather than authority over one target tenant. Register and List operate
    /// over the whole control plane, not a single tenant.
    #[must_use]
    pub fn is_platform_scoped(self) -> bool {
        matches!(self, Self::Register | Self::List)
    }
}

/// What the PEP is asking the authorizer to decide: a verified caller acting
/// on a target. Per-tenant actions carry the target tenant id (the URL `{id}`,
/// which by itself authorizes NOTHING). Platform-scoped actions carry `None`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizationQuery<'a> {
    pub caller: &'a CallerIdentity,
    pub action: TenantLifecycleAction,
    /// The target tenant id for a per-tenant action; `None` for a
    /// platform-scoped action (register/list).
    pub target_tenant_id: Option<&'a str>,
}

/// The decision the authorizer reached. Exactly two outcomes (deny-by-default).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthorizationDecision {
    Allow,
    Deny,
}

impl AuthorizationDecision {
    /// Whether the decision permits the request.
    #[must_use]
    pub fn is_allow(self) -> bool {
        matches!(self, Self::Allow)
    }
}

/// The attributable outcome returned by the authorizer: the decision plus the
/// forensic fields needed for audit. Every call to `authorize` produces an
/// [`AuthorizationOutcome`]; the PEP MUST emit a structured audit record from it
/// (message `"tenancy.authz.decision"`) so EVERY decision — allow and deny — is
/// traceable. Discarding the outcome fields is a policy violation (AC-W-13).
///
/// `decision_id` is a ULID minted by the backing PDP engine for this decision;
/// `determining_policy_ids` are the Cedar policy ids that drove the outcome
/// (empty on a deny-by-default where no policy matched).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizationOutcome {
    pub decision: AuthorizationDecision,
    /// Opaque, globally unique id for this decision (PDP-minted ULID).
    /// Non-empty on every successful call. Key the audit trail on this id.
    pub decision_id: String,
    /// The Cedar policy ids that determined the outcome. Non-empty on an
    /// explicit allow; may be empty on a deny-by-default (no matching permit)
    /// but non-empty when a forbid drove the deny.
    pub determining_policy_ids: Vec<String>,
}

/// Why the authorizer could not return a clean decision. EVERY variant is
/// fail-closed: a PEP MUST treat an [`AuthzError`] as a deny.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthzError {
    /// The query was malformed (e.g. a per-tenant action with no target, or a
    /// target id that is not a valid slug). The PEP denies.
    InvalidQuery(String),
    /// The backing decision engine refused to decide (bundle/evaluation
    /// failure). The PEP denies (fail-closed), never bypasses.
    EngineRefused(String),
}

impl fmt::Display for AuthzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidQuery(detail) => write!(f, "invalid authorization query: {detail}"),
            Self::EngineRefused(detail) => write!(f, "authorization engine refused: {detail}"),
        }
    }
}

impl std::error::Error for AuthzError {}

/// The authorization decision port. The facade (PEP) depends on this trait and
/// is wired to a concrete adapter (the Cedar-backed PDP) at the composition
/// root. `Send + Sync` so axum handlers can share one instance behind an
/// `Arc`.
///
/// Implementations evaluate deny-by-default and forbid-overrides-permit: a
/// cross-tenant request (a caller scoped to tenant A acting on tenant B) MUST
/// receive [`AuthorizationDecision::Deny`] regardless of any matching permit.
pub trait TenantLifecycleAuthorizer: Send + Sync {
    /// Decide one authorization query. The returned [`AuthorizationOutcome`]
    /// carries both the decision AND the forensic audit fields; the PEP MUST
    /// emit a structured audit record from the outcome for EVERY call.
    ///
    /// `Ok(outcome)` where `outcome.decision == Deny` AND `Err(_)` are BOTH
    /// refusals the PEP enforces; only `Ok(outcome)` where
    /// `outcome.decision == Allow` permits the request.
    ///
    /// # Errors
    /// [`AuthzError`] when the query is malformed or the backing engine
    /// refuses — the PEP treats either as a deny (fail-closed).
    fn authorize(&self, query: &AuthorizationQuery<'_>)
    -> Result<AuthorizationOutcome, AuthzError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_slugs_are_stable_and_lowercase_dotted() {
        for action in [
            TenantLifecycleAction::Register,
            TenantLifecycleAction::List,
            TenantLifecycleAction::Read,
            TenantLifecycleAction::Provision,
            TenantLifecycleAction::Suspend,
            TenantLifecycleAction::Resume,
            TenantLifecycleAction::Retire,
        ] {
            let slug = action.slug();
            assert!(slug.starts_with("tenancy."));
            assert!(
                slug.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.'),
                "slug {slug:?} must be lowercase dotted",
            );
        }
    }

    #[test]
    fn register_and_list_are_platform_scoped_others_are_not() {
        assert!(TenantLifecycleAction::Register.is_platform_scoped());
        assert!(TenantLifecycleAction::List.is_platform_scoped());
        for action in [
            TenantLifecycleAction::Read,
            TenantLifecycleAction::Provision,
            TenantLifecycleAction::Suspend,
            TenantLifecycleAction::Resume,
            TenantLifecycleAction::Retire,
        ] {
            assert!(!action.is_platform_scoped());
        }
    }

    #[test]
    fn decision_is_allow_only_for_allow() {
        assert!(AuthorizationDecision::Allow.is_allow());
        assert!(!AuthorizationDecision::Deny.is_allow());
    }

    #[test]
    fn authz_error_messages_are_legible() {
        assert!(
            AuthzError::EngineRefused("bundle rejected".to_owned())
                .to_string()
                .contains("bundle rejected")
        );
    }
}
