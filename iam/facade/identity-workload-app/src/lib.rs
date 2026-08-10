//! Workload-identity service core (usecase layer).
//!
//! This crate is the *application* that wires the three workload-identity
//! crates into the end-to-end machine-to-machine identity flow described by
//! `iam/identity/workload-identity/PRD.md`:
//!
//! ```text
//! raw workload JWT ─validate─▶ verified WorkloadPrincipal ─lifecycle/denylist─▶ authorize ─▶ AuthorizationDecision
//! ```
//!
//! It owns **no** policy algorithm, **no** crypto, and **no** state-machine
//! rules of its own — those live inward:
//! - [`iam_identity_workload_domain`] — the pure principal + lifecycle +
//!   PARC-decision kernel.
//! - [`iam_identity_workload_oidc`] — `ring`-backed JWS validation that
//!   yields a verified [`WorkloadPrincipal`].
//! - [`iam_identity_workload_authz_cedar`] — the real `cedar-policy`
//!   engine behind the [`WorkloadAuthorizer`] trait.
//!
//! ## Layering invariant (ADR-0131 / layered-architecture discipline)
//!
//! This is the `application`/usecase ring. It depends inward on a `domain`
//! crate and two adapter crates and exposes its own ports
//! ([`WorkloadPrincipalRepository`], [`RevocationDenylist`]) so the control
//! plane's persistence and the hot authorize path stay swappable. The provided
//! [`InMemoryWorkloadPrincipalRepository`] / [`InMemoryRevocationDenylist`] are
//! the reference adapters for tests and single-node bring-up.
//!
//! ## Hot-path posture (ADR-0083 Tier 3 — panic-free; PRD §3.4/§3.5)
//!
//! [`authorize_with_token`] is **default-deny on every error** and contains no
//! `unwrap`/`expect`/`panic` on the request path. The revocation semantics are
//! exactly the PRD's: the hot path is gated on the *denylist* for
//! suspend/retire, and on the verified principal being operational — it is NOT
//! gated on a just-written control-plane activation (the control plane is
//! eventually consistent; the short token TTL + denylist bound revocation
//! latency, PRD §3.5).

// ADR-0083 Tier 3: production code stays panic-free (deny in release); inline
// `mod tests` may use unwrap/expect/panic under cfg(test) only.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

use iam_identity_workload_authz_cedar::WorkloadAuthorizer;
use iam_identity_workload_domain::{
    Action, AuthorizationDecision, AuthorizationRequest, ClaimValue, Resource, WorkloadId,
    WorkloadIdentityError, WorkloadPrincipal, WorkloadState,
};
use iam_identity_workload_oidc::{Jwks, ValidationConfig, validate_workload_token};

// =====================================================================
// Ports
// =====================================================================

/// Persistence port for [`WorkloadPrincipal`] aggregates, keyed by
/// [`WorkloadId`].
///
/// The control-plane lifecycle use-cases load/save through this port; the hot
/// authorize path resolves the principal through it. Implementations are the
/// integration seam (an in-memory map for tests/bring-up; a sharded store in
/// production). Errors are surfaced as [`RepositoryError`] so a backing-store
/// failure on the authorize path can be mapped to a fail-closed deny rather
/// than panicking.
pub trait WorkloadPrincipalRepository {
    /// Load the principal for `workload_id`, or `Ok(None)` if none is stored.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the backing store cannot be read.
    fn load(&self, workload_id: &WorkloadId) -> Result<Option<WorkloadPrincipal>, RepositoryError>;

    /// Persist `principal`, overwriting any existing record for its
    /// [`WorkloadId`].
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the backing store cannot be written.
    fn save(&mut self, principal: &WorkloadPrincipal) -> Result<(), RepositoryError>;
}

/// Fast revocation denylist consulted on the hot authorize path (PRD §3.5).
///
/// `suspend`/`retire` write the workload id here; CAEP-style revocation events
/// may also write an issue-time cutoff so credentials minted at/before that
/// cutoff are denied while newer credentials can continue after re-attestation.
/// [`authorize_with_token`] reads this port before delegating to the policy
/// engine. A denylist read failure is treated as a hard deny (fail-closed),
/// never an allow.
pub trait RevocationDenylist {
    /// Whether `workload_id` is currently revoked.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the denylist cannot be read; callers on
    /// the authorize path MUST treat this as a deny.
    fn is_revoked(&self, workload_id: &WorkloadId) -> Result<bool, RepositoryError>;

    /// Add `workload_id` to the denylist. Idempotent: revoking an already-revoked
    /// id is a no-op success.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the denylist cannot be written.
    fn revoke(&mut self, workload_id: &WorkloadId) -> Result<(), RepositoryError>;

    /// Highest issue-time cutoff for `workload_id`, if a revocation event has
    /// been received. Tokens with `iat <= cutoff` MUST be denied.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the cutoff store cannot be read; callers
    /// on the authorize path MUST treat this as a deny.
    fn issue_time_cutoff(&self, _workload_id: &WorkloadId) -> Result<Option<i64>, RepositoryError> {
        Ok(None)
    }

    /// Record a revocation event for credentials issued at or before `cutoff`.
    /// Implementations that support the event path MUST preserve cutoff
    /// semantics rather than silently converting the event into a whole-principal
    /// revoke; whole-principal suspension/retirement uses [`Self::revoke`].
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the denylist cannot be written.
    fn revoke_issued_at_or_before(
        &mut self,
        workload_id: &WorkloadId,
        cutoff_epoch_seconds: i64,
    ) -> Result<(), RepositoryError>;
}

/// An opaque backing-store failure from a [`WorkloadPrincipalRepository`] or
/// [`RevocationDenylist`]. Carries a human-facing detail for logs without
/// leaking store internals into the typed control flow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryError {
    detail: String, // data_class: INTERNAL_ONLY
}

impl RepositoryError {
    /// Construct a store error with a human-facing detail.
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

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "workload-identity store error: {}", self.detail)
    }
}

impl std::error::Error for RepositoryError {}

// =====================================================================
// In-memory reference adapters
// =====================================================================

/// In-memory [`WorkloadPrincipalRepository`] backed by a [`BTreeMap`]. The
/// reference adapter for tests and single-node bring-up; production swaps in a
/// sharded store behind the same port.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryWorkloadPrincipalRepository {
    principals: BTreeMap<WorkloadId, WorkloadPrincipal>, // data_class: PII_IDENTIFYING
}

impl InMemoryWorkloadPrincipalRepository {
    /// Build an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored principals.
    #[must_use]
    pub fn len(&self) -> usize {
        self.principals.len()
    }

    /// Whether the repository holds no principals.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.principals.is_empty()
    }
}

impl WorkloadPrincipalRepository for InMemoryWorkloadPrincipalRepository {
    fn load(&self, workload_id: &WorkloadId) -> Result<Option<WorkloadPrincipal>, RepositoryError> {
        Ok(self.principals.get(workload_id).cloned())
    }

    fn save(&mut self, principal: &WorkloadPrincipal) -> Result<(), RepositoryError> {
        self.principals
            .insert(principal.workload_id().clone(), principal.clone());
        Ok(())
    }
}

/// In-memory [`RevocationDenylist`] backed by sorted workload ids plus optional
/// issue-time cutoffs. The reference adapter; production swaps in a fast shared
/// store (e.g. Valkey) behind the same port.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryRevocationDenylist {
    revoked: std::collections::BTreeSet<WorkloadId>, // data_class: PII_IDENTIFYING
    issue_time_cutoffs: BTreeMap<WorkloadId, i64>,   // data_class: PII_IDENTIFYING
}

impl InMemoryRevocationDenylist {
    /// Build an empty denylist.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of workload ids with a whole-principal revoke or issue-time cutoff.
    #[must_use]
    pub fn len(&self) -> usize {
        let mut ids = self.revoked.clone();
        ids.extend(self.issue_time_cutoffs.keys().cloned());
        ids.len()
    }

    /// Whether the denylist is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.revoked.is_empty() && self.issue_time_cutoffs.is_empty()
    }
}

impl RevocationDenylist for InMemoryRevocationDenylist {
    fn is_revoked(&self, workload_id: &WorkloadId) -> Result<bool, RepositoryError> {
        Ok(self.revoked.contains(workload_id))
    }

    fn revoke(&mut self, workload_id: &WorkloadId) -> Result<(), RepositoryError> {
        self.revoked.insert(workload_id.clone());
        Ok(())
    }

    fn issue_time_cutoff(&self, workload_id: &WorkloadId) -> Result<Option<i64>, RepositoryError> {
        Ok(self.issue_time_cutoffs.get(workload_id).copied())
    }

    fn revoke_issued_at_or_before(
        &mut self,
        workload_id: &WorkloadId,
        cutoff_epoch_seconds: i64,
    ) -> Result<(), RepositoryError> {
        self.issue_time_cutoffs
            .entry(workload_id.clone())
            .and_modify(|existing| *existing = (*existing).max(cutoff_epoch_seconds))
            .or_insert(cutoff_epoch_seconds);
        Ok(())
    }
}

// =====================================================================
// Lifecycle use-cases
// =====================================================================

/// Errors from a control-plane lifecycle use-case (provision/activate/suspend/
/// retire). Distinct from the hot authorize path, which never errors — it
/// fail-closes to a deny [`AuthorizationDecision`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleError {
    /// The principal id was not found in the repository.
    PrincipalNotFound {
        /// The workload id that was looked up.
        workload_id: String,
    },
    /// A principal already exists for this id; provision must not clobber it
    /// (retired ids are tombstoned and never re-provisioned, PRD §3.5 / AC-W-14).
    PrincipalAlreadyExists {
        /// The workload id that already had a record.
        workload_id: String,
    },
    /// A domain rule rejected the operation (bad id shape or an illegal
    /// lifecycle transition).
    Domain(WorkloadIdentityError),
    /// The backing store or denylist failed.
    Repository(RepositoryError),
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrincipalNotFound { workload_id } => {
                write!(f, "workload principal not found: {workload_id}")
            }
            Self::PrincipalAlreadyExists { workload_id } => {
                write!(f, "workload principal already exists: {workload_id}")
            }
            Self::Domain(error) => {
                write!(f, "workload-identity domain rejected operation: {error}")
            }
            Self::Repository(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for LifecycleError {}

impl From<WorkloadIdentityError> for LifecycleError {
    fn from(error: WorkloadIdentityError) -> Self {
        Self::Domain(error)
    }
}

impl From<RepositoryError> for LifecycleError {
    fn from(error: RepositoryError) -> Self {
        Self::Repository(error)
    }
}

/// Provision a new workload principal (PRD §3.5 `provision`).
///
/// Creates a freshly-provisioned principal via the domain constructor and
/// persists it. Refuses to overwrite an existing record so a tombstoned
/// (retired) or live id is never silently re-created (AC-W-14).
///
/// # Errors
/// - [`LifecycleError::PrincipalAlreadyExists`] if a record already exists.
/// - [`LifecycleError::Domain`] if any id fails the `ten_`/`wl_`/`cap.` shape.
/// - [`LifecycleError::Repository`] on a store failure.
pub fn provision<R: WorkloadPrincipalRepository>(
    repository: &mut R,
    tenant_id: impl Into<String>,
    workload_id: impl Into<String>,
    owning_capability: impl Into<String>,
) -> Result<WorkloadPrincipal, LifecycleError> {
    let principal = WorkloadPrincipal::provision(tenant_id, workload_id, owning_capability)?;
    if repository.load(principal.workload_id())?.is_some() {
        return Err(LifecycleError::PrincipalAlreadyExists {
            workload_id: principal.workload_id().to_string(),
        });
    }
    repository.save(&principal)?;
    Ok(principal)
}

/// Activate a principal: `Provisioned`/`Suspended` -> `Active` (PRD §3.5).
///
/// Loads the principal, applies the domain transition (which rejects illegal
/// moves with [`WorkloadIdentityError::IllegalStateTransition`]), and persists.
///
/// # Errors
/// - [`LifecycleError::PrincipalNotFound`] if no record exists.
/// - [`LifecycleError::Domain`] if the transition is illegal (e.g. from
///   `Retired`).
/// - [`LifecycleError::Repository`] on a store failure.
pub fn activate<R: WorkloadPrincipalRepository>(
    repository: &mut R,
    workload_id: &WorkloadId,
) -> Result<WorkloadPrincipal, LifecycleError> {
    transition(repository, workload_id, WorkloadState::Active)
}

/// Suspend a principal: `Active` -> `Suspended`, and revoke on the denylist so
/// the hot authorize path denies it even before its short-lived token expires
/// (PRD §3.5 / AC-W-07).
///
/// The denylist write happens only after the domain transition succeeds and the
/// principal is persisted, so a rejected (illegal) suspend never revokes.
///
/// # Errors
/// - [`LifecycleError::PrincipalNotFound`] if no record exists.
/// - [`LifecycleError::Domain`] if the transition is illegal.
/// - [`LifecycleError::Repository`] on a store/denylist failure.
pub fn suspend<R: WorkloadPrincipalRepository, D: RevocationDenylist>(
    repository: &mut R,
    denylist: &mut D,
    workload_id: &WorkloadId,
) -> Result<WorkloadPrincipal, LifecycleError> {
    let principal = transition(repository, workload_id, WorkloadState::Suspended)?;
    denylist.revoke(workload_id)?;
    Ok(principal)
}

/// Retire a principal: any non-terminal state -> `Retired` (terminal), and
/// revoke on the denylist (PRD §3.5 / AC-W-14).
///
/// `Retired` is terminal in the domain state machine, so a subsequent
/// [`activate`] is rejected as an illegal transition. The id stays in the
/// repository as a tombstone so [`provision`] cannot reuse it.
///
/// # Errors
/// - [`LifecycleError::PrincipalNotFound`] if no record exists.
/// - [`LifecycleError::Domain`] if the transition is illegal.
/// - [`LifecycleError::Repository`] on a store/denylist failure.
pub fn retire<R: WorkloadPrincipalRepository, D: RevocationDenylist>(
    repository: &mut R,
    denylist: &mut D,
    workload_id: &WorkloadId,
) -> Result<WorkloadPrincipal, LifecycleError> {
    let principal = transition(repository, workload_id, WorkloadState::Retired)?;
    denylist.revoke(workload_id)?;
    Ok(principal)
}

/// Record a CAEP-style revocation event for credentials issued at or before the
/// supplied cutoff. The event path deliberately updates only the hot-path
/// revocation port; it does not transition the persisted principal lifecycle, so
/// a workload can re-attest with a newer credential after the cutoff.
///
/// # Errors
/// - [`LifecycleError::Repository`] when the denylist cannot be written.
pub fn record_revocation_event<D: RevocationDenylist>(
    denylist: &mut D,
    workload_id: &WorkloadId,
    issue_time_cutoff_epoch_seconds: i64,
) -> Result<(), LifecycleError> {
    denylist.revoke_issued_at_or_before(workload_id, issue_time_cutoff_epoch_seconds)?;
    Ok(())
}

/// Shared load -> domain-transition -> persist sequence for the lifecycle
/// use-cases. The domain's `transition_to` is the single source of truth for
/// which moves are legal.
fn transition<R: WorkloadPrincipalRepository>(
    repository: &mut R,
    workload_id: &WorkloadId,
    target: WorkloadState,
) -> Result<WorkloadPrincipal, LifecycleError> {
    let mut principal =
        repository
            .load(workload_id)?
            .ok_or_else(|| LifecycleError::PrincipalNotFound {
                workload_id: workload_id.to_string(),
            })?;
    principal.transition_to(target)?;
    repository.save(&principal)?;
    Ok(principal)
}

// =====================================================================
// Authorize use-case (hot path)
// =====================================================================

/// Why an authorize call resolved the way it did, surfaced for the audit chain
/// alongside the [`AuthorizationDecision`]. A deny carries the *stage* that
/// fail-closed so a forged token is distinguishable from a revoked principal or
/// a policy denial (PRD §3.3 / AC-W-13).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthorizeOutcome {
    /// The full pipeline ran and the policy engine returned this decision
    /// (permit or an explicit/implicit deny — the engine's own reason is inside).
    Decided(AuthorizationDecision),
    /// Token validation failed (forged/expired/wrong-issuer/...): default-deny
    /// without ever consulting the policy engine (PRD §3.4).
    TokenRejected,
    /// The verified principal had no record in the repository (unknown subject):
    /// default-deny.
    PrincipalUnknown,
    /// The principal id is on the revocation denylist (suspended/retired):
    /// default-deny on the hot path even before the token expires (PRD §3.5).
    Revoked,
    /// A backing-store/denylist read failed: fail-closed default-deny, never an
    /// allow (PRD §5 fail-closed posture).
    StoreUnavailable,
}

impl AuthorizeOutcome {
    /// The decision to return to the PEP. Every non-`Decided` outcome maps to a
    /// fresh default-deny so callers can treat [`AuthorizeOutcome`] uniformly.
    #[must_use]
    pub fn decision(&self) -> AuthorizationDecision {
        match self {
            Self::Decided(decision) => decision.clone(),
            Self::TokenRejected
            | Self::PrincipalUnknown
            | Self::Revoked
            | Self::StoreUnavailable => AuthorizationDecision::default_deny(),
        }
    }

    /// Whether this outcome allows the request. Only a `Decided` permit allows;
    /// every fail-closed stage denies.
    #[must_use]
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Decided(decision) if decision.is_allow())
    }
}

/// The hot authorize path (PRD §3.3/§3.4/§3.5): validate a raw workload JWT,
/// resolve the persisted principal, reject if non-operational or revoked, then
/// delegate to the policy engine.
///
/// Pipeline (fail-closed at every step; ADR-0083 Tier 3 panic-free):
/// 1. [`validate_workload_token`] the raw JWT against the JWKS/config — on ANY
///    error, [`AuthorizeOutcome::TokenRejected`] (default-deny; engine never
///    runs).
/// 2. Resolve the persisted [`WorkloadPrincipal`] by the verified token's
///    [`WorkloadId`]. Missing -> [`AuthorizeOutcome::PrincipalUnknown`]; store
///    read failure -> [`AuthorizeOutcome::StoreUnavailable`].
/// 3. Denylist check (the revocation gate): whole-principal revoke or an
///    issue-time cutoff matching the token's `iat` -> [`AuthorizeOutcome::Revoked`];
///    read failure -> [`AuthorizeOutcome::StoreUnavailable`].
/// 4. Operational check: a non-`Active` persisted principal ->
///    [`AuthorizeOutcome::Revoked`] (the resolved control-plane record, not the
///    token, decides operational state).
/// 5. Build the [`AuthorizationRequest`] (carrying the *persisted* principal so
///    the engine sees the authoritative lifecycle state + scopes/claims) and
///    delegate to [`WorkloadAuthorizer::authorize`], which is itself
///    default-deny on any internal error.
///
/// Note on PRD §3.5 eventual consistency: the resolved persisted principal is
/// the operational authority, but the gate is NOT a freshly-written activation
/// — it reflects whatever the (eventually-consistent) control plane has
/// committed. Suspend/retire take effect via the denylist (step 3) regardless
/// of token TTL.
#[allow(clippy::too_many_arguments)]
pub fn authorize_with_token<R, D, A>(
    repository: &R,
    denylist: &D,
    authorizer: &A,
    jwks: &Jwks,
    config: &ValidationConfig,
    now_epoch_seconds: i64,
    token: &str,
    action: Action,
    resource: Resource,
    context: BTreeMap<String, ClaimValue>,
) -> AuthorizeOutcome
where
    R: WorkloadPrincipalRepository,
    D: RevocationDenylist,
    A: WorkloadAuthorizer,
{
    // 1. Authenticate the workload. Any validation failure is a default-deny
    //    and the policy engine is never consulted (PRD §3.4).
    let verified = match validate_workload_token(token, jwks, config, now_epoch_seconds) {
        Ok(principal) => principal,
        Err(_error) => return AuthorizeOutcome::TokenRejected,
    };
    let workload_id = verified.workload_id().clone();

    // 2. Resolve the authoritative control-plane principal. A store failure is
    //    fail-closed (never an allow); an absent record is an unknown subject.
    let persisted = match repository.load(&workload_id) {
        Ok(Some(principal)) => principal,
        Ok(None) => return AuthorizeOutcome::PrincipalUnknown,
        Err(_error) => return AuthorizeOutcome::StoreUnavailable,
    };

    // 3. Revocation gate (PRD §3.5): the denylist is the fast suspend/retire
    //    enforcement point on the hot path. CAEP-style revocation events add an
    //    issue-time cutoff; a token whose iat is missing or at/before that
    //    cutoff is stale and denied. Any read failure fail-closes.
    match denylist.is_revoked(&workload_id) {
        Ok(true) => return AuthorizeOutcome::Revoked,
        Ok(false) => {}
        Err(_error) => return AuthorizeOutcome::StoreUnavailable,
    }
    match denylist.issue_time_cutoff(&workload_id) {
        Ok(Some(cutoff)) => {
            let stale = credential_issued_at_epoch_seconds(&verified)
                .is_none_or(|issued_at| issued_at <= cutoff);
            if stale {
                return AuthorizeOutcome::Revoked;
            }
        }
        Ok(None) => {}
        Err(_error) => return AuthorizeOutcome::StoreUnavailable,
    }

    // 4. Operational gate: the persisted lifecycle state is the revocation
    //    authority. A suspended/retired/provisioned control-plane record can
    //    never be authorized even if the denylist somehow lagged. This is the
    //    second, defence-in-depth half of the §3.5 revocation story (the
    //    denylist in step 3 being the fast first half).
    if !persisted.state().is_operational() {
        return AuthorizeOutcome::Revoked;
    }

    // 5. Authorize on the VERIFIED token principal. The scopes + claims the
    //    engine evaluates are the per-token capabilities the issuer minted (and
    //    the OIDC adapter projected) — the control-plane record carries identity
    //    + lifecycle authority, not the request-time grant set. The verified
    //    principal is Active (the OIDC adapter activates on successful
    //    validation) and we have already proved the authoritative record is
    //    operational + not revoked above, so authorizing on the token principal
    //    is sound and reflects exactly what this token is allowed to do.
    //
    //    The Cedar adapter is default-deny on any internal translation error, so
    //    the worst case here is still a deny.
    let mut request = AuthorizationRequest::new(verified, action, resource);
    request.context = context;
    AuthorizeOutcome::Decided(authorizer.authorize(&request))
}

fn credential_issued_at_epoch_seconds(principal: &WorkloadPrincipal) -> Option<i64> {
    match principal.claim("iat") {
        Some(ClaimValue::Int(issued_at)) => Some(*issued_at),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A repository whose `load`/`save` always error — proves the lifecycle and
    /// authorize paths fail-close on a store outage rather than panicking.
    struct FailingRepository;
    impl WorkloadPrincipalRepository for FailingRepository {
        fn load(
            &self,
            _workload_id: &WorkloadId,
        ) -> Result<Option<WorkloadPrincipal>, RepositoryError> {
            Err(RepositoryError::new("induced load failure"))
        }
        fn save(&mut self, _principal: &WorkloadPrincipal) -> Result<(), RepositoryError> {
            Err(RepositoryError::new("induced save failure"))
        }
    }

    /// A denylist whose `is_revoked`/`revoke` always error — proves the
    /// revocation gate fail-closes on an outage.
    struct FailingDenylist;
    impl RevocationDenylist for FailingDenylist {
        fn is_revoked(&self, _workload_id: &WorkloadId) -> Result<bool, RepositoryError> {
            Err(RepositoryError::new("induced denylist failure"))
        }
        fn revoke(&mut self, _workload_id: &WorkloadId) -> Result<(), RepositoryError> {
            Err(RepositoryError::new("induced revoke failure"))
        }

        fn revoke_issued_at_or_before(
            &mut self,
            _workload_id: &WorkloadId,
            _cutoff_epoch_seconds: i64,
        ) -> Result<(), RepositoryError> {
            Err(RepositoryError::new("induced cutoff write failure"))
        }
    }

    fn workload_id(value: &str) -> WorkloadId {
        WorkloadId::new(value).expect("valid workload id")
    }

    #[test]
    fn provision_persists_a_provisioned_principal() {
        let mut repo = InMemoryWorkloadPrincipalRepository::new();
        let principal =
            provision(&mut repo, "ten_acme", "wl_ci", "cap.cloud.kms").expect("provision");
        assert_eq!(principal.state(), WorkloadState::Provisioned);
        assert_eq!(repo.len(), 1);
        let stored = repo
            .load(&workload_id("wl_ci"))
            .expect("load")
            .expect("present");
        assert_eq!(stored.state(), WorkloadState::Provisioned);
    }

    #[test]
    fn provision_rejects_duplicate_and_invalid_ids() {
        let mut repo = InMemoryWorkloadPrincipalRepository::new();
        provision(&mut repo, "ten_acme", "wl_ci", "cap.cloud.kms").expect("first provision");
        assert!(matches!(
            provision(&mut repo, "ten_acme", "wl_ci", "cap.cloud.kms"),
            Err(LifecycleError::PrincipalAlreadyExists { .. })
        ));
        // Bad id shape surfaces the domain error.
        assert!(matches!(
            provision(&mut repo, "acme", "wl_ci2", "cap.cloud.kms"),
            Err(LifecycleError::Domain(
                WorkloadIdentityError::InvalidTenantId
            ))
        ));
    }

    #[test]
    fn activate_moves_provisioned_to_active() {
        let mut repo = InMemoryWorkloadPrincipalRepository::new();
        provision(&mut repo, "ten_acme", "wl_ci", "cap.cloud.kms").expect("provision");
        let active = activate(&mut repo, &workload_id("wl_ci")).expect("activate");
        assert_eq!(active.state(), WorkloadState::Active);
    }

    #[test]
    fn activate_missing_principal_is_not_found() {
        let mut repo = InMemoryWorkloadPrincipalRepository::new();
        assert!(matches!(
            activate(&mut repo, &workload_id("wl_ghost")),
            Err(LifecycleError::PrincipalNotFound { .. })
        ));
    }

    #[test]
    fn suspend_revokes_on_the_denylist() {
        let mut repo = InMemoryWorkloadPrincipalRepository::new();
        let mut denylist = InMemoryRevocationDenylist::new();
        provision(&mut repo, "ten_acme", "wl_ci", "cap.cloud.kms").expect("provision");
        activate(&mut repo, &workload_id("wl_ci")).expect("activate");
        let suspended = suspend(&mut repo, &mut denylist, &workload_id("wl_ci")).expect("suspend");
        assert_eq!(suspended.state(), WorkloadState::Suspended);
        assert!(
            denylist
                .is_revoked(&workload_id("wl_ci"))
                .expect("denylist read")
        );
    }

    #[test]
    fn illegal_suspend_does_not_revoke() {
        // Provisioned -> Suspended is illegal; the denylist must stay clean.
        let mut repo = InMemoryWorkloadPrincipalRepository::new();
        let mut denylist = InMemoryRevocationDenylist::new();
        provision(&mut repo, "ten_acme", "wl_ci", "cap.cloud.kms").expect("provision");
        assert!(matches!(
            suspend(&mut repo, &mut denylist, &workload_id("wl_ci")),
            Err(LifecycleError::Domain(
                WorkloadIdentityError::IllegalStateTransition { .. }
            ))
        ));
        assert!(denylist.is_empty());
    }

    #[test]
    fn retire_is_terminal_and_revokes() {
        let mut repo = InMemoryWorkloadPrincipalRepository::new();
        let mut denylist = InMemoryRevocationDenylist::new();
        provision(&mut repo, "ten_acme", "wl_ci", "cap.cloud.kms").expect("provision");
        activate(&mut repo, &workload_id("wl_ci")).expect("activate");
        let retired = retire(&mut repo, &mut denylist, &workload_id("wl_ci")).expect("retire");
        assert_eq!(retired.state(), WorkloadState::Retired);
        assert!(
            denylist
                .is_revoked(&workload_id("wl_ci"))
                .expect("denylist read")
        );
        // Re-activation of a retired principal is an illegal transition.
        assert!(matches!(
            activate(&mut repo, &workload_id("wl_ci")),
            Err(LifecycleError::Domain(
                WorkloadIdentityError::IllegalStateTransition {
                    from: WorkloadState::Retired,
                    to: WorkloadState::Active,
                }
            ))
        ));
    }

    #[test]
    fn authorize_outcome_decision_defaults_to_deny() {
        // Every fail-closed stage yields a default-deny decision.
        for outcome in [
            AuthorizeOutcome::TokenRejected,
            AuthorizeOutcome::PrincipalUnknown,
            AuthorizeOutcome::Revoked,
            AuthorizeOutcome::StoreUnavailable,
        ] {
            assert!(!outcome.is_allow());
            assert!(!outcome.decision().is_allow());
        }
    }

    #[test]
    fn store_outage_adapters_fail_closed_without_panicking() {
        // The fault-injecting adapters return Err rather than panicking, which is
        // what lets the authorize hot path map a store outage to a deny.
        let mut failing_repo = FailingRepository;
        assert!(failing_repo.load(&workload_id("wl_ci")).is_err());
        assert!(failing_repo.save(&dummy_active()).is_err());
        let mut failing_denylist = FailingDenylist;
        assert!(failing_denylist.revoke(&workload_id("wl_ci")).is_err());
        assert!(failing_denylist.is_revoked(&workload_id("wl_ci")).is_err());
    }

    fn dummy_active() -> WorkloadPrincipal {
        let mut principal =
            WorkloadPrincipal::provision("ten_acme", "wl_ci", "cap.cloud.kms").expect("valid");
        principal
            .transition_to(WorkloadState::Active)
            .expect("activate");
        principal
    }
}
