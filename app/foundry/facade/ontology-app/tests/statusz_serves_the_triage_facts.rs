mod facade_support;
mod failing_log;
mod migration_support;
mod out_of_band;
use facade_support as support;

use axum::http::StatusCode;
use support::{Fixture, Session, WRITE_BODY as WRITE};

const FRESH_TENANT: &str = r#""tenant":{"log_head":0,"applied_ordinal":0,"lag":0,"poisoned_entries":0,"first_poisoned_ordinal":null}"#;
const SEED_REGISTRY: &str = r#""registry":{"entity_types":[{"id":"ety_record","revision":1}],"action_types":[{"id":"aty_record_write","revision":1}]}"#;

#[tokio::test]
async fn a_fresh_process_serves_an_empty_log_and_the_seed_revisions() {
    let fixture = Fixture::new("statusz-facts-fresh");
    let session = fixture.session();

    let (status, body) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains(FRESH_TENANT), "{body}");
    assert!(body.contains(SEED_REGISTRY), "{body}");
}

#[tokio::test]
async fn a_write_moves_the_log_head_and_the_applied_ordinal_together() {
    let fixture = Fixture::new("statusz-facts-write");
    let session = fixture.session();
    let (status, reply) = session.post(Some(fixture.operator_token()), WRITE).await;
    assert_eq!(status, StatusCode::OK, "{reply}");

    let (_, body) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;

    assert!(
        body.contains(r#""tenant":{"log_head":1,"applied_ordinal":1,"lag":0"#),
        "{body}"
    );
}

#[tokio::test]
async fn an_entry_appended_behind_the_process_is_a_head_the_fold_has_not_reached() {
    let fixture = Fixture::new("statusz-facts-behind");
    let session = fixture.session();
    out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_behind");

    let (_, body) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;

    assert!(
        body.contains(r#""tenant":{"log_head":1,"applied_ordinal":0,"lag":1"#),
        "{body}"
    );
}

#[tokio::test]
async fn a_poison_consumed_at_boot_names_the_ordinal_an_operator_starts_from() {
    let fixture = Fixture::new("statusz-facts-poison");
    out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_poison");
    let session = Session::from_state(
        foundry_ontology_app::compose(&fixture.config()).expect("boots over a poisoned log"),
    );

    let (_, body) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;

    assert!(
        body.contains(r#""lag":0,"poisoned_entries":1,"first_poisoned_ordinal":1}"#),
        "{body}"
    );
}

/// Two tenants, one evolved: the shipped seeds serve identical bytes, so only
/// an evolution on ONE tenant can show whose registry a caller is served.
#[tokio::test]
async fn an_evolved_registry_is_served_at_its_current_revision_to_its_own_tenant() {
    let fixture = Fixture::new("statusz-facts-evolved");
    let mut config = fixture.config();
    config.tenants.push("ten_other".into());
    let session = Session::from_state(migration_support::state_with_two_revisions(&config));

    let (_, evolved) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    let (_, other) = session.get(Some(fixture.foreign_token()), "/statusz").await;

    assert!(
        evolved.contains(r#""registry":{"entity_types":[{"id":"ety_record","revision":2}]"#),
        "the revision in force, not the one the seed shipped: {evolved}"
    );
    assert!(
        other.contains(r#""registry":{"entity_types":[{"id":"ety_record","revision":1}]"#),
        "the other tenant was not evolved and is not served its neighbour's revision: {other}"
    );
}

/// In a one-tenant fixture the aggregate, the caller's own block, and the
/// roster's first block are the same numbers. A poison and a lag on the
/// OTHER tenant separate all three: its operator must read them, and the
/// first tenant's operator must not.
#[tokio::test]
async fn the_tenant_block_is_the_callers_own_not_the_aggregate() {
    let fixture = Fixture::new("statusz-facts-own-tenant");
    out_of_band::append_for(&fixture.action_log_path(), "ten_other", "idem_other_poison");
    let session = fixture.both_tenants_session();
    out_of_band::append_for(&fixture.action_log_path(), "ten_other", "idem_other_lag");

    let (_, body) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;

    assert!(
        body.contains(r#""projection_lag":1,"poisoned_entries":1,"contended_tenants":0"#),
        "the aggregate carries the other tenant's lag and poison: {body}"
    );
    assert!(body.contains(FRESH_TENANT), "{body}");

    let (_, other) = session.get(Some(fixture.foreign_token()), "/statusz").await;

    assert!(
        other.contains(
            r#""tenant":{"log_head":2,"applied_ordinal":1,"lag":1,"poisoned_entries":1,"first_poisoned_ordinal":1}"#
        ),
        "the other tenant's operator reads the other tenant's block: {other}"
    );
    assert!(
        other.contains(r#""entity_types":["ety_record"]"#) && other.contains(SEED_REGISTRY),
        "and the other tenant's own registry, not an empty one: {other}"
    );
}

/// The two registries agree in every state production reaches, so only the
/// fixture that evolves one of them can say which one is served. The one
/// served is the one the writer stamps revisions from.
#[tokio::test]
async fn the_registry_served_is_the_one_writes_are_stamped_from() {
    let fixture = Fixture::new("statusz-facts-stamped-from");
    let session = Session::from_state(migration_support::state_with_engine_only_evolved(
        &fixture.config(),
    ));

    let (_, body) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;

    assert!(
        body.contains(r#""registry":{"entity_types":[{"id":"ety_record","revision":1}]"#),
        "the fold input's revision, not the link-bearing engine's: {body}"
    );
}

/// Three tenants, the first and the last locked: each locked tenant's
/// operator gets `null`, and the middle tenant's operator gets every fact.
/// Faulting both ends is what separates "the caller's tenant is locked" from
/// "any tenant is", "the first is", and "the last is".
#[tokio::test]
async fn a_contended_tenant_serves_null_for_every_fact_read_under_its_lock() {
    let fixture = Fixture::new("statusz-facts-contended");
    let mut config = fixture.config();
    config.tenants.push("ten_other".into());
    config.tenants.push("ten_zzz".into());
    let state = std::sync::Arc::new(foundry_ontology_app::compose(&config).expect("boots"));
    let session = Session::from_shared(state.clone());
    let lock = |id: &'static str| state.tenants.get(id).expect("a served tenant").lock();
    let held = (lock("ten_acme").await, lock("ten_zzz").await);

    let (status, body) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains(r#""contended_tenants":2"#)
            && body.contains(r#""entity_types":null,"tenant":null,"registry":null"#),
        "{body}"
    );
    let (_, other) = session.get(Some(fixture.foreign_token()), "/statusz").await;
    assert!(
        other.contains(r#""contended_tenants":2"#)
            && other.contains(r#""entity_types":["ety_record"]"#)
            && other.contains(FRESH_TENANT)
            && other.contains(SEED_REGISTRY),
        "its neighbours' locks withhold nothing from this caller: {other}"
    );
    drop(held);
}

/// Three tenants, the first and the last with unreadable logs: `null` sync
/// facts for their operators, the fresh block for the middle tenant's.
#[tokio::test]
async fn an_unreadable_log_nulls_the_sync_facts_and_still_serves_the_registry() {
    let fixture = Fixture::new("statusz-facts-unreadable");
    let mut config = fixture.config();
    config.tenants.push("ten_other".into());
    config.tenants.push("ten_zzz".into());
    let mut state =
        failing_log::state_with_one_failing_tenant(&config, "ten_acme", "the head is gone");
    state
        .tenants
        .get_mut("ten_zzz")
        .expect("the last served tenant")
        .get_mut()
        .action_log = Box::new(failing_log::AlwaysFailingLog {
        detail: "the head is gone",
    });
    let session = Session::from_state(state);

    let (_, body) = session
        .get(Some(fixture.operator_token()), "/statusz")
        .await;
    let (_, other) = session.get(Some(fixture.foreign_token()), "/statusz").await;

    assert!(
        body.contains(r#""unreadable_tenants":2"#) && body.contains(r#""tenant":null"#),
        "{body}"
    );
    assert!(body.contains(SEED_REGISTRY), "{body}");
    assert!(
        other.contains(r#""unreadable_tenants":2"#)
            && other.contains(r#""entity_types":["ety_record"]"#)
            && other.contains(FRESH_TENANT)
            && other.contains(SEED_REGISTRY),
        "its neighbours' dead stores null nothing for this caller: {other}"
    );
}
