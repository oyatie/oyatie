//! D3 — GitHub payload normalisation to [`CiTriggerEvent`] tests.
//!
//! Every supported GitHub webhook shape normalises to a `CiTriggerEvent` with
//! the correct field values.  All 4 tests PASS at Stage-4 RED.

use ci_webhook_gateway_kernel::{CiAction, CiTriggerEvent, RouteOutcome, route_github_event};

// ---------------------------------------------------------------------------
// Helper builders
// ---------------------------------------------------------------------------

fn delivery(n: u64) -> String {
    format!("delivery-norm-{n:016x}")
}

fn pr_payload(
    action: &str,
    base_ref: &str,
    base_sha: &str,
    head_sha: &str,
    pr_num: u64,
) -> Vec<u8> {
    format!(
        r#"{{
            "action": "{action}",
            "number": {pr_num},
            "pull_request": {{
                "number": {pr_num},
                "base": {{"ref": "{base_ref}", "sha": "{base_sha}"}},
                "head": {{"ref": "feature/test", "sha": "{head_sha}"}},
                "draft": false
            }}
        }}"#
    )
    .into_bytes()
}

fn assert_trigger(outcome: RouteOutcome) -> CiTriggerEvent {
    match outcome {
        RouteOutcome::Trigger(ev) => ev,
        RouteOutcome::Ignored { reason } => {
            panic!("expected Trigger, got Ignored: {reason}");
        }
    }
}

// ---------------------------------------------------------------------------
// D3-1: PR-open normalises with correct head_sha, base_sha, pr_number, branch
// ---------------------------------------------------------------------------

#[test]
fn d3_pr_open_normalises_all_fields() {
    let body = pr_payload("opened", "dev", "base111sha", "head222sha", 42);
    let outcome = route_github_event("pull_request", &body, &delivery(1), "dev").unwrap();
    let ev = assert_trigger(outcome);

    assert_eq!(ev.action, CiAction::PrOpened);
    assert_eq!(ev.pr_number, 42);
    assert_eq!(ev.head_sha, "head222sha");
    assert_eq!(ev.base_sha, "base111sha");
    assert_eq!(ev.branch, "dev");
    assert_eq!(ev.delivery_id, delivery(1));
}

// ---------------------------------------------------------------------------
// D3-2: PR-synchronized (GitHub spelling) normalises with PrSynchronized action
// ---------------------------------------------------------------------------

#[test]
fn d3_pr_synchronized_normalises_with_correct_action() {
    let body = pr_payload("synchronized", "dev", "base333sha", "head444sha", 55);
    let outcome = route_github_event("pull_request", &body, &delivery(2), "dev").unwrap();
    let ev = assert_trigger(outcome);

    assert_eq!(ev.action, CiAction::PrSynchronized);
    assert_eq!(ev.pr_number, 55);
    assert_eq!(ev.head_sha, "head444sha");
    assert_eq!(ev.base_sha, "base333sha");
    assert_eq!(ev.branch, "dev");
}

// ---------------------------------------------------------------------------
// D3-3: PR-close normalises with PrClosed action
// ---------------------------------------------------------------------------

#[test]
fn d3_pr_closed_normalises_with_closed_action() {
    let body = pr_payload("closed", "dev", "base555sha", "head666sha", 66);
    let outcome = route_github_event("pull_request", &body, &delivery(3), "dev").unwrap();
    let ev = assert_trigger(outcome);

    assert_eq!(ev.action, CiAction::PrClosed);
    assert_eq!(ev.pr_number, 66);
    assert_eq!(ev.head_sha, "head666sha");
    assert_eq!(ev.base_sha, "base555sha");
}

// ---------------------------------------------------------------------------
// D3-4: ping normalises to Ignored (not a CiTriggerEvent — no Jenkins trigger)
// ---------------------------------------------------------------------------

#[test]
fn d3_ping_normalises_to_ignored_not_trigger() {
    let outcome = route_github_event("ping", b"{}", &delivery(4), "dev").unwrap();
    assert!(
        matches!(outcome, RouteOutcome::Ignored { .. }),
        "ping should produce Ignored (no Jenkins trigger), got: {outcome:?}"
    );
}
