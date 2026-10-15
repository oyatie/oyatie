use crate::{
    baseline::{caller, path},
    model_support::*,
};

#[tokio::test]
async fn baseline_requires_fresh_read_membership_and_existing_owner_without_metadata_leakage() {
    let fixture = crate::facade_support::Fixture::new("model-base-access");
    let state = instrument(fixture.state(), &["foundry-model-reader"], Some(origin()));
    let mut store = state.reference.clone();
    let saved = store.create_draft(&caller(), 1, changed()).unwrap();
    let held = state.app.tenants[TENANT].lock().await;
    let mut anonymous = request("GET", &path(&saved), Body::empty());
    anonymous.headers_mut().remove("cookie");
    refusal(
        tokio::time::timeout(
            std::time::Duration::from_millis(200),
            send(&state.router, anonymous),
        )
        .await
        .unwrap(),
        StatusCode::UNAUTHORIZED,
        ModelRefusalCode::Unauthenticated,
    )
    .await;
    for (answer, status, code) in [
        (
            Err(SessionRefusal::Unauthenticated),
            StatusCode::UNAUTHORIZED,
            ModelRefusalCode::Unauthenticated,
        ),
        (
            Err(SessionRefusal::Expired),
            StatusCode::UNAUTHORIZED,
            ModelRefusalCode::Unauthenticated,
        ),
        (
            Err(SessionRefusal::MissingMembership),
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            Err(SessionRefusal::InactiveMembership),
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            Err(SessionRefusal::AmbiguousMembership),
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            Err(SessionRefusal::MembershipUnavailable),
            StatusCode::SERVICE_UNAVAILABLE,
            ModelRefusalCode::ModelUnavailable,
        ),
        (
            Ok(view(&["foundry-operator"])),
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            Ok(SessionView {
                tenant_id: "ten_foreign".into(),
                ..view(&["foundry-model-reader"])
            }),
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            Ok(SessionView {
                expires_at: now(),
                ..view(&["foundry-model-reader"])
            }),
            StatusCode::UNAUTHORIZED,
            ModelRefusalCode::Unauthenticated,
        ),
    ] {
        *state.sessions.answer.lock().unwrap() = answer;
        refusal(
            tokio::time::timeout(
                std::time::Duration::from_millis(200),
                send(&state.router, request("GET", &path(&saved), Body::empty())),
            )
            .await
            .unwrap(),
            status,
            code,
        )
        .await;
    }
    *state.sessions.answer.lock().unwrap() = Ok(view(&["foundry-model-reader"]));
    state.pdp.unavailable.store(true, Ordering::SeqCst);
    refusal(
        tokio::time::timeout(
            std::time::Duration::from_millis(200),
            send(&state.router, request("GET", &path(&saved), Body::empty())),
        )
        .await
        .unwrap(),
        StatusCode::SERVICE_UNAVAILABLE,
        ModelRefusalCode::ModelUnavailable,
    )
    .await;
    assert_eq!(state.store_calls.load(Ordering::SeqCst), 0);
    state.pdp.unavailable.store(false, Ordering::SeqCst);
    drop(held);
    *state.sessions.answer.lock().unwrap() = Ok(SessionView {
        user_id: "usr_other".into(),
        ..view(&["foundry-model-reader"])
    });
    refusal(
        send(
            &state.router,
            request(
                "GET",
                &path(&saved).replace("expected_version=1", "expected_version=99"),
                Body::empty(),
            ),
        )
        .await,
        StatusCode::FORBIDDEN,
        ModelRefusalCode::Forbidden,
    )
    .await;
    assert_eq!(state.store_calls.load(Ordering::SeqCst), 1);
    *state.sessions.answer.lock().unwrap() = Ok(view(&["foundry-model-reader"]));
    refusal(
        send(
            &state.router,
            request(
                "GET",
                &path(&saved).replace(TENANT, "ten_foreign"),
                Body::empty(),
            ),
        )
        .await,
        StatusCode::FORBIDDEN,
        ModelRefusalCode::Forbidden,
    )
    .await;
    assert_eq!(state.store_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn baseline_rechecks_identity_read_grant_and_pdp_after_mutex_wait() {
    for case in 0..4 {
        let fixture = crate::facade_support::Fixture::new("model-base-held-access");
        let state = instrument(fixture.state(), &["foundry-model-reader"], Some(origin()));
        let mut store = state.reference.clone();
        let saved = store.create_draft(&caller(), 1, changed()).unwrap();
        let held = state.app.tenants[TENANT].lock().await;
        let router = state.router.clone();
        let response = tokio::spawn(async move {
            send(&router, request("GET", &path(&saved), Body::empty())).await
        });
        wait_for_calls(&state.sessions, 1).await;
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while state.pdp.calls.load(Ordering::SeqCst) == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let (status, code) = match case {
            0 => {
                *state.sessions.answer.lock().unwrap() = Ok(view(&["foundry-operator"]));
                (StatusCode::FORBIDDEN, ModelRefusalCode::Forbidden)
            }
            1 => {
                *state.sessions.answer.lock().unwrap() = Ok(SessionView {
                    user_id: "usr_other".into(),
                    ..view(&["foundry-model-reader"])
                });
                (StatusCode::FORBIDDEN, ModelRefusalCode::Forbidden)
            }
            2 => {
                *state.sessions.answer.lock().unwrap() = Err(SessionRefusal::Unauthenticated);
                (StatusCode::UNAUTHORIZED, ModelRefusalCode::Unauthenticated)
            }
            _ => {
                state.pdp.unavailable.store(true, Ordering::SeqCst);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    ModelRefusalCode::ModelUnavailable,
                )
            }
        };
        drop(held);
        refusal(response.await.unwrap(), status, code).await;
        assert_eq!(state.sessions.calls.load(Ordering::SeqCst), 2);
        assert_eq!(state.store_calls.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn baseline_session_expiry_while_waiting_never_reaches_source() {
    let fixture = crate::facade_support::Fixture::new("model-base-held-expiry");
    let state = instrument(fixture.state(), &["foundry-model-reader"], Some(origin()));
    let mut store = state.reference.clone();
    let saved = store.create_draft(&caller(), 1, changed()).unwrap();
    *state.sessions.answer.lock().unwrap() = Ok(SessionView {
        expires_at: now() + 1,
        ..view(&["foundry-model-reader"])
    });
    let held = state.app.tenants[TENANT].lock().await;
    let router = state.router.clone();
    let response =
        tokio::spawn(
            async move { send(&router, request("GET", &path(&saved), Body::empty())).await },
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
