mod facade_support;
mod failing_store;
use facade_support as support;

use axum::http::StatusCode;
use failing_store::{AlwaysFailingStore, ApplyRefusingStore};
use foundry_ontology_app::{BootError, compose};
use foundry_projection_draft::ProjectionStore;
use foundry_projection_sqlite_draft::SqliteProjectionStore;
use support::{Fixture, Session, TENANT, WRITE_BODY as WRITE, scrape, value_of};

const FRESH_TENANT: &str = r#""tenant":{"log_head":0,"applied_ordinal":0,"lag":0,"poisoned_entries":0,"first_poisoned_ordinal":null}"#;

fn store_of(fixture: &Fixture) -> SqliteProjectionStore {
    SqliteProjectionStore::open(&fixture.projection_store_path()).expect("the store opens")
}

fn name_in(store: &SqliteProjectionStore, object_ref: &str) -> Option<(String, u64)> {
    store
        .get(TENANT, object_ref)
        .expect("the store reads")
        .map(|object| {
            let name = match &object
                .entity
                .properties
                .get("name")
                .expect("the seeded type requires a name")
                .value
                .value
            {
                data_ontology_kernel::PropertyValue::String(name) => name.clone(),
                other => panic!("a name is a string: {other:?}"),
            };
            (name, object.last_ordinal)
        })
}

#[tokio::test]
async fn an_accepted_write_is_in_the_durable_store_when_the_response_returns() {
    let fixture = Fixture::new("durable-write");
    let session = fixture.session();
    let (status, _) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(status, StatusCode::OK);
    let store = store_of(&fixture);
    assert_eq!(store.applied_head(TENANT).expect("head reads"), 1);
    assert_eq!(name_in(&store, "ent_alpha"), Some(("Ada".to_owned(), 1)));
    assert_eq!(fixture.log_head(), 1);
}

/// The HTTP read and a direct read of the store agree on the name; the
/// store's last ordinal for that object is the write's.
#[tokio::test]
async fn a_read_through_http_agrees_with_the_durable_store() {
    let fixture = Fixture::new("durable-read");
    let session = fixture.session();
    session.post(Some(fixture.operator_token()), WRITE).await;
    let (status, body) = session
        .get(
            Some(fixture.operator_token()),
            "/v1/objects/ent_alpha?revision=1",
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains(r#""value":"Ada""#), "{body}");
    assert_eq!(
        name_in(&store_of(&fixture), "ent_alpha"),
        Some(("Ada".to_owned(), 1))
    );
}

#[tokio::test]
async fn boot_rebuilds_a_missing_store_from_the_log() {
    let fixture = Fixture::new("durable-rebuild");
    {
        let session = fixture.session();
        session.post(Some(fixture.operator_token()), WRITE).await;
        let second = WRITE
            .replace("ent_alpha", "ent_beta")
            .replace("idem_1", "idem_2");
        let (status, _) = session.post(Some(fixture.operator_token()), &second).await;
        assert_eq!(status, StatusCode::OK);
    }
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!(
            "{}{suffix}",
            fixture.projection_store_path().display()
        ));
    }
    let session = fixture.session();
    let store = store_of(&fixture);
    assert_eq!(store.applied_head(TENANT).expect("head reads"), 2);
    assert_eq!(name_in(&store, "ent_beta"), Some(("Ada".to_owned(), 2)));
    let (_, status) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    assert!(
        status.contains(r#""log_head":2,"applied_ordinal":2,"lag":0"#),
        "{status}"
    );
}

#[tokio::test]
async fn boot_refuses_a_store_built_under_another_log() {
    let ours = Fixture::new("durable-ours");
    let theirs = Fixture::new("durable-theirs");
    ours.session()
        .post(Some(ours.operator_token()), WRITE)
        .await;
    theirs
        .session()
        .post(
            Some(theirs.operator_token()),
            &WRITE.replace("ent_alpha", "ent_beta"),
        )
        .await;
    let mut config = theirs.config();
    config.projection_store = ours.projection_store_path();
    let refused = compose(&config).expect_err("a store from another log must not serve");
    assert!(
        matches!(refused, BootError::CatchUpRefused { .. }),
        "{refused:?}"
    );
}

#[tokio::test]
async fn boot_refuses_an_unopenable_or_aliased_store() {
    let fixture = Fixture::new("durable-unopenable");
    let mut config = fixture.config();
    config.projection_store = std::env::temp_dir();
    assert!(matches!(
        compose(&config).expect_err("a directory is not a store"),
        BootError::ProjectionStoreUnopenable { .. }
    ));
    for alias in [fixture.action_log_path(), fixture.denial_log_path()] {
        let mut config = fixture.config();
        config.projection_store = alias;
        assert!(matches!(
            compose(&config).expect_err("a log is not a store"),
            BootError::StorePathAliased
        ));
    }
}

/// The sync facts are the STORE's: a write whose mirror the store refused
/// is lag one and un-ready until a catch-up repairs the store.
#[tokio::test]
async fn a_mirror_the_store_refuses_is_lag_the_store_reports() {
    let fixture = Fixture::new("durable-mirror-refused");
    let mut state = fixture.state();
    state
        .tenants
        .get_mut(TENANT)
        .expect("served")
        .get_mut()
        .projection_store = Box::new(ApplyRefusingStore::refusing_the_first(1));
    let session = Session::from_state(state);
    let (status, body) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(status, StatusCode::OK, "the log took it: {body}");
    assert_eq!(fixture.log_head(), 1);
    let (_, status) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    assert!(
        status.contains(r#""log_head":1,"applied_ordinal":0,"lag":1"#),
        "{status}"
    );
    let (ready, _) = session.get(None, "/readyz").await;
    assert_eq!(ready, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        value_of(&scrape(&session).await, "foundry_projection_lag"),
        1
    );

    // ONE refused mirror wedges the store: the next ordinal is not dense
    // against its head, so the store refuses that too, and only a catch-up
    // repairs it (boot; or a migration run, in its own file). No write does.
    let second = WRITE
        .replace("ent_alpha", "ent_beta")
        .replace("idem_1", "idem_2");
    let (status, _) = session.post(Some(fixture.operator_token()), &second).await;
    assert_eq!(status, StatusCode::OK);
    let (_, status) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    assert!(
        status.contains(r#""log_head":2,"applied_ordinal":0,"lag":2"#),
        "{status}"
    );
}

/// Three tenants, first and last with unreadable stores: `null` sync facts
/// for their operators, the fresh block for the middle's; the caller's own
/// registry is served from memory either way.
#[tokio::test]
async fn an_unreadable_store_nulls_the_sync_facts_and_still_serves_the_registry() {
    let fixture = Fixture::new("durable-unreadable");
    let mut config = fixture.config();
    config.tenants.push("ten_other".into());
    config.tenants.push("ten_zzz".into());
    let mut state = compose(&config).expect("boots");
    for id in ["ten_acme", "ten_zzz"] {
        state
            .tenants
            .get_mut(id)
            .expect("served")
            .get_mut()
            .projection_store = Box::new(AlwaysFailingStore {
            detail: "the store is gone",
        });
    }
    let session = Session::from_state(state);
    let (_, body) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    let (_, other) = session.get(Some(fixture.foreign_token()), "/statusz").await;
    assert!(
        body.contains(r#""unreadable_tenants":2"#)
            && body.contains(r#""tenant":null"#)
            && body.contains(r#""entity_types":["ety_record"]"#),
        "{body}"
    );
    assert!(
        other.contains(r#""unreadable_tenants":2"#) && other.contains(FRESH_TENANT),
        "its neighbours' dead stores null nothing for this caller: {other}"
    );
    let (ready, _) = session.get(None, "/readyz").await;
    assert_eq!(ready, StatusCode::SERVICE_UNAVAILABLE);
}
