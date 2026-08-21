//! An in-memory [`LockStore`] adapter.
//!
//! Backed by a [`BTreeMap`] keyed by `(tenant_id, lock_id)`, so iteration and
//! every derived list are ordered and reproducible, and every per-tenant read
//! is a `range` over that tenant's contiguous key block rather than a scan of
//! the whole multi-tenant map.
//!
//! # Nothing is dropped silently
//!
//! Expired locks are RETAINED rather than dropped: an operator asking "why was
//! this refused last Tuesday" needs the lapsed row. A re-acquisition of a
//! lapsed id does not overwrite it either - the displaced row moves to the
//! superseded list, readable through
//! [`InMemoryLockStore::superseded_locks`] and included in
//! [`InMemoryLockStore::all_locks`]. Without that, any caller could erase a
//! lapsed legal-hold record simply by taking its id back with a weaker reason.
//!
//! [`LockStore::purge_expired`] is the explicit, callable garbage collection
//! instead of a silent one, and it is on the PORT so a holder of a
//! `&mut dyn LockStore` can reclaim as well as a holder of the concrete type.
//!
//! This is the only [`LockStore`] implementation today; see the crate-level
//! Gaps for why the durable adapter is deferred, and for what this store does
//! NOT do about retention (there is no cap and no automatic eviction: growth is
//! bounded only by how often somebody calls purge).

use std::collections::BTreeMap;

use crate::precedence::check_acquisition;
use crate::release::{authorize_quorum_release, authorize_release, authorize_renewal, check_lease};
use crate::{LifecycleLock, LockId, LockKernelError, LockStore, ReleaseApproval};

/// An in-memory store of lifecycle locks.
#[derive(Clone, Debug, Default)]
pub struct InMemoryLockStore {
    locks: BTreeMap<(String, String), LifecycleLock>,
    superseded: Vec<LifecycleLock>,
}

impl InMemoryLockStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The key a tenant/id pair is stored under. Both halves are trimmed, so a
    /// non-canonical `" ten_acme"` cannot open a shadow namespace that a later
    /// `"ten_acme"` lookup fails to see.
    fn key(tenant_id: &str, id: &LockId) -> (String, String) {
        (tenant_id.trim().to_owned(), id.as_str().trim().to_owned())
    }

    /// The current rows of one tenant, ordered by lock id.
    ///
    /// A `range` over the tenant's own contiguous key block: the cost is
    /// proportional to that tenant's rows, not to the whole store.
    fn rows_for<'a>(&'a self, tenant_id: &'a str) -> impl Iterator<Item = &'a LifecycleLock> + 'a {
        self.locks
            .range((tenant_id.to_owned(), String::new())..)
            .take_while(move |(key, _)| key.0 == tenant_id)
            .map(|(_, lock)| lock)
    }

    /// Every row this store has ever held for `tenant_id` and not purged: the
    /// current rows, live or lapsed, plus every row a re-acquisition displaced.
    ///
    /// Ordered by lock id, then by expiry, then by holder - total and stable,
    /// so a superseded row and the row that replaced it appear together and in
    /// chronological order.
    #[must_use]
    pub fn all_locks(&self, tenant_id: &str) -> Vec<LifecycleLock> {
        let tenant = tenant_id.trim();
        let mut rows: Vec<LifecycleLock> = self
            .superseded
            .iter()
            .filter(|lock| lock.tenant_id.trim() == tenant)
            .cloned()
            .chain(self.rows_for(tenant).cloned())
            .collect();
        rows.sort_by(|left, right| {
            left.id
                .cmp(&right.id)
                .then_with(|| left.expires_at_epoch_s.cmp(&right.expires_at_epoch_s))
                .then_with(|| left.holder.cmp(&right.holder))
        });
        rows
    }

    /// The rows a re-acquisition displaced, oldest supersession first.
    ///
    /// A lapsed lock whose id was taken over by a new acquisition lands here
    /// rather than being overwritten, so the audit trail survives the takeover.
    #[must_use]
    pub fn superseded_locks(&self, tenant_id: &str) -> Vec<LifecycleLock> {
        let tenant = tenant_id.trim();
        self.superseded
            .iter()
            .filter(|lock| lock.tenant_id.trim() == tenant)
            .cloned()
            .collect()
    }

    /// The CURRENT lock under this tenant and id, live or lapsed. Superseded
    /// rows are not reachable here - by definition something else holds the id.
    #[must_use]
    pub fn get(&self, tenant_id: &str, id: &LockId) -> Option<&LifecycleLock> {
        self.locks.get(&Self::key(tenant_id, id))
    }

    /// Current rows held, across every tenant. Superseded rows are counted by
    /// [`Self::superseded_len`].
    #[must_use]
    pub fn len(&self) -> usize {
        self.locks.len()
    }

    /// Rows displaced by a re-acquisition and not yet purged, across every
    /// tenant.
    #[must_use]
    pub fn superseded_len(&self) -> usize {
        self.superseded.len()
    }

    /// Whether the store holds no current rows at all. Superseded history may
    /// still be present; see [`Self::superseded_len`].
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.locks.is_empty()
    }

    fn take_authorized<F>(
        &mut self,
        tenant_id: &str,
        id: &LockId,
        authorize: F,
    ) -> Result<LifecycleLock, LockKernelError>
    where
        F: FnOnce(&LifecycleLock) -> Result<(), LockKernelError>,
    {
        let key = Self::key(tenant_id, id);
        let lock = self.locks.get(&key).ok_or(LockKernelError::NotFound)?;
        authorize(lock)?;
        self.locks.remove(&key).ok_or(LockKernelError::NotFound)
    }
}

impl LockStore for InMemoryLockStore {
    fn acquire(
        &mut self,
        lock: LifecycleLock,
        now_epoch_s: u64,
    ) -> Result<LifecycleLock, LockKernelError> {
        // Rebuild through the constructor rather than trusting the caller: the
        // fields are public, so a lock handed in here may never have been
        // validated, or may have been edited after it was.
        let lock = lock.canonicalized()?;
        if lock.is_expired_at(now_epoch_s) {
            // Recording a lease that is already lapsed would put a row in the
            // store that blocks nothing; say so instead of accepting it.
            return Err(LockKernelError::Expired);
        }
        check_lease(lock.reason, lock.expires_at_epoch_s, now_epoch_s)?;
        let key = (lock.tenant_id.clone(), lock.id.as_str().to_owned());
        if self
            .locks
            .get(&key)
            .is_some_and(|existing| existing.is_live_at(now_epoch_s))
        {
            return Err(LockKernelError::AlreadyHeld);
        }
        let standing = self.live_locks(&lock.tenant_id, now_epoch_s);
        check_acquisition(&lock, &standing, now_epoch_s)?;
        if let Some(displaced) = self.locks.insert(key, lock.clone()) {
            // A lapsed row whose id was taken over. Keep it: the audit trail is
            // the reason lapsed rows are retained in the first place.
            self.superseded.push(displaced);
        }
        Ok(lock)
    }

    fn renew(
        &mut self,
        tenant_id: &str,
        id: &LockId,
        principal: &str,
        new_expires_at_epoch_s: u64,
        now_epoch_s: u64,
    ) -> Result<LifecycleLock, LockKernelError> {
        let key = Self::key(tenant_id, id);
        let lock = self.locks.get_mut(&key).ok_or(LockKernelError::NotFound)?;
        authorize_renewal(lock, principal, new_expires_at_epoch_s, now_epoch_s)?;
        lock.expires_at_epoch_s = new_expires_at_epoch_s;
        Ok(lock.clone())
    }

    fn release(
        &mut self,
        tenant_id: &str,
        id: &LockId,
        principal: &str,
        now_epoch_s: u64,
    ) -> Result<LifecycleLock, LockKernelError> {
        self.take_authorized(tenant_id, id, |lock| {
            authorize_release(lock, principal, now_epoch_s)
        })
    }

    fn release_with_quorum(
        &mut self,
        tenant_id: &str,
        id: &LockId,
        approvals: &[ReleaseApproval],
        now_epoch_s: u64,
    ) -> Result<LifecycleLock, LockKernelError> {
        self.take_authorized(tenant_id, id, |lock| {
            authorize_quorum_release(lock, approvals, now_epoch_s)
        })
    }

    fn live_locks(&self, tenant_id: &str, now_epoch_s: u64) -> Vec<LifecycleLock> {
        let tenant = tenant_id.trim();
        self.rows_for(tenant)
            .filter(|lock| lock.is_live_at(now_epoch_s))
            .cloned()
            .collect()
    }

    fn purge_expired(&mut self, now_epoch_s: u64) -> usize {
        let before = self.locks.len() + self.superseded.len();
        self.locks.retain(|_, lock| lock.is_live_at(now_epoch_s));
        self.superseded.retain(|lock| lock.is_live_at(now_epoch_s));
        before - (self.locks.len() + self.superseded.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LockReason;
    use crate::precedence::LifecycleAction;
    use crate::release::{ReleaseApproval, ReleaseRole};

    fn lock(id: &str, reason: LockReason, holder: &str, expires: u64) -> LifecycleLock {
        LifecycleLock::new(
            LockId(id.to_owned()),
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
    fn acquire_then_decide_blocks_the_gated_action() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(
                lock("lk-1", LockReason::PaymentDispute, "svc-billing", 1_000),
                10,
            )
            .unwrap();
        let delete = store.decide("ten_acme", LifecycleAction::DeleteTenant, 10);
        assert!(!delete.allow);
        assert_eq!(delete.governing_lock, Some(LockId("lk-1".to_owned())));
        let promote = store.decide("ten_acme", LifecycleAction::PromoteDrPair, 10);
        assert!(promote.allow);
    }

    #[test]
    fn a_lock_is_scoped_to_its_tenant() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(lock("lk-1", LockReason::LegalHold, "svc-legal", 9_999), 10)
            .unwrap();
        assert!(
            store
                .decide("ten_other", LifecycleAction::DeleteTenant, 10)
                .allow,
            "another tenant's hold must not block this tenant"
        );
        assert!(store.live_locks("ten_other", 10).is_empty());
    }

    /// A tenant id that is one space away from canonical must not open a second
    /// namespace: the hold would be on record and would block nothing.
    #[test]
    fn a_non_canonical_tenant_id_does_not_open_a_shadow_namespace() {
        let mut store = InMemoryLockStore::new();
        let mut sloppy = lock("lk-1", LockReason::LegalHold, "svc-legal", 9_999);
        sloppy.tenant_id = " ten_acme ".to_owned();
        sloppy.id = LockId(" lk-1 ".to_owned());
        store.acquire(sloppy, 10).unwrap();
        assert!(
            !store
                .decide("ten_acme", LifecycleAction::DeleteTenant, 10)
                .allow,
            "the statutory hold must block deletion under the canonical id"
        );
        assert_eq!(store.live_locks("ten_acme", 10).len(), 1);
        assert_eq!(store.live_locks(" ten_acme ", 10).len(), 1);
        assert!(
            store
                .get("ten_acme", &LockId("lk-1".to_owned()))
                .is_some_and(|held| held.tenant_id == "ten_acme" && held.holder == "svc-legal")
        );
        assert_eq!(
            store.acquire(lock("lk-1", LockReason::LegalHold, "thief", 9_999), 10),
            Err(LockKernelError::AlreadyHeld),
            "the canonical id must collide with the sloppy one, not sit beside it"
        );
    }

    #[test]
    fn re_acquiring_a_live_id_is_already_held() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(
                lock("lk-1", LockReason::PaymentDispute, "svc-billing", 1_000),
                10,
            )
            .unwrap();
        assert_eq!(
            store.acquire(lock("lk-1", LockReason::PaymentDispute, "thief", 1_000), 10),
            Err(LockKernelError::AlreadyHeld),
            "a live lock must not be overwritten by a second acquirer"
        );
        assert_eq!(
            store
                .get("ten_acme", &LockId("lk-1".to_owned()))
                .map(|l| l.holder.clone()),
            Some("svc-billing".to_owned())
        );
    }

    #[test]
    fn a_lapsed_id_can_be_re_acquired_by_someone_else() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(
                lock("lk-1", LockReason::PaymentDispute, "svc-billing", 1_000),
                10,
            )
            .unwrap();
        let taken = store
            .acquire(
                lock("lk-1", LockReason::PaymentDispute, "svc-other", 5_000),
                1_000,
            )
            .unwrap();
        assert_eq!(taken.holder, "svc-other");
    }

    /// Taking over a lapsed id must not erase what stood there. Otherwise any
    /// caller can delete a lapsed legal-hold record - the evidence that the
    /// hold ever existed - by re-acquiring its id with a weaker reason.
    #[test]
    fn re_acquiring_a_lapsed_id_archives_the_row_it_displaces() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(
                lock("lk-legal", LockReason::LegalHold, "svc-legal", 1_000),
                10,
            )
            .unwrap();
        store
            .acquire(
                lock("lk-legal", LockReason::DrPromotionWindow, "attacker", 9_999),
                1_001,
            )
            .unwrap();

        let current = store
            .get("ten_acme", &LockId("lk-legal".to_owned()))
            .cloned();
        assert_eq!(
            current.map(|held| (held.reason, held.holder)),
            Some((LockReason::DrPromotionWindow, "attacker".to_owned())),
            "the takeover itself is allowed - the id was free"
        );

        let history = store.superseded_locks("ten_acme");
        assert_eq!(history.len(), 1, "the displaced row must be kept");
        assert_eq!(history[0].reason, LockReason::LegalHold);
        assert_eq!(history[0].holder, "svc-legal");
        assert_eq!(history[0].expires_at_epoch_s, 1_000);

        let audit = store.all_locks("ten_acme");
        assert_eq!(
            audit.len(),
            2,
            "an operator reading the tenant's history must still see the hold"
        );
        assert_eq!(audit[0].reason, LockReason::LegalHold, "oldest first by id");
        assert_eq!(audit[1].reason, LockReason::DrPromotionWindow);
        assert_eq!(store.len(), 1);
        assert_eq!(store.superseded_len(), 1);
    }

    #[test]
    fn acquiring_an_already_lapsed_lease_is_refused() {
        let mut store = InMemoryLockStore::new();
        assert_eq!(
            store.acquire(
                lock("lk-1", LockReason::LegalHold, "svc-legal", 1_000),
                1_000
            ),
            Err(LockKernelError::Expired)
        );
        assert!(store.is_empty());
    }

    #[test]
    fn a_lease_longer_than_its_reason_permits_is_refused() {
        let mut store = InMemoryLockStore::new();
        assert_eq!(
            store.acquire(
                lock("lk-dr", LockReason::DrPromotionWindow, "svc-dr", u64::MAX),
                10
            ),
            Err(LockKernelError::LeaseTooLong),
            "an operational window must not be grantable for all practical time"
        );
        assert!(store.is_empty());
        // The same window inside its ceiling is fine.
        store
            .acquire(
                lock("lk-dr", LockReason::DrPromotionWindow, "svc-dr", 86_410),
                10,
            )
            .unwrap();
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn a_blank_lock_never_reaches_the_store() {
        let mut store = InMemoryLockStore::new();
        let mut blank = lock("lk-1", LockReason::LegalHold, "svc-legal", 9_999);
        blank.holder = String::new();
        assert_eq!(store.acquire(blank, 10), Err(LockKernelError::InvalidLock));
        assert!(store.is_empty());
        let mut forged = lock("lk-1", LockReason::LegalHold, "svc-legal", 9_999);
        forged.id = LockId("lk-1\nallowed".to_owned());
        assert_eq!(store.acquire(forged, 10), Err(LockKernelError::InvalidLock));
        assert!(store.is_empty());
    }

    #[test]
    fn acquisition_against_a_stronger_hold_is_a_precedence_conflict() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(
                lock("lk-legal", LockReason::LegalHold, "svc-legal", 9_999),
                10,
            )
            .unwrap();
        assert_eq!(
            store.acquire(
                lock(
                    "lk-move",
                    LockReason::JurisdictionMigration,
                    "svc-residency",
                    9_999
                ),
                10
            ),
            Err(LockKernelError::PrecedenceConflict)
        );
        assert_eq!(store.len(), 1);
    }

    /// The DSR clock must be recordable for a tenant that already has a
    /// retention basis, and in either arrival order (IP-021 §D.7).
    #[test]
    fn a_grace_window_opens_under_a_standing_hold_in_either_order() {
        for holds_first in [true, false] {
            let mut store = InMemoryLockStore::new();
            let hold = lock("lk-hold", LockReason::LegalHold, "svc-legal", 9_999);
            let grace = lock(
                "lk-grace",
                LockReason::PendingDeletionGrace,
                "svc-dsr",
                9_999,
            );
            if holds_first {
                store.acquire(hold, 10).unwrap();
                store.acquire(grace, 10).unwrap();
            } else {
                store.acquire(grace, 10).unwrap();
                store.acquire(hold, 10).unwrap();
            }
            let ids: Vec<String> = store
                .live_locks("ten_acme", 10)
                .into_iter()
                .map(|held| held.id.0)
                .collect();
            assert_eq!(
                ids,
                vec!["lk-grace".to_owned(), "lk-hold".to_owned()],
                "reachability depended on arrival order (hold first: {holds_first})"
            );
            assert!(
                !store
                    .decide("ten_acme", LifecycleAction::DeleteTenant, 10)
                    .allow,
                "deletion must still be delayed"
            );
        }
    }

    #[test]
    fn release_requires_the_holder_and_removes_the_row() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(
                lock("lk-1", LockReason::DrPromotionWindow, "svc-dr", 1_000),
                10,
            )
            .unwrap();
        assert_eq!(
            store.release("ten_acme", &LockId("lk-1".to_owned()), "svc-other", 10),
            Err(LockKernelError::ReleaseUnauthorized)
        );
        assert_eq!(store.len(), 1, "a refused release must not remove the row");
        let released = store
            .release("ten_acme", &LockId("lk-1".to_owned()), "svc-dr", 10)
            .unwrap();
        assert_eq!(released.id, LockId("lk-1".to_owned()));
        assert!(store.is_empty());
        assert_eq!(
            store.release("ten_acme", &LockId("lk-1".to_owned()), "svc-dr", 10),
            Err(LockKernelError::NotFound)
        );
    }

    #[test]
    fn a_legal_hold_survives_the_ordinary_release_and_yields_to_quorum() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(lock("lk-1", LockReason::LegalHold, "svc-legal", 9_999), 10)
            .unwrap();
        assert_eq!(
            store.release("ten_acme", &LockId("lk-1".to_owned()), "svc-legal", 10),
            Err(LockKernelError::ReleaseRequiresQuorum)
        );
        assert!(
            !store
                .decide("ten_acme", LifecycleAction::DeleteTenant, 10)
                .allow,
            "the hold must still stand after the refused release"
        );
        assert_eq!(
            store.release_with_quorum(
                "ten_acme",
                &LockId("lk-1".to_owned()),
                &[approval("dana", ReleaseRole::DataProtectionOfficer)],
                10
            ),
            Err(LockKernelError::QuorumNotMet)
        );
        store
            .release_with_quorum(
                "ten_acme",
                &LockId("lk-1".to_owned()),
                &[
                    approval("dana", ReleaseRole::DataProtectionOfficer),
                    approval("cleo", ReleaseRole::Counsel),
                ],
                10,
            )
            .unwrap();
        assert!(
            store
                .decide("ten_acme", LifecycleAction::DeleteTenant, 10)
                .allow
        );
    }

    #[test]
    fn renew_extends_the_lease_and_moves_the_expiry_boundary() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(lock("lk-1", LockReason::LegalHold, "svc-legal", 1_000), 10)
            .unwrap();
        assert!(
            store
                .decide("ten_acme", LifecycleAction::DeleteTenant, 1_000)
                .allow
        );
        let renewed = store
            .renew(
                "ten_acme",
                &LockId("lk-1".to_owned()),
                "svc-legal",
                2_000,
                999,
            )
            .unwrap();
        assert_eq!(renewed.expires_at_epoch_s, 2_000);
        assert!(
            !store
                .decide("ten_acme", LifecycleAction::DeleteTenant, 1_000)
                .allow
        );
        assert!(
            store
                .decide("ten_acme", LifecycleAction::DeleteTenant, 2_000)
                .allow
        );
    }

    #[test]
    fn a_refused_renewal_leaves_the_lease_untouched() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(lock("lk-1", LockReason::LegalHold, "svc-legal", 1_000), 10)
            .unwrap();
        assert_eq!(
            store.renew(
                "ten_acme",
                &LockId("lk-1".to_owned()),
                "svc-other",
                9_000,
                10
            ),
            Err(LockKernelError::ReleaseUnauthorized)
        );
        assert_eq!(
            store.renew("ten_acme", &LockId("lk-1".to_owned()), "svc-legal", 500, 10),
            Err(LockKernelError::RenewalNotExtending)
        );
        assert_eq!(
            store.renew(
                "ten_acme",
                &LockId("lk-1".to_owned()),
                "svc-legal",
                u64::MAX,
                10
            ),
            Err(LockKernelError::LeaseTooLong)
        );
        assert_eq!(
            store.renew(
                "ten_missing",
                &LockId("lk-1".to_owned()),
                "svc-legal",
                9_000,
                10
            ),
            Err(LockKernelError::NotFound)
        );
        assert_eq!(
            store
                .get("ten_acme", &LockId("lk-1".to_owned()))
                .map(|l| l.expires_at_epoch_s),
            Some(1_000)
        );
    }

    #[test]
    fn lapsed_rows_are_retained_until_purged() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(lock("lk-1", LockReason::LegalHold, "svc-legal", 1_000), 10)
            .unwrap();
        store
            .acquire(
                lock("lk-2", LockReason::PaymentDispute, "svc-billing", 9_999),
                10,
            )
            .unwrap();
        assert_eq!(store.live_locks("ten_acme", 2_000).len(), 1);
        assert_eq!(store.all_locks("ten_acme").len(), 2, "history is kept");
        assert_eq!(store.purge_expired(2_000), 1);
        assert_eq!(store.all_locks("ten_acme").len(), 1);
        assert_eq!(store.purge_expired(2_000), 0, "purge is idempotent");
    }

    /// Purge is on the PORT, not only on the concrete type: a consumer that
    /// took the seam the crate advertises must be able to reclaim.
    #[test]
    fn purge_is_reachable_through_the_port_and_reclaims_superseded_rows() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(
                lock("lk-1", LockReason::PaymentDispute, "svc-billing", 1_000),
                10,
            )
            .unwrap();
        store
            .acquire(
                lock("lk-1", LockReason::PaymentDispute, "svc-other", 5_000),
                1_000,
            )
            .unwrap();
        assert_eq!(store.superseded_len(), 1);
        let port: &mut dyn LockStore = &mut store;
        assert_eq!(
            port.purge_expired(2_000),
            1,
            "the superseded row is lapsed and reclaimable"
        );
        assert_eq!(store.superseded_len(), 0);
        assert_eq!(store.len(), 1, "the live row stays");
        let port: &mut dyn LockStore = &mut store;
        assert_eq!(port.purge_expired(6_000), 1, "then the row itself lapses");
        assert!(store.is_empty());
    }

    #[test]
    fn live_locks_are_ordered_by_id_regardless_of_insertion_order() {
        let mut store = InMemoryLockStore::new();
        for id in ["lk-c", "lk-a", "lk-b"] {
            store
                .acquire(
                    lock(id, LockReason::PaymentDispute, "svc-billing", 9_999),
                    10,
                )
                .unwrap();
        }
        let ids: Vec<String> = store
            .live_locks("ten_acme", 10)
            .into_iter()
            .map(|held| held.id.0)
            .collect();
        assert_eq!(ids, vec!["lk-a", "lk-b", "lk-c"]);
    }

    /// A per-tenant read must not see, or pay for, another tenant's rows.
    #[test]
    fn a_tenant_read_covers_exactly_that_tenants_key_block() {
        let mut store = InMemoryLockStore::new();
        for tenant in ["ten_a", "ten_acme", "ten_b"] {
            let mut held = lock("lk-1", LockReason::PaymentDispute, "svc-billing", 9_999);
            held.tenant_id = tenant.to_owned();
            store.acquire(held, 10).unwrap();
        }
        assert_eq!(store.len(), 3);
        for tenant in ["ten_a", "ten_acme", "ten_b"] {
            let rows = store.live_locks(tenant, 10);
            assert_eq!(rows.len(), 1, "{tenant} saw a neighbour's rows");
            assert_eq!(rows[0].tenant_id, tenant);
            assert_eq!(store.all_locks(tenant).len(), 1);
        }
        assert!(store.live_locks("ten_", 10).is_empty(), "no prefix bleed");
        assert!(store.live_locks("ten_acmex", 10).is_empty());
    }

    #[test]
    fn a_lapsed_lock_can_be_released_only_as_expired() {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(
                lock("lk-1", LockReason::PaymentDispute, "svc-billing", 1_000),
                10,
            )
            .unwrap();
        assert_eq!(
            store.release("ten_acme", &LockId("lk-1".to_owned()), "svc-billing", 1_000),
            Err(LockKernelError::Expired)
        );
        assert_eq!(store.len(), 1, "the lapsed row stays for the audit trail");
    }
}
