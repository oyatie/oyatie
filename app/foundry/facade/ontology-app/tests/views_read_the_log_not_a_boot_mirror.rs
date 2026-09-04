//! History and audit must show what this process just accepted.
//!
//! `TenantState.entries` is built once in `compose` and never appended to:
//! `write_handles` lends out the action log, the denial trail and the
//! projection, never the mirror. So a long-running process serves history and
//! audit from a snapshot of its own boot, and a write it accepted seconds ago
//! is absent from both — with `200 OK`, not an error.
//!
//! The object read is unaffected because it serves the projection, which the
//! write path does update. That asymmetry is why this survived: every read
//! test used the per-request helpers, which recompose `AppState` and re-read
//! the mirror from the log on every call, resetting the state the defect
//! lives in. Only a harness driving several requests through ONE composed
//! process can see it.

mod facade_support;
mod failing_log;
use facade_support as support;

use axum::http::StatusCode;
use support::{Fixture, Session, WRITE_BODY as WRITE, scrape, value_of};

#[tokio::test]
async fn a_live_write_is_visible_to_history_within_the_same_process() {
    let fixture = Fixture::new("views-live-history");
    let session = fixture.session();

    let (accepted, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(accepted, StatusCode::OK);

    let (status, body) = session
        .get(
            Some(fixture.operator_token()),
            "/v1/objects/ent_alpha/history",
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_ne!(
        body, "[]",
        "the process accepted this write and then served an empty history"
    );
    // `HistoryRow` carries no object_ref — the path implies it — so the tie
    // back to the write is the action type it invoked.
    assert!(
        body.contains("aty_record_write"),
        "history must name the action the write invoked: {body}"
    );
}

#[tokio::test]
async fn a_live_write_is_visible_to_the_audit_trail_within_the_same_process() {
    let fixture = Fixture::new("views-live-audit");
    let session = fixture.session();

    let (accepted, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(accepted, StatusCode::OK);

    let (status, body) = session
        .get(Some(fixture.operator_token()), "/v1/audit")
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_ne!(
        body, "[]",
        "the audit trail omitted a write this process accepted"
    );
}

/// Reading the log can fail, and a refusal this surface never counts is a
/// hole in the availability denominator — the defect #2372 spent five
/// revisions closing on every other refusing site.
#[tokio::test]
async fn a_log_that_cannot_be_read_refuses_and_counts() {
    let fixture = Fixture::new("views-log-unreadable");
    let session = Session::from_state(failing_log::state_with_a_failing_log(
        &fixture.config(),
        "the log went away",
    ));

    for path in ["/v1/objects/ent_alpha/history", "/v1/audit"] {
        let before = value_of(&scrape(&session).await, "foundry_read_refused_total");
        let (status, body) = session.get(Some(fixture.operator_token()), path).await;

        assert_eq!(
            status,
            StatusCode::SERVICE_UNAVAILABLE,
            "{path} must refuse rather than serve a view it could not read: {body}"
        );
        assert_eq!(
            body,
            r#"{"gate":"log","cause":"the action log could not be read; this view is unavailable"}"#,
            "{path}: the refusal body is exactly the authored one, with no adapter detail"
        );

        let after = value_of(&scrape(&session).await, "foundry_read_refused_total");
        assert_eq!(
            after,
            before + 1,
            "{path}: the refusal must count exactly once"
        );
    }
}
