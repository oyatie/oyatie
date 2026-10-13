use crate::model_support::*;
use axum::http::{HeaderValue, header};

#[tokio::test]
async fn cookie_membership_roles_and_pdp_refusals_finish_before_held_model_access() {
    let fixture = crate::facade_support::Fixture::new("model-identity-refusals");
    let state = instrument(fixture.state(), &["foundry-model-reader"], Some(origin()));
    let held = state.app.tenants[TENANT].lock().await;
    for cookie in [
        None,
        Some("__Host-oya_session=v1.bad"),
        Some("__Host-oya_session=v1.AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB"),
        Some("malformed"),
    ] {
        let mut input = request("GET", WORKSPACE, Body::empty());
        input.headers_mut().remove(header::COOKIE);
        input.headers_mut().insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer operator-token-for-tests"),
        );
        if let Some(cookie) = cookie {
            input
                .headers_mut()
                .insert(header::COOKIE, HeaderValue::from_str(cookie).unwrap());
        }
        let response = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            send(&state.router, input),
        )
        .await
        .unwrap();
        refusal(
            response,
            StatusCode::UNAUTHORIZED,
            ModelRefusalCode::Unauthenticated,
        )
        .await;
    }
    let mut duplicate = request("GET", WORKSPACE, Body::empty());
    duplicate.headers_mut().append(
        header::COOKIE,
        HeaderValue::from_str(&format!("__Host-oya_session={TOKEN}")).unwrap(),
    );
    refusal(
        tokio::time::timeout(
            std::time::Duration::from_millis(200),
            send(&state.router, duplicate),
        )
        .await
        .unwrap(),
        StatusCode::UNAUTHORIZED,
        ModelRefusalCode::Unauthenticated,
    )
    .await;
    assert_eq!(state.sessions.calls.load(Ordering::SeqCst), 0);
    for (answer, status, code) in [
        (
            Err(SessionRefusal::Unauthenticated),
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
            Ok(view(&[])),
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            Ok(view(&["foundry-operator"])),
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            Ok(view(&["foundry-model-reader", "foundry-model-reader"])),
            StatusCode::SERVICE_UNAVAILABLE,
            ModelRefusalCode::ModelUnavailable,
        ),
        (
            Ok(view(&[" "])),
            StatusCode::SERVICE_UNAVAILABLE,
            ModelRefusalCode::ModelUnavailable,
        ),
        (
            Ok(SessionView {
                tenant_id: "ten_other".into(),
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
                send(&state.router, request("GET", WORKSPACE, Body::empty())),
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
            send(&state.router, request("GET", WORKSPACE, Body::empty())),
        )
        .await
        .unwrap(),
        StatusCode::SERVICE_UNAVAILABLE,
        ModelRefusalCode::ModelUnavailable,
    )
    .await;
    assert_eq!(state.store_calls.load(Ordering::SeqCst), 0);
    drop(held);
}

#[tokio::test]
async fn unavailable_or_oversized_model_source_refuses_without_details_or_seed_fallback() {
    let fixture = crate::facade_support::Fixture::new("model-source-refusals");
    let app = Arc::new(fixture.state());
    let sessions = FakeSessions::new(Ok(view(&["foundry-model-reader"])));
    let router = foundry_ontology_app::router_with_user_sessions(app, sessions);
    refusal(
        send(&router, request("GET", WORKSPACE, Body::empty())).await,
        StatusCode::SERVICE_UNAVAILABLE,
        ModelRefusalCode::ModelUnavailable,
    )
    .await;
    let state = instrument(fixture.state(), &["foundry-model-reader"], Some(origin()));
    state.store_failure.store(true, Ordering::SeqCst);
    refusal(
        send(&state.router, request("GET", WORKSPACE, Body::empty())).await,
        StatusCode::SERVICE_UNAVAILABLE,
        ModelRefusalCode::ModelUnavailable,
    )
    .await;
    state.store_failure.store(false, Ordering::SeqCst);
    state.oversized_response.store(true, Ordering::SeqCst);
    refusal(
        send(&state.router, request("GET", WORKSPACE, Body::empty())).await,
        StatusCode::SERVICE_UNAVAILABLE,
        ModelRefusalCode::ModelUnavailable,
    )
    .await;
}

#[tokio::test]
async fn absent_origin_disables_mutation_and_wrong_established_binding_never_installs() {
    let fixture = crate::facade_support::Fixture::new("model-injection-refusal");
    let state = instrument(fixture.state(), &["foundry-model-author"], None);
    refusal(
        send(
            &state.router,
            request(
                "POST",
                DRAFTS,
                CreateModelDraft {
                    expected_generation: number(1),
                    document: changed(),
                }
                .encode()
                .unwrap(),
            ),
        )
        .await,
        StatusCode::FORBIDDEN,
        ModelRefusalCode::Forbidden,
    )
    .await;
    assert_eq!(state.store_calls.load(Ordering::SeqCst), 0);
    let store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    let mut app = fixture.state();
    let mut wrong = binding();
    wrong.baseline_id = "untrusted-baseline".into();
    assert_eq!(
        app.inject_established_model_store(TENANT, wrong, baseline(), Box::new(store)),
        Err(ModelError::BindingMismatch)
    );
    let router = foundry_ontology_app::router_with_user_sessions(
        Arc::new(app),
        FakeSessions::new(Ok(view(&["foundry-model-reader"]))),
    );
    refusal(
        send(&router, request("GET", WORKSPACE, Body::empty())).await,
        StatusCode::SERVICE_UNAVAILABLE,
        ModelRefusalCode::ModelUnavailable,
    )
    .await;
    for value in [
        "http://shell.example",
        "https://shell.example/editor",
        "https://user@shell.example",
        "https://shell.example?x=1",
        "https://shell.example#x",
    ] {
        assert!(foundry_ontology_app::model_control::ModelMutationOrigin::https(value).is_err());
    }
    assert!(
        foundry_ontology_app::model_control::ModelMutationOrigin::loopback_http(
            "http://127.0.0.1:9000"
        )
        .is_ok()
    );
    assert!(
        foundry_ontology_app::model_control::ModelMutationOrigin::loopback_http(
            "http://shell.example"
        )
        .is_err()
    );
}
