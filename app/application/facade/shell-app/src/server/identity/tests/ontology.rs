use super::support::*;
use axum::{
    body::Body,
    http::{HeaderMap, Method, StatusCode, header},
};

const SUFFIXES: [&str; 5] = [
    "types",
    "objects?type=ety_record&revision=1",
    "objects/ent_alpha?revision=1",
    "objects/ent_alpha/history",
    "audit",
];
#[tokio::test]
async fn finite_ontology_routes_guard_cookie_tenant_and_keep_no_store_on_all_refusals() {
    let iam = Fixture::new(&session()).await;
    let foundry = Fixture::new("[]").await;
    let shell = iam.shell(&foundry);
    for suffix in SUFFIXES {
        let path = format!("/api/ontology/ten_acme/{suffix}");
        let r = call(
            shell.clone(),
            Method::GET,
            &path,
            HeaderMap::new(),
            Body::empty(),
        )
        .await;
        expect_no_store(&r, StatusCode::UNAUTHORIZED);
        assert_eq!(json(r).await["error"], "unauthenticated");
        let mut duplicate = headers();
        duplicate.append(header::COOKIE, cookie().parse().unwrap());
        expect_no_store(
            &call(shell.clone(), Method::GET, &path, duplicate, Body::empty()).await,
            StatusCode::UNAUTHORIZED,
        );
        expect_no_store(
            &call(shell.clone(), Method::POST, &path, headers(), Body::empty()).await,
            StatusCode::METHOD_NOT_ALLOWED,
        );
    }
    expect_no_store(
        &call(
            shell.clone(),
            Method::GET,
            "/api/ontology/wrong/types",
            headers(),
            Body::empty(),
        )
        .await,
        StatusCode::BAD_REQUEST,
    );
    expect_no_store(
        &call(
            shell.clone(),
            Method::GET,
            "/api/ontology/ten_acme/objects?unknown=x",
            HeaderMap::new(),
            Body::empty(),
        )
        .await,
        StatusCode::UNAUTHORIZED,
    );
    expect_no_store(
        &call(
            shell,
            Method::GET,
            "/api/ontology/ten_acme/objects?unknown=x",
            headers(),
            Body::empty(),
        )
        .await,
        StatusCode::BAD_REQUEST,
    );
    assert!(foundry.calls.lock().unwrap().is_empty());
    assert_eq!(iam.calls.lock().unwrap().len(), 1);
}
#[tokio::test]
async fn shell_form_query_decodes_once_and_native_http_keeps_exact_literal_filter_bytes() {
    let iam = Fixture::new(&session()).await;
    let foundry = Fixture::new(r#"{"objects":[],"next":null}"#).await;
    let r=call(iam.shell(&foundry),Method::GET,"/api/ontology/ten_acme/objects?type=ety_record&revision=1&limit=2&cursor=ent_alpha&property=name&equals=text%3AAda%2B%2520%3D%3A%3F",headers(),Body::empty()).await;
    expect_no_store(&r, StatusCode::OK);
    assert_eq!(json(r).await["requested_revision"], 1);
    let calls = foundry.calls.lock().unwrap();
    assert_eq!(
        calls[0].0,
        "/v1/tenants/ten_acme/objects?type=ety_record&revision=1&limit=2&cursor=ent_alpha&property=name&equals=text:Ada+%20=:?"
    );
    assert_eq!(calls[0].1[header::COOKIE], cookie());
    assert!(!calls[0].1.contains_key(header::AUTHORIZATION));
}
#[tokio::test]
async fn native_typed_errors_do_not_collapse_into_empty_or_sample_success() {
    let iam = Fixture::new(&session()).await;
    let foundry = Fixture::new("{}").await;
    for (status, body, error) in [
        (StatusCode::UNAUTHORIZED, "{}", "unauthenticated"),
        (StatusCode::FORBIDDEN, "{}", "forbidden"),
        (
            StatusCode::BAD_REQUEST,
            r#"{"gate":"surface","cause":"bad query"}"#,
            "invalid_query",
        ),
        (
            StatusCode::NOT_FOUND,
            r#"{"gate":"surface","cause":"the durable projection holds no object at this reference"}"#,
            "absent_object",
        ),
        (
            StatusCode::CONFLICT,
            r#"{"gate":"surface","cause":"that revision was never accepted for this entity type"}"#,
            "revision_conflict",
        ),
        (
            StatusCode::CONFLICT,
            r#"{"gate":"surface","cause":"the stored values of that property are of another kind"}"#,
            "filter_conflict",
        ),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "secret backend body",
            "source_unavailable",
        ),
    ] {
        {
            let mut reply = foundry.reply.lock().unwrap();
            reply.status = status;
            reply.body = body.into();
        }
        let r = call(
            iam.shell(&foundry),
            Method::GET,
            "/api/ontology/ten_acme/objects/ent_alpha?revision=1",
            headers(),
            Body::empty(),
        )
        .await;
        expect_no_store(&r, status);
        assert_eq!(json(r).await, serde_json::json!({"error":error}));
    }
}

#[tokio::test]
async fn expired_verified_session_precedes_invalid_or_unrepresentable_query_refusal() {
    let iam=Fixture::new(&serde_json::json!({"tenantId":"ten_acme","userId":"usr_alice","expiresAt":1,"roles":["foundry-operator"]}).to_string()).await;
    let foundry = Fixture::new("[]").await;
    for path in [
        "objects?unknown=x",
        "objects?type=ety_record&revision=1&property=name&equals=text%3AAda%20Lovelace",
        "objects/ent_alpha?revision=invalid",
    ] {
        let r = call(
            iam.shell(&foundry),
            Method::GET,
            &format!("/api/ontology/ten_acme/{path}"),
            headers(),
            Body::empty(),
        )
        .await;
        expect_no_store(&r, StatusCode::UNAUTHORIZED);
        assert_eq!(json(r).await["error"], "unauthenticated");
    }
    assert_eq!(iam.calls.lock().unwrap().len(), 3);
    assert!(foundry.calls.lock().unwrap().is_empty());
}
