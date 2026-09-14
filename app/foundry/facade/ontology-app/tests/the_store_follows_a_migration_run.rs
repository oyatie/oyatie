mod facade_support;
mod failing_store;
mod migration_support;
use facade_support as support;

use axum::http::StatusCode;
use failing_store::ApplyRefusingStore;
use foundry_projection_draft::ProjectionStore;
use foundry_projection_sqlite_draft::SqliteProjectionStore;
use migration_support::{
    plan_for, run, state_with_two_revisions, state_with_two_revisions_for, write_owing,
    write_settled,
};
use support::{Fixture, Session, TENANT};

const CAUGHT_UP_AT_TWO: &str = r#""log_head":2,"applied_ordinal":2,"lag":0"#;

fn store_head(fixture: &Fixture) -> u64 {
    SqliteProjectionStore::open(&fixture.projection_store_path())
        .expect("the store opens")
        .applied_head(TENANT)
        .expect("head reads")
}

/// A wedged store is repaired by the run handler's catch-up.
#[tokio::test]
async fn a_migration_run_repairs_a_wedged_store() {
    let fixture = Fixture::new("durable-run-repairs");
    let mut state = state_with_two_revisions(&fixture.config());
    state
        .tenants
        .get_mut(TENANT)
        .expect("served")
        .get_mut()
        .projection_store = Box::new(ApplyRefusingStore::refusing_the_first(1));
    let session = Session::from_state(state);
    write_owing(
        &session,
        Some(fixture.operator_token()),
        "ent_alpha",
        "idem_1",
    )
    .await;
    let (_, status) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    assert!(
        status.contains(r#""log_head":1,"applied_ordinal":0,"lag":1"#),
        "{status}"
    );
    let (status, body) = run(&session, Some(fixture.operator_token()), &plan_for(TENANT)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, status) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    assert!(
        status.contains(r#""log_head":2,"applied_ordinal":2,"lag":0"#),
        "{status}"
    );
}

/// A migration run is many writes; the store is caught up once the run
/// returns, before the tenant lock is released.
#[tokio::test]
async fn a_migration_run_leaves_the_store_at_the_log_head() {
    let fixture = Fixture::new("durable-migration");
    let session = Session::from_state(state_with_two_revisions(&fixture.config()));
    write_owing(
        &session,
        Some(fixture.operator_token()),
        "ent_alpha",
        "idem_1",
    )
    .await;
    let (status, body) = run(&session, Some(fixture.operator_token()), &plan_for(TENANT)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains(r#""upcast":1"#), "{body}");
    let head = fixture.log_head();
    assert_eq!(head, 2, "the write and its upcast");
    assert_eq!(store_head(&fixture), head);

    let (_, status) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    assert!(
        status.contains(r#""log_head":2,"applied_ordinal":2,"lag":0"#),
        "{status}"
    );
}

/// The run handler's catch-up is keyed by the caller's tenant: another
/// tenant's run catches up ITS store, over its own log.
#[tokio::test]
async fn another_tenants_run_catches_up_its_own_store() {
    let fixture = Fixture::new("durable-run-other-tenant");
    let mut config = fixture.config();
    config.tenants.push("ten_other".into());
    let state = state_with_two_revisions_for(&config, &["ten_acme", "ten_other"]);
    let session = Session::from_state(state);
    write_owing(
        &session,
        Some(fixture.foreign_token()),
        "ent_gamma",
        "idem_g",
    )
    .await;
    let (status, body) = run(
        &session,
        Some(fixture.foreign_token()),
        &plan_for("ten_other"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains(r#""upcast":1"#), "{body}");
    let (_, status) = session.get(Some(fixture.foreign_token()), "/statusz").await;
    assert!(status.contains(CAUGHT_UP_AT_TWO), "{status}");
}

/// A run that appends nothing still catches the store up.
#[tokio::test]
async fn a_run_that_owes_nothing_still_repairs_a_wedged_store() {
    let fixture = Fixture::new("durable-run-owes-nothing");
    let mut state = state_with_two_revisions(&fixture.config());
    state
        .tenants
        .get_mut(TENANT)
        .expect("served")
        .get_mut()
        .projection_store = Box::new(ApplyRefusingStore::refusing_the_first(1));
    let session = Session::from_state(state);
    write_settled(
        &session,
        Some(fixture.operator_token()),
        "ent_settled",
        "idem_s",
    )
    .await;
    let (_, status) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    assert!(
        status.contains(r#""log_head":1,"applied_ordinal":0,"lag":1"#),
        "{status}"
    );
    let (status, body) = run(&session, Some(fixture.operator_token()), &plan_for(TENANT)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains(r#""upcast":0"#), "{body}");
    let (_, status) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    assert!(
        status.contains(r#""log_head":1,"applied_ordinal":1,"lag":0"#),
        "{status}"
    );
}
