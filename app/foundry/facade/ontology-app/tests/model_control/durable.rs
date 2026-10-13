use crate::model_support::*;
use foundry_model_sqlite_draft::SqliteModelStore;
use foundry_ontology_app::router_with_model_control;

#[tokio::test]
async fn mixed_owner_paged_discovery_and_real_sqlite_reopen_share_exact_server_state() {
    let fixture = crate::facade_support::Fixture::new("model-durable-reopen");
    let path = std::env::temp_dir().join(format!(
        "foundry-model-route-{}-{}.sqlite",
        std::process::id(),
        now()
    ));
    let mut store = SqliteModelStore::initialize(&path, binding(), baseline()).unwrap();
    let alice = ModelCaller {
        tenant_id: TENANT.into(),
        owner: "usr_alice".into(),
    };
    let other = ModelCaller {
        owner: "usr_other".into(),
        ..alice.clone()
    };
    let first = store.create_draft(&alice, 1, changed()).unwrap();
    store.create_draft(&other, 1, changed()).unwrap();
    let third = store.create_draft(&alice, 1, changed()).unwrap();
    let mut app = fixture.state();
    app.inject_established_model_store(TENANT, binding(), baseline(), Box::new(store))
        .unwrap();
    let sessions = FakeSessions::new(Ok(view(&["foundry-model-reader"])));
    let router = router_with_model_control(Arc::new(app), sessions.clone(), Some(origin()));
    let page = ModelDraftPage::decode(
        &body(
            send(
                &router,
                request("GET", &format!("{DRAFTS}?limit=1"), Body::empty()),
            )
            .await,
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(page.drafts[0].id, number(first.id));
    assert_eq!(page.next, Some(number(first.id)));
    let second = ModelDraftPage::decode(
        &body(
            send(
                &router,
                request("GET", &format!("{DRAFTS}?limit=1&after=1"), Body::empty()),
            )
            .await,
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(second.drafts[0].id, number(third.id));
    assert_eq!(second.next, None);
    refusal(
        send(
            &router,
            request("GET", &format!("{DRAFTS}/2"), Body::empty()),
        )
        .await,
        StatusCode::FORBIDDEN,
        ModelRefusalCode::Forbidden,
    )
    .await;
    refusal(
        send(
            &router,
            request("GET", &format!("{DRAFTS}/99"), Body::empty()),
        )
        .await,
        StatusCode::NOT_FOUND,
        ModelRefusalCode::DraftNotFound,
    )
    .await;
    drop(router);
    let store = SqliteModelStore::open_existing(&path, binding(), baseline()).unwrap();
    let mut app = fixture.state();
    app.inject_established_model_store(TENANT, binding(), baseline(), Box::new(store))
        .unwrap();
    let router = router_with_model_control(Arc::new(app), sessions, Some(origin()));
    let opened = ModelDraftWire::decode(
        &body(
            send(
                &router,
                request("GET", &format!("{DRAFTS}/{}", third.id), Body::empty()),
            )
            .await,
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(opened, ModelDraftWire::try_from(&third).unwrap());
    drop(router);
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
}
