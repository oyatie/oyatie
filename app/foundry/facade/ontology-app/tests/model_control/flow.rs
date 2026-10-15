use crate::model_support::*;

#[tokio::test]
async fn real_owner_create_save_reopen_prepare_keeps_published_projection_and_logs_unchanged() {
    let fixture = crate::facade_support::Fixture::new("model-lifecycle");
    let state = instrument(fixture.state(), &["foundry-model-author"], Some(origin()));
    let before = state.app.tenants[TENANT].lock().await.projection().clone();
    let created = create(&state.router).await;
    assert_eq!(created.owner, "usr_alice");
    assert_eq!(created.tenant_id, TENANT);
    assert_eq!(created.version, number(1));
    let mut incomplete = created.document.clone();
    incomplete
        .entities
        .iter_mut()
        .find(|entity| entity.id == "ety_record")
        .unwrap()
        .display_name
        .clear();
    let path = format!("{DRAFTS}/{}/save", created.id.get());
    let input = SaveModelDraft {
        expected_version: number(1),
        expected_base: number(1),
        document: incomplete.clone(),
    };
    let saved = ModelDraftWire::decode(
        &body(
            send(
                &state.router,
                request("POST", &path, input.encode().unwrap()),
            )
            .await,
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(saved.version, number(2));
    assert_eq!(saved.document, incomplete);
    let detail = format!("{DRAFTS}/{}", saved.id.get());
    assert_eq!(
        ModelDraftWire::decode(
            &body(
                send(&state.router, request("GET", &detail, Body::empty())).await,
                StatusCode::OK
            )
            .await
        )
        .unwrap(),
        saved
    );
    let validation = format!("{detail}/validate");
    let input = PrepareModelDraft {
        expected_version: saved.version,
        expected_base: saved.base_generation,
    };
    let invalid = ModelValidation::decode(
        &body(
            send(
                &state.router,
                request("POST", &validation, input.encode().unwrap()),
            )
            .await,
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert!(!invalid.valid);
    assert_eq!(invalid.draft_version, saved.version);
    assert_eq!(invalid.diagnostics[0].code, "empty_display_name");
    assert_eq!(
        invalid.diagnostics[0].path,
        "entities.ety_record.display_name"
    );
    let valid = SaveModelDraft {
        expected_version: saved.version,
        expected_base: saved.base_generation,
        document: changed(),
    };
    let saved = ModelDraftWire::decode(
        &body(
            send(
                &state.router,
                request("POST", &path, valid.encode().unwrap()),
            )
            .await,
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    let input = PrepareModelDraft {
        expected_version: saved.version,
        expected_base: saved.base_generation,
    };
    let result = ModelValidation::decode(
        &body(
            send(
                &state.router,
                request("POST", &validation, input.encode().unwrap()),
            )
            .await,
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert!(result.valid);
    assert!(result.diagnostics.is_empty());
    let published = ModelWorkspace::decode(
        &body(
            send(&state.router, request("GET", WORKSPACE, Body::empty())).await,
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(published.document, baseline().document);
    assert_eq!(published.generation, number(1));
    assert_eq!(state.app.tenants[TENANT].lock().await.projection(), &before);
    assert_eq!(fixture.log_head(), 0);
    assert_eq!(fixture.denial_head(), 0);
    assert_eq!(
        state
            .reference
            .head(&ModelCaller {
                tenant_id: TENANT.into(),
                owner: "usr_alice".into()
            })
            .unwrap()
            .generation,
        1
    );
    refusal(
        send(
            &state.router,
            request("POST", &path, valid.encode().unwrap()),
        )
        .await,
        StatusCode::CONFLICT,
        ModelRefusalCode::DraftConflict,
    )
    .await;
    let mut wrong = valid.clone();
    wrong.expected_version = saved.version;
    wrong.expected_base = number(2);
    refusal(
        send(
            &state.router,
            request("POST", &path, wrong.encode().unwrap()),
        )
        .await,
        StatusCode::CONFLICT,
        ModelRefusalCode::HeadConflict,
    )
    .await;
    assert_eq!(
        ModelDraftWire::decode(
            &body(
                send(&state.router, request("GET", &detail, Body::empty())).await,
                StatusCode::OK
            )
            .await
        )
        .unwrap(),
        saved
    );
    *state.sessions.answer.lock().unwrap() = Ok(view(&["foundry-model-publisher"]));
    let workspace = ModelWorkspace::decode(
        &body(
            send(&state.router, request("GET", WORKSPACE, Body::empty())).await,
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(
        workspace.capabilities,
        ModelCapabilities {
            author: false,
            publish: true
        }
    );
    body(
        send(
            &state.router,
            request("POST", &validation, input.encode().unwrap()),
        )
        .await,
        StatusCode::OK,
    )
    .await;
    refusal(
        send(
            &state.router,
            request("POST", &path, wrong.encode().unwrap()),
        )
        .await,
        StatusCode::FORBIDDEN,
        ModelRefusalCode::Forbidden,
    )
    .await;
    assert_eq!(
        send(
            &state.router,
            request("POST", &format!("{detail}/publish"), Body::empty())
        )
        .await
        .status(),
        StatusCode::BAD_REQUEST
    );
}
