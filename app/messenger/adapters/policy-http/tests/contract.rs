#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use axum::{Json, Router, routing::post};
use messenger_domain::Error;
use messenger_policy_api::{Action, Policy};
use messenger_policy_http::OyatiePolicy;

#[test]
fn hosted_https_requires_workload_identity() {
    assert!(OyatiePolicy::new("https://policy.example.org", "v1", None).is_err());
    assert!(OyatiePolicy::new("https://policy.example.org", "v1", Some(b"not-pem")).is_err());
    assert!(OyatiePolicy::new("http://example.org", "v1", None).is_err());
    assert!(OyatiePolicy::new("http://127.0.0.1:1", "", None).is_err());
}

#[tokio::test]
async fn unavailable_policy_cannot_be_an_allow() {
    let policy = OyatiePolicy::new("http://127.0.0.1:1", "v1", None).unwrap();
    assert!(
        matches!(
            policy
                .authorize("ten_acme", "alice", Action::Send, "!room:local")
                .await,
            Err(Error::Unavailable(_))
        ),
        "transport outage must remain retryable without allowing admission"
    );
}

#[tokio::test]
async fn empty_identity_is_denied_without_transport() {
    let policy = OyatiePolicy::new("http://127.0.0.1:1", "v1", None).unwrap();
    assert!(matches!(
        policy
            .authorize("acme", "usr_alice", Action::Send, "")
            .await,
        Err(Error::Denied)
    ));
}

#[tokio::test]
async fn only_correlated_fresh_complete_allows_pass() {
    for mode in [
        "allow",
        "deny",
        "wrong-request",
        "stale",
        "obligation",
        "unattributed",
        "http-error",
        "overloaded",
        "oversized",
    ] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = Router::new().route(
            "/v1/authorize",
            post(move |Json(body): Json<serde_json::Value>| async move {
                assert_eq!(body["request"]["tenant_id"], "acme");
                assert_eq!(body["request"]["action"], "messenger.message.send");
                assert_eq!(body["entities"][0]["attributes"]["tenant_id"], "acme");
                assert_eq!(
                    body["request"]["resource"]["entity_id"],
                    r#"["acme","!work:local"]"#
                );
                let mut response = serde_json::json!({
                    "request_id": body["request"]["request_id"],
                    "decision_id": "d1",
                    "decision": "allow",
                    "policy_version": "v1",
                    "determining_policy_ids": ["permit-1"],
                    "obligations": []
                });
                match mode {
                    "deny" => response["decision"] = serde_json::json!("deny"),
                    "wrong-request" => response["request_id"] = serde_json::json!("other"),
                    "stale" => response["policy_version"] = serde_json::json!("v0"),
                    "obligation" => {
                        response["obligations"] =
                            serde_json::json!([{"obligation_id":"unknown","parameters":{}}])
                    }
                    "unattributed" => response["determining_policy_ids"] = serde_json::json!([]),
                    "oversized" => response["decision_id"] = serde_json::json!("x".repeat(70_000)),
                    _ => {}
                }
                (
                    match mode {
                        "http-error" => axum::http::StatusCode::SERVICE_UNAVAILABLE,
                        "overloaded" => axum::http::StatusCode::TOO_MANY_REQUESTS,
                        _ => axum::http::StatusCode::OK,
                    },
                    Json(response),
                )
            }),
        );
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let policy = OyatiePolicy::new(&format!("http://{address}"), "v1", None).unwrap();
        let result = policy
            .authorize("acme", "usr_alice", Action::Send, "!work:local")
            .await;
        assert_eq!(result.is_ok(), mode == "allow", "{mode}");
        if matches!(mode, "http-error" | "overloaded" | "oversized") {
            assert!(
                matches!(result, Err(Error::Unavailable(_))),
                "{mode} must remain retryable"
            );
        }
        server.abort();
    }
}
