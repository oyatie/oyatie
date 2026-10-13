use crate::model_support::*;
use axum::http::{HeaderValue, header};

#[tokio::test]
async fn encoded_invalid_utf8_paths_return_closed_refusals_before_identity_or_store_access() {
    let fixture = crate::facade_support::Fixture::new("model-path-refusal");
    let state = instrument(fixture.state(), &["foundry-model-author"], Some(origin()));
    let held = state.app.tenants[TENANT].lock().await;
    let mut failures = Vec::new();
    for (method, path) in [
        ("GET", "/v1/tenants/%FF/model"),
        ("GET", "/v1/tenants/%FF/model/drafts?limit=1"),
        ("POST", "/v1/tenants/%FF/model/drafts"),
        ("GET", "/v1/tenants/%FF/model/drafts/1"),
        ("POST", "/v1/tenants/%FF/model/drafts/1/save"),
        ("POST", "/v1/tenants/%FF/model/drafts/1/validate"),
        ("GET", "/v1/tenants/ten_acme/model/drafts/%C3%28"),
        ("POST", "/v1/tenants/ten_acme/model/drafts/%C3%28/save"),
        ("POST", "/v1/tenants/ten_acme/model/drafts/%C3%28/validate"),
    ] {
        let response = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            send(&state.router, request(method, path, Body::empty())),
        )
        .await
        .unwrap();
        let content_type = response.headers().get(header::CONTENT_TYPE).cloned();
        let bytes = body(response, StatusCode::BAD_REQUEST).await;
        let decoded = ModelRefusal::decode(&bytes);
        if content_type.as_ref().map(HeaderValue::as_bytes) != Some(b"application/json")
            || decoded.as_ref().map(|refusal| refusal.error)
                != Ok(ModelRefusalCode::MalformedRequest)
        {
            failures.push(format!(
                "{method} {path}: content_type={content_type:?}, body={:?}, decoded={decoded:?}",
                String::from_utf8_lossy(&bytes)
            ));
        }
    }
    assert_eq!(state.sessions.calls.load(Ordering::SeqCst), 0);
    assert_eq!(state.pdp.calls.load(Ordering::SeqCst), 0);
    assert_eq!(state.store_calls.load(Ordering::SeqCst), 0);
    drop(held);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[tokio::test]
async fn origin_content_type_closed_shapes_and_queries_refuse_before_held_store_access() {
    let fixture = crate::facade_support::Fixture::new("model-transport-refusal");
    let state = instrument(fixture.state(), &["foundry-model-author"], Some(origin()));
    let held = state.app.tenants[TENANT].lock().await;
    let payload = CreateModelDraft {
        expected_generation: number(1),
        document: changed(),
    }
    .encode()
    .unwrap();
    for (name, value, status, code) in [
        (
            header::ORIGIN,
            None,
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            header::ORIGIN,
            Some("https://other.example"),
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            header::ORIGIN,
            Some("null"),
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            header::CONTENT_TYPE,
            None,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ModelRefusalCode::UnsupportedContentType,
        ),
        (
            header::CONTENT_TYPE,
            Some("text/plain"),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ModelRefusalCode::UnsupportedContentType,
        ),
        (
            header::CONTENT_TYPE,
            Some("application/json; charset=utf-8"),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ModelRefusalCode::UnsupportedContentType,
        ),
    ] {
        let mut input = request("POST", DRAFTS, payload.clone());
        input.headers_mut().remove(&name);
        if let Some(value) = value {
            input
                .headers_mut()
                .insert(name, HeaderValue::from_str(value).unwrap());
        }
        refusal(
            tokio::time::timeout(
                std::time::Duration::from_millis(200),
                send(&state.router, input),
            )
            .await
            .unwrap(),
            status,
            code,
        )
        .await;
    }
    for (header, value, status, code) in [
        (
            header::ORIGIN,
            ORIGIN,
            StatusCode::FORBIDDEN,
            ModelRefusalCode::Forbidden,
        ),
        (
            header::CONTENT_TYPE,
            "application/json",
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ModelRefusalCode::UnsupportedContentType,
        ),
    ] {
        let mut input = request("POST", DRAFTS, payload.clone());
        input
            .headers_mut()
            .append(header, HeaderValue::from_static(value));
        refusal(
            tokio::time::timeout(
                std::time::Duration::from_millis(200),
                send(&state.router, input),
            )
            .await
            .unwrap(),
            status,
            code,
        )
        .await;
    }
    let bytes = String::from_utf8(payload.clone()).unwrap();
    for wire in [
        "[]".into(),
        "{}".into(),
        format!("{{\"owner\":\"usr_other\",{}", &bytes[1..]),
        format!("{{\"roles\":[\"foundry-model-author\"],{}", &bytes[1..]),
        format!("{{\"tenant_id\":\"ten_other\",{}", &bytes[1..]),
        format!("{{\"expected_generation\":\"1\",{}", &bytes[1..]),
        bytes.replace("\"expected_generation\":\"1\"", "\"expected_generation\":1"),
    ] {
        refusal(
            tokio::time::timeout(
                std::time::Duration::from_millis(200),
                send(&state.router, request("POST", DRAFTS, wire)),
            )
            .await
            .unwrap(),
            StatusCode::BAD_REQUEST,
            ModelRefusalCode::MalformedRequest,
        )
        .await;
    }
    for query in [
        "",
        "?limit=0",
        "?limit=257",
        "?limit=1&limit=2",
        "?limit=1&after=0",
        "?limit=1&after=01",
        "?limit=1&after=9223372036854775808",
        "?limit=1&after=1&after=2",
        "?limit=1&unknown=1",
    ] {
        refusal(
            tokio::time::timeout(
                std::time::Duration::from_millis(200),
                send(
                    &state.router,
                    request("GET", &format!("{DRAFTS}{query}"), Body::empty()),
                ),
            )
            .await
            .unwrap(),
            StatusCode::BAD_REQUEST,
            ModelRefusalCode::MalformedRequest,
        )
        .await;
    }
    let too_large = vec![b' '; model_wire_limits().unwrap().create + 1];
    refusal(
        tokio::time::timeout(
            std::time::Duration::from_millis(200),
            send(&state.router, request("POST", DRAFTS, too_large.clone())),
        )
        .await
        .unwrap(),
        StatusCode::PAYLOAD_TOO_LARGE,
        ModelRefusalCode::BodyTooLarge,
    )
    .await;
    let chunked = crate::chunked::body(too_large);
    refusal(
        tokio::time::timeout(
            std::time::Duration::from_millis(200),
            send(&state.router, request("POST", DRAFTS, chunked)),
        )
        .await
        .unwrap(),
        StatusCode::PAYLOAD_TOO_LARGE,
        ModelRefusalCode::BodyTooLarge,
    )
    .await;
    assert_eq!(state.store_calls.load(Ordering::SeqCst), 0);
    drop(held);
}

#[tokio::test]
async fn canonical_large_identifiers_reach_native_cas_without_float_rounding() {
    let fixture = crate::facade_support::Fixture::new("model-decimal-route");
    let state = instrument(fixture.state(), &["foundry-model-author"], Some(origin()));
    for value in [
        9_007_199_254_740_991,
        9_007_199_254_740_992,
        9_007_199_254_740_993,
        MAX_MODEL_VERSION,
    ] {
        refusal(
            send(
                &state.router,
                request("GET", &format!("{DRAFTS}/{value}"), Body::empty()),
            )
            .await,
            StatusCode::NOT_FOUND,
            ModelRefusalCode::DraftNotFound,
        )
        .await;
        let input = CreateModelDraft {
            expected_generation: number(value),
            document: changed(),
        };
        refusal(
            send(
                &state.router,
                request("POST", DRAFTS, input.encode().unwrap()),
            )
            .await,
            StatusCode::CONFLICT,
            ModelRefusalCode::HeadConflict,
        )
        .await;
    }
    for value in ["0", "01", "+1", "1e3", "9223372036854775808"] {
        refusal(
            send(
                &state.router,
                request("GET", &format!("{DRAFTS}/{value}"), Body::empty()),
            )
            .await,
            StatusCode::BAD_REQUEST,
            ModelRefusalCode::MalformedRequest,
        )
        .await;
    }
}
