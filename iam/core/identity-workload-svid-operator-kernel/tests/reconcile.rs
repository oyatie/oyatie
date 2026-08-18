use iam_identity_workload_svid_operator_kernel::{
    Action, Clock, DesiredState, ObservedState, reconcile,
};

#[derive(Clone, Copy)]
struct FixedClock {
    now: u64,
}

impl Clock for FixedClock {
    fn now_epoch_seconds(&self) -> u64 {
        self.now
    }
}

fn desired() -> DesiredState {
    DesiredState {
        spiffe_id: "spiffe://oyatie.cell-7/platform/cloud-iam-pdp".to_owned(),
        ttl_secs: 3_600,
        rotation_window_secs: 600,
        secret_name: "oya-cloud-iam-pdp-svid".to_owned(),
        secret_namespace: "cloud-iam".to_owned(),
    }
}

#[test]
fn issues_when_no_secret_is_present() {
    let want = desired();
    let action = reconcile(&ObservedState::absent(), &want, &FixedClock { now: 1_000 });
    assert_eq!(
        action,
        Action::Issue {
            desired: want,
            requested_at_epoch_seconds: 1_000,
        }
    );
}

#[test]
fn noops_when_leaf_is_comfortably_fresh() {
    let want = desired();
    // Leaf expires at 5_000; now=1_000 → 4_000s remaining, far above the 600s window.
    let action = reconcile(
        &ObservedState::present(5_000),
        &want,
        &FixedClock { now: 1_000 },
    );
    assert_eq!(action, Action::Noop);
}

#[test]
fn rotates_when_leaf_is_within_the_rotation_window() {
    let want = desired();
    // Leaf expires at 1_500; now=1_000 → 500s remaining, at/below the 600s window.
    let action = reconcile(
        &ObservedState::present(1_500),
        &want,
        &FixedClock { now: 1_000 },
    );
    assert_eq!(
        action,
        Action::Rotate {
            desired: want,
            observed_leaf_not_after_epoch_seconds: 1_500,
            requested_at_epoch_seconds: 1_000,
        }
    );
}

#[test]
fn rotates_exactly_at_the_window_boundary() {
    let want = desired();
    // Leaf expires at 1_600; now=1_000 → exactly 600s remaining == window ⇒ rotate.
    let action = reconcile(
        &ObservedState::present(1_600),
        &want,
        &FixedClock { now: 1_000 },
    );
    assert!(matches!(action, Action::Rotate { .. }));
}

#[test]
fn noops_one_second_above_the_window_boundary() {
    let want = desired();
    // Leaf expires at 1_601; now=1_000 → 601s remaining, one above the window ⇒ noop.
    let action = reconcile(
        &ObservedState::present(1_601),
        &want,
        &FixedClock { now: 1_000 },
    );
    assert_eq!(action, Action::Noop);
}

#[test]
fn rotates_an_already_expired_leaf_without_underflow() {
    let want = desired();
    // Leaf expired at 500; now=1_000 → saturating remaining = 0 ⇒ rotate, no panic.
    let action = reconcile(
        &ObservedState::present(500),
        &want,
        &FixedClock { now: 1_000 },
    );
    assert_eq!(
        action,
        Action::Rotate {
            desired: want,
            observed_leaf_not_after_epoch_seconds: 500,
            requested_at_epoch_seconds: 1_000,
        }
    );
}

#[test]
fn applying_issue_then_observing_the_fresh_leaf_is_idempotent() {
    let want = desired();
    // Cold start → Issue.
    let issue = reconcile(&ObservedState::absent(), &want, &FixedClock { now: 1_000 });
    assert!(matches!(issue, Action::Issue { .. }));
    // The adapter would mint a leaf valid for ttl_secs (notAfter = 1_000 + 3_600).
    let observed = ObservedState::present(1_000 + want.ttl_secs);
    // Re-observing the fresh leaf at the same instant must converge to Noop.
    assert_eq!(
        reconcile(&observed, &want, &FixedClock { now: 1_000 }),
        Action::Noop
    );
}
