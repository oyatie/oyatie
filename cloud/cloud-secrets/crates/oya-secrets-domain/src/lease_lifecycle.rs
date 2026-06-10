//! Dynamic secret-lease lifecycle (story G002: cloud-secrets dynamic
//! leasing; zero static secrets anywhere).
//!
//! Doctrine: secret material is handed out only under a live, TTL-bounded,
//! principal-bound lease (HashiCorp Vault dynamic-secrets precedent; CAEP
//! revocation posture from ADR-0536 D-1). There is no unexpiring handout:
//!
//! - issuance requires a workload principal and a policy-validated TTL
//! - renewal extends from "now" (never stacks onto the old expiry), is
//!   capped, and is only possible while the lease is LIVE — an expired
//!   lease can never be revived, the workload must re-authenticate
//! - revocation is immediate, idempotent, and dominates every other state
//! - every liveness check fails closed with the reason and timestamp
//!
//! This module is the domain state machine the cloud-secrets service binary
//! drives; persistence arrives via the G03 port when that lane's adapters
//! land (cross-lane law: ports, never direct imports of unfinished work).

use std::fmt;
use std::num::NonZeroU64;

/// Bounds for lease TTLs: no sub-minute churn, no day-plus static handouts.
pub const MIN_LEASE_TTL_SECONDS: u64 = 60;
/// Upper TTL bound (24h): anything longer is a static secret in disguise.
pub const MAX_LEASE_TTL_SECONDS: u64 = 24 * 60 * 60;

const LEASE_ID_PREFIX: &str = "lease/";

/// Errors from the lease lifecycle. Liveness failures carry the timestamp
/// that ended the lease so callers can emit precise audit records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaseError {
    /// Lease id must be `lease/<slug>` with a non-empty, `/`-free slug.
    InvalidLeaseId,
    /// The secret reference must be non-empty.
    InvalidSecretReference,
    /// Zero-static-secrets doctrine: every lease binds to a principal.
    InvalidPrincipal,
    /// Requested TTL is outside `[MIN_LEASE_TTL_SECONDS, MAX_LEASE_TTL_SECONDS]`.
    TtlOutOfBounds {
        /// The rejected TTL.
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
}

impl fmt::Display for LeaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLeaseId => f.write_str("lease: id must be 'lease/<slug>'"),
            Self::InvalidSecretReference => f.write_str("lease: secret reference must be non-empty"),
            Self::InvalidPrincipal => {
                f.write_str("lease: a workload principal is required (zero static secrets)")
            }
            Self::TtlOutOfBounds { requested_seconds } => write!(
                f,
                "lease: ttl {requested_seconds}s outside [{MIN_LEASE_TTL_SECONDS}, {MAX_LEASE_TTL_SECONDS}]"
            ),
            Self::Expired { at_epoch_seconds } => {
                write!(f, "lease: expired at {at_epoch_seconds}; re-authenticate to re-issue")
            }
            Self::Revoked { at_epoch_seconds } => {
                write!(f, "lease: revoked at {at_epoch_seconds}")
            }
            Self::RenewalsExhausted { max_renewals } => {
                write!(f, "lease: {max_renewals} renewals exhausted; re-authenticate to re-issue")
            }
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
        let slug = value.strip_prefix(LEASE_ID_PREFIX).ok_or(LeaseError::InvalidLeaseId)?;
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

/// Issuance policy: validated TTL plus a renewal budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeasePolicy {
    ttl_seconds: NonZeroU64,
    max_renewals: u32,
}

impl LeasePolicy {
    /// Construct a policy; the TTL must sit inside the doctrine bounds.
    pub fn new(ttl_seconds: u64, max_renewals: u32) -> Result<Self, LeaseError> {
        if !(MIN_LEASE_TTL_SECONDS..=MAX_LEASE_TTL_SECONDS).contains(&ttl_seconds) {
            return Err(LeaseError::TtlOutOfBounds { requested_seconds: ttl_seconds });
        }
        let ttl_seconds = NonZeroU64::new(ttl_seconds)
            .ok_or(LeaseError::TtlOutOfBounds { requested_seconds: 0 })?;
        Ok(Self { ttl_seconds, max_renewals })
    }

    /// The validated TTL.
    pub fn ttl_seconds(&self) -> u64 {
        self.ttl_seconds.get()
    }

    /// The renewal budget.
    pub fn max_renewals(&self) -> u32 {
        self.max_renewals
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

/// A dynamic, TTL-bounded, principal-bound secret lease.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicLease {
    lease_id: LeaseId,
    secret_reference: String, // data_class: INTERNAL_ONLY
    principal: String,        // data_class: INTERNAL_ONLY
    policy: LeasePolicy,
    issued_at_epoch_seconds: u64,
    expires_at_epoch_seconds: u64,
    renewals_used: u32,
    revoked_at_epoch_seconds: Option<u64>,
}

impl DynamicLease {
    /// Issue a lease bound to a secret reference and a workload principal.
    pub fn issue(
        lease_id: LeaseId,
        secret_reference: impl Into<String>,
        principal: impl Into<String>,
        policy: LeasePolicy,
        now_epoch_seconds: u64,
    ) -> Result<Self, LeaseError> {
        let secret_reference = secret_reference.into();
        if secret_reference.trim().is_empty() {
            return Err(LeaseError::InvalidSecretReference);
        }
        let principal = principal.into();
        if principal.trim().is_empty() {
            return Err(LeaseError::InvalidPrincipal);
        }
        Ok(Self {
            lease_id,
            secret_reference,
            principal,
            policy,
            issued_at_epoch_seconds: now_epoch_seconds,
            expires_at_epoch_seconds: now_epoch_seconds.saturating_add(policy.ttl_seconds()),
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

    /// The workload principal holding this lease.
    pub fn principal(&self) -> &str {
        &self.principal
    }

    /// Current expiry.
    pub fn expires_at_epoch_seconds(&self) -> u64 {
        self.expires_at_epoch_seconds
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
            LeaseState::Expired => {
                Err(LeaseError::Expired { at_epoch_seconds: self.expires_at_epoch_seconds })
            }
            LeaseState::Revoked => Err(LeaseError::Revoked {
                at_epoch_seconds: self.revoked_at_epoch_seconds.unwrap_or(0),
            }),
        }
    }

    /// Renew a LIVE lease: extends to `now + ttl` (never stacking onto the
    /// old expiry), consuming one renewal from the budget. Expired or
    /// revoked leases cannot renew — re-authentication is the only path.
    pub fn renew(&mut self, now_epoch_seconds: u64) -> Result<u64, LeaseError> {
        self.assert_live(now_epoch_seconds)?;
        if self.renewals_used >= self.policy.max_renewals() {
            return Err(LeaseError::RenewalsExhausted {
                max_renewals: self.policy.max_renewals(),
            });
        }
        self.renewals_used += 1;
        self.expires_at_epoch_seconds =
            now_epoch_seconds.saturating_add(self.policy.ttl_seconds());
        Ok(self.expires_at_epoch_seconds)
    }

    /// Revoke immediately (CAEP-style). Idempotent: the first revocation
    /// timestamp wins.
    pub fn revoke(&mut self, now_epoch_seconds: u64) {
        if self.revoked_at_epoch_seconds.is_none() {
            self.revoked_at_epoch_seconds = Some(now_epoch_seconds);
        }
    }
}
