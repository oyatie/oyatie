//! Lifecycle-locks kernel (IP-021) - pure logic for lock creation, precedence,
//! release authorization, expiry, and decision explanation. Usecases consume it
//! before `RequestTenantDeletion`, jurisdiction migration, payment-method
//! removal, KYB/KYC re-verification, and DR pair promotion.
//!
//! Precedent: AWS S3 Object Lock (governance/compliance retention that an
//! ordinary principal cannot lift) and Cloudflare zone lock (human-gated
//! protection of destructive account changes), lifted to tenant lifecycle
//! transitions.
//!
//! # Shape
//!
//! IP-021 plans four crates (`lock` / `precedence` / `release` / `errors`).
//! The tenancy capability is capped at twelve crates and the workspace
//! lockfile is frozen, so that layout is collapsed into this single crate as a
//! module tree:
//!
//! - [`precedence`] - the [`LifecycleAction`] enum and the reason-vs-action
//!   matrix, plus acquisition conflict detection.
//! - [`release`] - holder authorization, lease ceilings, renewal, and the
//!   multi-party release quorum.
//! - [`inmemory`] - an in-memory [`LockStore`] adapter.
//!
//! # Determinism
//!
//! Nothing here reads a clock or a random source. Every expiry-sensitive entry
//! point takes `now_epoch_s` as an explicit parameter, so a test can name the
//! instant and replay it exactly.
//!
//! A lock is LIVE at `now` iff `now < expires_at_epoch_s`. Expiry is therefore
//! exclusive at its own instant: at `expires_at_epoch_s` the lock has already
//! lapsed and blocks nothing. An `expires_at_epoch_s` of `0` is a lock that
//! never had force.
//!
//! # Identifiers are canonical
//!
//! [`LifecycleLock::new`] and [`release::ReleaseApproval::new`] are the only
//! way a valid identifier is minted, and they STORE what they validated:
//! surrounding whitespace is stripped, control characters and over-long strings
//! are refused. Every lookup ([`LockStore`] and its in-memory adapter) and
//! every comparison ([`release::is_holder`]) trims as well.
//!
//! That is load-bearing, not tidiness. A lock stored under `" ten_acme"` while
//! the deletion path asks about `"ten_acme"` is a shadow namespace that fails
//! OPEN: a statutory hold is on record and blocks nothing. A holder recorded as
//! `"svc-dr "` can never release its own lock. A control character in an id
//! forges a line in the audit log that renders it.
//!
//! The contract is exactly ASCII/Unicode whitespace trimming. There is NO case
//! folding and NO Unicode normalization - see Gaps.
//!
//! # Gaps (deliberately deferred)
//!
//! - **Persistence.** Only the in-memory [`inmemory::InMemoryLockStore`] exists.
//!   The Postgres/`sqlx` adapter IP-021 implies is not written here: this crate
//!   may not gain a dependency (the workspace `Cargo.lock` is owned by another
//!   lane and this capability holds no waiver), and a real adapter needs
//!   `sqlx` + a pool. The [`LockStore`] port is the seam it will land behind.
//! - **Async.** [`LockStore`] is a SYNC trait for the same reason - no `tokio`.
//!   An async port is a mechanical widening once the lockfile opens.
//! - **Domain events.** `oya.tenancy.lifecycle-lock-applied`,
//!   `-release-requested` and `-released` (IP-021 §E) are not emitted; there is
//!   no in-crate event bus and emitting them needs `serde` + a transport. Every
//!   mutation returns the affected [`LifecycleLock`], which is the payload an
//!   emitting layer needs.
//! - **Audit-chain sealing.** Merkle sealing of release decisions belongs to
//!   the audit capability and is not reproduced here.
//! - **Property tests.** Coverage is exhaustive-by-enumeration over the closed
//!   matrix (every reason x every action, and every ordering of a four-approval
//!   quorum) rather than randomized, because the input domains are finite and
//!   enumerating them is strictly stronger.
//! - **Unicode canonicalization.** Identifier canonicalization is whitespace
//!   trimming only. Two ids that differ by NFC/NFD form, or by a homoglyph, are
//!   two different locks here. Closing that needs `unicode-normalization`,
//!   which is a dependency; the seam is [`canonical_identifier`], which every
//!   constructor already funnels through.
//! - **Error payloads.** [`LockKernelError`] variants are fieldless, so a
//!   refusal does not itself name the lock, tenant or conflicting row. The
//!   detail is reachable beside the error -
//!   [`precedence::acquisition_conflict`] returns the contradicting lock and
//!   [`release::quorum_shortfall`] returns the unfilled role - but a caller has
//!   to ask for it. Widening the variants is a public-API change that belongs
//!   with the durable adapter, not ahead of it.
//! - **Retention policy.** Lapsed and superseded rows are retained until
//!   [`LockStore::purge_expired`] is called. There is no row cap, no age-out
//!   and no automatic eviction: unbounded growth is bounded only by how often
//!   the operator purges. A durable adapter should carry a real retention
//!   window instead.
//! - **IP-021 §D.1 lock kinds.** [`LockReason`] models the DSR grace window,
//!   jurisdiction migration, KYB re-verification, DR promotion, payment
//!   dispute, legal hold and the manual soft lock. IP-021's
//!   `IncidentContainment` and `RegulatorInvestigation` kinds are NOT modelled:
//!   both behave as holds a legal hold already covers, and inventing rows in
//!   the precedence matrix for them without an owning usecase would be
//!   guessing. They land with the incident-response usecase that needs them.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod inmemory;
pub mod precedence;
pub mod release;

pub use inmemory::InMemoryLockStore;
pub use precedence::LifecycleAction;
pub use release::{ReleaseApproval, ReleaseRole};

/// Longest identifier - lock id, tenant id, holder, approving principal - this
/// kernel will hold. A bound is part of the contract: an unbounded id is an
/// unbounded audit line and an unbounded map key.
pub const MAX_IDENTIFIER_CHARS: usize = 256;

/// Longest run of caller-supplied text echoed into an operator-facing
/// explanation. See [`sanitize_for_explanation`].
pub const MAX_ECHOED_CHARS: usize = 64;

/// Canonicalize and validate one identifier.
///
/// Trims surrounding whitespace and returns what it validated, so a caller can
/// never validate one string and store another. This is the single seam every
/// identifier in the crate passes through.
///
/// # Errors
///
/// [`LockKernelError::InvalidLock`] when the trimmed value is empty, contains a
/// control character (which would forge a line in any log that renders it), or
/// is longer than [`MAX_IDENTIFIER_CHARS`].
pub fn canonical_identifier(raw: &str) -> Result<String, LockKernelError> {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(char::is_control)
        || trimmed.chars().count() > MAX_IDENTIFIER_CHARS
    {
        return Err(LockKernelError::InvalidLock);
    }
    Ok(trimmed.to_owned())
}

/// Render caller-supplied text safe for an operator-facing explanation.
///
/// Control characters become `?` and the result is capped at
/// [`MAX_ECHOED_CHARS`] characters with a visible truncation marker. Both
/// halves matter: an explanation is written to logs and audit records, so an
/// unescaped newline lets a caller inject a whole extra line - a forged
/// `allowed` verdict of its own (CWE-117) - and an uncapped echo turns a 10 MB
/// request field into a 10 MB audit row.
#[must_use]
pub fn sanitize_for_explanation(raw: &str) -> String {
    let mut rendered: String = raw
        .chars()
        .take(MAX_ECHOED_CHARS)
        .map(|character| {
            if character.is_control() {
                '?'
            } else {
                character
            }
        })
        .collect();
    if raw.chars().nth(MAX_ECHOED_CHARS).is_some() {
        rendered.push_str("...(truncated)");
    }
    rendered
}

/// Stable identifier of a single lifecycle lock. Unique within a tenant.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LockId(pub String); // data_class: INTERNAL_ONLY

impl LockId {
    /// The identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for LockId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why a lifecycle lock stands. The reason - not the holder - decides which
/// lifecycle actions the lock blocks (see [`precedence`]) and who may lift it
/// (see [`release`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockReason {
    /// The tenant is inside the DSR deletion grace window.
    PendingDeletionGrace,
    /// A residency / jurisdiction migration is in flight.
    JurisdictionMigration,
    /// KYB/KYC re-verification of the legal entity is in flight.
    KybReverification,
    /// A DR pair promotion window is open.
    DrPromotionWindow,
    /// An open payment dispute imposes a retention obligation.
    PaymentDispute,
    /// A statutory preservation order. The strongest reason, and the only one
    /// that no ordinary holder release can lift.
    LegalHold,
    /// An operator placed a manual protection lock against accidental
    /// destructive change (IP-021 §D.1's `ManualSoftLock`, the Cloudflare
    /// zone-lock shape). The weakest reason, and the only one a tenant admin
    /// can lift alone (IP-021 §D.4).
    ManualSoftLock,
}

/// One lock standing against a tenant.
///
/// The record carries no cell or region: locks survive DR pair promotion
/// precisely because kernel state is independent of cell location (IP-021 §D.5).
///
/// The fields are public so a caller can read them; they are NOT a construction
/// path. Build through [`LifecycleLock::new`], which canonicalizes, and note
/// that [`LockStore::acquire`] re-canonicalizes anything handed to it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleLock {
    /// Tenant-unique lock identifier.
    pub id: LockId, // data_class: INTERNAL_ONLY
    /// Tenant the lock stands against.
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    /// Why the lock stands.
    pub reason: LockReason, // data_class: INTERNAL_ONLY
    /// Principal that acquired the lock and holds its lease. Classified
    /// `TENANT_SCOPED` because a [`LockReason::ManualSoftLock`] is placed and
    /// lifted tenant-side, so this can carry a tenant's own user identifier.
    pub holder: String, // data_class: TENANT_SCOPED
    /// Instant the lease lapses. The lock is live iff `now < expires_at_epoch_s`.
    pub expires_at_epoch_s: u64, // data_class: INTERNAL_ONLY
}

impl LifecycleLock {
    /// Build a lock, canonicalizing the identifiers and rejecting the shapes no
    /// store may hold.
    ///
    /// The id, tenant and holder are stored TRIMMED - what was validated is
    /// what is kept, so no lookup can miss a lock it should have found.
    ///
    /// # Errors
    ///
    /// [`LockKernelError::InvalidLock`] when the id, tenant, or holder is
    /// blank after trimming, carries a control character, or exceeds
    /// [`MAX_IDENTIFIER_CHARS`]. See [`canonical_identifier`].
    pub fn new(
        id: LockId,
        tenant_id: String,
        reason: LockReason,
        holder: String,
        expires_at_epoch_s: u64,
    ) -> Result<Self, LockKernelError> {
        Ok(Self {
            id: LockId(canonical_identifier(id.as_str())?),
            tenant_id: canonical_identifier(&tenant_id)?,
            reason,
            holder: canonical_identifier(&holder)?,
            expires_at_epoch_s,
        })
    }

    /// This lock with every identifier re-canonicalized.
    ///
    /// The fields are public, so a lock reaching a store may never have been
    /// through [`Self::new`], or may have been edited afterwards. A store calls
    /// this rather than trusting what it was handed.
    ///
    /// # Errors
    ///
    /// [`LockKernelError::InvalidLock`], as [`Self::new`].
    pub fn canonicalized(self) -> Result<Self, LockKernelError> {
        Self::new(
            self.id,
            self.tenant_id,
            self.reason,
            self.holder,
            self.expires_at_epoch_s,
        )
    }

    /// Whether the lease still stands at `now_epoch_s`.
    ///
    /// Exclusive at the expiry instant: a lock expiring at `E` is live at
    /// `E - 1` and lapsed at `E`.
    #[must_use]
    pub const fn is_live_at(&self, now_epoch_s: u64) -> bool {
        now_epoch_s < self.expires_at_epoch_s
    }

    /// Whether the lease has lapsed at `now_epoch_s`. The complement of
    /// [`Self::is_live_at`].
    #[must_use]
    pub const fn is_expired_at(&self, now_epoch_s: u64) -> bool {
        !self.is_live_at(now_epoch_s)
    }

    /// Operator-facing fragment naming this lock and why it stands.
    ///
    /// The id is passed through [`sanitize_for_explanation`]: it is normally
    /// canonical, but the field is public and this string lands in audit text.
    #[must_use]
    pub fn describe(&self) -> String {
        format!(
            "{} ({}: {})",
            sanitize_for_explanation(self.id.as_str()),
            self.reason.as_slug(),
            self.reason.rationale()
        )
    }
}

/// The verdict of an evaluation: whether the action may proceed, which locks
/// stop it, which of those governs, and a sentence an operator can act on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockDecision {
    /// Whether the lifecycle action may proceed.
    pub allow: bool, // data_class: INTERNAL_ONLY
    /// Blocking locks, strongest reason first, ties broken by ascending id.
    pub blocking_locks: Vec<LockId>, // data_class: INTERNAL_ONLY
    /// The highest-precedence blocking lock, i.e. the first of
    /// [`Self::blocking_locks`]. `None` exactly when nothing blocks.
    pub governing_lock: Option<LockId>, // data_class: INTERNAL_ONLY
    /// Deterministic operator-facing explanation naming WHICH locks blocked and
    /// WHY. An operator who cannot tell why an action was refused will force it,
    /// so this text is part of the contract and is asserted by tests.
    ///
    /// Every caller-supplied fragment in it has been through
    /// [`sanitize_for_explanation`], because this string is written to logs and
    /// audit records.
    pub explanation: String, // data_class: INTERNAL_ONLY
}

/// Every way this kernel can refuse.
///
/// The variants are fieldless; where a refusal has an interesting subject, the
/// companion query returns it ([`precedence::acquisition_conflict`],
/// [`release::quorum_shortfall`]). See the crate-level Gaps.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LockKernelError {
    /// The acquisition contradicts a lock already standing: the new lock exists
    /// to perform an action that the standing lock blocks.
    PrecedenceConflict,
    /// The principal may not lift this lock - it is not the holder.
    ReleaseUnauthorized,
    /// This lock's reason admits no holder release at all; only a multi-party
    /// quorum can lift it. Distinct from [`Self::ReleaseUnauthorized`] so the
    /// holder is told to convene the quorum rather than to retry as somebody
    /// else.
    ReleaseRequiresQuorum,
    /// The lease has already lapsed at the supplied instant.
    Expired,
    /// The id, tenant, or holder is blank, control-bearing, or over-long.
    InvalidLock,
    /// A live lock with the same id already stands against the tenant.
    AlreadyHeld,
    /// No lock with that id stands against the tenant.
    NotFound,
    /// A renewal must extend the lease; it may not shorten or freeze it.
    RenewalNotExtending,
    /// The requested lease runs further ahead than this reason permits at one
    /// grant. See [`release::max_lease_seconds`].
    LeaseTooLong,
    /// The release quorum is short a required role, or one principal tried to
    /// fill two required roles at once.
    QuorumNotMet,
}

impl core::fmt::Display for LockKernelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::PrecedenceConflict => "acquisition contradicts a lock already standing",
            Self::ReleaseUnauthorized => "principal is not the holder of this lock",
            Self::ReleaseRequiresQuorum => {
                "this lock's reason can only be lifted by a multi-party quorum, not by its holder"
            }
            Self::Expired => "the lock lease has already lapsed",
            Self::InvalidLock => {
                "lock id, tenant, and holder must be non-blank, free of control characters, \
                 and within the identifier length limit"
            }
            Self::AlreadyHeld => "a live lock with that id already stands against the tenant",
            Self::NotFound => "no lock with that id stands against the tenant",
            Self::RenewalNotExtending => "a renewal must extend the lease",
            Self::LeaseTooLong => "the lease runs further ahead than this lock reason permits",
            Self::QuorumNotMet => "the release quorum is not met",
        };
        f.write_str(message)
    }
}

impl std::error::Error for LockKernelError {}

/// The instant [`evaluate`] evaluates at.
///
/// Epoch zero: the earliest instant at which every lock with any positive
/// expiry is still live. See [`evaluate`] for why that is the conservative
/// choice for a signature that carries no instant.
pub const LEGACY_EVALUATION_INSTANT: u64 = 0;

/// Evaluate `action` against `locks` WITHOUT an instant.
///
/// Preserved for callers built against the original signature. It carries no
/// time, so it cannot honour expiry; rather than guess, it delegates to
/// [`evaluate_at`] at [`LEGACY_EVALUATION_INSTANT`] (epoch zero), where every
/// lock with a positive expiry is still live. That is the fail-CLOSED reading:
/// a caller with no clock is told the lock still stands, and can only ever be
/// more conservative than the truth - never less. The opposite convention
/// (evaluating at `u64::MAX`) would silently expire every lock and let a legal
/// hold be walked through.
///
/// `action` is matched against [`LifecycleAction::from_slug`]. An unrecognized
/// action also fails closed: `allow` is `false` and the explanation says the
/// action was not recognized. In that case `blocking_locks` may be empty while
/// `allow` is `false` - the refusal is the kernel's, not a lock's.
///
/// The unrecognized slug is echoed back through
/// [`sanitize_for_explanation`], never raw: `action` is wire text, and this
/// explanation is written to audit records.
///
/// New callers should use [`evaluate_at`], which honours expiry.
#[must_use]
pub fn evaluate(action: &str, locks: &[LifecycleLock]) -> LockDecision {
    match LifecycleAction::from_slug(action) {
        Some(known) => evaluate_at(known, locks, LEGACY_EVALUATION_INSTANT),
        None => LockDecision {
            allow: false,
            blocking_locks: Vec::new(),
            governing_lock: None,
            explanation: format!(
                "action={} refused: unrecognized lifecycle action, failing closed; \
                 known actions are {}",
                sanitize_for_explanation(action),
                LifecycleAction::slug_list()
            ),
        },
    }
}

/// Evaluate `action` against `locks` at `now_epoch_s`.
///
/// A lock blocks iff it is LIVE at `now_epoch_s` (see [`LifecycleLock::is_live_at`])
/// AND its reason blocks that action (see [`precedence`]). Expired locks are
/// invisible: an expired lock blocks nothing, whatever its reason.
///
/// Blocking locks - and the explanation naming them - are ordered by descending
/// reason precedence and then ascending lock id, so the same input always
/// produces byte-identical output.
///
/// Locks belonging to other tenants are NOT filtered here; the caller supplies
/// the tenant's own locks (see [`LockStore::live_locks`]). A caller that cannot
/// guarantee that should use [`evaluate_for_tenant`], which enforces it - the
/// explanation names lock ids, and one tenant must not be shown another's.
#[must_use]
pub fn evaluate_at(
    action: LifecycleAction,
    locks: &[LifecycleLock],
    now_epoch_s: u64,
) -> LockDecision {
    let mut blockers: Vec<&LifecycleLock> = locks
        .iter()
        .filter(|lock| lock.is_live_at(now_epoch_s) && lock.reason.blocks(action))
        .collect();
    blockers.sort_by(|left, right| {
        right
            .reason
            .precedence()
            .cmp(&left.reason.precedence())
            .then_with(|| left.id.cmp(&right.id))
    });

    let considered = locks.len();
    let explanation = if blockers.is_empty() {
        format!(
            "action={} allowed: 0 of {considered} lock(s) block it",
            action.as_slug()
        )
    } else {
        let detail: Vec<String> = blockers.iter().map(|lock| lock.describe()).collect();
        format!(
            "action={} denied: {} of {considered} lock(s) block it: {}",
            action.as_slug(),
            blockers.len(),
            detail.join("; ")
        )
    };

    LockDecision {
        allow: blockers.is_empty(),
        governing_lock: blockers.first().map(|lock| lock.id.clone()),
        blocking_locks: blockers.into_iter().map(|lock| lock.id.clone()).collect(),
        explanation,
    }
}

/// Evaluate `action` for ONE tenant against a slice that may hold any tenant's
/// locks.
///
/// Rows of other tenants are dropped before evaluation and before the count in
/// the explanation. [`precedence::acquisition_conflict`] guards the same hazard
/// on the acquisition path; this is the same invariant on the evaluation path,
/// so a mixed-tenant slice can neither deny tenant A because of tenant B's hold
/// nor render B's lock id into text an operator of A will read.
#[must_use]
pub fn evaluate_for_tenant(
    tenant_id: &str,
    action: LifecycleAction,
    locks: &[LifecycleLock],
    now_epoch_s: u64,
) -> LockDecision {
    let tenant = tenant_id.trim();
    let scoped: Vec<LifecycleLock> = locks
        .iter()
        .filter(|lock| lock.tenant_id.trim() == tenant)
        .cloned()
        .collect();
    evaluate_at(action, &scoped, now_epoch_s)
}

/// The storage port lifecycle locks live behind.
///
/// Sync by design: this crate takes no dependencies, so there is no async
/// runtime to await on (see the crate-level Gaps). The only implementation
/// today is [`inmemory::InMemoryLockStore`]; a durable adapter lands behind
/// this same trait.
pub trait LockStore {
    /// Place `lock` against its tenant at `now_epoch_s`.
    ///
    /// Implementations MUST canonicalize the lock they were handed
    /// ([`LifecycleLock::canonicalized`]) rather than trust its public fields.
    ///
    /// # Errors
    ///
    /// [`LockKernelError::InvalidLock`] for a blank, control-bearing or
    /// over-long id/tenant/holder, [`LockKernelError::Expired`] if the lease is
    /// already lapsed at acquisition, [`LockKernelError::LeaseTooLong`] if it
    /// runs past the reason's ceiling, [`LockKernelError::AlreadyHeld`] if a
    /// live lock with that id already stands, and
    /// [`LockKernelError::PrecedenceConflict`] if a live lock blocks the action
    /// this lock exists to serve.
    fn acquire(
        &mut self,
        lock: LifecycleLock,
        now_epoch_s: u64,
    ) -> Result<LifecycleLock, LockKernelError>;

    /// Extend the lease of a lock the principal holds.
    ///
    /// # Errors
    ///
    /// [`LockKernelError::NotFound`], [`LockKernelError::Expired`] for a lapsed
    /// lease, [`LockKernelError::ReleaseUnauthorized`] if `principal` is not the
    /// holder, [`LockKernelError::RenewalNotExtending`] if the new expiry does
    /// not strictly extend the lease, or [`LockKernelError::LeaseTooLong`] if it
    /// runs past the reason's ceiling.
    fn renew(
        &mut self,
        tenant_id: &str,
        id: &LockId,
        principal: &str,
        new_expires_at_epoch_s: u64,
        now_epoch_s: u64,
    ) -> Result<LifecycleLock, LockKernelError>;

    /// Lift a lock by the ordinary holder path.
    ///
    /// # Errors
    ///
    /// [`LockKernelError::NotFound`], [`LockKernelError::Expired`],
    /// [`LockKernelError::ReleaseRequiresQuorum`] - always for
    /// [`LockReason::LegalHold`], which the ordinary path can never lift - or
    /// [`LockKernelError::ReleaseUnauthorized`] for a non-holder.
    fn release(
        &mut self,
        tenant_id: &str,
        id: &LockId,
        principal: &str,
        now_epoch_s: u64,
    ) -> Result<LifecycleLock, LockKernelError>;

    /// Lift a lock by the multi-party quorum path.
    ///
    /// # Errors
    ///
    /// [`LockKernelError::NotFound`], [`LockKernelError::Expired`], or
    /// [`LockKernelError::QuorumNotMet`].
    fn release_with_quorum(
        &mut self,
        tenant_id: &str,
        id: &LockId,
        approvals: &[ReleaseApproval],
        now_epoch_s: u64,
    ) -> Result<LifecycleLock, LockKernelError>;

    /// The tenant's locks that are live at `now_epoch_s`, ordered by lock id.
    fn live_locks(&self, tenant_id: &str, now_epoch_s: u64) -> Vec<LifecycleLock>;

    /// Drop every retained row that has lapsed at `now_epoch_s`, returning how
    /// many went.
    ///
    /// On the PORT rather than only on the concrete store: retention is
    /// unbounded by design (a lapsed row is audit evidence), so a consumer
    /// holding a `&mut dyn LockStore` must be able to reclaim it. Explicit, so
    /// nothing disappears as a side effect of a read.
    fn purge_expired(&mut self, now_epoch_s: u64) -> usize;

    /// Evaluate `action` for `tenant_id` against that tenant's live locks.
    fn decide(&self, tenant_id: &str, action: LifecycleAction, now_epoch_s: u64) -> LockDecision {
        evaluate_for_tenant(
            tenant_id,
            action,
            &self.live_locks(tenant_id, now_epoch_s),
            now_epoch_s,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock(id: &str, reason: LockReason, expires: u64) -> LifecycleLock {
        LifecycleLock::new(
            LockId(id.to_owned()),
            "ten_acme".to_owned(),
            reason,
            "svc-lifecycle".to_owned(),
            expires,
        )
        .unwrap()
    }

    #[test]
    fn blank_components_are_rejected() {
        for (id, tenant, holder) in [
            ("", "ten_acme", "svc"),
            ("lk-1", "  ", "svc"),
            ("lk-1", "ten_acme", ""),
        ] {
            let built = LifecycleLock::new(
                LockId(id.to_owned()),
                tenant.to_owned(),
                LockReason::LegalHold,
                holder.to_owned(),
                100,
            );
            assert_eq!(built, Err(LockKernelError::InvalidLock));
        }
    }

    /// Validating a trimmed value and storing the raw one is what creates a
    /// shadow namespace that fails OPEN. What is validated is what is stored.
    #[test]
    fn identifiers_are_stored_as_they_were_validated() {
        let built = LifecycleLock::new(
            LockId(" lk-1\t".to_owned()),
            " ten_acme".to_owned(),
            LockReason::LegalHold,
            "svc-legal ".to_owned(),
            100,
        )
        .unwrap();
        assert_eq!(built.id, LockId("lk-1".to_owned()));
        assert_eq!(built.tenant_id, "ten_acme");
        assert_eq!(built.holder, "svc-legal");
        assert_eq!(
            built.clone().canonicalized(),
            Ok(built),
            "canonicalization is idempotent"
        );
    }

    #[test]
    fn control_bearing_and_over_long_identifiers_are_rejected() {
        for bad in ["lk\n1", "lk\r1", "lk\u{0}1"] {
            assert_eq!(
                LifecycleLock::new(
                    LockId(bad.to_owned()),
                    "ten_acme".to_owned(),
                    LockReason::LegalHold,
                    "svc".to_owned(),
                    100,
                ),
                Err(LockKernelError::InvalidLock),
                "{bad:?} would forge a line in any log that renders it"
            );
        }
        let long = "x".repeat(MAX_IDENTIFIER_CHARS);
        assert!(canonical_identifier(&long).is_ok());
        let too_long = "x".repeat(MAX_IDENTIFIER_CHARS + 1);
        assert_eq!(
            canonical_identifier(&too_long),
            Err(LockKernelError::InvalidLock)
        );
    }

    #[test]
    fn expiry_boundary_is_exclusive_at_the_instant() {
        let held = lock("lk-1", LockReason::LegalHold, 1_000);
        assert!(held.is_live_at(999), "one second before expiry: still live");
        assert!(
            held.is_expired_at(1_000),
            "exactly at expiry: already lapsed"
        );
        assert!(held.is_expired_at(1_001), "one second after: lapsed");
    }

    #[test]
    fn expired_lock_blocks_nothing() {
        let locks = [lock("lk-1", LockReason::LegalHold, 1_000)];
        for action in LifecycleAction::ALL {
            let before = evaluate_at(action, &locks, 999);
            assert!(!before.allow, "{action:?} must be blocked at 999");
            let at = evaluate_at(action, &locks, 1_000);
            assert!(at.allow, "{action:?} must be allowed at the expiry instant");
            let after = evaluate_at(action, &locks, 1_001);
            assert!(after.allow, "{action:?} must be allowed after expiry");
        }
    }

    #[test]
    fn no_locks_allows_every_action() {
        for action in LifecycleAction::ALL {
            let decision = evaluate_at(action, &[], 500);
            assert!(decision.allow);
            assert!(decision.blocking_locks.is_empty());
            assert_eq!(decision.governing_lock, None);
            assert_eq!(
                decision.explanation,
                format!(
                    "action={} allowed: 0 of 0 lock(s) block it",
                    action.as_slug()
                )
            );
        }
    }

    #[test]
    fn non_blocking_reason_does_not_block_its_unrelated_action() {
        // The whole point of the type: not every lock blocks every action.
        let locks = [lock("lk-1", LockReason::PaymentDispute, 9_999)];
        let promote = evaluate_at(LifecycleAction::PromoteDrPair, &locks, 10);
        assert!(
            promote.allow,
            "a payment dispute must not block DR promotion"
        );
        assert_eq!(
            promote.explanation,
            "action=promote-dr-pair allowed: 0 of 1 lock(s) block it"
        );
        let delete = evaluate_at(LifecycleAction::DeleteTenant, &locks, 10);
        assert!(!delete.allow, "a payment dispute must block deletion");
    }

    #[test]
    fn explanation_names_which_locks_and_why_in_precedence_order() {
        let locks = [
            lock("lk-zz-dr", LockReason::DrPromotionWindow, 9_999),
            lock("lk-aa-legal", LockReason::LegalHold, 9_999),
            lock("lk-mm-pay", LockReason::PaymentDispute, 9_999),
        ];
        let decision = evaluate_at(LifecycleAction::DeleteTenant, &locks, 10);
        assert!(!decision.allow);
        assert_eq!(
            decision.blocking_locks,
            vec![
                LockId("lk-aa-legal".to_owned()),
                LockId("lk-mm-pay".to_owned()),
                LockId("lk-zz-dr".to_owned()),
            ]
        );
        assert_eq!(
            decision.governing_lock,
            Some(LockId("lk-aa-legal".to_owned()))
        );
        assert_eq!(
            decision.explanation,
            format!(
                "action=delete-tenant denied: 3 of 3 lock(s) block it: \
                 lk-aa-legal (legal-hold: {}); \
                 lk-mm-pay (payment-dispute: {}); \
                 lk-zz-dr (dr-promotion-window: {})",
                LockReason::LegalHold.rationale(),
                LockReason::PaymentDispute.rationale(),
                LockReason::DrPromotionWindow.rationale()
            )
        );
    }

    #[test]
    fn explanation_is_stable_under_input_permutation() {
        let a = lock("lk-b", LockReason::PaymentDispute, 9_999);
        let b = lock("lk-a", LockReason::PaymentDispute, 9_999);
        let forward = evaluate_at(LifecycleAction::DeleteTenant, &[a.clone(), b.clone()], 10);
        let reverse = evaluate_at(LifecycleAction::DeleteTenant, &[b, a], 10);
        assert_eq!(forward, reverse);
        assert_eq!(
            forward.blocking_locks,
            vec![LockId("lk-a".to_owned()), LockId("lk-b".to_owned())],
            "ties break on ascending id, not on input order"
        );
    }

    #[test]
    fn expired_locks_still_count_in_the_considered_total() {
        let locks = [
            lock("lk-1", LockReason::LegalHold, 100),
            lock("lk-2", LockReason::PaymentDispute, 9_999),
        ];
        let decision = evaluate_at(LifecycleAction::DeleteTenant, &locks, 200);
        assert_eq!(decision.blocking_locks, vec![LockId("lk-2".to_owned())]);
        assert!(
            decision.explanation.starts_with(
                "action=delete-tenant denied: 1 of 2 lock(s) block it: lk-2 (payment-dispute:"
            ),
            "operator must see that 2 locks existed and 1 bit: {}",
            decision.explanation
        );
    }

    /// A mixed-tenant slice must not deny one tenant because of another's hold,
    /// nor render the other tenant's lock id into operator-facing text.
    #[test]
    fn a_tenant_scoped_evaluation_never_sees_another_tenants_lock() {
        let mut foreign = lock("lk-b-legal", LockReason::LegalHold, 9_999);
        foreign.tenant_id = "ten_other".to_owned();
        let mine = lock("lk-a-dr", LockReason::DrPromotionWindow, 9_999);

        let leaky = evaluate_at(
            LifecycleAction::RemovePaymentCredential,
            &[foreign.clone(), mine.clone()],
            10,
        );
        assert!(!leaky.allow, "the unscoped entry point sees both");
        assert!(leaky.explanation.contains("lk-b-legal"));

        let scoped = evaluate_for_tenant(
            "ten_acme",
            LifecycleAction::RemovePaymentCredential,
            &[foreign, mine],
            10,
        );
        assert!(
            scoped.allow,
            "no lock of ten_acme blocks removing a payment credential"
        );
        assert_eq!(
            scoped.explanation, "action=remove-payment-credential allowed: 0 of 1 lock(s) block it",
            "the count and the text must both be tenant-scoped"
        );
        assert!(!scoped.explanation.contains("lk-b-legal"));
    }

    #[test]
    fn legacy_evaluate_fails_closed_at_epoch_zero() {
        // A far-future legal hold must still block through the instant-free
        // signature; it must not be silently treated as expired.
        let locks = [lock("lk-1", LockReason::LegalHold, u64::MAX)];
        let decision = evaluate("delete-tenant", &locks);
        assert!(!decision.allow);
        assert_eq!(decision.blocking_locks, vec![LockId("lk-1".to_owned())]);
    }

    #[test]
    fn legacy_evaluate_honours_the_matrix_rather_than_blocking_everything() {
        let locks = [lock("lk-1", LockReason::PaymentDispute, u64::MAX)];
        assert!(!evaluate("delete-tenant", &locks).allow);
        assert!(
            evaluate("promote-dr-pair", &locks).allow,
            "the stub behaviour of blocking every action is the bug this fixes"
        );
    }

    #[test]
    fn legacy_evaluate_refuses_an_unrecognized_action() {
        let decision = evaluate("rm -rf tenant", &[]);
        assert!(!decision.allow, "unknown actions fail closed");
        assert!(decision.blocking_locks.is_empty());
        assert_eq!(decision.governing_lock, None);
        assert!(
            decision
                .explanation
                .starts_with("action=rm -rf tenant refused: unrecognized lifecycle action"),
            "{}",
            decision.explanation
        );
    }

    /// A wire slug is untrusted text and the explanation is an audit line. A
    /// newline in the echo would let the caller write a second line of its own
    /// - a forged `allowed` verdict inside the legal-hold audit trail.
    #[test]
    fn an_unrecognized_action_cannot_forge_a_line_in_the_audit_text() {
        let forged = "x\naction=delete-tenant allowed: 0 of 0 lock(s) block it";
        let decision = evaluate(forged, &[]);
        assert!(!decision.allow);
        assert!(
            !decision.explanation.contains('\n'),
            "no newline may survive into the explanation: {:?}",
            decision.explanation
        );
        assert_eq!(
            decision.explanation.lines().count(),
            1,
            "the whole refusal must stay on one line: {:?}",
            decision.explanation
        );
        assert!(decision.explanation.starts_with("action=x?action=delete"));
    }

    /// The echo is capped, so a caller cannot turn one request field into a
    /// megabyte of audit record.
    #[test]
    fn the_echoed_action_is_capped() {
        let huge = "z".repeat(1_000_000);
        let decision = evaluate(&huge, &[]);
        assert!(!decision.allow);
        assert!(
            decision.explanation.len() < 400,
            "explanation grew with the input: {} bytes",
            decision.explanation.len()
        );
        assert!(
            decision.explanation.starts_with(&format!(
                "action={}...(truncated) refused:",
                "z".repeat(MAX_ECHOED_CHARS)
            )),
            "{}",
            decision.explanation
        );
        assert_eq!(sanitize_for_explanation("short"), "short");
        assert_eq!(sanitize_for_explanation("a\tb"), "a?b");
    }

    #[test]
    fn a_lock_that_never_had_force_blocks_nothing() {
        let locks = [lock("lk-1", LockReason::LegalHold, 0)];
        assert!(evaluate_at(LifecycleAction::DeleteTenant, &locks, 0).allow);
        assert!(evaluate("delete-tenant", &locks).allow);
    }

    #[test]
    fn error_display_is_distinct_and_non_empty_per_variant() {
        let variants = [
            LockKernelError::PrecedenceConflict,
            LockKernelError::ReleaseUnauthorized,
            LockKernelError::ReleaseRequiresQuorum,
            LockKernelError::Expired,
            LockKernelError::InvalidLock,
            LockKernelError::AlreadyHeld,
            LockKernelError::NotFound,
            LockKernelError::RenewalNotExtending,
            LockKernelError::LeaseTooLong,
            LockKernelError::QuorumNotMet,
        ];
        let mut rendered: Vec<String> = variants.iter().map(ToString::to_string).collect();
        assert!(rendered.iter().all(|text| !text.is_empty()));
        rendered.sort();
        let before = rendered.len();
        rendered.dedup();
        assert_eq!(
            rendered.len(),
            before,
            "each variant needs its own sentence"
        );
        let as_error: &dyn std::error::Error = &LockKernelError::Expired;
        assert_eq!(as_error.to_string(), "the lock lease has already lapsed");
        assert!(
            LockKernelError::ReleaseRequiresQuorum
                .to_string()
                .contains("quorum"),
            "the remedy must be in the sentence"
        );
    }
}
