use super::{
    faults::Fault,
    served_recovery::{Sessions, lifecycle, send},
    support::*,
};
use axum::{body::Body, http::StatusCode};
use foundry_model_api::{control::*, *};
use http_body_util::BodyExt;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn served_uncommitted_unknown_preserves_metadata_for_exact_recovery() {
    check_metadata_freeze(Fault::RejectAndLoseReadback, false).await;
}

#[tokio::test]
async fn served_committed_unknown_preserves_metadata_for_exact_recovery() {
    check_metadata_freeze(Fault::CommitAndLoseReadback, true).await;
}

async fn check_metadata_freeze(mode: Fault, committed: bool) {
    let files = Files::new();
    let (mut app, fault) = files.compose(mode);
    let tenant = app.tenants.get_mut(TENANT).unwrap().get_mut();
    let saved = draft(tenant);
    let other = draft(tenant);
    let app = Arc::new(app);
    let roles = || {
        vec![
            "foundry-model-author".into(),
            "foundry-model-publisher".into(),
        ]
    };
    let sessions = Arc::new(Sessions(Mutex::new(roles())));
    let router = crate::router_with_model_control(
        app.clone(),
        sessions.clone(),
        Some(files.models.origin.clone()),
    );
    let second_sessions = Arc::new(Sessions(Mutex::new(roles())));
    let second_client = crate::router_with_model_control(
        app.clone(),
        second_sessions.clone(),
        Some(files.models.origin.clone()),
    );
    let drafts_path = format!("/v1/tenants/{TENANT}/model/drafts");
    let path = format!("{drafts_path}/{}/publish", saved.id);
    let publish_input = input(&saved).encode().unwrap();
    assert_eq!(
        send(&router, "POST", &path, publish_input.clone(), false)
            .await
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    *fault.lock().unwrap() = Fault::None;
    let tenant = app.tenants[TENANT].lock().await;
    let store = &tenant.model_store.as_ref().unwrap().store;
    let history = store.history(&caller()).unwrap();
    let before_saved = store.open_draft(&caller(), saved.id).unwrap();
    let page = DraftPageRequest {
        limit: 10,
        after: None,
    };
    let before_drafts = store.owned_drafts(&caller(), &page).unwrap();
    let projection = tenant.projection.clone();
    let pending = tenant.publication_unknown.clone();
    assert!(pending.is_some());
    let writes = files.writes.lock().unwrap().clone();
    drop(tenant);

    let create_input = CreateModelDraft {
        expected_generation: ModelNumber::new(if committed { 2 } else { 1 }).unwrap(),
        document: saved.document.clone(),
    }
    .encode()
    .unwrap();
    // Revocation still takes precedence over the unknown-outcome refusal.
    *second_sessions.0.lock().unwrap() = vec!["foundry-model-reader".into()];
    assert_eq!(
        send(
            &second_client,
            "POST",
            &drafts_path,
            create_input.clone(),
            false
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    *second_sessions.0.lock().unwrap() = roles();
    for draft in [&saved, &other] {
        let save_input = SaveModelDraft {
            expected_version: ModelNumber::new(draft.version).unwrap(),
            expected_base: ModelNumber::new(draft.base_generation).unwrap(),
            document: draft.document.clone(),
        }
        .encode()
        .unwrap();
        assert_eq!(
            send(
                &second_client,
                "POST",
                &format!("{drafts_path}/{}/save", draft.id),
                save_input,
                false
            )
            .await
            .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
    assert_eq!(
        send(&second_client, "POST", &drafts_path, create_input, false)
            .await
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    let detail = format!("{drafts_path}/{}", saved.id);
    let read = send(&second_client, "GET", &detail, Body::empty(), false).await;
    assert_eq!(read.status(), StatusCode::OK);
    assert_eq!(
        ModelDraftWire::decode(&read.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        ModelDraftWire::try_from(&before_saved).unwrap()
    );
    let read = send(
        &second_client,
        "GET",
        &format!(
            "{detail}/base?expected_version={}&expected_base={}",
            saved.version, saved.base_generation
        ),
        Body::empty(),
        false,
    )
    .await;
    assert_eq!(read.status(), StatusCode::OK);
    assert_eq!(read.headers()["cache-control"], "no-store");
    let exact_base =
        ModelDraftBase::decode(&read.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(exact_base.draft_id.get(), saved.id);
    assert_eq!(exact_base.draft_version.get(), saved.version);
    assert_eq!(exact_base.base_generation.get(), saved.base_generation);
    assert_eq!(exact_base.document, history.publications[0].document);
    let validation = send(
        &second_client,
        "POST",
        &format!("{detail}/validate"),
        publish_input.clone(),
        false,
    )
    .await;
    assert_eq!(
        validation.status(),
        if committed {
            StatusCode::CONFLICT
        } else {
            StatusCode::OK
        }
    );
    if !committed {
        assert!(
            ModelValidation::decode(&validation.into_body().collect().await.unwrap().to_bytes())
                .unwrap()
                .valid
        );
    }
    let unknown = lifecycle(&second_client).await;
    assert_eq!(unknown.phase, ModelLifecyclePhase::CommitUnknown);
    assert!(!unknown.ready);
    let tenant = app.tenants[TENANT].lock().await;
    let store = &tenant.model_store.as_ref().unwrap().store;
    assert_eq!(store.history(&caller()).unwrap(), history);
    assert_eq!(store.open_draft(&caller(), saved.id).unwrap(), before_saved);
    assert_eq!(store.open_draft(&caller(), other.id).unwrap(), other);
    assert_eq!(store.owned_drafts(&caller(), &page).unwrap(), before_drafts);
    assert_eq!(tenant.projection, projection);
    assert_eq!(tenant.publication_unknown, pending);
    assert_eq!(tenant.action_log.head(TENANT).unwrap(), 0);
    assert_eq!(tenant.denial_log.head(TENANT).unwrap(), 0);
    assert_eq!(*files.writes.lock().unwrap(), writes);
    drop(tenant);

    let recovered = send(&router, "POST", &path, publish_input.clone(), false).await;
    assert_eq!(
        recovered.status(),
        if committed {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        }
    );
    let tenant = app.tenants[TENANT].lock().await;
    assert!(tenant.publication_unknown.is_none());
    assert_eq!(
        tenant.observed_model_generation(),
        Some(if committed { 2 } else { 1 })
    );
    assert_eq!(*files.writes.lock().unwrap(), writes);
    drop(tenant);
    assert!(lifecycle(&router).await.ready);
    if !committed {
        let save_input = SaveModelDraft {
            expected_version: ModelNumber::new(other.version).unwrap(),
            expected_base: ModelNumber::new(other.base_generation).unwrap(),
            document: other.document.clone(),
        }
        .encode()
        .unwrap();
        assert_eq!(
            send(
                &second_client,
                "POST",
                &format!("{drafts_path}/{}/save", other.id),
                save_input,
                false
            )
            .await
            .status(),
            StatusCode::OK
        );
    }
    let published = send(&router, "POST", &path, publish_input.clone(), false).await;
    assert_eq!(published.status(), StatusCode::OK);
    let published =
        ModelPublished::decode(&published.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(published.receipt.generation.get(), 2);
    let expected_publishes = if committed { 1 } else { 2 };
    assert_eq!(files.writes.lock().unwrap().publishes, expected_publishes);
    let repeated = send(&router, "POST", &path, publish_input, false).await;
    assert_eq!(repeated.status(), StatusCode::OK);
    assert_eq!(
        ModelPublished::decode(&repeated.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        published
    );
    assert_eq!(files.writes.lock().unwrap().publishes, expected_publishes);
    let created = send(
        &second_client,
        "POST",
        &drafts_path,
        CreateModelDraft {
            expected_generation: ModelNumber::new(2).unwrap(),
            document: saved.document,
        }
        .encode()
        .unwrap(),
        false,
    )
    .await;
    assert_eq!(created.status(), StatusCode::CREATED);
    let created =
        ModelDraftWire::decode(&created.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(created.id.get(), other.id + 1);
    let save_input = SaveModelDraft {
        expected_version: created.version,
        expected_base: created.base_generation,
        document: created.document,
    }
    .encode()
    .unwrap();
    assert_eq!(
        send(
            &second_client,
            "POST",
            &format!("{drafts_path}/{}/save", created.id.get()),
            save_input,
            false
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        lifecycle(&router).await.observed_generation.unwrap().get(),
        2
    );
}
