//! Cedar-gated quorum crypto-shred lifecycle (story G002; AWS KMS
//! ScheduleKeyDeletion precedent).
//!
//! Ladder rungs (AMENDMENT 7): unit + RED/GREEN fixture pair for the
//! fail-closed gates (PDP deny, quorum short, window not elapsed). The PDP
//! arrives through `ShredAuthorizationPort` — these tests are the contract
//! the G04 embedded Cedar PDP adapter must satisfy.

use std::num::NonZeroU32;

use secrets_kms_enclave::{
    DekId, KekId, KekMaterial, KekVersion, KekVersionChain, MIN_WAITING_WINDOW_SECONDS,
    QuorumPolicy, ScheduledKeyDeletion, ShredAction, ShredAuthorizationPort,
    ShredAuthorizationRequest, ShredDecision, ShredDecisionEvidence, ShredError,
};

/// Fixed-answer PDP fixture; records nothing, answers everything the same.
struct FixedPdp {
    permit: bool,
}

impl ShredAuthorizationPort for FixedPdp {
    fn authorize(&self, request: &ShredAuthorizationRequest) -> ShredDecision {
        let evidence = ShredDecisionEvidence {
            decision_id: format!("dec-{:?}-{}", request.action, request.actor),
            policy_version: "policy-v7".to_owned(),
        };
        if self.permit {
            ShredDecision::Permit(evidence)
        } else {
            ShredDecision::Deny(evidence)
        }
    }
}

fn chain_with_history(kek_id: &str) -> (KekVersionChain, Vec<u8>, secrets_kms_enclave::WrappedDek) {
    let mut chain = KekVersionChain::new(
        KekMaterial::generate(KekId::new(kek_id).expect("kek id"), KekVersion::INITIAL)
            .expect("kek"),
    );
    let (dek, wrapped) = chain
        .generate_dek(DekId::new("dek/obj_1").expect("dek id"))
        .expect("dek");
    let blob = dek.seal(b"ctx", b"tenant payload").expect("seal");
    chain.rotate().expect("rotate");
    (chain, blob, wrapped)
}

fn quorum(n: u32) -> QuorumPolicy {
    QuorumPolicy {
        required_approvals: NonZeroU32::new(n).expect("quorum"),
    }
}

const T0: u64 = 1_750_000_000;

#[test]
fn green_full_lifecycle_schedule_approve_wait_execute() {
    let (chain, blob, wrapped) = chain_with_history("kek/ten_alpha");
    let pdp = FixedPdp { permit: true };

    let mut scheduled = ScheduledKeyDeletion::schedule(
        chain,
        "ten_alpha".to_owned(),
        "requester@ops".to_owned(),
        &pdp,
        quorum(2),
        MIN_WAITING_WINDOW_SECONDS,
        T0,
    )
    .expect("schedule under permit");

    // Decrypt-only availability during the window: reads still serve.
    let dek = scheduled
        .pending_chain()
        .unwrap_dek(&wrapped)
        .expect("window read");
    assert_eq!(
        dek.open(b"ctx", &blob).expect("open").as_slice(),
        b"tenant payload"
    );

    assert_eq!(
        scheduled
            .approve("approver-a@ops".to_owned())
            .expect("first"),
        1
    );
    assert_eq!(
        scheduled
            .approve("approver-b@ops".to_owned())
            .expect("second"),
        2
    );

    let executed_at = T0 + MIN_WAITING_WINDOW_SECONDS;
    let proof = scheduled
        .execute(executed_at)
        .expect("execute after window");

    assert_eq!(proof.kek_id.value(), "kek/ten_alpha");
    assert_eq!(proof.tenant_id, "ten_alpha");
    assert_eq!(proof.versions_destroyed, 2); // v1 retired + v2 current
    assert_eq!(proof.approvers.len(), 2);
    assert_eq!(proof.schedule_decision.policy_version, "policy-v7");

    let destruction = proof.to_destruction_request("proof/audit-001".to_owned());
    assert_eq!(destruction.key_id, "kek/ten_alpha");
    assert_eq!(destruction.requested_at_epoch_seconds, T0);
    assert_eq!(destruction.completed_at_epoch_seconds, executed_at);
    // `scheduled` and the chain are consumed — use-after-shred does not
    // compile, which IS the assertion.
}

#[test]
fn red_pdp_deny_fails_closed_and_returns_custody() {
    let (chain, blob, wrapped) = chain_with_history("kek/ten_alpha");
    let pdp = FixedPdp { permit: false };

    let (returned, err) = ScheduledKeyDeletion::schedule(
        chain,
        "ten_alpha".to_owned(),
        "requester@ops".to_owned(),
        &pdp,
        quorum(2),
        MIN_WAITING_WINDOW_SECONDS,
        T0,
    )
    .expect_err("deny must fail closed");

    assert!(matches!(err, ShredError::NotPermitted { .. }));
    // Custody intact: chain still decrypts and still encrypts forward.
    let dek = returned.unwrap_dek(&wrapped).expect("custody preserved");
    assert_eq!(
        dek.open(b"ctx", &blob).expect("open").as_slice(),
        b"tenant payload"
    );
    assert!(
        returned
            .generate_dek(DekId::new("dek/obj_2").unwrap())
            .is_ok()
    );
}

#[test]
fn red_quorum_short_blocks_execution() {
    let (chain, _, _) = chain_with_history("kek/ten_alpha");
    let pdp = FixedPdp { permit: true };
    let mut scheduled = ScheduledKeyDeletion::schedule(
        chain,
        "ten_alpha".to_owned(),
        "requester@ops".to_owned(),
        &pdp,
        quorum(2),
        MIN_WAITING_WINDOW_SECONDS,
        T0,
    )
    .expect("schedule");
    scheduled
        .approve("approver-a@ops".to_owned())
        .expect("one approval");

    let (returned, err) = scheduled
        .execute(T0 + MIN_WAITING_WINDOW_SECONDS + 1)
        .expect_err("1/2 approvals must not shred");
    assert_eq!(err, ShredError::QuorumNotReached { have: 1, need: 2 });
    // Custody (decrypt-only) survives the refused execution.
    assert_eq!(returned.pending_chain().kek_id().value(), "kek/ten_alpha");
}

#[test]
fn red_window_not_elapsed_blocks_execution() {
    let (chain, _, _) = chain_with_history("kek/ten_alpha");
    let pdp = FixedPdp { permit: true };
    let mut scheduled = ScheduledKeyDeletion::schedule(
        chain,
        "ten_alpha".to_owned(),
        "requester@ops".to_owned(),
        &pdp,
        quorum(1),
        MIN_WAITING_WINDOW_SECONDS,
        T0,
    )
    .expect("schedule");
    scheduled
        .approve("approver-a@ops".to_owned())
        .expect("approval");

    let (_, err) = scheduled
        .execute(T0 + MIN_WAITING_WINDOW_SECONDS - 1)
        .expect_err("window must gate execution");
    assert_eq!(
        err,
        ShredError::WindowNotElapsed {
            earliest_at_epoch_seconds: T0 + MIN_WAITING_WINDOW_SECONDS
        }
    );
}

#[test]
fn approval_rules_distinct_and_no_self_approval() {
    let (chain, _, _) = chain_with_history("kek/ten_alpha");
    let pdp = FixedPdp { permit: true };
    let mut scheduled = ScheduledKeyDeletion::schedule(
        chain,
        "ten_alpha".to_owned(),
        "requester@ops".to_owned(),
        &pdp,
        quorum(2),
        MIN_WAITING_WINDOW_SECONDS,
        T0,
    )
    .expect("schedule");

    assert_eq!(
        scheduled.approve("requester@ops".to_owned()),
        Err(ShredError::RequesterCannotApprove)
    );
    scheduled
        .approve("approver-a@ops".to_owned())
        .expect("first");
    assert_eq!(
        scheduled.approve("approver-a@ops".to_owned()),
        Err(ShredError::DuplicateApprover)
    );
}

#[test]
fn cancel_restores_encrypt_capable_custody_with_attribution() {
    let (chain, blob, wrapped) = chain_with_history("kek/ten_alpha");
    let pdp = FixedPdp { permit: true };
    let mut scheduled = ScheduledKeyDeletion::schedule(
        chain,
        "ten_alpha".to_owned(),
        "requester@ops".to_owned(),
        &pdp,
        quorum(2),
        MIN_WAITING_WINDOW_SECONDS,
        T0,
    )
    .expect("schedule");
    scheduled
        .approve("approver-a@ops".to_owned())
        .expect("approval before cancel");

    let (restored, evidence) = scheduled
        .cancel("operator@ops".to_owned(), &pdp, T0 + 60)
        .expect("permitted cancel");

    // Cancellation is fully attributable: actor, both policy decisions,
    // accumulated approvals, and the schedule/cancel timeline.
    assert_eq!(evidence.kek_id.value(), "kek/ten_alpha");
    assert_eq!(evidence.tenant_id, "ten_alpha");
    assert_eq!(evidence.cancelled_by, "operator@ops");
    assert_eq!(evidence.cancel_decision.policy_version, "policy-v7");
    assert!(evidence.cancel_decision.decision_id.contains("Cancel"));
    assert!(evidence.schedule_decision.decision_id.contains("Schedule"));
    assert_eq!(
        evidence.approvals_at_cancel,
        vec!["approver-a@ops".to_owned()]
    );
    assert_eq!(evidence.scheduled_at_epoch_seconds, T0);
    assert_eq!(evidence.cancelled_at_epoch_seconds, T0 + 60);

    let dek = restored.unwrap_dek(&wrapped).expect("reads restored");
    assert_eq!(
        dek.open(b"ctx", &blob).expect("open").as_slice(),
        b"tenant payload"
    );
    assert!(
        restored
            .generate_dek(DekId::new("dek/obj_3").unwrap())
            .is_ok()
    );
}

#[test]
fn property_sweep_quorum_arithmetic() {
    // Property: for every (required M, granted N) pair, execute() succeeds
    // iff N >= M once the window elapsed — never on any other arithmetic.
    for required in 1u32..=5 {
        for granted in 0u32..=6 {
            let (chain, _, _) = chain_with_history("kek/ten_alpha");
            let pdp = FixedPdp { permit: true };
            let mut scheduled = ScheduledKeyDeletion::schedule(
                chain,
                "ten_alpha".to_owned(),
                "requester@ops".to_owned(),
                &pdp,
                quorum(required),
                MIN_WAITING_WINDOW_SECONDS,
                T0,
            )
            .expect("schedule");
            for approver in 0..granted {
                scheduled
                    .approve(format!("approver-{approver}@ops"))
                    .expect("distinct approver");
            }
            let outcome = scheduled.execute(T0 + MIN_WAITING_WINDOW_SECONDS);
            if granted >= required {
                let proof = outcome.expect("N >= M must shred");
                assert_eq!(
                    proof.approvers.len(),
                    granted as usize,
                    "M={required} N={granted}"
                );
            } else {
                let (_, err) = outcome.expect_err("N < M must fail closed");
                assert_eq!(
                    err,
                    ShredError::QuorumNotReached {
                        have: granted,
                        need: required
                    },
                    "M={required} N={granted}"
                );
            }
        }
    }
}

#[test]
fn cancel_denied_keeps_pending_custody() {
    let (chain, _, wrapped) = chain_with_history("kek/ten_alpha");
    let permit_pdp = FixedPdp { permit: true };
    let deny_pdp = FixedPdp { permit: false };
    let scheduled = ScheduledKeyDeletion::schedule(
        chain,
        "ten_alpha".to_owned(),
        "requester@ops".to_owned(),
        &permit_pdp,
        quorum(1),
        MIN_WAITING_WINDOW_SECONDS,
        T0,
    )
    .expect("schedule");

    let (still_pending, err) = scheduled
        .cancel("operator@ops".to_owned(), &deny_pdp, T0 + 60)
        .expect_err("denied cancel fails closed");
    assert!(matches!(err, ShredError::NotPermitted { .. }));
    assert!(still_pending.pending_chain().unwrap_dek(&wrapped).is_ok());
}

#[test]
fn window_floor_enforced() {
    let (chain, _, _) = chain_with_history("kek/ten_alpha");
    let pdp = FixedPdp { permit: true };
    let (_, err) = ScheduledKeyDeletion::schedule(
        chain,
        "ten_alpha".to_owned(),
        "requester@ops".to_owned(),
        &pdp,
        quorum(1),
        MIN_WAITING_WINDOW_SECONDS - 1,
        T0,
    )
    .expect_err("sub-floor window rejected");
    assert_eq!(
        err,
        ShredError::WindowTooShort {
            floor_seconds: MIN_WAITING_WINDOW_SECONDS
        }
    );
}
