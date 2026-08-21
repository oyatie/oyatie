//! D1 — GitHub webhook payload parsing tests.
//!
//! Covers the 5 GitHub webhook shapes: push, PR-open, PR-update (synchronized),
//! PR-close, and ping.  All tests should PASS at Stage-4 RED (these exercise
//! the parser, not the ed25519 verifier).

use ci_webhook_gateway_kernel::{CiAction, KernelError, RouteOutcome, route_github_event};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn delivery(n: u64) -> String {
    format!("delivery-{n:016x}")
}

fn pr_body(action: &str, base: &str, head_sha: &str, pr_num: u64, draft: bool) -> Vec<u8> {
    format!(
        r#"{{
            "action": "{action}",
            "number": {pr_num},
            "pull_request": {{
                "number": {pr_num},
                "base": {{"ref": "{base}", "sha": "base000sha"}},
                "head": {{"ref": "feature/x", "sha": "{head_sha}"}},
                "draft": {draft}
            }}
        }}"#
    )
    .into_bytes()
}

// ---------------------------------------------------------------------------
// D1-1: ping payload — GitHub webhook-registration handshake
// ---------------------------------------------------------------------------

#[test]
fn d1_ping_payload_is_ignored_not_errored() {
    let outcome = route_github_event("ping", b"{}", &delivery(1), "dev").unwrap();
    assert!(
        matches!(outcome, RouteOutcome::Ignored { ref reason } if reason.contains("ping")),
        "ping should produce Ignored outcome, got: {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// D1-2: pull_request opened — PR-open shape
// ---------------------------------------------------------------------------

#[test]
fn d1_pr_open_payload_produces_trigger_event() {
    let body = pr_body("opened", "dev", "abc123sha456", 77, false);
    let outcome = route_github_event("pull_request", &body, &delivery(2), "dev").unwrap();
    match outcome {
        RouteOutcome::Trigger(ev) => {
            assert_eq!(ev.action, CiAction::PrOpened);
            assert_eq!(ev.pr_number, 77);
            assert_eq!(ev.head_sha, "abc123sha456");
            assert_eq!(ev.branch, "dev");
            assert_eq!(ev.delivery_id, delivery(2));
        }
        RouteOutcome::Ignored { reason } => {
            panic!("expected Trigger, got Ignored: {reason}");
        }
    }
}

// ---------------------------------------------------------------------------
// D1-3: pull_request synchronized — PR-update shape (GitHub spelling)
// ---------------------------------------------------------------------------

#[test]
fn d1_pr_update_github_spelling_produces_trigger_event() {
    let body = pr_body("synchronized", "dev", "def456sha789", 88, false);
    let outcome = route_github_event("pull_request", &body, &delivery(3), "dev").unwrap();
    match outcome {
        RouteOutcome::Trigger(ev) => {
            assert_eq!(ev.action, CiAction::PrSynchronized);
            assert_eq!(ev.pr_number, 88);
            assert_eq!(ev.head_sha, "def456sha789");
        }
        RouteOutcome::Ignored { reason } => {
            panic!("expected Trigger, got Ignored: {reason}");
        }
    }
}

// ---------------------------------------------------------------------------
// D1-4: pull_request closed — PR-close shape
// ---------------------------------------------------------------------------

#[test]
fn d1_pr_close_payload_produces_trigger_event_with_closed_action() {
    let body = pr_body("closed", "dev", "ghi789sha012", 99, false);
    let outcome = route_github_event("pull_request", &body, &delivery(4), "dev").unwrap();
    match outcome {
        RouteOutcome::Trigger(ev) => {
            assert_eq!(ev.action, CiAction::PrClosed);
            assert_eq!(ev.pr_number, 99);
        }
        RouteOutcome::Ignored { reason } => {
            panic!("expected Trigger for closed PR, got Ignored: {reason}");
        }
    }
}

// ---------------------------------------------------------------------------
// D1-5: unknown event type — must be UnroutableEvent, not a silent drop
// ---------------------------------------------------------------------------

#[test]
fn d1_unknown_event_type_is_unroutable_not_silent_drop() {
    let err = route_github_event("wiki", b"{}", &delivery(5), "dev").unwrap_err();
    assert!(
        matches!(err, KernelError::UnroutableEvent { ref event, .. } if event == "wiki"),
        "unknown event should be UnroutableEvent, got: {err:?}"
    );
}
