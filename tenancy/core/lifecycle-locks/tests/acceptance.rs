//! IP-021 §E acceptance scenarios end to end: delete-lock over a weaker hold,
//! legal-hold release quorum, expired soft-lock, DR-promotion lock survival,
//! and the DSR "request accepted, deletion delayed" example from IP-009.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use tenancy_lifecycle_locks::precedence::{acquisition_conflict, check_acquisition};
use tenancy_lifecycle_locks::release::{
    ReleaseRole, holder_release_permitted, quorum_shortfall, required_roles,
};
use tenancy_lifecycle_locks::{
    InMemoryLockStore, LifecycleAction, LifecycleLock, LockId, LockKernelError, LockReason,
    LockStore, ReleaseApproval, evaluate, evaluate_at,
};

const TENANT: &str = "ten_acme";

fn lock(id: &str, reason: LockReason, holder: &str, expires: u64) -> LifecycleLock {
    LifecycleLock::new(
        LockId(id.to_owned()),
        TENANT.to_owned(),
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
fn delete_lock_outranks_the_weaker_hold_in_the_explanation() {
    // Both block deletion; the operator must be shown the stronger one first.
    let locks = [
        lock("lk-dr", LockReason::DrPromotionWindow, "svc-dr", 9_999),
        lock(
            "lk-grace",
            LockReason::PendingDeletionGrace,
            "svc-dsr",
            9_999,
        ),
    ];
    let decision = evaluate_at(LifecycleAction::DeleteTenant, &locks, 10);
    assert!(!decision.allow);
    assert_eq!(
        decision.governing_lock,
        Some(LockId("lk-grace".to_owned())),
        "pending-deletion-grace (4) outranks dr-promotion-window (1)"
    );
    assert_eq!(
        decision.blocking_locks,
        vec![LockId("lk-grace".to_owned()), LockId("lk-dr".to_owned())]
    );
}

#[test]
fn legal_hold_release_needs_the_full_quorum_and_nothing_less() {
    let mut store = InMemoryLockStore::new();
    store
        .acquire(
            lock("lk-hold", LockReason::LegalHold, "svc-legal", 9_999),
            10,
        )
        .unwrap();
    assert!(!holder_release_permitted(LockReason::LegalHold));
    assert_eq!(
        required_roles(LockReason::LegalHold),
        &[ReleaseRole::DataProtectionOfficer, ReleaseRole::Counsel]
    );
    let id = LockId("lk-hold".to_owned());
    assert_eq!(
        store.release(TENANT, &id, "svc-legal", 10),
        Err(LockKernelError::ReleaseRequiresQuorum),
        "the holder is told to convene the quorum, not that it used the wrong identity"
    );
    let counsel_only = [approval("cleo", ReleaseRole::Counsel)];
    assert_eq!(
        store.release_with_quorum(TENANT, &id, &counsel_only, 10),
        Err(LockKernelError::QuorumNotMet)
    );
    assert_eq!(
        quorum_shortfall(LockReason::LegalHold, &counsel_only),
        Some(ReleaseRole::DataProtectionOfficer),
        "the empty seat is nameable even though the error is not"
    );
    store
        .release_with_quorum(
            TENANT,
            &id,
            &[
                approval("dana", ReleaseRole::DataProtectionOfficer),
                approval("cleo", ReleaseRole::Counsel),
            ],
            10,
        )
        .unwrap();
    assert!(
        store
            .decide(TENANT, LifecycleAction::DeleteTenant, 10)
            .allow
    );
}

/// A lawful two-principal quorum must not be refused because of the order the
/// signatures were collected in. `dana` is cross-listed as DPO and counsel;
/// `cleo` is a second DPO, so `cleo` + `dana` covers both seats.
#[test]
fn a_legal_hold_yields_to_a_lawful_quorum_whatever_order_it_arrived_in() {
    let collected = [
        vec![
            approval("dana", ReleaseRole::DataProtectionOfficer),
            approval("cleo", ReleaseRole::DataProtectionOfficer),
            approval("dana", ReleaseRole::Counsel),
        ],
        vec![
            approval("cleo", ReleaseRole::DataProtectionOfficer),
            approval("dana", ReleaseRole::DataProtectionOfficer),
            approval("dana", ReleaseRole::Counsel),
        ],
    ];
    for approvals in collected {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(
                lock("lk-hold", LockReason::LegalHold, "svc-legal", 9_999),
                10,
            )
            .unwrap();
        store
            .release_with_quorum(TENANT, &LockId("lk-hold".to_owned()), &approvals, 10)
            .unwrap();
        assert!(
            store
                .decide(TENANT, LifecycleAction::DeleteTenant, 10)
                .allow
        );
    }
}

#[test]
fn an_expired_soft_lock_stops_gating_at_its_own_instant() {
    let mut store = InMemoryLockStore::new();
    store
        .acquire(
            lock("lk-kyb", LockReason::KybReverification, "svc-kyb", 1_000),
            10,
        )
        .unwrap();
    let gated = LifecycleAction::RemovePaymentCredential;
    assert!(
        !store.decide(TENANT, gated, 999).allow,
        "live one second before"
    );
    assert!(
        store.decide(TENANT, gated, 1_000).allow,
        "lapsed exactly at the expiry instant"
    );
    assert!(store.decide(TENANT, gated, 1_001).allow, "lapsed after");
}

/// IP-021 §D.4's manual soft lock end to end: an operator places it, it stops
/// the destructive changes, and a tenant admin - and only a tenant admin -
/// lifts it through the quorum path.
#[test]
fn a_manual_soft_lock_is_placed_by_ops_and_lifted_by_a_tenant_admin() {
    let mut store = InMemoryLockStore::new();
    store
        .acquire(
            lock("lk-soft", LockReason::ManualSoftLock, "svc-ops", 9_999),
            10,
        )
        .unwrap();
    assert!(
        !store
            .decide(TENANT, LifecycleAction::DeleteTenant, 10)
            .allow
    );
    assert!(
        !store
            .decide(TENANT, LifecycleAction::ChangeJurisdiction, 10)
            .allow
    );
    assert!(
        store
            .decide(TENANT, LifecycleAction::PromoteDrPair, 10)
            .allow,
        "a protection lock must not stop a failover"
    );
    let id = LockId("lk-soft".to_owned());
    assert_eq!(
        store.release_with_quorum(
            TENANT,
            &id,
            &[approval("otto", ReleaseRole::OpsSecurity)],
            10
        ),
        Err(LockKernelError::QuorumNotMet)
    );
    store
        .release_with_quorum(
            TENANT,
            &id,
            &[approval("tina", ReleaseRole::TenantAdmin)],
            10,
        )
        .unwrap();
    assert!(
        store
            .decide(TENANT, LifecycleAction::DeleteTenant, 10)
            .allow
    );
}

#[test]
fn locks_survive_dr_pair_promotion_because_state_carries_no_cell() {
    // The kernel record names no cell or region (IP-021 D.5), so promoting the
    // DR pair cannot drop a hold: the same store answers the same way.
    let mut store = InMemoryLockStore::new();
    store
        .acquire(
            lock("lk-hold", LockReason::LegalHold, "svc-legal", 9_999),
            10,
        )
        .unwrap();
    let before = store.decide(TENANT, LifecycleAction::DeleteTenant, 10);

    // Promotion itself is gated by the same hold ...
    assert!(
        !store
            .decide(TENANT, LifecycleAction::PromoteDrPair, 10)
            .allow
    );
    // ... and once counsel lifts the hold, promotion runs under its own window.
    store
        .release_with_quorum(
            TENANT,
            &LockId("lk-hold".to_owned()),
            &[
                approval("dana", ReleaseRole::DataProtectionOfficer),
                approval("cleo", ReleaseRole::Counsel),
            ],
            10,
        )
        .unwrap();
    store
        .acquire(
            lock("lk-dr", LockReason::DrPromotionWindow, "svc-dr", 9_999),
            11,
        )
        .unwrap();
    // A dispute hold placed before promotion is still answering after it.
    store
        .acquire(
            lock("lk-pay", LockReason::PaymentDispute, "svc-billing", 9_999),
            12,
        )
        .unwrap();
    let after = store.decide(TENANT, LifecycleAction::DeleteTenant, 13);
    assert!(!after.allow);
    assert!(!before.allow);
    assert_eq!(
        after.blocking_locks,
        vec![LockId("lk-pay".to_owned()), LockId("lk-dr".to_owned())],
        "both holds survived the promotion window"
    );
}

#[test]
fn dsr_request_is_accepted_while_deletion_stays_delayed() {
    // IP-009 example: the DSR grace window and a statutory retention basis both
    // stand; deletion is refused with a sentence an operator can act on, while
    // the unrelated payment-credential removal still proceeds.
    let mut store = InMemoryLockStore::new();
    store
        .acquire(
            lock(
                "lk-grace",
                LockReason::PendingDeletionGrace,
                "svc-dsr",
                5_000,
            ),
            10,
        )
        .unwrap();
    let decision = store.decide(TENANT, LifecycleAction::DeleteTenant, 10);
    assert!(!decision.allow);
    assert_eq!(
        decision.explanation,
        format!(
            "action=delete-tenant denied: 1 of 1 lock(s) block it: lk-grace \
             (pending-deletion-grace: {})",
            LockReason::PendingDeletionGrace.rationale()
        )
    );
    assert!(
        store
            .decide(TENANT, LifecycleAction::RemovePaymentCredential, 10)
            .allow,
        "the grace window has no opinion about payment credentials"
    );
    // Once the window lapses, deletion is allowed with no code change.
    assert!(
        store
            .decide(TENANT, LifecycleAction::DeleteTenant, 5_000)
            .allow
    );
}

/// IP-021 §D.7's headline scenario in full: the tenant ALREADY carries a
/// statutory retention basis when the erasure request lands. The grace window
/// must be recordable anyway - a DSR clock the kernel refuses to hold is the
/// "every usecase invents its own do-not-delete-yet flag" failure IP-021 §A
/// exists to prevent - and deletion must stay refused, naming the hold.
#[test]
fn a_dsr_grace_window_is_recordable_under_an_existing_retention_basis() {
    // Both are retention bases that outrank the grace timer, so each must be
    // the lock that GOVERNS the refusal an operator is shown.
    for basis in [LockReason::LegalHold, LockReason::PaymentDispute] {
        for basis_first in [true, false] {
            let mut store = InMemoryLockStore::new();
            let hold = lock("lk-basis", basis, "svc-basis", 9_999);
            let grace = lock(
                "lk-grace",
                LockReason::PendingDeletionGrace,
                "svc-dsr",
                5_000,
            );
            if basis_first {
                store.acquire(hold, 10).unwrap();
                store.acquire(grace, 10).unwrap();
            } else {
                store.acquire(grace, 10).unwrap();
                store.acquire(hold, 10).unwrap();
            }
            assert_eq!(
                store.live_locks(TENANT, 10).len(),
                2,
                "{} + grace was unreachable with basis_first={basis_first}",
                basis.as_slug()
            );
            let decision = store.decide(TENANT, LifecycleAction::DeleteTenant, 10);
            assert!(!decision.allow, "deletion must stay delayed");
            assert_eq!(
                decision.governing_lock,
                Some(LockId("lk-basis".to_owned())),
                "the retention basis governs the refusal, not the grace timer"
            );
            assert!(decision.explanation.contains(basis.as_slug()));
            assert!(decision.explanation.contains("pending-deletion-grace"));

            // The grace window lapses first; the retention basis still holds.
            assert!(
                !store
                    .decide(TENANT, LifecycleAction::DeleteTenant, 5_000)
                    .allow,
                "{} must still delay deletion after the DSR clock runs out",
                basis.as_slug()
            );
        }
    }
}

/// A window whose action a standing lock forbids is still refused - that is the
/// half of the acquisition rule the grace-window fix must not have removed.
#[test]
fn a_migration_window_cannot_be_opened_under_a_dispute_hold() {
    let mut store = InMemoryLockStore::new();
    store
        .acquire(
            lock("lk-pay", LockReason::PaymentDispute, "svc-billing", 9_999),
            10,
        )
        .unwrap();
    // payment-dispute does not block change-jurisdiction, so it alone is not
    // enough; the manual soft lock the operator adds is.
    store
        .acquire(
            lock("lk-soft", LockReason::ManualSoftLock, "svc-ops", 9_999),
            10,
        )
        .unwrap();
    let candidate = lock(
        "lk-move",
        LockReason::JurisdictionMigration,
        "svc-residency",
        9_999,
    );
    let standing = store.all_locks(TENANT);
    assert_eq!(
        acquisition_conflict(&candidate, &standing, 10).map(|held| held.id.clone()),
        Some(LockId("lk-soft".to_owned())),
        "the contradicting lock is nameable for the operator"
    );
    assert_eq!(
        check_acquisition(&candidate, &standing, 10),
        Err(LockKernelError::PrecedenceConflict)
    );
    assert_eq!(
        store.acquire(candidate, 10),
        Err(LockKernelError::PrecedenceConflict)
    );
}

#[test]
fn the_matrix_is_not_uniform_over_reasons_or_actions() {
    // Guards against a regression to the scaffold behaviour, where every lock
    // blocked every action.
    for action in LifecycleAction::ALL {
        let blocked_by: Vec<LockReason> = LockReason::ALL
            .into_iter()
            .filter(|reason| reason.blocks(action))
            .collect();
        assert!(
            !blocked_by.is_empty(),
            "{} is gated by nothing at all",
            action.as_slug()
        );
        if action != LifecycleAction::DeleteTenant {
            assert!(
                blocked_by.len() < LockReason::ALL.len(),
                "{} is blocked by every reason, which is the stub bug",
                action.as_slug()
            );
        }
    }
}

#[test]
fn the_instant_free_entry_point_agrees_with_the_dated_one_for_live_locks() {
    let locks = [
        lock(
            "lk-pay",
            LockReason::PaymentDispute,
            "svc-billing",
            u64::MAX,
        ),
        lock("lk-kyb", LockReason::KybReverification, "svc-kyb", u64::MAX),
    ];
    for action in LifecycleAction::ALL {
        let legacy = evaluate(action.as_slug(), &locks);
        let dated = evaluate_at(action, &locks, 0);
        assert_eq!(legacy, dated, "{} disagreed", action.as_slug());
    }
}

#[test]
fn every_reason_and_action_pair_produces_a_decision_naming_its_locks() {
    for reason in LockReason::ALL {
        let locks = [lock("lk-1", reason, "svc", 9_999)];
        for action in LifecycleAction::ALL {
            let decision = evaluate_at(action, &locks, 10);
            assert_eq!(decision.allow, !reason.blocks(action));
            assert_eq!(decision.allow, decision.governing_lock.is_none());
            assert_eq!(decision.allow, decision.blocking_locks.is_empty());
            assert!(decision.explanation.contains(action.as_slug()));
            if !decision.allow {
                assert!(
                    decision.explanation.contains(reason.as_slug())
                        && decision.explanation.contains("lk-1"),
                    "explanation must name which lock and why: {}",
                    decision.explanation
                );
            }
        }
    }
}

#[test]
fn a_lock_is_never_silently_stolen_across_the_whole_reason_set() {
    for reason in LockReason::ALL {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(lock("lk-1", reason, "owner", 9_999), 10)
            .unwrap();
        assert_eq!(
            store.acquire(lock("lk-1", reason, "thief", 9_999), 10),
            Err(LockKernelError::AlreadyHeld),
            "{} was overwritable",
            reason.as_slug()
        );
        let expected = if holder_release_permitted(reason) {
            LockKernelError::ReleaseUnauthorized
        } else {
            LockKernelError::ReleaseRequiresQuorum
        };
        assert_eq!(
            store.release(TENANT, &LockId("lk-1".to_owned()), "thief", 10),
            Err(expected),
            "{} was releasable by a non-holder",
            reason.as_slug()
        );
        assert_eq!(store.len(), 1);
    }
}

/// Whatever a reason's release rule is, SOME path must be able to lift it -
/// otherwise a lock can only ever be waited out.
#[test]
fn every_reason_has_a_reachable_release_path() {
    for reason in LockReason::ALL {
        let mut store = InMemoryLockStore::new();
        store
            .acquire(lock("lk-1", reason, "owner", 9_999), 10)
            .unwrap();
        let approvals: Vec<ReleaseApproval> = required_roles(reason)
            .iter()
            .enumerate()
            .map(|(index, role)| approval(&format!("approver-{index}"), *role))
            .collect();
        store
            .release_with_quorum(TENANT, &LockId("lk-1".to_owned()), &approvals, 10)
            .unwrap_or_else(|error| {
                panic!("{} has no reachable quorum: {error}", reason.as_slug())
            });
        assert!(store.is_empty());
    }
}
