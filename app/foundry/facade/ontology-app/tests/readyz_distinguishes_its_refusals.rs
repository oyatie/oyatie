//! `/readyz` says WHICH kind of not-ready, because they are different facts.
//!
//! A tenant that is behind and a tenant nobody could read are both not-ready,
//! and Kubernetes reads only the status code — both are 503. The body is for
//! the human reading the event, and answering "lagging" for a tenant whose
//! head could not be read names a state the process never observed. That is
//! the failure this vertical's lag signal was rebuilt to stop, one surface
//! along, so the distinction is asserted rather than described.

mod facade_support;
mod failing_log;
mod out_of_band;
use facade_support as support;

use axum::http::StatusCode;
use support::{Fixture, Session};

/// A head the process cannot read is not evidence of being caught up.
///
/// `sync_status` became fallible in this change, which is a failure mode
/// the readiness predicate could not previously face at all — the head came from an
/// in-memory vector and could not fail. An unpinned `Err` branch on a
/// readiness probe is a process that answers "ready" over a store it cannot
/// read.
#[tokio::test]
async fn an_unreadable_head_is_not_ready() {
    let fixture = Fixture::new("lag-unreadable-readyz");
    let state = failing_log::state_with_a_failing_log(&fixture.config(), "the head is gone");

    assert!(
        !foundry_ontology_app::observation::observe(&state).is_caught_up(),
        "a tenant whose head cannot be read must not report ready"
    );

    let session = Session::from_state(failing_log::state_with_a_failing_log(
        &fixture.config(),
        "the head is gone",
    ));
    let (status, body) = session.get(None, "/readyz").await;
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "the probe must refuse, not answer ready over an unreadable log"
    );
    // WHICH refusal, not merely that it refused. Reporting an unobservable
    // tenant as "lagging" names a state the process never observed.
    assert_eq!(
        body, "unobserved\n",
        "a tenant nobody could read is not a tenant known to be behind"
    );
}

/// The other refusal keeps its own name. A projection genuinely behind its
/// log is "lagging", and an unobservable one is not — collapsing them would
/// put a state the process never saw in front of an operator.
#[tokio::test]
async fn a_lagging_process_says_lagging() {
    let fixture = Fixture::new("readyz-lagging");
    let session = Session::from_state(fixture.state());
    let (ready, body) = session.get(None, "/readyz").await;
    assert_eq!(ready, StatusCode::OK);
    assert_eq!(body, "ready\n");

    out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_lagging_probe");

    let (status, body) = session.get(None, "/readyz").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        body, "lagging\n",
        "this tenant was read, and it is behind: that is a different answer"
    );
}
