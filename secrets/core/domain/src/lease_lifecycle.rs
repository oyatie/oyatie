//! Dynamic secret-lease lifecycle (story G002: cloud-secrets dynamic
//! leasing; zero static secrets anywhere).
//!
//! Doctrine: secret material is handed out only under a live, TTL-bounded,
//! workload-identity-bound lease (HashiCorp Vault dynamic-secrets precedent;
//! CAEP revocation posture from ADR-0536 D-1). There is no unexpiring
//! handout:
//!
//! - issuance binds to a VALIDATED G001 [`Principal`] contract (Workload
//!   kind, Active state) — not a free-form string; the SPIFFE identity the
//!   G05 lane mints is what arrives here
//! - every lease has an ABSOLUTE lifetime ceiling: renewals extend from
//!   "now" (never stacking), consume a budget, and can never push the
//!   expiry past `issued_at + max_lifetime` — a fully-renewed lease still
//!   dies on schedule
//! - an expired lease can never be revived; the workload re-authenticates
//! - revocation is immediate, idempotent, dominates every other state, and
//!   EMITS a typed [`LeaseRevocationEvent`] exactly once — the CAEP-style
//!   shared-signals record the service publishes on the revocation stream
//!   (transport rides the G09 bus behind its port); the in-memory flag is
//!   the local state, the event is the propagation contract
//! - every liveness check fails closed with the reason and timestamp
//!
//! This module is the domain state machine the cloud-secrets service binary
//! drives; persistence arrives via the G03 port when that lane's adapters
//! land (cross-lane law: ports, never direct imports of unfinished work).

use std::fmt;
use std::num::NonZeroU64;

use oya_shared_platform_contracts_kernel::identity::{Principal, PrincipalKind, PrincipalState};

/// Bounds for lease TTLs: no sub-minute churn, no day-plus static handouts.
pub const MIN_LEASE_TTL_SECONDS: u64 = 60;
/// Upper TTL bound (24h): anything longer is a static secret in disguise.
pub const MAX_LEASE_TTL_SECONDS: u64 = 24 * 60 * 60;
/// Absolute lifetime ceiling (7d): no renewal chain may outlive this.
pub const MAX_LEASE_LIFETIME_SECONDS: u64 = 7 * 24 * 60 * 60;

const LEASE_ID_PREFIX: &str = "lease/";

/// Errors from the lease lifecycle. Liveness failures carry the timestamp
/// that ended the lease so callers can emit precise audit records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaseError {
    /// Lease id must be `lease/<slug>` with a non-empty, `/`-free slug.
    InvalidLeaseId,
    /// The secret reference must be non-empty.
    InvalidSecretReference,
    /// The G001 principal contract failed its own validation.
    PrincipalContractViolation {
        /// First violation rendered, for the audit record.
        detail: String,
    },
    /// Leases bind to workload identities only (zero static secrets:
    /// SPIFFE-minted workloads, not humans or federated externals).
    PrincipalNotWorkload {
        /// The rejected kind.
        kind: PrincipalKind,
    },
    /// Only Active principals may hold leases (PDP fail-closed mirror).
    PrincipalNotOperational {
        /// The rejected state.
        state: PrincipalState,
    },
    /// Requested TTL is outside `[MIN_LEASE_TTL_SECONDS, MAX_LEASE_TTL_SECONDS]`.
    TtlOutOfBounds {
        /// The rejected TTL.
        requested_seconds: u64,
    },
    /// Max lifetime must sit in `[ttl, MAX_LEASE_LIFETIME_SECONDS]`.
    LifetimeOutOfBounds {
        /// The rejected lifetime.
        requested_seconds: u64,
    },
    /// The lease TTL elapsed.
    Expired {
        /// Epoch second at which the lease expired.
        at_epoch_seconds: u64,
    },
    /// The lease was revoked.
    Revoked {
        /// Epoch second at which the lease was revoked.
        at_epoch_seconds: u64,
    },
    /// The renewal budget is spent; the workload must re-authenticate.
    RenewalsExhausted {
        /// Renewals the policy allowed.
        max_renewals: u32,
    },
    /// The absolute lifetime ceiling makes further extension meaningless.
    MaxLifetimeReached {
        /// The absolute expiry that cannot be extended.
        absolute_expiry_epoch_seconds: u64,
    },
}

impl fmt::Display for LeaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLeaseId => f.write_str("lease: id must be 'lease/<slug>'"),
            Self::InvalidSecretReference => {
                f.write_str("lease: secret reference must be non-empty")
            }
            Self::PrincipalContractViolation { detail } => {
                write!(f, "lease: principal contract violation: {detail}")
            }
            Self::PrincipalNotWorkload { kind } => {
                write!(
                    f,
                    "lease: principal kind {kind:?} rejected; leases bind to workloads"
                )
            }
            Self::PrincipalNotOperational { state } => {
                write!(
                    f,
                    "lease: principal state {state:?} rejected; Active only (fail closed)"
                )
            }
            Self::TtlOutOfBounds { requested_seconds } => write!(
                f,
                "lease: ttl {requested_seconds}s outside [{MIN_LEASE_TTL_SECONDS}, {MAX_LEASE_TTL_SECONDS}]"
            ),
            Self::LifetimeOutOfBounds { requested_seconds } => write!(
                f,
                "lease: max lifetime {requested_seconds}s outside [ttl, {MAX_LEASE_LIFETIME_SECONDS}]"
            ),
            Self::Expired { at_epoch_seconds } => {
                write!(
                    f,
                    "lease: expired at {at_epoch_seconds}; re-authenticate to re-issue"
                )
            }
            Self::Revoked { at_epoch_seconds } => {
                write!(f, "lease: revoked at {at_epoch_seconds}")
            }
            Self::RenewalsExhausted { max_renewals } => {
                write!(
                    f,
                    "lease: {max_renewals} renewals exhausted; re-authenticate to re-issue"
                )
            }
            Self::MaxLifetimeReached {
                absolute_expiry_epoch_seconds,
            } => write!(
                f,
                "lease: absolute lifetime ceiling {absolute_expiry_epoch_seconds} reached; re-authenticate to re-issue"
            ),
        }
    }
}

impl std::error::Error for LeaseError {}

/// Validated lease identifier (`lease/<slug>`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LeaseId {
    value: String, // data_class: INTERNAL_ONLY
}

impl LeaseId {
    /// Construct a validated lease id.
    pub fn new(value: impl Into<String>) -> Result<Self, LeaseError> {
        let value = value.into();
        let slug = value
            .strip_prefix(LEASE_ID_PREFIX)
            .ok_or(LeaseError::InvalidLeaseId)?;
        if slug.is_empty() || slug.contains('/') {
            return Err(LeaseError::InvalidLeaseId);
        }
        Ok(Self { value })
    }

    /// Full value (`lease/<slug>`).
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for LeaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

/// Issuance policy: validated TTL, renewal budget, and the absolute
/// lifetime ceiling no renewal chain may outlive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeasePolicy {
    ttl_seconds: NonZeroU64,
    max_renewals: u32,
    max_lifetime_seconds: NonZeroU64,
}

impl LeasePolicy {
    /// Construct a policy. The TTL must sit inside the doctrine bounds and
    /// the absolute lifetime inside `[ttl, MAX_LEASE_LIFETIME_SECONDS]`.
    pub fn new(
        ttl_seconds: u64,
        max_renewals: u32,
        max_lifetime_seconds: u64,
    ) -> Result<Self, LeaseError> {
        if !(MIN_LEASE_TTL_SECONDS..=MAX_LEASE_TTL_SECONDS).contains(&ttl_seconds) {
            return Err(LeaseError::TtlOutOfBounds {
                requested_seconds: ttl_seconds,
            });
        }
        if !(ttl_seconds..=MAX_LEASE_LIFETIME_SECONDS).contains(&max_lifetime_seconds) {
            return Err(LeaseError::LifetimeOutOfBounds {
                requested_seconds: max_lifetime_seconds,
            });
        }
        let ttl_seconds = NonZeroU64::new(ttl_seconds).ok_or(LeaseError::TtlOutOfBounds {
            requested_seconds: 0,
        })?;
        let max_lifetime_seconds =
            NonZeroU64::new(max_lifetime_seconds).ok_or(LeaseError::LifetimeOutOfBounds {
                requested_seconds: 0,
            })?;
        Ok(Self {
            ttl_seconds,
            max_renewals,
            max_lifetime_seconds,
        })
    }

    /// The validated TTL.
    pub fn ttl_seconds(&self) -> u64 {
        self.ttl_seconds.get()
    }

    /// The renewal budget.
    pub fn max_renewals(&self) -> u32 {
        self.max_renewals
    }

    /// The absolute lifetime ceiling.
    pub fn max_lifetime_seconds(&self) -> u64 {
        self.max_lifetime_seconds.get()
    }
}

/// Observable lease state, derived from time + revocation, never stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseState {
    /// Within TTL and not revoked.
    Live,
    /// TTL elapsed (and not revoked first).
    Expired,
    /// Revoked; dominates expiry.
    Revoked,
}

/// Why a lease was revoked — carried on the revocation event so downstream
/// consumers (PDP issue-time cutoffs, audit) can differentiate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationReason {
    /// Credential or workload compromise suspected.
    CompromiseSuspected,
    /// The holding principal was suspended/deprovisioned.
    PrincipalDeprovisioned,
    /// Policy or rotation mandated early termination.
    PolicyChange,
    /// Operator-initiated administrative revocation.
    Administrative,
}

/// CAEP-style shared-signals revocation record (ADR-0536 D-1 posture),
/// emitted EXACTLY ONCE by [`DynamicLease::revoke`]. The service publishes
/// this on the revocation stream (G09 bus behind its port); every consumer
/// treats receipt as an issue-time cutoff for the lease.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseRevocationEvent {
    /// The revoked lease.
    pub lease_id: LeaseId,
    /// Workload principal that held the lease.
    pub principal_id: String, // data_class: PII_QUASI_IDENTIFIER
    /// Tenant scope of the principal.
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// The secret the lease granted access to.
    pub secret_reference: String, // data_class: INTERNAL_ONLY
    /// Why the lease was revoked.
    pub reason: RevocationReason,
    /// When the revocation took effect.
    pub revoked_at_epoch_seconds: u64,
}

/// A dynamic, TTL-bounded, workload-identity-bound secret lease.
#[derive(Clone, PartialEq, Eq)]
pub struct DynamicLease {
    lease_id: LeaseId,
    secret_reference: String, // data_class: INTERNAL_ONLY
    principal_id: String,     // data_class: PII_QUASI_IDENTIFIER
    tenant_id: String,        // data_class: TENANT_SCOPED
    policy: LeasePolicy,
    issued_at_epoch_seconds: u64,
    expires_at_epoch_seconds: u64,
    absolute_expiry_epoch_seconds: u64,
    renewals_used: u32,
    revoked_at_epoch_seconds: Option<u64>,
}

impl DynamicLease {
    /// Issue a lease bound to a secret reference and a VALIDATED workload
    /// principal: the G001 contract must validate, the kind must be
    /// `Workload` (SPIFFE-minted by the G05 lane), and the state must be
    /// `Active` — anything else fails closed before a lease exists.
    pub fn issue(
        lease_id: LeaseId,
        secret_reference: impl Into<String>,
        principal: &Principal,
        policy: LeasePolicy,
        now_epoch_seconds: u64,
    ) -> Result<Self, LeaseError> {
        let secret_reference = secret_reference.into();
        if secret_reference.trim().is_empty() {
            return Err(LeaseError::InvalidSecretReference);
        }
        if let Err(violations) = principal.validate() {
            let detail = violations
                .first()
                .map(ToString::to_string)
                .unwrap_or_else(|| "unknown violation".to_owned());
            return Err(LeaseError::PrincipalContractViolation { detail });
        }
        if principal.kind != PrincipalKind::Workload {
            return Err(LeaseError::PrincipalNotWorkload {
                kind: principal.kind,
            });
        }
        if !principal.state.is_operational() {
            return Err(LeaseError::PrincipalNotOperational {
                state: principal.state,
            });
        }
        let absolute_expiry_epoch_seconds =
            now_epoch_seconds.saturating_add(policy.max_lifetime_seconds());
        let expires_at_epoch_seconds = now_epoch_seconds
            .saturating_add(policy.ttl_seconds())
            .min(absolute_expiry_epoch_seconds);
        Ok(Self {
            lease_id,
            secret_reference,
            principal_id: principal.principal_id.clone(),
            tenant_id: principal.tenant_id.clone(),
            policy,
            issued_at_epoch_seconds: now_epoch_seconds,
            expires_at_epoch_seconds,
            absolute_expiry_epoch_seconds,
            renewals_used: 0,
            revoked_at_epoch_seconds: None,
        })
    }

    /// Lease identifier.
    pub fn lease_id(&self) -> &LeaseId {
        &self.lease_id
    }

    /// The secret this lease grants access to.
    pub fn secret_reference(&self) -> &str {
        &self.secret_reference
    }

    /// The workload principal id holding this lease.
    pub fn principal_id(&self) -> &str {
        &self.principal_id
    }

    /// Tenant scope inherited from the principal.
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Current expiry.
    pub fn expires_at_epoch_seconds(&self) -> u64 {
        self.expires_at_epoch_seconds
    }

    /// The ceiling no renewal can push the expiry past.
    pub fn absolute_expiry_epoch_seconds(&self) -> u64 {
        self.absolute_expiry_epoch_seconds
    }

    /// Renewals consumed so far.
    pub fn renewals_used(&self) -> u32 {
        self.renewals_used
    }

    /// Derive the state at `now`. Revocation dominates; expiry is inclusive
    /// (a lease IS expired at its expiry second — fail closed on the bound).
    pub fn state(&self, now_epoch_seconds: u64) -> LeaseState {
        if self.revoked_at_epoch_seconds.is_some() {
            return LeaseState::Revoked;
        }
        if now_epoch_seconds >= self.expires_at_epoch_seconds {
            return LeaseState::Expired;
        }
        LeaseState::Live
    }

    /// Fail-closed liveness gate: material handout paths call this before
    /// every release of secret material.
    pub fn assert_live(&self, now_epoch_seconds: u64) -> Result<(), LeaseError> {
        match self.state(now_epoch_seconds) {
            LeaseState::Live => Ok(()),
            LeaseState::Expired => Err(LeaseError::Expired {
                at_epoch_seconds: self.expires_at_epoch_seconds,
            }),
            LeaseState::Revoked => Err(LeaseError::Revoked {
                at_epoch_seconds: self.revoked_at_epoch_seconds.unwrap_or(0),
            }),
        }
    }

    /// Renew a LIVE lease: extends to `min(now + ttl, absolute expiry)`,
    /// consuming one renewal from the budget. Expired or revoked leases
    /// cannot renew, and once the absolute ceiling is the binding bound no
    /// further extension is possible — re-authentication is the only path.
    pub fn renew(&mut self, now_epoch_seconds: u64) -> Result<u64, LeaseError> {
        self.assert_live(now_epoch_seconds)?;
        if self.renewals_used >= self.policy.max_renewals() {
            return Err(LeaseError::RenewalsExhausted {
                max_renewals: self.policy.max_renewals(),
            });
        }
        let candidate = now_epoch_seconds
            .saturating_add(self.policy.ttl_seconds())
            .min(self.absolute_expiry_epoch_seconds);
        if candidate <= self.expires_at_epoch_seconds
            && self.expires_at_epoch_seconds == self.absolute_expiry_epoch_seconds
        {
            return Err(LeaseError::MaxLifetimeReached {
                absolute_expiry_epoch_seconds: self.absolute_expiry_epoch_seconds,
            });
        }
        self.renewals_used += 1;
        self.expires_at_epoch_seconds = candidate;
        Ok(self.expires_at_epoch_seconds)
    }

    /// Revoke immediately (CAEP shared-signals posture). Idempotent: the
    /// FIRST revocation wins and returns the [`LeaseRevocationEvent`] to be
    /// published on the revocation stream; subsequent calls return `None`
    /// (state already terminal, no duplicate signal).
    pub fn revoke(
        &mut self,
        now_epoch_seconds: u64,
        reason: RevocationReason,
    ) -> Option<LeaseRevocationEvent> {
        if self.revoked_at_epoch_seconds.is_some() {
            return None;
        }
        self.revoked_at_epoch_seconds = Some(now_epoch_seconds);
        Some(LeaseRevocationEvent {
            lease_id: self.lease_id.clone(),
            principal_id: self.principal_id.clone(),
            tenant_id: self.tenant_id.clone(),
            secret_reference: self.secret_reference.clone(),
            reason,
            revoked_at_epoch_seconds: now_epoch_seconds,
        })
    }
}

impl fmt::Debug for DynamicLease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // principal_id (PII_QUASI_IDENTIFIER) and secret_reference
        // (INTERNAL_ONLY) are redacted, mirroring the kms material types.
        write!(
            f,
            "DynamicLease {{ lease_id: {}, tenant_id: {}, principal_id: [REDACTED], secret_reference: [REDACTED], expires_at: {}, absolute_expiry: {}, renewals_used: {}, revoked: {} }}",
            self.lease_id,
            self.tenant_id,
            self.expires_at_epoch_seconds,
            self.absolute_expiry_epoch_seconds,
            self.renewals_used,
            self.revoked_at_epoch_seconds.is_some()
        )
    }
}
