//! Workload-identity domain kernel — PURE (zero external dependencies).
//!
//! This crate is the canonical principal/identity state machine for *workload*
//! identities (machine-to-machine: CI runners, microservice service accounts,
//! agents) as distinct from the human-identity surface owned by the
//! `identity-oidc-issuer-*` / `identity-webauthn-*` crate families.
//!
//! Scope per task T3 (workload-identity service):
//! - the workload principal model: tenant + workload identity + claims,
//! - the authorization decision types (PARC: principal / action / resource /
//!   context request → decision),
//! - a pure principal lifecycle state machine.
//!
//! ## Layering invariant (ADR-0131 / architecture-boundaries gate)
//!
//! This is a `domain` crate. It MUST have zero dependencies (not even
//! workspace crates): the architecture-boundaries gate permits a `domain`
//! crate to import only `kernel`/`domain` peers, and this crate deliberately
//! imports none so it stays a self-contained, deterministic core. Outer
//! adapter crates (Cedar authz-gate, OIDC token validation) depend inward on
//! these types; nothing here depends outward.
//!
//! ## Determinism
//!
//! Every function here is total and deterministic: no clock, no RNG, no I/O.
//! Time is always passed in as `now_epoch_seconds`. Crypto and policy
//! evaluation live in adapter crates, never here.

// ADR-0083 Tier 3: production code stays panic-free (deny in release); inline
// `mod tests` may use unwrap/expect/panic under cfg(test) only.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::fmt;

/// Schema version for the serialized [`WorkloadPrincipal`] shape.
pub const WORKLOAD_PRINCIPAL_SCHEMA_VERSION: u32 = 1;

/// Hard ceiling on a workload credential's lifetime. Workload tokens are
/// short-lived by doctrine (ADR-0002 tenant+identity kernel): a workload
/// re-attests frequently rather than holding a long-lived secret.
pub const MAX_WORKLOAD_TOKEN_TTL_SECONDS: u64 = 60 * 60;

/// Errors produced while constructing or transitioning workload-identity
/// domain values. Exhaustive and `#[non_exhaustive]`-free on purpose: callers
/// in this workspace match all variants and the gate forbids `anyhow` in
/// domain crates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkloadIdentityError {
    /// Tenant id did not match the canonical `ten_<slug>` shape.
    InvalidTenantId,
    /// Workload id did not match the canonical `wl_<slug>` shape.
    InvalidWorkloadId,
    /// Owning-capability id did not match the canonical `cap.<dotted>` shape.
    InvalidCapabilityId,
    /// Trust domain was not a bare `spiffe://<authority>` SPIFFE trust-domain id.
    InvalidTrustDomain,
    /// A claim name was empty or contained control/whitespace characters.
    InvalidClaimName,
    /// A scope string was empty or whitespace-only.
    InvalidScope,
    /// Requested TTL was zero seconds.
    TokenTtlZero,
    /// Requested TTL exceeded [`MAX_WORKLOAD_TOKEN_TTL_SECONDS`].
    TokenTtlTooLong,
    /// Attempted an illegal lifecycle transition for the current state.
    IllegalStateTransition {
        /// State the principal was in.
        from: WorkloadState,
        /// State the caller attempted to move to.
        to: WorkloadState,
    },
}

impl fmt::Display for WorkloadIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTenantId => f.write_str("invalid tenant id (expected ten_<slug>)"),
            Self::InvalidWorkloadId => f.write_str("invalid workload id (expected wl_<slug>)"),
            Self::InvalidCapabilityId => {
                f.write_str("invalid owning-capability id (expected cap.<dotted>)")
            }
            Self::InvalidTrustDomain => {
                f.write_str("invalid trust domain (expected bare spiffe://<authority>)")
            }
            Self::InvalidClaimName => f.write_str("invalid claim name"),
            Self::InvalidScope => f.write_str("invalid scope"),
            Self::TokenTtlZero => f.write_str("token ttl must be greater than zero"),
            Self::TokenTtlTooLong => f.write_str("token ttl exceeds the workload ceiling"),
            Self::IllegalStateTransition { from, to } => {
                write!(f, "illegal workload state transition: {from:?} -> {to:?}")
            }
        }
    }
}

impl std::error::Error for WorkloadIdentityError {}

/// Canonical tenant identifier (`ten_<slug>`).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TenantId(String);

impl TenantId {
    /// Construct, validating the `ten_<slug>` shape.
    ///
    /// # Errors
    /// Returns [`WorkloadIdentityError::InvalidTenantId`] when the value does
    /// not start with `ten_` followed by at least one character.
    pub fn new(value: impl Into<String>) -> Result<Self, WorkloadIdentityError> {
        let value = value.into();
        if value.starts_with("ten_") && value.len() > 4 {
            Ok(Self(value))
        } else {
            Err(WorkloadIdentityError::InvalidTenantId)
        }
    }

    /// Borrow the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Canonical workload identifier (`wl_<slug>`).
///
/// A workload is a non-human principal: a CI job, a microservice instance, or
/// an autonomous agent. The id is stable across token rotations.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorkloadId(String);

impl WorkloadId {
    /// Construct, validating the `wl_<slug>` shape.
    ///
    /// # Errors
    /// Returns [`WorkloadIdentityError::InvalidWorkloadId`] when the value does
    /// not start with `wl_` followed by at least one character.
    pub fn new(value: impl Into<String>) -> Result<Self, WorkloadIdentityError> {
        let value = value.into();
        if value.starts_with("wl_") && value.len() > 3 {
            Ok(Self(value))
        } else {
            Err(WorkloadIdentityError::InvalidWorkloadId)
        }
    }

    /// Borrow the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkloadId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Canonical owning-capability identifier (`cap.<dotted>`), mirroring the
/// `cap.` convention already used by the human-identity service-principal model.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CapabilityId(String);

impl CapabilityId {
    /// Construct, validating the `cap.<dotted>` shape.
    ///
    /// # Errors
    /// Returns [`WorkloadIdentityError::InvalidCapabilityId`] when the value
    /// does not start with `cap.` followed by at least one character.
    pub fn new(value: impl Into<String>) -> Result<Self, WorkloadIdentityError> {
        let value = value.into();
        if value.starts_with("cap.") && value.len() > 4 {
            Ok(Self(value))
        } else {
            Err(WorkloadIdentityError::InvalidCapabilityId)
        }
    }

    /// Borrow the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// SPIFFE-shaped trust domain of a workload principal (`spiffe://<authority>`).
///
/// In the Oyatie workload model the SPIFFE trust-domain authority is the
/// tenant: every workload's SPIFFE identity is rooted at `spiffe://<tenant>`,
/// so a token minted for `ten_acme` can only ever speak for the `ten_acme`
/// trust domain. The authority segment is therefore *always* equal to the
/// owning [`TenantId`] (enforced by [`TrustDomain::for_tenant`]); this binding
/// is what lets the mesh (Istio Ambient ztunnel / SPIFFE SVID, ADR-0148) and
/// the authz layer agree on a single tenant-scoped identity root and reject a
/// cross-trust-domain (cross-tenant) token at the boundary.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TrustDomain(String);

impl TrustDomain {
    /// SPIFFE URI scheme prefix every trust domain carries.
    const SPIFFE_SCHEME: &'static str = "spiffe://";

    /// Derive the trust domain for a tenant: `spiffe://<tenant>`.
    ///
    /// This is the only constructor that builds a trust domain from identity
    /// material, and it is total: a valid [`TenantId`] always yields a valid
    /// `spiffe://ten_<slug>` authority, preserving the `trust_domain == tenant`
    /// invariant by construction.
    #[must_use]
    pub fn for_tenant(tenant: &TenantId) -> Self {
        Self(format!("{}{}", Self::SPIFFE_SCHEME, tenant.as_str()))
    }

    /// Construct from a raw `spiffe://<authority>` string, validating the SPIFFE
    /// shape (scheme present, non-empty authority, no path/query/fragment — a
    /// trust domain is the authority alone, not a full SVID path).
    ///
    /// # Errors
    /// Returns [`WorkloadIdentityError::InvalidTrustDomain`] when the value is
    /// not a bare `spiffe://<authority>` trust-domain id.
    pub fn new(value: impl Into<String>) -> Result<Self, WorkloadIdentityError> {
        let value = value.into();
        let Some(authority) = value.strip_prefix(Self::SPIFFE_SCHEME) else {
            return Err(WorkloadIdentityError::InvalidTrustDomain);
        };
        // A trust domain is the authority only: reject SVID paths/queries so a
        // full `spiffe://td/ns/sa` SVID is never mistaken for a trust domain.
        let well_formed = !authority.is_empty()
            && !authority.contains('/')
            && !authority.contains('?')
            && !authority.contains('#')
            && authority
                .chars()
                .all(|c| !c.is_whitespace() && !c.is_control());
        if well_formed {
            Ok(Self(value))
        } else {
            Err(WorkloadIdentityError::InvalidTrustDomain)
        }
    }

    /// The SPIFFE authority segment (the part after `spiffe://`).
    #[must_use]
    pub fn authority(&self) -> &str {
        self.0
            .strip_prefix(Self::SPIFFE_SCHEME)
            .unwrap_or(self.0.as_str())
    }

    /// Whether this trust domain's authority equals `tenant` — the
    /// `trust_domain == tenant` invariant the authz/mesh boundary relies on.
    #[must_use]
    pub fn matches_tenant(&self, tenant: &TenantId) -> bool {
        self.authority() == tenant.as_str()
    }

    /// Borrow the full `spiffe://<authority>` string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TrustDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A single, validated claim value carried by a workload principal.
///
/// Claims are the typed projection of the verified token payload. Keeping the
/// value variants closed (rather than a free-form JSON blob) is deliberate:
/// the authz layer matches on these without re-parsing untyped data.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ClaimValue {
    /// A textual claim (e.g. `aud`, `iss`, environment label).
    Text(String),
    /// A boolean claim (e.g. `email_verified`-style assertions).
    Bool(bool),
    /// An integer claim (e.g. a numeric trust tier).
    Int(i64),
    /// A list of textual claims (e.g. group memberships, roles).
    TextList(Vec<String>),
}

impl ClaimValue {
    /// Return the text if this is a [`ClaimValue::Text`], else `None`.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value.as_str()),
            _ => None,
        }
    }

    /// Return `true` if this is a `Bool(true)` or a `TextList`/`Text` that
    /// contains the needle. Used by the authz layer for membership tests.
    #[must_use]
    pub fn contains(&self, needle: &str) -> bool {
        match self {
            Self::Text(value) => value == needle,
            Self::TextList(values) => values.iter().any(|value| value == needle),
            Self::Bool(_) | Self::Int(_) => false,
        }
    }
}

/// Lifecycle state of a workload principal. The state machine is intentionally
/// small and explicit; transitions are validated by
/// [`WorkloadPrincipal::transition_to`].
///
/// ```text
///   Provisioned ──activate──▶ Active ──suspend──▶ Suspended ──activate──▶ Active
///        │                      │                     │
///        └──────────────────────┴─────────────────────┴──retire──▶ Retired (terminal)
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorkloadState {
    /// Registered but not yet allowed to authenticate.
    Provisioned,
    /// May authenticate and receive authorization decisions.
    Active,
    /// Temporarily blocked (e.g. anomaly detected); reversible.
    Suspended,
    /// Permanently decommissioned; terminal.
    Retired,
}

impl WorkloadState {
    /// Whether a principal in this state is permitted to authenticate and be
    /// evaluated for authorization. Only [`WorkloadState::Active`] is.
    #[must_use]
    pub fn is_operational(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Whether moving from `self` to `target` is a legal transition.
    #[must_use]
    pub fn can_transition_to(self, target: Self) -> bool {
        match (self, target) {
            // Retired is terminal: no outbound transitions.
            (Self::Retired, _) => false,
            // Activation paths.
            (Self::Provisioned | Self::Suspended, Self::Active) => true,
            // Suspension is only from Active.
            (Self::Active, Self::Suspended) => true,
            // Retirement is reachable from any non-terminal state.
            (Self::Provisioned | Self::Active | Self::Suspended, Self::Retired) => true,
            // Everything else (incl. no-op self transitions) is rejected so the
            // caller cannot silently mask a logic error.
            _ => false,
        }
    }
}

/// A verified workload principal: the typed result of authenticating a workload
/// (e.g. via [`crate`]'s downstream OIDC adapter) plus its lifecycle state and
/// claims. This is the subject the authz layer reasons about.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkloadPrincipal {
    tenant_id: TenantId,
    workload_id: WorkloadId,
    owning_capability: CapabilityId,
    trust_domain: TrustDomain,
    state: WorkloadState,
    claims: BTreeMap<String, ClaimValue>,
    scopes: Vec<String>,
    schema_version: u32,
}

impl WorkloadPrincipal {
    /// Construct a freshly-provisioned workload principal with no claims.
    ///
    /// # Errors
    /// Propagates id-validation errors from [`TenantId::new`],
    /// [`WorkloadId::new`], and [`CapabilityId::new`].
    pub fn provision(
        tenant_id: impl Into<String>,
        workload_id: impl Into<String>,
        owning_capability: impl Into<String>,
    ) -> Result<Self, WorkloadIdentityError> {
        let tenant_id = TenantId::new(tenant_id)?;
        // The trust domain is derived from (and therefore always equal to) the
        // tenant: a workload's SPIFFE identity is rooted at `spiffe://<tenant>`.
        // Deriving rather than accepting it keeps the `trust_domain == tenant`
        // invariant unbreakable at construction.
        let trust_domain = TrustDomain::for_tenant(&tenant_id);
        Ok(Self {
            tenant_id,
            workload_id: WorkloadId::new(workload_id)?,
            owning_capability: CapabilityId::new(owning_capability)?,
            trust_domain,
            state: WorkloadState::Provisioned,
            claims: BTreeMap::new(),
            scopes: Vec::new(),
            schema_version: WORKLOAD_PRINCIPAL_SCHEMA_VERSION,
        })
    }

    /// Tenant the workload belongs to.
    #[must_use]
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Stable workload identifier.
    #[must_use]
    pub fn workload_id(&self) -> &WorkloadId {
        &self.workload_id
    }

    /// Capability that owns (is accountable for) this workload.
    #[must_use]
    pub fn owning_capability(&self) -> &CapabilityId {
        &self.owning_capability
    }

    /// SPIFFE trust domain (`spiffe://<tenant>`) the workload's identity is
    /// rooted at. Always equal to the [`tenant_id`](Self::tenant_id) authority.
    #[must_use]
    pub fn trust_domain(&self) -> &TrustDomain {
        &self.trust_domain
    }

    /// Current lifecycle state.
    #[must_use]
    pub fn state(&self) -> WorkloadState {
        self.state
    }

    /// Serialized-shape schema version.
    #[must_use]
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Borrow the claim map.
    #[must_use]
    pub fn claims(&self) -> &BTreeMap<String, ClaimValue> {
        &self.claims
    }

    /// Look up a single claim by name.
    #[must_use]
    pub fn claim(&self, name: &str) -> Option<&ClaimValue> {
        self.claims.get(name)
    }

    /// Borrow the granted scopes.
    #[must_use]
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }

    /// Whether the principal currently holds `scope`.
    #[must_use]
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|granted| granted == scope)
    }

    /// Attach or overwrite a claim, validating the claim name.
    ///
    /// # Errors
    /// Returns [`WorkloadIdentityError::InvalidClaimName`] when `name` is empty
    /// or contains whitespace/control characters.
    pub fn set_claim(
        &mut self,
        name: impl Into<String>,
        value: ClaimValue,
    ) -> Result<(), WorkloadIdentityError> {
        let name = name.into();
        if name.is_empty() || name.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(WorkloadIdentityError::InvalidClaimName);
        }
        self.claims.insert(name, value);
        Ok(())
    }

    /// Builder-style claim attachment.
    ///
    /// # Errors
    /// See [`WorkloadPrincipal::set_claim`].
    pub fn with_claim(
        mut self,
        name: impl Into<String>,
        value: ClaimValue,
    ) -> Result<Self, WorkloadIdentityError> {
        self.set_claim(name, value)?;
        Ok(self)
    }

    /// Grant a scope, validating it is non-empty. Duplicate grants are
    /// idempotent.
    ///
    /// # Errors
    /// Returns [`WorkloadIdentityError::InvalidScope`] for an empty or
    /// whitespace-only scope.
    pub fn grant_scope(&mut self, scope: impl Into<String>) -> Result<(), WorkloadIdentityError> {
        let scope = scope.into();
        if scope.trim().is_empty() {
            return Err(WorkloadIdentityError::InvalidScope);
        }
        if !self.scopes.iter().any(|granted| granted == &scope) {
            self.scopes.push(scope);
        }
        Ok(())
    }

    /// Builder-style scope grant.
    ///
    /// # Errors
    /// See [`WorkloadPrincipal::grant_scope`].
    pub fn with_scope(mut self, scope: impl Into<String>) -> Result<Self, WorkloadIdentityError> {
        self.grant_scope(scope)?;
        Ok(self)
    }

    /// Apply a lifecycle transition.
    ///
    /// # Errors
    /// Returns [`WorkloadIdentityError::IllegalStateTransition`] if the move is
    /// not permitted by [`WorkloadState::can_transition_to`].
    pub fn transition_to(&mut self, target: WorkloadState) -> Result<(), WorkloadIdentityError> {
        if self.state.can_transition_to(target) {
            self.state = target;
            Ok(())
        } else {
            Err(WorkloadIdentityError::IllegalStateTransition {
                from: self.state,
                to: target,
            })
        }
    }
}

/// The action a [`WorkloadPrincipal`] is asking to perform, in PARC terms.
/// Free-form by design — the authz layer maps these onto policy actions.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Action(String);

impl Action {
    /// Construct an action (e.g. `cloud.kms.Decrypt`). Always succeeds; the
    /// authz layer is responsible for recognizing known actions.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the action string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The resource an action targets, identified by a type + id pair (mirrors
/// Cedar's `Type::"id"` entity-uid shape).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Resource {
    resource_type: String,
    resource_id: String,
    attributes: BTreeMap<String, ClaimValue>,
}

impl Resource {
    /// Construct a resource reference.
    #[must_use]
    pub fn new(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            attributes: BTreeMap::new(),
        }
    }

    /// The resource type (e.g. `Secret`, `Bucket`).
    #[must_use]
    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }

    /// The resource id (e.g. a secret name).
    #[must_use]
    pub fn resource_id(&self) -> &str {
        &self.resource_id
    }

    /// Typed resource attributes visible to policy conditions.
    #[must_use]
    pub fn attributes(&self) -> &BTreeMap<String, ClaimValue> {
        &self.attributes
    }

    /// Attach a policy-visible resource attribute (builder style).
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: ClaimValue) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::\"{}\"", self.resource_type, self.resource_id)
    }
}

/// A complete authorization request: the verified principal plus the PARC
/// action/resource/context. This is the input to any authz adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationRequest {
    /// The authenticated workload making the request.
    pub principal: WorkloadPrincipal,
    /// The action requested.
    pub action: Action,
    /// The resource targeted.
    pub resource: Resource,
    /// Free-form request context (e.g. `source_ip`, `mfa`), as typed claims.
    pub context: BTreeMap<String, ClaimValue>,
}

impl AuthorizationRequest {
    /// Construct a request with empty context.
    #[must_use]
    pub fn new(principal: WorkloadPrincipal, action: Action, resource: Resource) -> Self {
        Self {
            principal,
            action,
            resource,
            context: BTreeMap::new(),
        }
    }

    /// Attach a context attribute (builder style).
    #[must_use]
    pub fn with_context(mut self, key: impl Into<String>, value: ClaimValue) -> Self {
        self.context.insert(key.into(), value);
        self
    }
}

/// The binary effect of an authorization decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Effect {
    /// The request is allowed.
    Allow,
    /// The request is denied.
    Deny,
}

impl Effect {
    /// `true` only for [`Effect::Allow`].
    #[must_use]
    pub fn is_allow(self) -> bool {
        matches!(self, Self::Allow)
    }
}

/// The reason a decision came out the way it did. Surfacing this is essential
/// for audit-chain emission and operator debugging (deny-by-default must be
/// distinguishable from an explicit forbid).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionReason {
    /// An explicit `permit` policy matched.
    ExplicitPermit {
        /// Identifier of the matched policy.
        policy_id: String,
    },
    /// An explicit `forbid` policy matched and overrode any permit (Cedar
    /// forbid-wins semantics).
    ExplicitForbid {
        /// Identifier of the matched policy.
        policy_id: String,
    },
    /// No policy matched; deny-by-default.
    DefaultDeny,
    /// The principal was not in an operational lifecycle state.
    PrincipalNotOperational {
        /// State the principal was in.
        state: WorkloadState,
    },
}

/// The outcome of evaluating an [`AuthorizationRequest`].
///
/// Constructed by authz adapters via the provided constructors so that the
/// `effect`/`reason` pair stays internally consistent (an `Allow` can only be
/// produced from an explicit permit; everything else denies).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationDecision {
    effect: Effect,
    reason: DecisionReason,
}

impl AuthorizationDecision {
    /// Construct an allow decision backed by an explicit permit policy.
    #[must_use]
    pub fn permit(policy_id: impl Into<String>) -> Self {
        Self {
            effect: Effect::Allow,
            reason: DecisionReason::ExplicitPermit {
                policy_id: policy_id.into(),
            },
        }
    }

    /// Construct a deny decision from an explicit forbid policy.
    #[must_use]
    pub fn forbid(policy_id: impl Into<String>) -> Self {
        Self {
            effect: Effect::Deny,
            reason: DecisionReason::ExplicitForbid {
                policy_id: policy_id.into(),
            },
        }
    }

    /// Construct the deny-by-default decision (no policy matched).
    #[must_use]
    pub fn default_deny() -> Self {
        Self {
            effect: Effect::Deny,
            reason: DecisionReason::DefaultDeny,
        }
    }

    /// Construct a deny decision because the principal is not operational.
    #[must_use]
    pub fn principal_not_operational(state: WorkloadState) -> Self {
        Self {
            effect: Effect::Deny,
            reason: DecisionReason::PrincipalNotOperational { state },
        }
    }

    /// The decision effect.
    #[must_use]
    pub fn effect(&self) -> Effect {
        self.effect
    }

    /// Convenience: whether the request was allowed.
    #[must_use]
    pub fn is_allow(&self) -> bool {
        self.effect.is_allow()
    }

    /// The reason backing the effect.
    #[must_use]
    pub fn reason(&self) -> &DecisionReason {
        &self.reason
    }
}

/// A single candidate policy outcome produced by a policy engine (e.g. Cedar)
/// for an authorization request. The `effect` indicates whether the matching
/// policy was a permit or forbid, and `policy_id` identifies the policy that
/// produced it. An ordered slice of these is the input to
/// [`evaluate_decision`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyOutcome {
    /// The effect of the matching policy: allow (permit) or deny (forbid).
    pub effect: Effect,
    /// The identifier of the policy that produced this outcome.
    pub policy_id: String,
}

impl PolicyOutcome {
    /// Construct a permit outcome.
    #[must_use]
    pub fn permit(policy_id: impl Into<String>) -> Self {
        Self {
            effect: Effect::Allow,
            policy_id: policy_id.into(),
        }
    }

    /// Construct a forbid outcome.
    #[must_use]
    pub fn forbid(policy_id: impl Into<String>) -> Self {
        Self {
            effect: Effect::Deny,
            policy_id: policy_id.into(),
        }
    }
}

/// Evaluate an ordered slice of candidate [`PolicyOutcome`]s for `principal`,
/// folding them into a single [`AuthorizationDecision`] using Cedar-compatible
/// precedence rules:
///
/// 1. **Not-operational short-circuit**: if `principal.state().is_operational()`
///    is `false`, return
///    [`AuthorizationDecision::principal_not_operational`] immediately without
///    examining any outcomes.
/// 2. **Forbid-wins**: if any outcome has [`Effect::Deny`], the result is the
///    first such `ExplicitForbid` decision regardless of any permits.
/// 3. **Explicit permit**: if at least one outcome has [`Effect::Allow`] and
///    none has [`Effect::Deny`], return the first
///    [`AuthorizationDecision::permit`].
/// 4. **Deny-by-default**: if `outcomes` is empty or no outcome matched either
///    effect, return [`AuthorizationDecision::default_deny`].
///
/// This function is total, deterministic, panic-free, and has no I/O or clock.
#[must_use]
pub fn evaluate_decision(
    principal: &WorkloadPrincipal,
    outcomes: &[PolicyOutcome],
) -> AuthorizationDecision {
    // Rule 1: not-operational short-circuit.
    if !principal.state().is_operational() {
        return AuthorizationDecision::principal_not_operational(principal.state());
    }

    // Scan once: track the first forbid and first permit.
    let mut first_forbid: Option<&str> = None;
    let mut first_permit: Option<&str> = None;

    for outcome in outcomes {
        match outcome.effect {
            Effect::Deny if first_forbid.is_none() => {
                first_forbid = Some(&outcome.policy_id);
            }
            Effect::Allow if first_permit.is_none() => {
                first_permit = Some(&outcome.policy_id);
            }
            _ => {}
        }
    }

    // Rule 2: forbid-wins.
    if let Some(policy_id) = first_forbid {
        return AuthorizationDecision::forbid(policy_id);
    }

    // Rule 3: explicit permit.
    if let Some(policy_id) = first_permit {
        return AuthorizationDecision::permit(policy_id);
    }

    // Rule 4: deny-by-default.
    AuthorizationDecision::default_deny()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_principal() -> WorkloadPrincipal {
        let mut principal =
            WorkloadPrincipal::provision("ten_acme", "wl_ci_runner", "cap.cloud.kms")
                .expect("valid ids");
        principal
            .transition_to(WorkloadState::Active)
            .expect("provisioned -> active is legal");
        principal
    }

    #[test]
    fn tenant_id_rejects_bad_prefix() {
        assert_eq!(
            TenantId::new("acme"),
            Err(WorkloadIdentityError::InvalidTenantId)
        );
        assert_eq!(
            TenantId::new("ten_"),
            Err(WorkloadIdentityError::InvalidTenantId)
        );
        assert!(TenantId::new("ten_acme").is_ok());
    }

    #[test]
    fn workload_id_rejects_bad_prefix() {
        assert_eq!(
            WorkloadId::new("svc"),
            Err(WorkloadIdentityError::InvalidWorkloadId)
        );
        assert!(WorkloadId::new("wl_x").is_ok());
    }

    #[test]
    fn capability_id_requires_cap_dot() {
        assert_eq!(
            CapabilityId::new("cloud.kms"),
            Err(WorkloadIdentityError::InvalidCapabilityId)
        );
        assert!(CapabilityId::new("cap.cloud.kms").is_ok());
    }

    #[test]
    fn provision_starts_non_operational() {
        let principal = WorkloadPrincipal::provision("ten_acme", "wl_a", "cap.x.y").expect("valid");
        assert_eq!(principal.state(), WorkloadState::Provisioned);
        assert!(!principal.state().is_operational());
        assert_eq!(
            principal.schema_version(),
            WORKLOAD_PRINCIPAL_SCHEMA_VERSION
        );
    }

    #[test]
    fn trust_domain_is_spiffe_shaped_and_equals_tenant() {
        let principal = WorkloadPrincipal::provision("ten_acme", "wl_a", "cap.x.y").expect("valid");
        assert_eq!(principal.trust_domain().as_str(), "spiffe://ten_acme");
        assert_eq!(principal.trust_domain().authority(), "ten_acme");
        // The derived trust domain always matches the owning tenant.
        assert!(
            principal
                .trust_domain()
                .matches_tenant(principal.tenant_id())
        );
    }

    #[test]
    fn trust_domain_rejects_non_spiffe_and_svid_paths() {
        // Missing scheme.
        assert_eq!(
            TrustDomain::new("ten_acme"),
            Err(WorkloadIdentityError::InvalidTrustDomain)
        );
        // Empty authority.
        assert_eq!(
            TrustDomain::new("spiffe://"),
            Err(WorkloadIdentityError::InvalidTrustDomain)
        );
        // A full SVID path is NOT a bare trust domain.
        assert_eq!(
            TrustDomain::new("spiffe://ten_acme/ns/sa"),
            Err(WorkloadIdentityError::InvalidTrustDomain)
        );
        // A bare authority is accepted.
        assert!(TrustDomain::new("spiffe://ten_acme").is_ok());
    }

    #[test]
    fn trust_domain_matches_tenant_is_tenant_scoped() {
        let acme = TenantId::new("ten_acme").expect("valid");
        let globex = TenantId::new("ten_globex").expect("valid");
        let td = TrustDomain::for_tenant(&acme);
        assert!(td.matches_tenant(&acme));
        assert!(!td.matches_tenant(&globex));
    }

    #[test]
    fn lifecycle_transitions_follow_the_state_machine() {
        let mut principal =
            WorkloadPrincipal::provision("ten_acme", "wl_a", "cap.x.y").expect("valid");
        // Provisioned -> Active -> Suspended -> Active -> Retired
        principal
            .transition_to(WorkloadState::Active)
            .expect("activate");
        assert!(principal.state().is_operational());
        principal
            .transition_to(WorkloadState::Suspended)
            .expect("suspend");
        assert!(!principal.state().is_operational());
        principal
            .transition_to(WorkloadState::Active)
            .expect("reactivate");
        principal
            .transition_to(WorkloadState::Retired)
            .expect("retire");
        // Retired is terminal.
        assert_eq!(
            principal.transition_to(WorkloadState::Active),
            Err(WorkloadIdentityError::IllegalStateTransition {
                from: WorkloadState::Retired,
                to: WorkloadState::Active,
            })
        );
    }

    #[test]
    fn illegal_transition_is_rejected() {
        let mut principal =
            WorkloadPrincipal::provision("ten_acme", "wl_a", "cap.x.y").expect("valid");
        // Provisioned -> Suspended is not a legal edge.
        assert!(principal.transition_to(WorkloadState::Suspended).is_err());
        // No-op self transition is rejected too.
        assert!(principal.transition_to(WorkloadState::Provisioned).is_err());
    }

    #[test]
    fn claims_and_scopes_validate_and_query() {
        let principal = active_principal()
            .with_claim("env", ClaimValue::Text("prod".into()))
            .expect("claim ok")
            .with_claim(
                "groups",
                ClaimValue::TextList(vec!["deployers".into(), "readers".into()]),
            )
            .expect("claim ok")
            .with_scope("cloud.kms.decrypt")
            .expect("scope ok");

        assert_eq!(
            principal.claim("env").and_then(ClaimValue::as_text),
            Some("prod")
        );
        assert!(
            principal
                .claim("groups")
                .expect("present")
                .contains("deployers")
        );
        assert!(principal.has_scope("cloud.kms.decrypt"));
        assert!(!principal.has_scope("cloud.kms.encrypt"));
    }

    #[test]
    fn empty_claim_name_and_scope_rejected() {
        let mut principal = active_principal();
        assert_eq!(
            principal.set_claim("bad name", ClaimValue::Bool(true)),
            Err(WorkloadIdentityError::InvalidClaimName)
        );
        assert_eq!(
            principal.grant_scope("   "),
            Err(WorkloadIdentityError::InvalidScope)
        );
    }

    #[test]
    fn scope_grants_are_idempotent() {
        let mut principal = active_principal();
        principal.grant_scope("a").expect("ok");
        principal.grant_scope("a").expect("ok");
        assert_eq!(principal.scopes(), &["a".to_string()]);
    }

    #[test]
    fn decision_constructors_keep_effect_and_reason_consistent() {
        let permit = AuthorizationDecision::permit("p1");
        assert!(permit.is_allow());
        assert!(matches!(
            permit.reason(),
            DecisionReason::ExplicitPermit { policy_id } if policy_id == "p1"
        ));

        let forbid = AuthorizationDecision::forbid("f1");
        assert!(!forbid.is_allow());
        assert_eq!(forbid.effect(), Effect::Deny);

        assert_eq!(AuthorizationDecision::default_deny().effect(), Effect::Deny);
        assert_eq!(
            AuthorizationDecision::principal_not_operational(WorkloadState::Suspended).effect(),
            Effect::Deny
        );
    }

    #[test]
    fn authorization_request_builds_with_context() {
        let request = AuthorizationRequest::new(
            active_principal(),
            Action::new("cloud.kms.Decrypt"),
            Resource::new("Secret", "db-password"),
        )
        .with_context("source_ip", ClaimValue::Text("10.0.0.1".into()));
        assert_eq!(request.action.as_str(), "cloud.kms.Decrypt");
        assert_eq!(request.resource.resource_type(), "Secret");
        assert_eq!(
            request
                .context
                .get("source_ip")
                .and_then(ClaimValue::as_text),
            Some("10.0.0.1")
        );
    }

    #[test]
    fn resource_display_matches_entity_uid_shape() {
        let resource = Resource::new("Bucket", "logs");
        assert_eq!(resource.to_string(), "Bucket::\"logs\"");
    }

    #[test]
    fn resource_builds_with_policy_attributes() {
        let resource = Resource::new("QuotaRecord", "ten_acme")
            .with_attribute("tenant_id", ClaimValue::Text("ten_acme".into()));

        assert_eq!(resource.resource_type(), "QuotaRecord");
        assert_eq!(
            resource
                .attributes()
                .get("tenant_id")
                .and_then(ClaimValue::as_text),
            Some("ten_acme")
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // evaluate_decision: table-driven acceptance tests
    // ──────────────────────────────────────────────────────────────────────────

    fn provisioned_principal() -> WorkloadPrincipal {
        WorkloadPrincipal::provision("ten_acme", "wl_ci_runner", "cap.cloud.kms")
            .expect("valid ids")
        // stays in Provisioned — not operational
    }

    fn suspended_principal() -> WorkloadPrincipal {
        let mut p = active_principal();
        p.transition_to(WorkloadState::Suspended)
            .expect("active -> suspended");
        p
    }

    fn retired_principal() -> WorkloadPrincipal {
        let mut p = active_principal();
        p.transition_to(WorkloadState::Retired)
            .expect("active -> retired");
        p
    }

    /// Rule 1: Provisioned principal → PrincipalNotOperational, no outcomes examined.
    #[test]
    fn evaluate_decision_not_operational_provisioned() {
        let principal = provisioned_principal();
        let outcomes = vec![PolicyOutcome::permit("p1")];
        let decision = evaluate_decision(&principal, &outcomes);
        assert_eq!(decision.effect(), Effect::Deny);
        assert!(matches!(
            decision.reason(),
            DecisionReason::PrincipalNotOperational {
                state: WorkloadState::Provisioned
            }
        ));
    }

    /// Rule 1: Suspended principal → PrincipalNotOperational.
    #[test]
    fn evaluate_decision_not_operational_suspended() {
        let principal = suspended_principal();
        let outcomes = vec![PolicyOutcome::permit("p1")];
        let decision = evaluate_decision(&principal, &outcomes);
        assert_eq!(decision.effect(), Effect::Deny);
        assert!(matches!(
            decision.reason(),
            DecisionReason::PrincipalNotOperational {
                state: WorkloadState::Suspended
            }
        ));
    }

    /// Rule 1: Retired principal → PrincipalNotOperational.
    #[test]
    fn evaluate_decision_not_operational_retired() {
        let principal = retired_principal();
        let outcomes = vec![PolicyOutcome::permit("p1"), PolicyOutcome::forbid("f1")];
        let decision = evaluate_decision(&principal, &outcomes);
        assert_eq!(decision.effect(), Effect::Deny);
        assert!(matches!(
            decision.reason(),
            DecisionReason::PrincipalNotOperational {
                state: WorkloadState::Retired
            }
        ));
    }

    /// Rule 2: forbid-wins — any ExplicitForbid beats any ExplicitPermit.
    #[test]
    fn evaluate_decision_forbid_wins_over_permit() {
        let principal = active_principal();
        // Permit appears before forbid; forbid still wins.
        let outcomes = vec![PolicyOutcome::permit("p1"), PolicyOutcome::forbid("f1")];
        let decision = evaluate_decision(&principal, &outcomes);
        assert_eq!(decision.effect(), Effect::Deny);
        assert!(matches!(
            decision.reason(),
            DecisionReason::ExplicitForbid { policy_id } if policy_id == "f1"
        ));
    }

    /// Rule 2: forbid-wins — forbid appearing first also wins over later permit.
    #[test]
    fn evaluate_decision_forbid_wins_forbid_first() {
        let principal = active_principal();
        let outcomes = vec![PolicyOutcome::forbid("f2"), PolicyOutcome::permit("p2")];
        let decision = evaluate_decision(&principal, &outcomes);
        assert_eq!(decision.effect(), Effect::Deny);
        assert!(matches!(
            decision.reason(),
            DecisionReason::ExplicitForbid { policy_id } if policy_id == "f2"
        ));
    }

    /// Rule 3: permit-only → Allow with first permit policy_id.
    #[test]
    fn evaluate_decision_permit_only_allows() {
        let principal = active_principal();
        let outcomes = vec![PolicyOutcome::permit("p1"), PolicyOutcome::permit("p2")];
        let decision = evaluate_decision(&principal, &outcomes);
        assert_eq!(decision.effect(), Effect::Allow);
        assert!(matches!(
            decision.reason(),
            DecisionReason::ExplicitPermit { policy_id } if policy_id == "p1"
        ));
    }

    /// Rule 4: empty outcomes → deny-by-default.
    #[test]
    fn evaluate_decision_empty_outcomes_default_deny() {
        let principal = active_principal();
        let decision = evaluate_decision(&principal, &[]);
        assert_eq!(decision.effect(), Effect::Deny);
        assert!(matches!(decision.reason(), DecisionReason::DefaultDeny));
    }

    /// Rule 4 (variant): no matches at all → deny-by-default (same as empty,
    /// confirming the empty slice path).
    #[test]
    fn evaluate_decision_no_match_default_deny() {
        let principal = active_principal();
        // An empty slice is the canonical "no policy matched" input.
        let decision = evaluate_decision(&principal, &[]);
        assert!(matches!(decision.reason(), DecisionReason::DefaultDeny));
        assert!(!decision.is_allow());
    }

    /// Rule 2 (variant): forbid-with-no-permit → ExplicitForbid, not DefaultDeny.
    #[test]
    fn evaluate_decision_forbid_with_no_permit() {
        let principal = active_principal();
        let outcomes = vec![PolicyOutcome::forbid("f3")];
        let decision = evaluate_decision(&principal, &outcomes);
        assert_eq!(decision.effect(), Effect::Deny);
        assert!(matches!(
            decision.reason(),
            DecisionReason::ExplicitForbid { policy_id } if policy_id == "f3"
        ));
    }
}
