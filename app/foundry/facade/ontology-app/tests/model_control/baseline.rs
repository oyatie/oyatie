use crate::model_support::*;

pub(super) fn caller() -> ModelCaller {
    ModelCaller {
        tenant_id: TENANT.into(),
        owner: "usr_alice".into(),
    }
}

pub(super) fn path(draft: &ModelDraft) -> String {
    format!(
        "{DRAFTS}/{}/base?expected_version={}&expected_base={}",
        draft.id, draft.version, draft.base_generation
    )
}

#[tokio::test]
async fn stale_saved_baseline_is_exact_and_does_not_authorize_stale_publication() {
    let fixture = crate::facade_support::Fixture::new("model-exact-saved-base");
    let state = instrument(fixture.state(), &["foundry-model-reader"], Some(origin()));
    let mut store = state.reference.clone();
    let saved = store.create_draft(&caller(), 1, changed()).unwrap();
    publish_next(&mut store, &caller());
    let before = store.history(&caller()).unwrap();
    let projection = state.app.tenants[TENANT].lock().await.projection().clone();
    assert_eq!(
        state.app.tenants[TENANT]
            .lock()
            .await
            .observed_model_generation(),
        None
    );
    let response = send(&state.router, request("GET", &path(&saved), Body::empty())).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/json");
    let bytes = body(response, StatusCode::OK).await;
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(wire["draft_id"], "1");
    assert_eq!(wire["draft_version"], "1");
    assert_eq!(wire["base_generation"], "1");
    assert_eq!(
        wire["document"],
        serde_json::to_value(baseline().document).unwrap()
    );
    assert_eq!(wire.as_object().unwrap().len(), 4);
    assert_eq!(store.history(&caller()).unwrap(), before);
    assert_eq!(store.open_draft(&caller(), saved.id).unwrap(), saved);
    assert_eq!(store.head(&caller()).unwrap().generation, 2);
    assert!(matches!(
        store.prepare(&caller(), saved.id, 1, 1),
        Err(ModelError::HeadConflict {
            current_generation: 2
        })
    ));
    assert_eq!(
        state.app.tenants[TENANT].lock().await.projection(),
        &projection
    );
    assert_eq!(fixture.log_head(), 0);
    assert_eq!(fixture.denial_head(), 0);
    assert_eq!(
        state.app.tenants[TENANT]
            .lock()
            .await
            .observed_model_generation(),
        None
    );
}

#[tokio::test]
async fn baseline_keeps_invalid_saved_candidate_and_all_logical_state_unchanged() {
    let fixture = crate::facade_support::Fixture::new("model-base-invalid-saved");
    let state = instrument(fixture.state(), &["foundry-model-reader"], Some(origin()));
    let mut store = state.reference.clone();
    let created = store.create_draft(&caller(), 1, changed()).unwrap();
    let mut invalid = created.document.clone();
    invalid.entities.reverse();
    invalid.entities.push(invalid.entities[0].clone());
    invalid.links.push(ModelLink {
        id: "lty_unresolved".into(),
        from_entity_type: "missing_from".into(),
        to_entity_type: "missing_to".into(),
        cardinality: ModelCardinality::OneToMany,
        allow_cross_tenant: false,
        revision: 1,
        display: None,
    });
    let saved = store
        .save_draft(&caller(), created.id, 1, 1, invalid.clone())
        .unwrap();
    let before = store.history(&caller()).unwrap();
    let bytes = body(
        send(&state.router, request("GET", &path(&saved), Body::empty())).await,
        StatusCode::OK,
    )
    .await;
    let row = ModelDraftBase::decode(&bytes).unwrap();
    assert_eq!(row.draft_version, number(2));
    assert_eq!(row.document, baseline().document);
    assert_eq!(store.open_draft(&caller(), saved.id).unwrap(), saved);
    assert_eq!(store.history(&caller()).unwrap(), before);
    assert!(store.prepare(&caller(), saved.id, 2, 1).is_err());
    assert_eq!(fixture.log_head(), 0);
    assert_eq!(fixture.denial_head(), 0);
}

#[tokio::test]
async fn history_corruption_or_unavailability_never_substitutes_current_or_seed_document() {
    let fixture = crate::facade_support::Fixture::new("model-base-history-refusals");
    let state = instrument(fixture.state(), &["foundry-model-reader"], Some(origin()));
    let mut store = state.reference.clone();
    let saved = store.create_draft(&caller(), 1, changed()).unwrap();
    publish_next(&mut store, &caller());
    let before = store.history(&caller()).unwrap();
    for fault in [
        HistoryFault::Rebound,
        HistoryFault::MissingBase,
        HistoryFault::Duplicate,
        HistoryFault::InvalidDocument,
        HistoryFault::Oversized,
        HistoryFault::Failure,
    ] {
        *state.history_fault.lock().unwrap() = Some(fault);
        refusal(
            send(&state.router, request("GET", &path(&saved), Body::empty())).await,
            StatusCode::SERVICE_UNAVAILABLE,
            ModelRefusalCode::ModelUnavailable,
        )
        .await;
        assert_eq!(store.history(&caller()).unwrap(), before);
        assert_eq!(store.open_draft(&caller(), saved.id).unwrap(), saved);
    }
    *state.history_fault.lock().unwrap() = None;
    state.store_failure.store(true, Ordering::SeqCst);
    refusal(
        send(&state.router, request("GET", &path(&saved), Body::empty())).await,
        StatusCode::SERVICE_UNAVAILABLE,
        ModelRefusalCode::ModelUnavailable,
    )
    .await;
    assert_eq!(fixture.log_head(), 0);
    assert_eq!(fixture.denial_head(), 0);
}

#[tokio::test]
async fn concurrent_append_between_saved_draft_and_history_reads_preserves_exact_base() {
    let fixture = crate::facade_support::Fixture::new("model-base-concurrent-append");
    let state = instrument(fixture.state(), &["foundry-model-reader"], Some(origin()));
    let mut store = state.reference.clone();
    let saved = store.create_draft(&caller(), 1, changed()).unwrap();
    *state.history_fault.lock().unwrap() = Some(HistoryFault::Append);
    let bytes = body(
        send(&state.router, request("GET", &path(&saved), Body::empty())).await,
        StatusCode::OK,
    )
    .await;
    let row = ModelDraftBase::decode(&bytes).unwrap();
    assert_eq!(row.document, baseline().document);
    assert_eq!(row.base_generation, number(1));
    assert_eq!(store.head(&caller()).unwrap().generation, 2);
    assert_eq!(store.open_draft(&caller(), saved.id).unwrap(), saved);
    assert_eq!(fixture.log_head(), 0);
    assert_eq!(fixture.denial_head(), 0);
}
