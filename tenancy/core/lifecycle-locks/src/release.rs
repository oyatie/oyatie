//! Release, renewal, lease bounds, and holder authorization.
//!
//! Two paths lift a lock:
//!
//! 1. **The ordinary path** ([`authorize_release`]): the principal that holds
//!    the lease lifts it. Nobody else can - a lock is never silently stolen.
//! 2. **The quorum path** ([`authorize_quorum_release`]): the multi-party
//!    approval set for the lock's reason lifts it, regardless of holder.
//!
//! [`crate::LockReason::LegalHold`] is closed to the ordinary path entirely
//! (IP-021 §D.4): a statutory preservation order is not the holder's to lift,
//! even when the holder placed it. Its quorum is DPO **and** counsel, and the
//! two must be different principals. That refusal is its own error variant,
//! [`crate::LockKernelError::ReleaseRequiresQuorum`], so an operator holding
//! the lease is told to convene the quorum rather than to try another identity.
//!
//! # Required roles by reason
//!
//! ```text
//! legal-hold             -> data-protection-officer + counsel
//! payment-dispute        -> finance-compliance
//! pending-deletion-grace -> data-protection-officer
//! jurisdiction-migration -> ops-security
//! kyb-reverification     -> finance-compliance
//! dr-promotion-window    -> ops-security
//! manual-soft-lock       -> tenant-admin
//! ```
//!
//! # Lease bounds
//!
//! Every reason carries a maximum lease ([`max_lease_seconds`]). Without one, a
//! purely operational window could be taken out to `u64::MAX` and deny deletion
//! and jurisdiction change for all practical time. A hold that genuinely needs
//! longer is RENEWED, which is a fresh, authorized, auditable act by the holder
//! - the ceiling bounds each lease, not the total.

use crate::{LifecycleLock, LockKernelError, LockReason};

/// An organizational role that can sit on a release quorum.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ReleaseRole {
    /// Data protection officer.
    DataProtectionOfficer,
    /// Legal counsel.
    Counsel,
    /// Operations security.
    OpsSecurity,
    /// Finance / compliance.
    FinanceCompliance,
    /// Tenant-side administrator. The quorum for
    /// [`crate::LockReason::ManualSoftLock`], which is IP-021 §D.4's
    /// "soft lock can be released by tenant admin" rule.
    TenantAdmin,
}

impl ReleaseRole {
    /// Every role, in declaration order.
    pub const ALL: [Self; 5] = [
        Self::DataProtectionOfficer,
        Self::Counsel,
        Self::OpsSecurity,
        Self::FinanceCompliance,
        Self::TenantAdmin,
    ];

    /// The stable wire slug for this role.
    #[must_use]
    pub const fn as_slug(self) -> &'static str {
        match self {
            Self::DataProtectionOfficer => "data-protection-officer",
            Self::Counsel => "counsel",
            Self::OpsSecurity => "ops-security",
            Self::FinanceCompliance => "finance-compliance",
            Self::TenantAdmin => "tenant-admin",
        }
    }
}

impl core::fmt::Display for ReleaseRole {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_slug())
    }
}

/// One principal signing off on a release in one role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseApproval {
    /// The approving principal, canonicalized by
    /// [`ReleaseApproval::new`].
    ///
    /// Classified `TENANT_SCOPED` rather than `INTERNAL_ONLY` because
    /// [`ReleaseRole::TenantAdmin`] is a real quorum role (it releases a
    /// [`crate::LockReason::ManualSoftLock`]), so this field can and does carry
    /// a tenant's own user identifier - the same class
    /// `tenancy/core/reserved-namespace` gives its `principal`. A redaction or
    /// export pipeline driven by these annotations must scope it per tenant.
    pub principal: String, // data_class: TENANT_SCOPED
    /// The role the principal signs in.
    pub role: ReleaseRole, // data_class: INTERNAL_ONLY
}

impl ReleaseApproval {
    /// Build an approval, canonicalizing the principal exactly as
    /// [`crate::LifecycleLock::new`] canonicalizes a holder: surrounding
    /// whitespace is stripped, so `"dana "` and `"dana"` are one approver and
    /// cannot be passed off as two.
    ///
    /// # Errors
    ///
    /// [`LockKernelError::InvalidLock`] when the principal is blank, carries a
    /// control character, or is longer than
    /// [`crate::MAX_IDENTIFIER_CHARS`] - an anonymous, log-forging or unbounded
    /// approval is not an approval.
    pub fn new(principal: String, role: ReleaseRole) -> Result<Self, LockKernelError> {
        Ok(Self {
            principal: crate::canonical_identifier(&principal)?,
            role,
        })
    }
}

/// The roles a quorum release of `reason` requires. All of them, each filled by
/// a DIFFERENT principal.
#[must_use]
pub const fn required_roles(reason: LockReason) -> &'static [ReleaseRole] {
    match reason {
        LockReason::LegalHold => &[ReleaseRole::DataProtectionOfficer, ReleaseRole::Counsel],
        LockReason::PaymentDispute | LockReason::KybReverification => {
            &[ReleaseRole::FinanceCompliance]
        }
        LockReason::PendingDeletionGrace => &[ReleaseRole::DataProtectionOfficer],
        LockReason::JurisdictionMigration | LockReason::DrPromotionWindow => {
            &[ReleaseRole::OpsSecurity]
        }
        LockReason::ManualSoftLock => &[ReleaseRole::TenantAdmin],
    }
}

/// The longest lease, in seconds, a lock of this reason may hold at one
/// acquisition or renewal.
///
/// Compliance holds get a year because a preservation order or a dispute
/// retention genuinely runs that long; the operational windows get hours or
/// weeks because they describe a running operation, not an obligation. A hold
/// that outlives its ceiling is renewed rather than granted indefinitely, so
/// somebody re-authorizes it on the record.
#[must_use]
pub const fn max_lease_seconds(reason: LockReason) -> u64 {
    const DAY: u64 = 86_400;
    match reason {
        // A statutory order and a dispute retention are measured in months.
        LockReason::LegalHold | LockReason::PaymentDispute => 366 * DAY,
        // Statutory DSR clocks and identity checks are measured in weeks.
        LockReason::PendingDeletionGrace | LockReason::KybReverification => 90 * DAY,
        // An operator's protection lock should be revisited each quarter.
        LockReason::ManualSoftLock => 90 * DAY,
        // A residency migration is a project, not an obligation.
        LockReason::JurisdictionMigration => 30 * DAY,
        // A promotion window is an operation in progress.
        LockReason::DrPromotionWindow => DAY,
    }
}

/// Check a proposed lease against [`max_lease_seconds`] for `reason`.
///
/// # Errors
///
/// [`LockKernelError::LeaseTooLong`] when `expires_at_epoch_s` is further than
/// the reason's ceiling beyond `now_epoch_s`. A lease that has already lapsed
/// is not this function's business and passes.
pub const fn check_lease(
    reason: LockReason,
    expires_at_epoch_s: u64,
    now_epoch_s: u64,
) -> Result<(), LockKernelError> {
    if expires_at_epoch_s.saturating_sub(now_epoch_s) > max_lease_seconds(reason) {
        return Err(LockKernelError::LeaseTooLong);
    }
    Ok(())
}

/// Whether `reason` can ever be lifted by its holder alone.
///
/// False for [`LockReason::LegalHold`] and only for it.
#[must_use]
pub const fn holder_release_permitted(reason: LockReason) -> bool {
    !matches!(reason, LockReason::LegalHold)
}

/// Whether `principal` is the holder of `lock`.
///
/// Compared after trimming, matching the canonicalization
/// [`crate::LifecycleLock::new`] applies to the stored holder: without it a
/// lock recorded through a sloppy ingest path as `"svc-dr "` could never be
/// released by `svc-dr` and would sit until it lapsed.
#[must_use]
pub fn is_holder(lock: &LifecycleLock, principal: &str) -> bool {
    lock.holder.trim() == principal.trim()
}

/// Authorize the ordinary, holder-driven release of `lock` at `now_epoch_s`.
///
/// # Errors
///
/// - [`LockKernelError::Expired`] if the lease has already lapsed: there is
///   nothing left to lift, and reporting success would tell the operator a
///   release happened that did not.
/// - [`LockKernelError::ReleaseRequiresQuorum`] for any
///   [`LockReason::LegalHold`], whoever asks - including its own holder. This
///   is a distinct variant from the one below on purpose: the remedy is to
///   convene DPO and counsel, not to retry with a different identity.
/// - [`LockKernelError::ReleaseUnauthorized`] for any principal that is not the
///   holder.
pub fn authorize_release(
    lock: &LifecycleLock,
    principal: &str,
    now_epoch_s: u64,
) -> Result<(), LockKernelError> {
    if lock.is_expired_at(now_epoch_s) {
        return Err(LockKernelError::Expired);
    }
    if !holder_release_permitted(lock.reason) {
        return Err(LockKernelError::ReleaseRequiresQuorum);
    }
    if !is_holder(lock, principal) {
        return Err(LockKernelError::ReleaseUnauthorized);
    }
    Ok(())
}

/// A required role that the approval set cannot fill, or `None` when the whole
/// quorum can be satisfied.
///
/// This is the operator-facing companion to
/// [`LockKernelError::QuorumNotMet`], which carries no payload: it names a role
/// that no still-unused approver could cover. With more than one role short,
/// which of them is named is deterministic but not otherwise meaningful.
#[must_use]
pub fn quorum_shortfall(reason: LockReason, approvals: &[ReleaseApproval]) -> Option<ReleaseRole> {
    quorum_assignment(required_roles(reason), approvals).err()
}

/// Authorize the multi-party quorum release of `lock` at `now_epoch_s`.
///
/// Every role in [`required_roles`] must be filled, and no single principal may
/// fill two required roles - a DPO who is also counsel is one pair of eyes, not
/// two. Approvals in roles the reason does not require are ignored rather than
/// rejected, so an over-signed request still succeeds.
///
/// The assignment of principals to roles is a full bipartite matching, not a
/// first-fit walk. A greedy pass rejects sets that a legal assignment does
/// satisfy - `[(dana, DPO), (cleo, DPO), (dana, counsel)]` loses `dana` to the
/// DPO seat and then finds counsel unfillable - which made the verdict depend
/// on the ORDER a caller happened to collect signatures in. Duplicate approvals
/// are folded and the candidate lists are sorted, so the answer depends only on
/// the SET of `(principal, role)` pairs supplied.
///
/// # Errors
///
/// - [`LockKernelError::Expired`] if the lease has already lapsed.
/// - [`LockKernelError::QuorumNotMet`] if no assignment of distinct principals
///   covers every required role. [`quorum_shortfall`] names one that failed.
pub fn authorize_quorum_release(
    lock: &LifecycleLock,
    approvals: &[ReleaseApproval],
    now_epoch_s: u64,
) -> Result<(), LockKernelError> {
    if lock.is_expired_at(now_epoch_s) {
        return Err(LockKernelError::Expired);
    }
    quorum_assignment(required_roles(lock.reason), approvals)
        .map_err(|_| LockKernelError::QuorumNotMet)
}

/// Match each required role to a distinct approving principal, or name a role
/// that could not be filled.
fn quorum_assignment(
    required: &[ReleaseRole],
    approvals: &[ReleaseApproval],
) -> Result<(), ReleaseRole> {
    let mut candidates: Vec<Vec<&str>> = Vec::with_capacity(required.len());
    for role in required {
        let mut principals: Vec<&str> = approvals
            .iter()
            .filter(|approval| approval.role == *role)
            .map(|approval| approval.principal.trim())
            .collect();
        principals.sort_unstable();
        principals.dedup();
        candidates.push(principals);
    }
    let mut assigned: Vec<(&str, usize)> = Vec::with_capacity(required.len());
    for (index, role) in required.iter().enumerate() {
        let mut visited: Vec<&str> = Vec::new();
        if !augment(index, &candidates, &mut assigned, &mut visited) {
            return Err(*role);
        }
    }
    Ok(())
}

/// One augmenting-path step of the matching: seat `role`, displacing an
/// already-seated role only if that role can be re-seated elsewhere.
///
/// `visited` is the set of principals this search has already tried, which is
/// what bounds the recursion at one pass per principal.
fn augment<'a>(
    role: usize,
    candidates: &[Vec<&'a str>],
    assigned: &mut Vec<(&'a str, usize)>,
    visited: &mut Vec<&'a str>,
) -> bool {
    for principal in &candidates[role] {
        if visited.contains(principal) {
            continue;
        }
        visited.push(*principal);
        match assigned.iter().position(|(held, _)| held == principal) {
            None => {
                assigned.push((*principal, role));
                return true;
            }
            Some(index) => {
                let displaced = assigned[index].1;
                if augment(displaced, candidates, assigned, visited) {
                    assigned[index] = (*principal, role);
                    return true;
                }
            }
        }
    }
    false
}

/// Authorize a lease extension by the holder at `now_epoch_s`.
///
/// # Errors
///
/// - [`LockKernelError::Expired`] if the lease already lapsed: a lapsed lease
///   is re-acquired, never renewed, so the precedence check runs again.
/// - [`LockKernelError::ReleaseUnauthorized`] if `principal` is not the holder.
///   Renewal is a holder power for EVERY reason, legal hold included: extending
///   a preservation order is not lifting it.
/// - [`LockKernelError::RenewalNotExtending`] if the new expiry does not move
///   strictly later, which would silently shorten the hold.
/// - [`LockKernelError::LeaseTooLong`] if the new expiry is further out than
///   [`max_lease_seconds`] allows, so a holder cannot ratchet a lock to
///   `u64::MAX` in one call.
pub fn authorize_renewal(
    lock: &LifecycleLock,
    principal: &str,
    new_expires_at_epoch_s: u64,
    now_epoch_s: u64,
) -> Result<(), LockKernelError> {
    if lock.is_expired_at(now_epoch_s) {
        return Err(LockKernelError::Expired);
    }
    if !is_holder(lock, principal) {
        return Err(LockKernelError::ReleaseUnauthorized);
    }
    if new_expires_at_epoch_s <= lock.expires_at_epoch_s {
        return Err(LockKernelError::RenewalNotExtending);
    }
    check_lease(lock.reason, new_expires_at_epoch_s, now_epoch_s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LockId;

    fn lock(reason: LockReason, holder: &str, expires: u64) -> LifecycleLock {
        LifecycleLock::new(
            LockId("lk-1".to_owned()),
            "ten_acme".to_owned(),
            reason,
            holder.to_owned(),
            expires,
        )
        .unwrap()
    }

    fn approval(principal: &str, role: ReleaseRole) -> ReleaseApproval {
        ReleaseApproval::new(principal.to_owned(), role).unwrap()
    }

    #[test]
    fn holder_releases_an_ordinary_lock() {
        let held = lock(LockReason::PaymentDispute, "svc-billing", 1_000);
        assert_eq!(authorize_release(&held, "svc-billing", 10), Ok(()));
    }

    #[test]
    fn a_non_holder_cannot_release() {
        let held = lock(LockReason::PaymentDispute, "svc-billing", 1_000);
        assert_eq!(
            authorize_release(&held, "svc-support", 10),
            Err(LockKernelError::ReleaseUnauthorized),
            "a lock must never be silently stolen"
        );
    }

    /// A holder stored through a sloppy ingest path must still be able to
    /// release its own lock; the alternative is a lease nobody can lift.
    #[test]
    fn holder_identity_is_compared_after_trimming() {
        let mut held = lock(LockReason::DrPromotionWindow, "svc-dr", 1_000);
        held.holder = "svc-dr ".to_owned();
        assert!(is_holder(&held, "svc-dr"));
        assert_eq!(authorize_release(&held, "svc-dr", 10), Ok(()));
        assert_eq!(authorize_release(&held, " svc-dr", 10), Ok(()));
        assert_eq!(
            authorize_release(&held, "svc-drx", 10),
            Err(LockKernelError::ReleaseUnauthorized),
            "trimming must not turn into prefix matching"
        );
    }

    #[test]
    fn legal_hold_cannot_be_released_by_the_ordinary_path() {
        let held = lock(LockReason::LegalHold, "svc-legal", 9_999);
        // Not even by its own holder - and the refusal names the remedy.
        assert_eq!(
            authorize_release(&held, "svc-legal", 10),
            Err(LockKernelError::ReleaseRequiresQuorum),
            "the holder must be told to convene the quorum, not to retry"
        );
        assert_eq!(
            authorize_release(&held, "root", 10),
            Err(LockKernelError::ReleaseRequiresQuorum)
        );
        assert!(!holder_release_permitted(LockReason::LegalHold));
        for reason in LockReason::ALL {
            assert_eq!(
                holder_release_permitted(reason),
                reason != LockReason::LegalHold,
                "{} disagrees with the legal-hold carve-out",
                reason.as_slug()
            );
        }
    }

    #[test]
    fn releasing_a_lapsed_lease_is_reported_as_expired() {
        let held = lock(LockReason::PaymentDispute, "svc-billing", 1_000);
        assert_eq!(authorize_release(&held, "svc-billing", 999), Ok(()));
        assert_eq!(
            authorize_release(&held, "svc-billing", 1_000),
            Err(LockKernelError::Expired)
        );
        assert_eq!(
            authorize_release(&held, "svc-billing", 1_001),
            Err(LockKernelError::Expired)
        );
    }

    #[test]
    fn legal_hold_quorum_needs_dpo_and_counsel() {
        let held = lock(LockReason::LegalHold, "svc-legal", 9_999);
        assert_eq!(
            authorize_quorum_release(&held, &[], 10),
            Err(LockKernelError::QuorumNotMet)
        );
        assert_eq!(
            authorize_quorum_release(
                &held,
                &[approval("dana", ReleaseRole::DataProtectionOfficer)],
                10
            ),
            Err(LockKernelError::QuorumNotMet),
            "one role short"
        );
        assert_eq!(
            quorum_shortfall(
                LockReason::LegalHold,
                &[approval("dana", ReleaseRole::DataProtectionOfficer)]
            ),
            Some(ReleaseRole::Counsel),
            "the operator must be told WHICH seat is empty"
        );
        assert_eq!(
            authorize_quorum_release(
                &held,
                &[
                    approval("dana", ReleaseRole::DataProtectionOfficer),
                    approval("cleo", ReleaseRole::Counsel),
                ],
                10
            ),
            Ok(())
        );
        assert_eq!(
            quorum_shortfall(
                LockReason::LegalHold,
                &[
                    approval("dana", ReleaseRole::DataProtectionOfficer),
                    approval("cleo", ReleaseRole::Counsel),
                ]
            ),
            None
        );
    }

    #[test]
    fn one_principal_cannot_fill_both_legal_hold_roles() {
        let held = lock(LockReason::LegalHold, "svc-legal", 9_999);
        assert_eq!(
            authorize_quorum_release(
                &held,
                &[
                    approval("dana", ReleaseRole::DataProtectionOfficer),
                    approval("dana", ReleaseRole::Counsel),
                ],
                10
            ),
            Err(LockKernelError::QuorumNotMet),
            "one pair of eyes is not two"
        );
        // Nor by writing the same principal twice in one role.
        assert_eq!(
            authorize_quorum_release(
                &held,
                &[
                    approval("dana", ReleaseRole::DataProtectionOfficer),
                    approval("dana ", ReleaseRole::Counsel),
                ],
                10
            ),
            Err(LockKernelError::QuorumNotMet),
            "whitespace must not manufacture a second approver"
        );
    }

    /// The cross-listed-approver case: `dana` is both DPO and counsel, `cleo`
    /// is a second DPO. `cleo` = DPO and `dana` = counsel is two distinct
    /// principals covering both seats, so the quorum IS met - and it must be
    /// met whichever order the signatures were collected in.
    #[test]
    fn a_cross_listed_approver_does_not_defeat_a_legal_quorum() {
        let held = lock(LockReason::LegalHold, "svc-legal", 9_999);
        let orders = [
            [
                approval("dana", ReleaseRole::DataProtectionOfficer),
                approval("cleo", ReleaseRole::DataProtectionOfficer),
                approval("dana", ReleaseRole::Counsel),
            ],
            [
                approval("cleo", ReleaseRole::DataProtectionOfficer),
                approval("dana", ReleaseRole::DataProtectionOfficer),
                approval("dana", ReleaseRole::Counsel),
            ],
            [
                approval("dana", ReleaseRole::Counsel),
                approval("dana", ReleaseRole::DataProtectionOfficer),
                approval("cleo", ReleaseRole::DataProtectionOfficer),
            ],
        ];
        for (index, approvals) in orders.iter().enumerate() {
            assert_eq!(
                authorize_quorum_release(&held, approvals, 10),
                Ok(()),
                "collection order {index} was refused a quorum that exists"
            );
        }
    }

    /// The verdict must be a function of the SET of approvals, never of its
    /// order. Exhaustive over every permutation of a four-signature set.
    #[test]
    fn the_quorum_verdict_is_independent_of_approval_order() {
        let held = lock(LockReason::LegalHold, "svc-legal", 9_999);
        let base = [
            approval("dana", ReleaseRole::DataProtectionOfficer),
            approval("dana", ReleaseRole::Counsel),
            approval("cleo", ReleaseRole::DataProtectionOfficer),
            approval("otto", ReleaseRole::OpsSecurity),
        ];
        let indices = [0_usize, 1, 2, 3];
        let mut permutations = 0_usize;
        for a in indices {
            for b in indices {
                for c in indices {
                    for d in indices {
                        let picked = [a, b, c, d];
                        let mut seen = picked.to_vec();
                        seen.sort_unstable();
                        seen.dedup();
                        if seen.len() != 4 {
                            continue;
                        }
                        permutations += 1;
                        let approvals: Vec<ReleaseApproval> =
                            picked.iter().map(|i| base[*i].clone()).collect();
                        assert_eq!(
                            authorize_quorum_release(&held, &approvals, 10),
                            Ok(()),
                            "permutation {picked:?} disagreed"
                        );
                    }
                }
            }
        }
        assert_eq!(permutations, 24, "all 4! orderings must have been tried");
    }

    #[test]
    fn a_wrong_role_does_not_satisfy_the_quorum_and_extras_are_ignored() {
        let held = lock(LockReason::JurisdictionMigration, "svc-residency", 9_999);
        assert_eq!(
            authorize_quorum_release(&held, &[approval("tina", ReleaseRole::TenantAdmin)], 10),
            Err(LockKernelError::QuorumNotMet)
        );
        assert_eq!(
            authorize_quorum_release(
                &held,
                &[
                    approval("tina", ReleaseRole::TenantAdmin),
                    approval("otto", ReleaseRole::OpsSecurity),
                ],
                10
            ),
            Ok(()),
            "an over-signed request still meets the quorum"
        );
    }

    /// IP-021 §D.4's tenant-admin rule. Before `manual-soft-lock` existed,
    /// `TenantAdmin` was a public variant that satisfied no quorum at all.
    #[test]
    fn a_tenant_admin_releases_the_manual_soft_lock() {
        let held = lock(LockReason::ManualSoftLock, "svc-ops", 9_999);
        assert_eq!(
            required_roles(LockReason::ManualSoftLock),
            &[ReleaseRole::TenantAdmin]
        );
        assert_eq!(
            authorize_quorum_release(&held, &[approval("tina", ReleaseRole::TenantAdmin)], 10),
            Ok(())
        );
        assert_eq!(
            authorize_quorum_release(&held, &[approval("otto", ReleaseRole::OpsSecurity)], 10),
            Err(LockKernelError::QuorumNotMet)
        );
        assert!(
            holder_release_permitted(LockReason::ManualSoftLock),
            "the operator who placed it may also lift it"
        );
    }

    /// Every public role must be reachable: a role no quorum names is an
    /// authorization path the API advertises and can never grant.
    #[test]
    fn every_role_can_satisfy_some_quorum() {
        for role in ReleaseRole::ALL {
            assert!(
                LockReason::ALL
                    .into_iter()
                    .any(|reason| required_roles(reason).contains(&role)),
                "{role} sits on no quorum and can never authorize anything"
            );
        }
    }

    #[test]
    fn every_reason_has_a_non_empty_quorum() {
        for reason in LockReason::ALL {
            assert!(
                !required_roles(reason).is_empty(),
                "{} has no release path at all",
                reason.as_slug()
            );
        }
        assert_eq!(required_roles(LockReason::LegalHold).len(), 2);
    }

    #[test]
    fn quorum_on_a_lapsed_lease_is_expired_not_granted() {
        let held = lock(LockReason::LegalHold, "svc-legal", 1_000);
        assert_eq!(
            authorize_quorum_release(
                &held,
                &[
                    approval("dana", ReleaseRole::DataProtectionOfficer),
                    approval("cleo", ReleaseRole::Counsel),
                ],
                1_000
            ),
            Err(LockKernelError::Expired)
        );
    }

    #[test]
    fn renewal_must_extend_and_must_be_the_holder() {
        let held = lock(LockReason::DrPromotionWindow, "svc-dr", 1_000);
        assert_eq!(authorize_renewal(&held, "svc-dr", 1_001, 10), Ok(()));
        assert_eq!(
            authorize_renewal(&held, "svc-dr", 1_000, 10),
            Err(LockKernelError::RenewalNotExtending),
            "freezing the lease is not a renewal"
        );
        assert_eq!(
            authorize_renewal(&held, "svc-dr", 500, 10),
            Err(LockKernelError::RenewalNotExtending),
            "shortening the lease is not a renewal"
        );
        assert_eq!(
            authorize_renewal(&held, "svc-other", 2_000, 10),
            Err(LockKernelError::ReleaseUnauthorized)
        );
        assert_eq!(
            authorize_renewal(&held, "svc-dr", 2_000, 1_000),
            Err(LockKernelError::Expired),
            "a lapsed lease is re-acquired, not renewed"
        );
    }

    #[test]
    fn legal_hold_may_be_extended_by_its_holder() {
        let held = lock(LockReason::LegalHold, "svc-legal", 1_000);
        assert_eq!(
            authorize_renewal(&held, "svc-legal", 5_000, 10),
            Ok(()),
            "extending a preservation order is not lifting it"
        );
    }

    #[test]
    fn a_renewal_cannot_ratchet_the_lease_past_its_ceiling() {
        for reason in LockReason::ALL {
            let held = lock(reason, "svc", 1_000);
            let ceiling = max_lease_seconds(reason);
            assert_eq!(
                authorize_renewal(&held, "svc", 10 + ceiling, 10),
                Ok(()),
                "{} refused a lease exactly at its ceiling",
                reason.as_slug()
            );
            assert_eq!(
                authorize_renewal(&held, "svc", 10 + ceiling + 1, 10),
                Err(LockKernelError::LeaseTooLong),
                "{} accepted a lease one second past its ceiling",
                reason.as_slug()
            );
            assert_eq!(
                authorize_renewal(&held, "svc", u64::MAX, 10),
                Err(LockKernelError::LeaseTooLong),
                "{} could be ratcheted to forever",
                reason.as_slug()
            );
        }
    }

    #[test]
    fn every_reason_has_a_positive_and_finite_lease_ceiling() {
        for reason in LockReason::ALL {
            let ceiling = max_lease_seconds(reason);
            assert!(ceiling > 0, "{} may never be held at all", reason.as_slug());
            assert!(
                ceiling <= 366 * 86_400,
                "{} may be held longer than a year at one grant",
                reason.as_slug()
            );
        }
        assert!(
            max_lease_seconds(LockReason::DrPromotionWindow)
                < max_lease_seconds(LockReason::LegalHold),
            "an operational window must not be grantable for as long as a statutory hold"
        );
        assert_eq!(check_lease(LockReason::LegalHold, 0, 10), Ok(()));
    }

    #[test]
    fn a_blank_approver_is_rejected() {
        assert_eq!(
            ReleaseApproval::new("   ".to_owned(), ReleaseRole::Counsel),
            Err(LockKernelError::InvalidLock)
        );
        assert_eq!(
            ReleaseApproval::new("da\nna".to_owned(), ReleaseRole::Counsel),
            Err(LockKernelError::InvalidLock),
            "a control character in an approver name is a forged audit line"
        );
        assert_eq!(
            approval(" dana ", ReleaseRole::Counsel).principal,
            "dana",
            "the stored principal is canonical"
        );
    }

    #[test]
    fn role_slugs_are_unique() {
        let mut slugs: Vec<&str> = ReleaseRole::ALL.iter().map(|role| role.as_slug()).collect();
        let distinct = slugs.len();
        slugs.sort_unstable();
        slugs.dedup();
        assert_eq!(slugs.len(), distinct);
        assert_eq!(ReleaseRole::OpsSecurity.to_string(), "ops-security");
    }
}
