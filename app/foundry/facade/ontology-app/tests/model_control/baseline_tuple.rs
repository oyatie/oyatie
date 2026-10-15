use crate::{
    baseline::{caller, path},
    model_support::*,
};

#[tokio::test]
async fn exact_saved_tuple_conflicts_and_canonical_queries_refuse_before_model_access() {
    let fixture = crate::facade_support::Fixture::new("model-base-exact-tuple");
    let state = instrument(fixture.state(), &["foundry-model-reader"], Some(origin()));
    let mut store = state.reference.clone();
    let saved = store.create_draft(&caller(), 1, changed()).unwrap();
    for (query, code) in [
        (
            "expected_version=2&expected_base=1",
            ModelRefusalCode::DraftConflict,
        ),
        (
            "expected_version=1&expected_base=2",
            ModelRefusalCode::HeadConflict,
        ),
        (
            "expected_version=9007199254740993&expected_base=1",
            ModelRefusalCode::DraftConflict,
        ),
        (
            "expected_version=9223372036854775807&expected_base=1",
            ModelRefusalCode::DraftConflict,
        ),
        (
            "expected_version=1&expected_base=9007199254740993",
            ModelRefusalCode::HeadConflict,
        ),
    ] {
        refusal(
            send(
                &state.router,
                request(
                    "GET",
                    &format!("{DRAFTS}/{}/base?{query}", saved.id),
                    Body::empty(),
                ),
            )
            .await,
            StatusCode::CONFLICT,
            code,
        )
        .await;
    }
    store
        .save_draft(&caller(), saved.id, 1, 1, changed())
        .unwrap();
    refusal(
        send(&state.router, request("GET", &path(&saved), Body::empty())).await,
        StatusCode::CONFLICT,
        ModelRefusalCode::DraftConflict,
    )
    .await;
    for id in ["99", "9007199254740993", "9223372036854775807"] {
        refusal(
            send(
                &state.router,
                request(
                    "GET",
                    &format!("{DRAFTS}/{id}/base?expected_version=1&expected_base=1"),
                    Body::empty(),
                ),
            )
            .await,
            StatusCode::NOT_FOUND,
            ModelRefusalCode::DraftNotFound,
        )
        .await;
    }
    state.store_calls.store(0, Ordering::SeqCst);
    let held = state.app.tenants[TENANT].lock().await;
    for query in [
        "",
        "?",
        "?expected_version=1",
        "?expected_base=1",
        "?expected_version=1&expected_version=1&expected_base=1",
        "?expected_version=1&expected_base=1&expected_base=1",
        "?expected_version=1&expected_base=1&extra=1",
        "?expected_version=1&expected_base=1&",
        "?expected_version=1&expected_base",
        "?expected_version=1&expected_base=",
        "?expected_version=1&expected_base=01",
        "?expected_version=1&expected_base=0",
        "?expected_version=1&expected_base=-1",
        "?expected_version=1&expected_base=+1",
        "?expected_version=1&expected_base=1e3",
        "?expected_version=1&expected_base=1.0",
        "?expected_version=1&expected_base=9223372036854775808",
        "?expected_version=1&expected_base=%31",
        "?expected_version=1&expected_base=1%20",
        "?expected%5Fversion=1&expected_base=1",
        "?expected_version=1=1&expected_base=1",
    ] {
        let response = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            send(
                &state.router,
                request("GET", &format!("{DRAFTS}/1/base{query}"), Body::empty()),
            ),
        )
        .await
        .unwrap();
        refusal(
            response,
            StatusCode::BAD_REQUEST,
            ModelRefusalCode::MalformedRequest,
        )
        .await;
    }
    for id in [
        "0",
        "01",
        "+1",
        "-1",
        "1e3",
        "1.0",
        "9223372036854775808",
        "1%20",
        "%FF",
    ] {
        let response = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            send(
                &state.router,
                request(
                    "GET",
                    &format!("{DRAFTS}/{id}/base?expected_version=1&expected_base=1"),
                    Body::empty(),
                ),
            ),
        )
        .await
        .unwrap();
        refusal(
            response,
            StatusCode::BAD_REQUEST,
            ModelRefusalCode::MalformedRequest,
        )
        .await;
    }
    assert_eq!(state.store_calls.load(Ordering::SeqCst), 0);
    drop(held);
}
