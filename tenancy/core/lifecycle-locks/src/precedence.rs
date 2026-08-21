//! The precedence matrix: which lock reason blocks which lifecycle action, how
//! strong each reason is, and when an acquisition contradicts a standing hold.
//!
//! # The matrix
//!
//! Rows are [`crate::LockReason`], columns are [`LifecycleAction`], `X` means
//! blocked. This table is the single place the matrix is stated in prose;
//! [`crate::LockReason::blocks`] is the same table in code and
//! `matrix_table_matches_documentation` in this module's tests asserts the two
//! agree cell by cell.
//!
//! ```text
//! reason (strength)             | delete | jurisdiction | payment-cred | kyb | dr-promote
//! ------------------------------|--------|--------------|--------------|-----|-----------
//! legal-hold            (6)     |   X    |      X       |      X       |  X  |     X
//! payment-dispute       (5)     |   X    |      .       |      X       |  .  |     .
//! pending-deletion-grace(4)     |   X    |      X       |      .       |  .  |     .
//! jurisdiction-migration(3)     |   X    |      X       |      .       |  X  |     X
//! kyb-reverification    (2)     |   X    |      X       |      X       |  X  |     .
//! dr-promotion-window   (1)     |   X    |      X       |      .       |  .  |     X
//! manual-soft-lock      (0)     |   X    |      X       |      .       |  .  |     .
//! ```
//!
//! Reading of the non-obvious cells:
//!
//! - Every reason blocks `delete-tenant`. Deletion is the one irreversible
//!   action, so any standing hold is reason enough to refuse it.
//! - `legal-hold` is total by construction, `reverify-kyb` and
//!   `remove-payment-credential` included. A KYB re-check writes a new
//!   attestation onto the entity record and a credential removal destroys an
//!   instrument, and both are preserved evidence under an order; the kernel
//!   refuses them rather than guess which writes a preservation order
//!   tolerates. A re-check that genuinely must run under a hold lifts the hold
//!   through the DPO-plus-counsel quorum, which is an act with names on it.
//!   This is deliberate, and `rationale()` says so in the sentence the operator
//!   is shown.
//! - `payment-dispute` is a pure retention obligation over the tenant and its
//!   payment instrument. It has no opinion about residency, identity, or cell
//!   topology, so it does not block those.
//! - `pending-deletion-grace` keeps the record stable while the DSR grace
//!   window runs, and blocks a jurisdiction change that would move the record
//!   out from under the window. It deliberately does NOT block DR promotion:
//!   availability work must never be gated on a grace timer.
//! - `jurisdiction-migration` and `kyb-reverification` both block themselves -
//!   a second concurrent run would race the first. [`acquisition_conflict`]
//!   enforces that at acquisition, so a self-blocking reason stands at most
//!   ONCE per tenant.
//! - `kyb-reverification` blocks `remove-payment-credential` because the payment
//!   instrument is evidence for the check in flight, but does NOT block DR
//!   promotion: identity paperwork must not stop a failover.
//! - `dr-promotion-window` blocks itself and jurisdiction change (cell topology
//!   is moving) but nothing financial or identity-related.
//! - `manual-soft-lock` is IP-021 §D.1's operator-placed protection lock (the
//!   Cloudflare-zone-lock shape): it guards the two changes a human would not
//!   want made by accident and nothing else, and it is the one reason a tenant
//!   admin can lift alone (IP-021 §D.4, see [`crate::release`]).
//!
//! # Holds and windows
//!
//! Every reason is one of two kinds, and [`LockReason::implied_action`] is
//! where the kind is stated:
//!
//! - A **hold** exists to STOP something. It serves no action, so it is always
//!   acquirable: a court order, a dispute retention, an operator's soft lock and
//!   a DSR grace window must each be recordable on a tenant whatever else
//!   already stands.
//! - A **window** exists to PERFORM one action, and is refused when any live
//!   lock blocks that action - the operation could never run, so recording the
//!   window would be a lie to the next operator who reads the lock list.
//!
//! # Strength
//!
//! [`LockReason::precedence`] orders the reasons. Strength decides which lock
//! GOVERNS a refusal - the one an operator is shown first, and the one named as
//! the contradiction by [`acquisition_conflict`]. It deliberately does NOT
//! decide WHETHER an acquisition is refused: a rule that let a strong lock
//! through where a weak one was stopped made the reachable set depend on the
//! order the two locks happened to arrive in. `legal-hold` is strongest by
//! construction and `manual-soft-lock`, which has the lowest release bar of any
//! reason, is weakest; see [`crate::release`] for the release consequence.

use crate::{LifecycleLock, LockKernelError, LockReason};

/// A lifecycle transition that lifecycle locks gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LifecycleAction {
    /// Irreversibly retire the tenant.
    DeleteTenant,
    /// Move the tenant to a different residency jurisdiction.
    ChangeJurisdiction,
    /// Remove a stored payment credential.
    RemovePaymentCredential,
    /// Run KYB/KYC re-verification of the legal entity.
    ReverifyKyb,
    /// Promote the tenant's DR pair.
    PromoteDrPair,
}

impl LifecycleAction {
    /// Every gated action, in declaration order. Closed set: adding a variant
    /// forces every matrix arm to be revisited.
    pub const ALL: [Self; 5] = [
        Self::DeleteTenant,
        Self::ChangeJurisdiction,
        Self::RemovePaymentCredential,
        Self::ReverifyKyb,
        Self::PromoteDrPair,
    ];

    /// The stable wire slug for this action.
    #[must_use]
    pub const fn as_slug(self) -> &'static str {
        match self {
            Self::DeleteTenant => "delete-tenant",
            Self::ChangeJurisdiction => "change-jurisdiction",
            Self::RemovePaymentCredential => "remove-payment-credential",
            Self::ReverifyKyb => "reverify-kyb",
            Self::PromoteDrPair => "promote-dr-pair",
        }
    }

    /// Parse a wire slug. Exact match only: a near-miss must not be guessed
    /// into a different action, so the caller fails closed instead.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|action| action.as_slug() == slug)
    }

    /// Every slug, comma-separated - for operator-facing error text.
    #[must_use]
    pub fn slug_list() -> String {
        Self::ALL
            .iter()
            .map(|action| action.as_slug())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl core::fmt::Display for LifecycleAction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_slug())
    }
}

impl LockReason {
    /// Every reason, weakest-first is NOT guaranteed; declaration order.
    pub const ALL: [Self; 7] = [
        Self::PendingDeletionGrace,
        Self::JurisdictionMigration,
        Self::KybReverification,
        Self::DrPromotionWindow,
        Self::PaymentDispute,
        Self::LegalHold,
        Self::ManualSoftLock,
    ];

    /// The stable wire slug for this reason.
    #[must_use]
    pub const fn as_slug(self) -> &'static str {
        match self {
            Self::PendingDeletionGrace => "pending-deletion-grace",
            Self::JurisdictionMigration => "jurisdiction-migration",
            Self::KybReverification => "kyb-reverification",
            Self::DrPromotionWindow => "dr-promotion-window",
            Self::PaymentDispute => "payment-dispute",
            Self::LegalHold => "legal-hold",
            Self::ManualSoftLock => "manual-soft-lock",
        }
    }

    /// Operator-facing sentence explaining why a lock of this reason stands.
    /// Rendered into every [`crate::LockDecision::explanation`].
    ///
    /// Each sentence must account for EVERYTHING the reason's matrix row
    /// blocks. A rationale narrower than the row - `legal-hold` once said only
    /// "no irreversible or jurisdiction-affecting change", while the row also
    /// refuses `reverify-kyb` and `remove-payment-credential` - leaves the
    /// operator unable to tell an intended refusal from a matrix bug.
    #[must_use]
    pub const fn rationale(self) -> &'static str {
        match self {
            Self::PendingDeletionGrace => {
                "tenant is inside the DSR deletion grace window - the record must stay stable \
                 until the window lapses"
            }
            Self::JurisdictionMigration => {
                "residency migration is in flight - the tenant's home region is mid-move"
            }
            Self::KybReverification => {
                "KYB/KYC re-verification is in flight - the legal entity is unconfirmed"
            }
            Self::DrPromotionWindow => "DR pair promotion window is open - cell topology is moving",
            Self::PaymentDispute => {
                "open payment dispute - the tenant and its payment instrument must be retained \
                 until the dispute settles"
            }
            Self::LegalHold => {
                "statutory preservation order - EVERY gated lifecycle change is refused while it \
                 stands, identity re-verification and payment-credential removal as well as \
                 deletion and jurisdiction change, because each of them mutates preserved evidence"
            }
            Self::ManualSoftLock => {
                "an operator placed a manual protection lock - irreversible and \
                 jurisdiction-affecting changes are held until a tenant admin lifts it"
            }
        }
    }

    /// How strong this reason is. Higher wins. Total and stable.
    ///
    /// `legal-hold` is strictly the strongest. `manual-soft-lock` is weakest:
    /// it is the only reason a tenant admin can lift alone, so it must never
    /// outrank a hold that takes a compliance quorum to lift.
    #[must_use]
    pub const fn precedence(self) -> u8 {
        match self {
            Self::LegalHold => 6,
            Self::PaymentDispute => 5,
            Self::PendingDeletionGrace => 4,
            Self::JurisdictionMigration => 3,
            Self::KybReverification => 2,
            Self::DrPromotionWindow => 1,
            Self::ManualSoftLock => 0,
        }
    }

    /// Whether a LIVE lock with this reason blocks `action`.
    ///
    /// This is the module-doc table in code. Expiry is NOT considered here -
    /// [`crate::evaluate_at`] filters lapsed locks before consulting the matrix.
    #[must_use]
    pub const fn blocks(self, action: LifecycleAction) -> bool {
        use LifecycleAction as A;
        use LockReason as R;
        matches!(
            (self, action),
            // legal-hold: every gated action.
            (R::LegalHold, _)
            // payment-dispute: retention of the tenant and its instrument.
            | (R::PaymentDispute, A::DeleteTenant | A::RemovePaymentCredential)
            // pending-deletion-grace: keep the record where it is.
            | (R::PendingDeletionGrace, A::DeleteTenant | A::ChangeJurisdiction)
            // jurisdiction-migration: residency is mid-move.
            | (
                R::JurisdictionMigration,
                A::DeleteTenant | A::ChangeJurisdiction | A::ReverifyKyb | A::PromoteDrPair
            )
            // kyb-reverification: entity unconfirmed, evidence frozen.
            | (
                R::KybReverification,
                A::DeleteTenant
                    | A::ChangeJurisdiction
                    | A::RemovePaymentCredential
                    | A::ReverifyKyb
            )
            // dr-promotion-window: cell topology is moving.
            | (
                R::DrPromotionWindow,
                A::DeleteTenant | A::ChangeJurisdiction | A::PromoteDrPair
            )
            // manual-soft-lock: guard the irreversible and the residency-moving.
            | (R::ManualSoftLock, A::DeleteTenant | A::ChangeJurisdiction)
        )
    }

    /// The action a lock of this reason exists to perform, if any - i.e.
    /// whether this reason is a WINDOW (`Some`) or a HOLD (`None`).
    ///
    /// A `jurisdiction-migration` lock is taken IN ORDER TO change jurisdiction,
    /// so it is a window. A `legal-hold`, a `payment-dispute`, a
    /// `manual-soft-lock` and a `pending-deletion-grace` all exist to STOP
    /// something, so they are holds and serve no action.
    ///
    /// `pending-deletion-grace` is the one worth spelling out. It is the DSR
    /// WAITING window, not an intent to delete - it blocks `delete-tenant`
    /// itself. Reading it as an intent made the statutory clock unrecordable
    /// for exactly the tenants that carry a residual-retention basis, which is
    /// IP-021 §D.7's headline scenario: an erasure request arriving for a
    /// tenant already under legal hold could not be booked at all.
    ///
    /// A window's action is one this reason also blocks - that self-block is
    /// what makes a window mutually exclusive with itself.
    #[must_use]
    pub const fn implied_action(self) -> Option<LifecycleAction> {
        match self {
            Self::JurisdictionMigration => Some(LifecycleAction::ChangeJurisdiction),
            Self::KybReverification => Some(LifecycleAction::ReverifyKyb),
            Self::DrPromotionWindow => Some(LifecycleAction::PromoteDrPair),
            Self::LegalHold
            | Self::PaymentDispute
            | Self::PendingDeletionGrace
            | Self::ManualSoftLock => None,
        }
    }

    /// Whether this reason is a hold - it stops actions and serves none, so a
    /// lock of this reason is always acquirable.
    #[must_use]
    pub const fn is_hold(self) -> bool {
        self.implied_action().is_none()
    }
}

/// The standing lock, if any, that `candidate` contradicts at `now_epoch_s`.
///
/// A candidate contradicts a standing lock when the standing lock is LIVE, is a
/// DIFFERENT lock of the SAME tenant, and blocks the action the candidate
/// exists to perform. Opening a jurisdiction-migration window on a tenant under
/// legal hold is the canonical case: the migration the window serves can never
/// run while the hold stands.
///
/// Holds ([`LockReason::implied_action`] of `None`) serve no action and are
/// therefore ALWAYS acquirable. A legal hold, a dispute retention, an
/// operator's soft lock and a DSR grace window must each be placeable on a
/// tenant no matter what else stands - a court order does not wait for a
/// migration to finish, and an erasure request must be bookable against a
/// tenant that already has a retention basis.
///
/// Strength is NOT part of the test. It once was - only a STRICTLY STRONGER
/// standing lock could contradict - and that made the reachable set depend on
/// arrival order: `{jurisdiction-migration, dr-promotion-window}` was refused
/// when the migration came first and accepted when the DR window did, though
/// each blocks the action the other exists to perform and neither could ever
/// run. Strength survives only as the tie-break that decides WHICH
/// contradicting lock is named.
///
/// Because a window's action is one the window itself blocks, this is also the
/// rule that keeps a self-blocking reason to at most one live lock per tenant:
/// two migration windows held by two services, each waiting on a migration the
/// other forbids, is precisely the state to refuse.
///
/// A lock never contradicts ITSELF: standing rows sharing the candidate's id
/// are skipped, so passing the candidate inside `standing` is safe.
///
/// Returns the strongest contradicting lock, ties broken by ascending id, so
/// the answer does not depend on `standing` order.
#[must_use]
pub fn acquisition_conflict<'a>(
    candidate: &LifecycleLock,
    standing: &'a [LifecycleLock],
    now_epoch_s: u64,
) -> Option<&'a LifecycleLock> {
    let intent = candidate.reason.implied_action()?;
    standing
        .iter()
        .filter(|held| held.tenant_id.trim() == candidate.tenant_id.trim())
        .filter(|held| held.id.as_str().trim() != candidate.id.as_str().trim())
        .filter(|held| held.is_live_at(now_epoch_s))
        .filter(|held| held.reason.blocks(intent))
        .min_by(|left, right| {
            right
                .reason
                .precedence()
                .cmp(&left.reason.precedence())
                .then_with(|| left.id.cmp(&right.id))
        })
}

/// [`acquisition_conflict`] as a `Result`, for call sites that only need the
/// verdict.
///
/// [`LockKernelError`] variants carry no payload, so a caller that must tell an
/// operator WHICH lock stopped the acquisition calls [`acquisition_conflict`]
/// instead and renders [`LifecycleLock::describe`] on the answer.
///
/// # Errors
///
/// [`LockKernelError::PrecedenceConflict`] when a live lock contradicts the
/// candidate.
pub fn check_acquisition(
    candidate: &LifecycleLock,
    standing: &[LifecycleLock],
    now_epoch_s: u64,
) -> Result<(), LockKernelError> {
    match acquisition_conflict(candidate, standing, now_epoch_s) {
        Some(_) => Err(LockKernelError::PrecedenceConflict),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LockId;

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

    /// The module-doc table, transcribed by hand from the prose. If the code
    /// matrix and the documented matrix ever diverge, this fails.
    const DOCUMENTED: [(LockReason, [bool; 5]); 7] = [
        //                                  del   juris  paycred kyb   dr
        (LockReason::LegalHold, [true, true, true, true, true]),
        (
            LockReason::PaymentDispute,
            [true, false, true, false, false],
        ),
        (
            LockReason::PendingDeletionGrace,
            [true, true, false, false, false],
        ),
        (
            LockReason::JurisdictionMigration,
            [true, true, false, true, true],
        ),
        (
            LockReason::KybReverification,
            [true, true, true, true, false],
        ),
        (
            LockReason::DrPromotionWindow,
            [true, true, false, false, true],
        ),
        (
            LockReason::ManualSoftLock,
            [true, true, false, false, false],
        ),
    ];

    /// Hold or window, pinned by equality. `pending-deletion-grace` is the row
    /// that matters: classifying the DSR waiting window as an intent to delete
    /// is what made IP-021 §D.7 unrepresentable.
    const CLASSIFICATION: [(LockReason, Option<LifecycleAction>); 7] = [
        (LockReason::LegalHold, None),
        (LockReason::PaymentDispute, None),
        (LockReason::PendingDeletionGrace, None),
        (LockReason::ManualSoftLock, None),
        (
            LockReason::JurisdictionMigration,
            Some(LifecycleAction::ChangeJurisdiction),
        ),
        (
            LockReason::KybReverification,
            Some(LifecycleAction::ReverifyKyb),
        ),
        (
            LockReason::DrPromotionWindow,
            Some(LifecycleAction::PromoteDrPair),
        ),
    ];

    #[test]
    fn matrix_table_matches_documentation() {
        for (reason, row) in DOCUMENTED {
            for (column, action) in LifecycleAction::ALL.into_iter().enumerate() {
                assert_eq!(
                    reason.blocks(action),
                    row[column],
                    "matrix cell ({}, {}) disagrees with the documented table",
                    reason.as_slug(),
                    action.as_slug()
                );
            }
        }
        assert_eq!(
            DOCUMENTED.len(),
            LockReason::ALL.len(),
            "every reason needs a documented row"
        );
    }

    #[test]
    fn every_reason_is_classified_as_a_hold_or_a_window() {
        for (reason, expected) in CLASSIFICATION {
            assert_eq!(
                reason.implied_action(),
                expected,
                "{} changed kind; a hold that starts serving an action becomes \
                 unacquirable under the very holds it is meant to coexist with",
                reason.as_slug()
            );
            assert_eq!(reason.is_hold(), expected.is_none());
        }
        assert_eq!(CLASSIFICATION.len(), LockReason::ALL.len());
    }

    /// A window's action must be one the window itself blocks, or the mutual
    /// exclusion the window claims does not exist.
    #[test]
    fn a_window_blocks_the_action_it_serves() {
        for reason in LockReason::ALL {
            if let Some(intent) = reason.implied_action() {
                assert!(
                    reason.blocks(intent),
                    "{} serves {} but does not block it, so two could run at once",
                    reason.as_slug(),
                    intent.as_slug()
                );
            }
        }
    }

    #[test]
    fn deletion_is_blocked_by_every_reason() {
        for reason in LockReason::ALL {
            assert!(
                reason.blocks(LifecycleAction::DeleteTenant),
                "{} must block the irreversible action",
                reason.as_slug()
            );
        }
    }

    /// No reason serves `delete-tenant`: deletion is the one action that is
    /// never held open by a window, so nothing can claim to exist to perform
    /// it and thereby become unacquirable under every retention hold.
    #[test]
    fn nothing_claims_to_exist_in_order_to_delete() {
        for reason in LockReason::ALL {
            assert_ne!(
                reason.implied_action(),
                Some(LifecycleAction::DeleteTenant),
                "{} claims to exist in order to delete the tenant",
                reason.as_slug()
            );
        }
    }

    #[test]
    fn no_reason_blocks_every_action_except_legal_hold() {
        for reason in LockReason::ALL {
            let blocks_all = LifecycleAction::ALL
                .into_iter()
                .all(|action| reason.blocks(action));
            assert_eq!(
                blocks_all,
                reason == LockReason::LegalHold,
                "only legal-hold may be total; {} is not",
                reason.as_slug()
            );
        }
    }

    #[test]
    fn legal_hold_is_strictly_the_strongest() {
        for reason in LockReason::ALL {
            if reason != LockReason::LegalHold {
                assert!(
                    LockReason::LegalHold.precedence() > reason.precedence(),
                    "legal-hold must outrank {}",
                    reason.as_slug()
                );
            }
        }
    }

    #[test]
    fn precedence_is_a_total_order_with_no_ties() {
        let mut ranks: Vec<u8> = LockReason::ALL.iter().map(|r| r.precedence()).collect();
        ranks.sort_unstable();
        let distinct = ranks.len();
        ranks.dedup();
        assert_eq!(ranks.len(), distinct, "two reasons share a rank");
    }

    #[test]
    fn slug_round_trips_and_is_unique() {
        let mut slugs: Vec<&str> = LifecycleAction::ALL
            .iter()
            .map(|action| action.as_slug())
            .collect();
        let distinct = slugs.len();
        slugs.sort_unstable();
        slugs.dedup();
        assert_eq!(slugs.len(), distinct);
        for action in LifecycleAction::ALL {
            assert_eq!(LifecycleAction::from_slug(action.as_slug()), Some(action));
        }
        assert_eq!(LifecycleAction::from_slug("delete"), None);
        assert_eq!(LifecycleAction::from_slug("Delete-Tenant"), None);
        assert_eq!(LifecycleAction::from_slug(""), None);
    }

    #[test]
    fn reason_slugs_are_unique_and_every_reason_has_a_rationale() {
        let mut slugs: Vec<&str> = LockReason::ALL
            .iter()
            .map(|reason| reason.as_slug())
            .collect();
        let distinct = slugs.len();
        slugs.sort_unstable();
        slugs.dedup();
        assert_eq!(slugs.len(), distinct);
        for reason in LockReason::ALL {
            assert!(
                !reason.rationale().is_empty(),
                "{} has no operator sentence",
                reason.as_slug()
            );
        }
    }

    /// A refusal must be explained by a sentence that covers it. `legal-hold`
    /// blocks `reverify-kyb`, so its rationale may not stop at "irreversible or
    /// jurisdiction-affecting change".
    #[test]
    fn the_legal_hold_rationale_accounts_for_the_whole_row() {
        let text = LockReason::LegalHold.rationale();
        assert!(
            text.contains("EVERY gated lifecycle change"),
            "the sentence must claim the whole row: {text}"
        );
        assert!(
            text.contains("re-verification") && text.contains("payment-credential"),
            "the two non-obvious cells must be named: {text}"
        );
    }

    #[test]
    fn migration_window_conflicts_with_a_standing_legal_hold() {
        let standing = [lock("lk-legal", LockReason::LegalHold, 9_999)];
        let candidate = lock("lk-move", LockReason::JurisdictionMigration, 9_999);
        assert_eq!(
            check_acquisition(&candidate, &standing, 10),
            Err(LockKernelError::PrecedenceConflict)
        );
        assert_eq!(
            acquisition_conflict(&candidate, &standing, 10).map(|l| l.id.clone()),
            Some(LockId("lk-legal".to_owned()))
        );
    }

    #[test]
    fn a_hold_is_always_acquirable() {
        // Every reason that serves no action must be placeable no matter what
        // already stands - a legal hold, a dispute retention, an operator soft
        // lock, and above all the DSR grace window, whose whole job is to be
        // recorded while a stronger retention basis delays the erasure.
        let standing: Vec<LifecycleLock> = LockReason::ALL
            .into_iter()
            .enumerate()
            .map(|(index, reason)| lock(&format!("lk-{index}"), reason, 9_999))
            .collect();
        for reason in LockReason::ALL {
            if !reason.is_hold() {
                continue;
            }
            let candidate = lock("lk-new", reason, 9_999);
            assert_eq!(
                check_acquisition(&candidate, &standing, 10),
                Ok(()),
                "{} serves no action and must always be acquirable",
                reason.as_slug()
            );
        }
    }

    #[test]
    fn an_expired_stronger_lock_does_not_conflict() {
        let standing = [lock("lk-legal", LockReason::LegalHold, 1_000)];
        let candidate = lock("lk-move", LockReason::JurisdictionMigration, 9_999);
        assert_eq!(
            check_acquisition(&candidate, &standing, 999),
            Err(LockKernelError::PrecedenceConflict)
        );
        assert_eq!(check_acquisition(&candidate, &standing, 1_000), Ok(()));
    }

    #[test]
    fn a_weaker_standing_lock_that_blocks_the_intent_still_conflicts() {
        // manual-soft-lock (0) is the weakest reason there is, and it blocks
        // change-jurisdiction; a migration window (3) may not be opened over it
        // just because the window outranks it.
        let standing = [lock("lk-soft", LockReason::ManualSoftLock, 9_999)];
        let candidate = lock("lk-move", LockReason::JurisdictionMigration, 9_999);
        assert!(candidate.reason.precedence() > standing[0].reason.precedence());
        assert_eq!(
            check_acquisition(&candidate, &standing, 10),
            Err(LockKernelError::PrecedenceConflict)
        );
    }

    #[test]
    fn a_stronger_standing_lock_that_does_not_block_the_intent_is_fine() {
        // payment-dispute (5) outranks dr-promotion-window (1) but does not
        // block DR promotion, so the window may be opened.
        let standing = [lock("lk-pay", LockReason::PaymentDispute, 9_999)];
        let candidate = lock("lk-dr", LockReason::DrPromotionWindow, 9_999);
        assert_eq!(check_acquisition(&candidate, &standing, 10), Ok(()));
    }

    /// A self-blocking reason is a singleton per tenant. Two migration windows
    /// held by two services, each waiting on a migration the other forbids, is
    /// exactly the state the conflict rule exists to refuse.
    #[test]
    fn a_window_cannot_stand_twice_for_one_tenant() {
        for reason in LockReason::ALL {
            if reason.is_hold() {
                continue;
            }
            let standing = [lock("lk-1", reason, 9_999)];
            let candidate = lock("lk-2", reason, 9_999);
            assert_eq!(
                check_acquisition(&candidate, &standing, 10),
                Err(LockKernelError::PrecedenceConflict),
                "a second live {} window must be refused",
                reason.as_slug()
            );
        }
    }

    #[test]
    fn a_lock_does_not_conflict_with_itself() {
        // Passing the candidate inside `standing` - what a caller who reads the
        // whole tenant row set does - must not refuse the candidate.
        let candidate = lock("lk-move", LockReason::JurisdictionMigration, 9_999);
        let standing = [candidate.clone()];
        assert_eq!(check_acquisition(&candidate, &standing, 10), Ok(()));
    }

    #[test]
    fn a_conflict_ignores_locks_of_another_tenant() {
        let mut other = lock("lk-legal", LockReason::LegalHold, 9_999);
        other.tenant_id = "ten_other".to_owned();
        let candidate = lock("lk-move", LockReason::JurisdictionMigration, 9_999);
        assert_eq!(check_acquisition(&candidate, &[other], 10), Ok(()));
    }

    #[test]
    fn conflict_selection_is_order_independent() {
        let legal = lock("lk-legal", LockReason::LegalHold, 9_999);
        let grace = lock("lk-grace", LockReason::PendingDeletionGrace, 9_999);
        let candidate = lock("lk-move", LockReason::JurisdictionMigration, 9_999);
        let forward_input = [legal.clone(), grace.clone()];
        let reverse_input = [grace, legal];
        let forward = acquisition_conflict(&candidate, &forward_input, 10);
        let reverse = acquisition_conflict(&candidate, &reverse_input, 10);
        assert_eq!(forward, reverse);
        assert_eq!(
            forward.map(|l| l.id.clone()),
            Some(LockId("lk-legal".to_owned())),
            "the strongest contradicting lock governs"
        );
    }

    /// Two windows are refused together in BOTH orders or in NEITHER. Any
    /// asymmetry here means the reachable state of a tenant depends on which
    /// service happened to call first.
    #[test]
    fn two_windows_are_co_acquirable_in_both_orders_or_in_neither() {
        for first in LockReason::ALL {
            for second in LockReason::ALL {
                if first.is_hold() || second.is_hold() || first == second {
                    continue;
                }
                let a = lock("lk-a", first, 9_999);
                let b = lock("lk-b", second, 9_999);
                let forward = check_acquisition(&b, std::slice::from_ref(&a), 10);
                let reverse = check_acquisition(&a, std::slice::from_ref(&b), 10);
                assert_eq!(
                    forward,
                    reverse,
                    "{} then {} disagrees with the reverse order",
                    first.as_slug(),
                    second.as_slug()
                );
            }
        }
    }

    /// IP-021 §D.7 in the small: the grace window and every retention hold must
    /// be co-acquirable in EITHER order, or whether a statutory erasure clock
    /// can be recorded depends on the order the two rows arrived in.
    #[test]
    fn a_grace_window_and_a_retention_hold_are_co_acquirable_in_either_order() {
        for hold in [
            LockReason::LegalHold,
            LockReason::PaymentDispute,
            LockReason::ManualSoftLock,
        ] {
            let held = lock("lk-hold", hold, 9_999);
            let grace = lock("lk-grace", LockReason::PendingDeletionGrace, 9_999);
            assert_eq!(
                check_acquisition(&grace, std::slice::from_ref(&held), 10),
                Ok(()),
                "grace window refused under a standing {}",
                hold.as_slug()
            );
            assert_eq!(
                check_acquisition(&held, std::slice::from_ref(&grace), 10),
                Ok(()),
                "{} refused under a standing grace window",
                hold.as_slug()
            );
        }
    }
}
