use super::support::*;
use axum::{
    body::Body,
    http::{Method, StatusCode, header},
};
use serde_json::json;

#[path = "ontology_relationships_support.rs"]
mod support;
use support::{Backend, PATH, graph, query};

#[tokio::test]
async fn one_hop_proxy_preserves_exact_wire_cookie_identity_and_independent_pins() {
    let iam = Fixture::new(&session()).await;
    let native = Backend::new().await;
    let shell = native.shell(&iam);
    let response = call(
        shell.clone(),
        Method::POST,
        PATH,
        headers(),
        query().to_string(),
    )
    .await;
    expect_no_store(&response, StatusCode::OK);
    assert_eq!(response.headers()["x-oyatie-tenant"], "ten_acme");
    assert_eq!(response.headers()["x-oyatie-user"], "usr_alice");
    assert_eq!(json(response).await, graph());
    {
        let calls = native.calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0, Method::GET);
        assert_eq!(calls[0].1, "/v1/tenants/ten_acme/link-types");
        assert_eq!(calls[1].0, Method::POST);
        assert_eq!(calls[1].1, "/v1/tenants/ten_acme/search-around");
        for (_, _, headers, _) in calls.iter() {
            assert_eq!(headers[header::COOKIE], cookie());
            assert!(!headers.contains_key(header::AUTHORIZATION));
        }
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&calls[1].3).unwrap(),
            query()
        );
    }
    iam.reply.lock().unwrap().body = session().replace("ten_acme", "ten_other");
    let response = call(shell, Method::POST, PATH, headers(), query().to_string()).await;
    expect_no_store(&response, StatusCode::UNAUTHORIZED);
    assert_eq!(json(response).await["error"], "unauthenticated");
    assert_eq!(native.calls.lock().unwrap().len(), 2);
    assert_eq!(iam.calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn origin_media_body_shape_and_scope_refuse_before_any_native_request() {
    let iam = Fixture::new(&session()).await;
    let native = Backend::new().await;
    let shell = native.shell(&iam);
    let mut cases = Vec::new();
    for value in [None, Some("https://untrusted.invalid")] {
        let mut h = headers();
        h.remove(header::ORIGIN);
        if let Some(value) = value {
            h.insert(header::ORIGIN, value.parse().unwrap());
        }
        cases.push((h, query().to_string(), StatusCode::FORBIDDEN));
    }
    let mut duplicate = headers();
    duplicate.append(header::ORIGIN, ORIGIN.parse().unwrap());
    cases.push((duplicate, query().to_string(), StatusCode::FORBIDDEN));
    for value in [
        None,
        Some("text/plain"),
        Some("application/json; charset=utf-8"),
    ] {
        let mut h = headers();
        h.remove(header::CONTENT_TYPE);
        if let Some(value) = value {
            h.insert(header::CONTENT_TYPE, value.parse().unwrap());
        }
        cases.push((h, query().to_string(), StatusCode::UNSUPPORTED_MEDIA_TYPE));
    }
    let mut duplicate = headers();
    duplicate.append(header::CONTENT_TYPE, "application/json".parse().unwrap());
    cases.push((
        duplicate,
        query().to_string(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
    ));
    for body in ["[]".into(), "{".into(), format!("{} ", " ".repeat(4096))] {
        let status = if body.len() > 4096 {
            StatusCode::PAYLOAD_TOO_LARGE
        } else {
            StatusCode::BAD_REQUEST
        };
        cases.push((headers(), body, status));
    }
    for field in ["tenant", "principal", "consent", "max_depth"] {
        let mut body = query();
        body[field] = json!("untrusted");
        cases.push((headers(), body.to_string(), StatusCode::BAD_REQUEST));
    }
    let mut nested = query();
    nested["root"]["extra"] = json!(true);
    cases.push((headers(), nested.to_string(), StatusCode::BAD_REQUEST));
    let duplicated = query()
        .to_string()
        .replace("\"revision\":1", "\"revision\":1,\"revision\":1");
    cases.push((headers(), duplicated, StatusCode::BAD_REQUEST));
    for (headers, body, status) in cases {
        let response = call(shell.clone(), Method::POST, PATH, headers, body).await;
        expect_no_store(&response, status);
        assert_eq!(
            json(response).await["error"],
            if status == StatusCode::FORBIDDEN {
                "forbidden"
            } else {
                "invalid_query"
            }
        );
    }
    let mut anonymous = headers();
    anonymous.remove(header::COOKIE);
    let response = call(shell.clone(), Method::POST, PATH, anonymous, Body::empty()).await;
    expect_no_store(&response, StatusCode::UNAUTHORIZED);
    let before = iam.calls.lock().unwrap().len();
    let response = call(
        shell,
        Method::POST,
        "/api/ontology/not-a-tenant/search-around",
        headers(),
        query().to_string(),
    )
    .await;
    expect_no_store(&response, StatusCode::BAD_REQUEST);
    assert_eq!(iam.calls.lock().unwrap().len(), before);
    assert!(native.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn upstream_refusals_and_malformed_graphs_do_not_become_empty_rows() {
    let iam = Fixture::new(&session()).await;
    let native = Backend::new().await;
    let shell = native.shell(&iam);
    for (upstream, body, expected, error) in [
        (401, "{}", 401, "unauthenticated"),
        (403, "{}", 403, "forbidden"),
        (
            400,
            r#"{"gate":"surface","cause":"changed intent"}"#,
            400,
            "invalid_query",
        ),
        (
            409,
            r#"{"gate":"surface","cause":"that revision was never accepted for this entity type"}"#,
            409,
            "revision_conflict",
        ),
        (404, "{}", 503, "source_unavailable"),
        (503, "details", 503, "source_unavailable"),
        (200, "{}", 503, "source_unavailable"),
    ] {
        {
            let mut reply = native.reply.lock().unwrap();
            reply.status = StatusCode::from_u16(upstream).unwrap();
            reply.body = body.into();
        }
        let response = call(
            shell.clone(),
            Method::POST,
            PATH,
            headers(),
            query().to_string(),
        )
        .await;
        expect_no_store(&response, StatusCode::from_u16(expected).unwrap());
        assert_eq!(json(response).await["error"], error);
    }
    let mut truncated = graph();
    truncated["truncated"] = json!(true);
    {
        let mut reply = native.reply.lock().unwrap();
        reply.status = StatusCode::OK;
        reply.body = truncated.to_string();
    }
    let response = call(shell, Method::POST, PATH, headers(), query().to_string()).await;
    expect_no_store(&response, StatusCode::OK);
    assert_eq!(json(response).await, truncated);
}

#[tokio::test]
async fn membership_expiring_during_upstream_read_cannot_emit_identity_or_graph() {
    let expires = super::super::config::now() + 1;
    let iam=Fixture::new(&format!(r#"{{"tenantId":"ten_acme","userId":"usr_alice","expiresAt":{expires},"roles":["foundry-operator"]}}"#)).await;
    let native = Backend::new().await;
    native.reply.lock().unwrap().delay_ms = 1100;
    let response = call(
        native.shell(&iam),
        Method::POST,
        PATH,
        headers(),
        query().to_string(),
    )
    .await;
    expect_no_store(&response, StatusCode::UNAUTHORIZED);
    assert!(!response.headers().contains_key("x-oyatie-user"));
    assert_eq!(json(response).await["error"], "unauthenticated");
    assert_eq!(
        native.calls.lock().unwrap().len(),
        2,
        "expiry follows the catalog and graph reads"
    );
}
