use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use mail_api::MetadataStore;
use mail_kernel::Account;
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";

#[tokio::test]
async fn jmap_reads_smtp_store_and_never_accepts_cross_tenant_account_claim() {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.provision(
        Account::new("b", "other", "bob", "bob@example.org").unwrap(),
        &"x".repeat(32),
    )
    .unwrap();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: preserved\r\n\r\nhello",
    )
    .unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    let app = mail_protocol_imap::jmap_router(service, "http://localhost:8080".into());
    let unauthenticated = app
        .clone()
        .oneshot(
            Request::post("/jmap")
                .header("content-type", "application/json")
                .body(Body::from(vec![b'x'; 1024 * 1024 + 1]))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        unauthenticated.status(),
        StatusCode::UNAUTHORIZED,
        "authentication must precede request body extraction"
    );
    let body = json!({"using":["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"], "methodCalls":[
        ["Email/query", {"accountId":"a"}, "q"],
        ["Email/get", {"accountId":"a", "#ids":{"resultOf":"q", "name":"Email/query", "path":"/ids"}}, "g"],
        ["Mailbox/get", {"accountId":"b"}, "deny"]
    ]});
    let response = app
        .clone()
        .oneshot(
            Request::post("/jmap")
                .header("authorization", format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let value: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(
        value["methodResponses"][1][1]["list"][0]["subject"],
        "preserved"
    );
    assert_eq!(value["methodResponses"][2][1]["type"], "accountNotFound");
    db.revoke("a").unwrap();
    let response = app
        .oneshot(
            Request::get("/.well-known/jmap")
                .header("authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn blob_routes_enforce_account_policy_and_make_downloads_safe() {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.provision(
        Account::new("b", "t", "bob", "bob@example.org").unwrap(),
        &"b".repeat(32),
    )
    .unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    let app = mail_protocol_imap::jmap_router(service, "http://localhost".into());
    let unauthenticated = app
        .clone()
        .oneshot(
            Request::post("/upload/a")
                .body(Body::from(vec![0; mail_kernel::MAX_MESSAGE_BYTES + 1]))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        unauthenticated.status(),
        StatusCode::UNAUTHORIZED,
        "authenticate before accepting an upload body"
    );
    for (account, expected) in [("a", StatusCode::OK), ("b", StatusCode::NOT_FOUND)] {
        let response = app
            .clone()
            .oneshot(
                Request::post(format!("/upload/{account}"))
                    .header("authorization", format!("Bearer {TOKEN}"))
                    .header("content-type", "application/octet-stream")
                    .body(Body::from(vec![0, 255, 13, 10]))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }
    let id = db.put_blob("a", b"\0\xff\r\n").unwrap();
    let url = format!("/download/a/{id}/%0D%0AX-Evil%3Ayes?type=text%2Fhtml");
    let response = app
        .clone()
        .oneshot(
            Request::get(&url)
                .header("authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/html");
    assert_eq!(response.headers()["x-content-type-options"], "nosniff");
    assert!(
        response.headers()["content-disposition"]
            .to_str()
            .unwrap()
            .starts_with("attachment;")
    );
    assert!(!response.headers().contains_key("x-evil"));
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .as_ref(),
        b"\0\xff\r\n"
    );
    for url in [
        format!("/download/b/{id}/file"),
        format!("/download/a/{id}/file?type=text%2Fplain%0D%0AX-Evil%3Ayes"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::get(url)
                    .header("authorization", format!("Bearer {TOKEN}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_client_error());
    }
    db.revoke("a").unwrap();
    let response = app
        .oneshot(
            Request::get(&url)
                .header("authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
