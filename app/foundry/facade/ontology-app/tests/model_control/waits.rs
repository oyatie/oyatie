use crate::model_support::*;

#[tokio::test]
async fn role_revocation_while_waiting_for_tenant_mutex_prevents_saved_mutation() {
    let fixture = crate::facade_support::Fixture::new("model-held-revocation");
    let state = instrument(fixture.state(), &["foundry-model-author"], Some(origin()));
    let held = state.app.tenants[TENANT].lock().await;
    let input = CreateModelDraft {
        expected_generation: number(1),
        document: changed(),
    };
    let router = state.router.clone();
    let request = request("POST", DRAFTS, input.encode().unwrap());
    let response = tokio::spawn(async move { send(&router, request).await });
    wait_for_calls(&state.sessions, 1).await;
    *state.sessions.answer.lock().unwrap() = Ok(view(&["foundry-model-reader"]));
    drop(held);
    refusal(
        response.await.unwrap(),
        StatusCode::FORBIDDEN,
        ModelRefusalCode::Forbidden,
    )
    .await;
    assert_eq!(state.sessions.calls.load(Ordering::SeqCst), 2);
    assert_eq!(state.store_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn expiry_during_mutex_or_iam_wait_never_reaches_model_source() {
    for mutex_wait in [true, false] {
        let fixture = crate::facade_support::Fixture::new("model-held-expiry");
        let state = instrument(fixture.state(), &["foundry-model-reader"], Some(origin()));
        *state.sessions.answer.lock().unwrap() = Ok(SessionView {
            expires_at: now() + 1,
            ..view(&["foundry-model-reader"])
        });
        if !mutex_wait {
            *state.sessions.delay.lock().unwrap() = std::time::Duration::from_millis(1200);
        }
        let held = if mutex_wait {
            Some(state.app.tenants[TENANT].lock().await)
        } else {
            None
        };
        let router = state.router.clone();
        let response =
            tokio::spawn(
                async move { send(&router, request("GET", WORKSPACE, Body::empty())).await },
            );
        wait_for_calls(&state.sessions, 1).await;
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        drop(held);
        refusal(
            response.await.unwrap(),
            StatusCode::UNAUTHORIZED,
            ModelRefusalCode::Unauthenticated,
        )
        .await;
        assert_eq!(state.store_calls.load(Ordering::SeqCst), 0);
    }
}
