use crate::{model_support::*, native::*};

#[tokio::test]
async fn only_canonical_publisher_and_exact_owner_can_publish() {
    for roles in [
        vec!["foundry-model-author"],
        vec!["foundry-operator"],
        vec!["foundry-model-reader"],
    ] {
        let fixture = NativeFixture::new("publish-role");
        let state = fixture.state(&["foundry-model-author"]);
        let saved = create(&state.router).await;
        *state.sessions.answer.lock().unwrap() = Ok(view(&roles));
        refusal(
            send(&state.router, publication_request(&saved)).await,
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        )
        .await;
        assert_eq!(
            state.app.tenants[TENANT]
                .lock()
                .await
                .observed_model_generation(),
            Some(1)
        );
    }
    let fixture = NativeFixture::new("publish-owner");
    let state = fixture.state(&["foundry-model-author"]);
    let saved = create(&state.router).await;
    *state.sessions.answer.lock().unwrap() = Ok(SessionView {
        user_id: "usr_foreign".into(),
        ..view(&["foundry-model-publisher"])
    });
    refusal(
        send(&state.router, publication_request(&saved)).await,
        StatusCode::FORBIDDEN,
        ModelRefusalCode::Forbidden,
    )
    .await;
}

#[tokio::test]
async fn publisher_requires_exact_origin_and_closed_saved_version_input() {
    let fixture = NativeFixture::new("publish-closed");
    let state = fixture.state(&["foundry-model-author", "foundry-model-publisher"]);
    let saved = create(&state.router).await;
    let path = format!("{DRAFTS}/{}/publish", saved.id.get());
    for input in [
        r#"{"expected_version":1,"expected_base":"1"}"#,
        r#"{"expected_version":"1","expected_base":"1","owner":"usr_alice"}"#,
        r#"{"expected_version":"1","expected_base":"1","role":"foundry-model-publisher"}"#,
        r#"{"expected_version":"1","expected_version":"1","expected_base":"1"}"#,
    ] {
        refusal(
            send(&state.router, request("POST", &path, input)).await,
            StatusCode::BAD_REQUEST,
            ModelRefusalCode::MalformedRequest,
        )
        .await;
    }
    let mut wrong = publication_request(&saved);
    wrong
        .headers_mut()
        .insert("origin", "https://foreign.example".parse().unwrap());
    refusal(
        send(&state.router, wrong).await,
        StatusCode::FORBIDDEN,
        ModelRefusalCode::Forbidden,
    )
    .await;
    assert_eq!(
        state.app.tenants[TENANT]
            .lock()
            .await
            .observed_model_generation(),
        Some(1)
    );
}

#[tokio::test]
async fn publication_rechecks_membership_after_waiting_for_tenant() {
    let fixture = NativeFixture::new("publish-wait");
    let state = fixture.state(&["foundry-model-author", "foundry-model-publisher"]);
    let saved = create(&state.router).await;
    let count = state.sessions.calls.load(Ordering::SeqCst);
    let held = state.app.tenants[TENANT].lock().await;
    let router = state.router.clone();
    let request = publication_request(&saved);
    let response = tokio::spawn(async move { send(&router, request).await });
    wait_for_calls(&state.sessions, count + 1).await;
    *state.sessions.answer.lock().unwrap() = Ok(view(&["foundry-model-author"]));
    drop(held);
    refusal(
        response.await.unwrap(),
        StatusCode::FORBIDDEN,
        ModelRefusalCode::Forbidden,
    )
    .await;
    assert_eq!(
        state.app.tenants[TENANT]
            .lock()
            .await
            .observed_model_generation(),
        Some(1)
    );
}

#[tokio::test]
async fn lifecycle_is_model_only_authorized_strict_and_no_store() {
    let fixture = NativeFixture::new("model-lifecycle-access");
    let state = fixture.state(&["foundry-model-reader"]);
    assert!(lifecycle(&state).await.ready);
    refusal(
        send(
            &state.router,
            request(
                "GET",
                &format!("{LIFECYCLE}?tenant=ten_other"),
                Body::empty(),
            ),
        )
        .await,
        StatusCode::BAD_REQUEST,
        ModelRefusalCode::MalformedRequest,
    )
    .await;
    let denied = send(&state.router, request("GET", "/statusz", Body::empty())).await;
    assert!(!denied.status().is_success());
    let denied = send(
        &state.router,
        request(
            "GET",
            "/v1/tenants/ten_acme/objects/ent_hot?type=ety_record&revision=1",
            Body::empty(),
        ),
    )
    .await;
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    *state.sessions.answer.lock().unwrap() = Ok(view(&[]));
    refusal(
        send(&state.router, request("GET", LIFECYCLE, Body::empty())).await,
        StatusCode::FORBIDDEN,
        ModelRefusalCode::Forbidden,
    )
    .await;
}
