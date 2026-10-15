use crate::{
    baseline::{caller, path},
    model_support::*,
};
use foundry_model_sqlite_draft::SqliteModelStore;
use foundry_ontology_app::router_with_model_control;

#[tokio::test]
async fn stale_base_remains_exact_after_further_publication_and_sqlite_reopen() {
    let fixture = crate::facade_support::Fixture::new("model-base-durable");
    let directory = std::env::temp_dir().join(format!(
        "foundry-owned-base-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let database = directory.join("model.sqlite");
    let mut store = SqliteModelStore::initialize(&database, binding(), baseline()).unwrap();
    let saved = store.create_draft(&caller(), 1, changed()).unwrap();
    publish_next(&mut store, &caller());
    for generation in [2, 3] {
        if generation == 3 {
            publish_next(&mut store, &caller());
        }
        let history = store.history(&caller()).unwrap();
        let page = DraftPageRequest {
            limit: MAX_MODEL_ITEMS,
            after: None,
        };
        let drafts = store.owned_drafts(&caller(), &page).unwrap();
        let exact_drafts: Vec<_> = drafts
            .drafts
            .iter()
            .map(|row| store.open_draft(&caller(), row.id).unwrap())
            .collect();
        let mut app = fixture.state();
        app.inject_established_model_store(TENANT, binding(), baseline(), Box::new(store))
            .unwrap();
        let app = Arc::new(app);
        let projection = app.tenants[TENANT].lock().await.projection().clone();
        let router = router_with_model_control(
            app.clone(),
            FakeSessions::new(Ok(view(&["foundry-model-reader"]))),
            None,
        );
        let mut input = request("GET", &path(&saved), Body::empty());
        input.headers_mut().remove("origin");
        input.headers_mut().remove("content-type");
        let bytes = body(send(&router, input).await, StatusCode::OK).await;
        let base = ModelDraftBase::decode(&bytes).unwrap();
        assert_eq!(base.document, history.publications[0].document);
        assert_eq!(base.base_generation, number(1));
        assert_eq!(base.draft_id, number(saved.id));
        let workspace = ModelWorkspace::decode(
            &body(
                send(&router, request("GET", WORKSPACE, Body::empty())).await,
                StatusCode::OK,
            )
            .await,
        )
        .unwrap();
        assert_eq!(workspace.generation, number(generation));
        assert_ne!(workspace.document, base.document);
        let tenant = app.tenants[TENANT].lock().await;
        assert_eq!(tenant.projection(), &projection);
        assert_eq!(tenant.observed_model_generation(), None);
        assert_eq!(fixture.log_head(), 0);
        assert_eq!(fixture.denial_head(), 0);
        drop(tenant);
        drop(router);
        drop(app);
        store = SqliteModelStore::open_existing(&database, binding(), baseline()).unwrap();
        assert_eq!(store.history(&caller()).unwrap(), history);
        assert_eq!(store.owned_drafts(&caller(), &page).unwrap(), drafts);
        for draft in exact_drafts {
            assert_eq!(store.open_draft(&caller(), draft.id).unwrap(), draft);
        }
        assert_eq!(store.open_draft(&caller(), saved.id).unwrap(), saved);
        assert!(
            matches!(store.prepare(&caller(), saved.id, saved.version, saved.base_generation), Err(ModelError::HeadConflict { current_generation }) if current_generation == generation)
        );
    }
    drop(store);
    std::fs::remove_dir_all(directory).unwrap();
}
