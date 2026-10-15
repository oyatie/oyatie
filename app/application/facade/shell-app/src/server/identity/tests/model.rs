use super::support::*;
use application_ontology_read::*;
use axum::{
    body::Body,
    http::{HeaderMap, Method, StatusCode, header},
};
fn workspace() -> String {
    String::from_utf8(
        ModelWorkspace {
            generation: ModelNumber::parse("9007199254740993").unwrap(),
            document: ModelDocument {
                named_string_types: vec![],
                entities: vec![],
                links: vec![],
            },
            capabilities: ModelCapabilities {
                author: true,
                publish: false,
            },
        }
        .encode()
        .unwrap(),
    )
    .unwrap()
}
#[tokio::test]
async fn model_proxy_checks_fresh_identity_headers_no_store_and_server_origin_before_upstream() {
    let iam=Fixture::new(&serde_json::json!({"tenantId":"ten_acme","userId":"usr_alice","expiresAt":now()+3600,"roles":["foundry-model-author"]}).to_string()).await;
    let foundry = Fixture::new(&workspace()).await;
    let shell = iam.shell(&foundry);
    for path in [
        "/api/ontology/ten_acme/model",
        "/api/ontology/ten_acme/model/drafts",
        "/api/ontology/ten_acme/model/drafts/9007199254740993",
    ] {
        expect_no_store(
            &call(
                shell.clone(),
                Method::GET,
                path,
                HeaderMap::new(),
                Body::empty(),
            )
            .await,
            StatusCode::UNAUTHORIZED,
        );
    }
    assert!(foundry.calls.lock().unwrap().is_empty());
    let response = call(
        shell.clone(),
        Method::GET,
        "/api/ontology/ten_acme/model",
        headers(),
        Body::empty(),
    )
    .await;
    expect_no_store(&response, StatusCode::OK);
    assert_eq!(response.headers()["x-oyatie-user"], "usr_alice");
    assert_eq!(response.headers()["x-oyatie-tenant"], "ten_acme");
    assert_eq!(json(response).await["generation"], "9007199254740993");
    for path in [
        "/api/ontology/ten_acme/model/drafts",
        "/api/ontology/ten_acme/model/drafts/1/save",
        "/api/ontology/ten_acme/model/drafts/1/validate",
    ] {
        let mut wrong = headers();
        wrong.insert(header::ORIGIN, "https://foreign.invalid".parse().unwrap());
        expect_no_store(
            &call(shell.clone(), Method::POST, path, wrong, "{}").await,
            StatusCode::FORBIDDEN,
        );
        let mut duplicate = headers();
        duplicate.append(header::CONTENT_TYPE, "application/json".parse().unwrap());
        expect_no_store(
            &call(shell.clone(), Method::POST, path, duplicate, "{}").await,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        );
    }
    assert_eq!(foundry.calls.lock().unwrap().len(), 1);
    iam.reply.lock().unwrap().body=serde_json::json!({"tenantId":"ten_acme","userId":"usr_alice","expiresAt":1,"roles":["foundry-model-author"]}).to_string();
    expect_no_store(
        &call(
            shell,
            Method::GET,
            "/api/ontology/ten_acme/model",
            headers(),
            Body::empty(),
        )
        .await,
        StatusCode::UNAUTHORIZED,
    );
    assert_eq!(foundry.calls.lock().unwrap().len(), 1);
}
#[tokio::test]
async fn model_proxy_preserves_exact_request_ack_and_keeps_closed_malformed_shapes_local() {
    let iam = Fixture::new(&session()).await;
    let draft = serde_json::json!({"id":"9007199254740993","tenant_id":"ten_acme","owner":"usr_alice","base_generation":"1","version":"1","document":{"entities":[],"links":[]},"publication":null});
    let foundry = Fixture::new(&draft.to_string()).await;
    foundry.reply.lock().unwrap().status = StatusCode::CREATED;
    let shell = iam.shell(&foundry);
    let create = CreateModelDraft {
        expected_generation: ModelNumber::new(1).unwrap(),
        document: ModelDocument {
            named_string_types: vec![],
            entities: vec![],
            links: vec![],
        },
    };
    let response = call(
        shell.clone(),
        Method::POST,
        "/api/ontology/ten_acme/model/drafts",
        headers(),
        create.encode().unwrap(),
    )
    .await;
    expect_no_store(&response, StatusCode::CREATED);
    assert_eq!(json(response).await, draft);
    {
        let calls = foundry.calls.lock().unwrap();
        assert_eq!(calls[0].0, "/v1/tenants/ten_acme/model/drafts");
        assert_eq!(calls[0].1[header::ORIGIN], ORIGIN);
        assert_eq!(calls[0].1[header::COOKIE], cookie());
        assert!(!calls[0].1.contains_key(header::AUTHORIZATION));
    }
    for body in [
        r#"{"expected_generation":1,"document":{"entities":[],"links":[]}}"#,
        r#"{"expected_generation":"1","expected_generation":"1","document":{"entities":[],"links":[]}}"#,
        r#"{"expected_generation":"1","tenant_id":"ten_other","document":{"entities":[],"links":[]}}"#,
        r#"["1",{"entities":[],"links":[]}]"#,
    ] {
        let response = call(
            shell.clone(),
            Method::POST,
            "/api/ontology/ten_acme/model/drafts",
            headers(),
            body,
        )
        .await;
        expect_no_store(&response, StatusCode::BAD_REQUEST);
        assert_eq!(json(response).await["error"], "malformed_request");
    }
    assert_eq!(foundry.calls.lock().unwrap().len(), 1);
    for query in [
        "limit=0",
        "limit=257",
        "limit=01",
        "after=9007199254740993&after=2",
        "after=9223372036854775808",
        "after=1&roles=author",
    ] {
        expect_no_store(
            &call(
                shell.clone(),
                Method::GET,
                &format!("/api/ontology/ten_acme/model/drafts?{query}"),
                headers(),
                Body::empty(),
            )
            .await,
            StatusCode::BAD_REQUEST,
        );
    }
    assert_eq!(foundry.calls.lock().unwrap().len(), 1);
    foundry.reply.lock().unwrap().body = draft.to_string().replace("usr_alice", "usr_foreign");
    let response = call(
        shell,
        Method::POST,
        "/api/ontology/ten_acme/model/drafts",
        headers(),
        create.encode().unwrap(),
    )
    .await;
    expect_no_store(&response, StatusCode::SERVICE_UNAVAILABLE);
    assert_ne!(json(response).await, draft);
}
fn now() -> i64 {
    super::super::config::now()
}

#[tokio::test]
async fn model_extractor_refusals_keep_closed_semantic_bodies_and_no_store() {
    let iam = Fixture::new(&session()).await;
    let foundry = Fixture::new("{}").await;
    let shell = iam.shell(&foundry);
    for (method, path) in [
        (Method::GET, "/api/ontology/ten_%FF/model"),
        (Method::GET, "/api/ontology/ten_%FF/model/drafts"),
        (Method::GET, "/api/ontology/ten_acme/model/drafts/%FF"),
        (Method::POST, "/api/ontology/ten_%FF/model/drafts"),
        (Method::POST, "/api/ontology/ten_acme/model/drafts/%FF/save"),
        (
            Method::POST,
            "/api/ontology/ten_acme/model/drafts/%FF/validate",
        ),
    ] {
        let response = call(shell.clone(), method, path, headers(), "{}").await;
        expect_no_store(&response, StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(
            body.as_ref(),
            br#"{"error":"malformed_request"}"#,
            "extractor refusals remain closed model data"
        );
    }
    let maximum = model_wire_limits()
        .unwrap()
        .create
        .max(model_wire_limits().unwrap().save);
    let response = call(
        shell,
        Method::POST,
        "/api/ontology/ten_acme/model/drafts",
        headers(),
        vec![b' '; maximum + 1],
    )
    .await;
    expect_no_store(&response, StatusCode::PAYLOAD_TOO_LARGE);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    assert_eq!(body.as_ref(), br#"{"error":"body_too_large"}"#);
    assert!(foundry.calls.lock().unwrap().is_empty());
}
