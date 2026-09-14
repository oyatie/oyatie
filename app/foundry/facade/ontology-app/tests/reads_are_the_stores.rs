//! Reads answer from the durable projection store. What the store holds is
//! what a reader sees; what it refused is lag on the sync surface.
mod facade_support;
mod failing_store;
use facade_support as support;

use axum::http::StatusCode;
use failing_store::{AlwaysFailingStore, ApplyRefusingStore};
use support::{Fixture, Session, TENANT, WRITE_BODY as WRITE, scrape, value_of};

const READ_ALPHA: &str = "/v1/objects/ent_alpha?revision=1";

fn with_store(fixture: &Fixture, taken: usize, refusals: usize) -> Session {
    let mut state = fixture.state();
    state
        .tenants
        .get_mut(TENANT)
        .expect("served")
        .get_mut()
        .projection_store = Box::new(ApplyRefusingStore::taking_then_refusing(taken, refusals));
    Session::from_state(state)
}

/// A read is the STORE's answer: a write whose mirror the store refused is
/// in the log and the fold, and the reader is told it is unknown while the
/// sync surface shows the lag. An unreadable store refuses the read.
#[tokio::test]
async fn an_object_read_answers_from_the_store_not_the_fold() {
    let fixture = Fixture::new("durable-read-is-the-stores");
    let session = with_store(&fixture, 0, 1);
    let (status, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = session
        .get(
            Some(fixture.operator_token()),
            "/v1/objects/ent_alpha?revision=1",
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    let (_, status) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    assert!(status.contains(r#""lag":1"#), "{status}");

    let mut state = fixture.state();
    state
        .tenants
        .get_mut(TENANT)
        .expect("served")
        .get_mut()
        .projection_store = Box::new(AlwaysFailingStore {
        detail: "the store is gone",
    });
    let session = Session::from_state(state);
    let (status, body) = session
        .get(
            Some(fixture.operator_token()),
            "/v1/objects/ent_alpha?revision=1",
        )
        .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{body}");
    assert!(body.contains(r#""gate":"store""#), "{body}");
    assert_eq!(
        value_of(&scrape(&session).await, "foundry_read_refused_total"),
        1
    );
}

/// The store's LAST TAKEN write is what a reader sees: a later write of the
/// same object the store refused leaves the earlier value served, unmarked,
/// while the sync surface reports the lag.
#[tokio::test]
async fn a_refused_later_write_leaves_the_earlier_value_served() {
    let fixture = Fixture::new("durable-read-stale");
    let session = with_store(&fixture, 1, 1);
    session.post(Some(fixture.operator_token()), WRITE).await;
    let bob = WRITE
        .replace(r#""name":"Ada""#, r#""name":"Bob""#)
        .replace("idem_1", "idem_2");
    let (status, _) = session.post(Some(fixture.operator_token()), &bob).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = session
        .get(Some(fixture.operator_token()), READ_ALPHA)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains(r#""value":"Ada""#) && !body.contains("Bob"),
        "{body}"
    );
    let (_, status) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    assert!(
        status.contains(r#""log_head":2,"applied_ordinal":1,"lag":1"#),
        "{status}"
    );
}

/// The store is one file for every served tenant; the read asks it for the
/// caller's tenant. The second tenant's operator reads its own object and
/// not the first's, and the first's operator does not see the second's.
#[tokio::test]
async fn a_served_second_tenant_reads_its_own_objects_from_the_shared_store() {
    let fixture = Fixture::new("durable-read-second-tenant");
    let session = fixture.both_tenants_session();
    session.post(Some(fixture.operator_token()), WRITE).await;
    let theirs = WRITE
        .replace("ent_alpha", "ent_beta")
        .replace(r#""name":"Ada""#, r#""name":"Zed""#);
    let (status, _) = session.post(Some(fixture.foreign_token()), &theirs).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = session
        .get(
            Some(fixture.foreign_token()),
            "/v1/objects/ent_beta?revision=1",
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains(r#""value":"Zed""#), "{body}");
    let (status, _) = session.get(Some(fixture.foreign_token()), READ_ALPHA).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "the first tenant's object is not theirs"
    );
    let (status, _) = session
        .get(
            Some(fixture.operator_token()),
            "/v1/objects/ent_beta?revision=1",
        )
        .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "the second tenant's object is not ours"
    );
}
