//! Lifecycle-locks kernel — pure logic for lock creation, precedence, release
//! authorization, expiry, and decision explanation. Consumed by usecases before
//! deletion, jurisdiction migration, payment-method removal, KYB/KYC
//! re-verification, and DR promotion.
//!
//! Wave 15-IMPL-truth-up scaffold; full implementation lands in IP-021 execution.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(dead_code)]

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LockId(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockReason {
    PendingDeletionGrace,
    JurisdictionMigration,
    KybReverification,
    DrPromotionWindow,
    PaymentDispute,
    LegalHold,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleLock {
    pub id: LockId,              // data_class: AUDIT
    pub tenant_id: String,       // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub reason: LockReason,      // data_class: AUDIT
    pub holder: String,          // data_class: INTERNAL_ONLY
    pub expires_at_epoch_s: u64, // data_class: AUDIT
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockDecision {
    pub allow: bool,                 // data_class: AUDIT
    pub blocking_locks: Vec<LockId>, // data_class: AUDIT
    pub explanation: String,         // data_class: AUDIT
}

pub fn evaluate(action: &str, locks: &[LifecycleLock]) -> LockDecision {
    let blockers: Vec<LockId> = locks.iter().map(|l| l.id.clone()).collect();
    let allow = blockers.is_empty();
    LockDecision {
        allow,
        blocking_locks: blockers,
        explanation: format!("action={action} stub-decision"),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LockKernelError {
    PrecedenceConflict,
    ReleaseUnauthorized,
    Expired,
}
