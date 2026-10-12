use super::super::{SignInConfig, router};
use super::support::*;
use axum::{
    body::Body,
    http::{HeaderMap, Method, StatusCode, header},
};

#[tokio::test]
async fn preview_disables_identity_without_publishing_administrator_or_wildcard_routes() {
    let shell = router(None);
    let response = call(
        shell.clone(),
        Method::GET,
        "/api/identity-config",
        HeaderMap::new(),
        Body::empty(),
    )
    .await;
    expect_no_store(&response, StatusCode::OK);
    assert_eq!(json(response).await["enabled"], false);
    let response = call(
        shell.clone(),
        Method::GET,
        "/api/ontology-status/ten_acme",
        headers(),
        Body::empty(),
    )
    .await;
    expect_no_store(&response, StatusCode::SERVICE_UNAVAILABLE);
    for path in [
        "/api/identity/ten_acme/passkeys/enrollment-options",
        "/api/arbitrary-control",
        "/api/identity/ten_acme/Users",
    ] {
        let response = call(shell.clone(), Method::POST, path, headers(), "{}").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
    let response = call(
        shell,
        Method::POST,
        "/api/identity/ten_acme/session",
        headers(),
        "{}",
    )
    .await;
    expect_no_store(&response, StatusCode::METHOD_NOT_ALLOWED);
    assert!(
        SignInConfig::new(
            "https://console.example",
            "http://iam.example",
            "https://foundry.example"
        )
        .is_err()
    );
    for bad in [
        "https://console.example/",
        "https://user:pass@console.example",
        "https://console.example/path",
        "https://console.example?query",
        "https://console.example#fragment",
    ] {
        assert!(SignInConfig::new(bad, "https://iam.example", "https://foundry.example").is_err());
    }
    assert!(
        SignInConfig::for_loopback(
            "http://console.example",
            "http://127.0.0.1",
            "http://localhost"
        )
        .is_err()
    );
}

#[tokio::test]
async fn mutations_require_single_exact_origin_content_type_and_bounded_body() {
    let iam = Fixture::new("{}").await;
    let foundry = Fixture::new(FACTS).await;
    for op in [
        "passkeys/enroll",
        "passkeys/authentication-options",
        "passkeys/authenticate",
        "session:logout",
    ] {
        for origin in [
            None,
            Some("https://attacker.example"),
            Some("null"),
            Some("http://localhost:3000/"),
        ] {
            let mut h = headers();
            h.remove(header::ORIGIN);
            if let Some(origin) = origin {
                h.insert(header::ORIGIN, origin.parse().unwrap());
            }
            let response = call(
                iam.shell(&foundry),
                Method::POST,
                &format!("/api/identity/ten_acme/{op}"),
                h,
                "{}",
            )
            .await;
            expect_no_store(&response, StatusCode::FORBIDDEN);
        }
    }
    let mut h = headers();
    h.append(header::ORIGIN, ORIGIN.parse().unwrap());
    let response = call(
        iam.shell(&foundry),
        Method::POST,
        "/api/identity/ten_acme/passkeys/enroll",
        h,
        "{}",
    )
    .await;
    expect_no_store(&response, StatusCode::FORBIDDEN);
    let mut h = headers();
    h.remove(header::CONTENT_TYPE);
    let response = call(
        iam.shell(&foundry),
        Method::POST,
        "/api/identity/ten_acme/passkeys/enroll",
        h,
        "{}",
    )
    .await;
    expect_no_store(&response, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let response = call(
        iam.shell(&foundry),
        Method::POST,
        "/api/identity/ten_acme/passkeys/enroll",
        headers(),
        "x".repeat(65537),
    )
    .await;
    expect_no_store(&response, StatusCode::PAYLOAD_TOO_LARGE);
    let response = call(
        iam.shell(&foundry),
        Method::GET,
        "/api/identity/anonymous/session",
        headers(),
        "",
    )
    .await;
    expect_no_store(&response, StatusCode::BAD_REQUEST);
    assert!(iam.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn only_canonical_single_cookie_and_required_headers_reach_fixed_endpoint() {
    let iam = Fixture::new("{}").await;
    let foundry = Fixture::new(FACTS).await;
    let mut h = headers();
    h.insert(
        header::AUTHORIZATION,
        "Bearer administrator-secret".parse().unwrap(),
    );
    h.insert("x-identity", "forged-user".parse().unwrap());
    let response = call(
        iam.shell(&foundry),
        Method::POST,
        "/api/identity/ten_acme/passkeys/authentication-options",
        h,
        r#"{"userId":"usr_alice"}"#,
    )
    .await;
    expect_no_store(&response, StatusCode::OK);
    {
        let calls = iam.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        let (path, h, b) = &calls[0];
        assert_eq!(
            path,
            "/identity/v1/ten_acme/passkeys/authentication-options"
        );
        assert_eq!(h[header::ORIGIN], ORIGIN);
        assert!(!h.contains_key(header::COOKIE));
        assert!(!h.contains_key(header::AUTHORIZATION));
        assert!(!h.contains_key("x-identity"));
        assert_eq!(b.as_ref(), br#"{"userId":"usr_alice"}"#);
    }
    for value in [
        format!("{}; {}", cookie(), cookie()),
        "__Host-oya_session=bad".into(),
    ] {
        let mut h = headers();
        h.insert(header::COOKIE, value.parse().unwrap());
        let response = call(
            iam.shell(&foundry),
            Method::GET,
            "/api/identity/ten_acme/session",
            h,
            "",
        )
        .await;
        expect_no_store(&response, StatusCode::UNAUTHORIZED);
    }
    let response = call(
        iam.shell(&foundry),
        Method::GET,
        "/api/identity/ten_acme/session",
        HeaderMap::new(),
        "",
    )
    .await;
    expect_no_store(&response, StatusCode::UNAUTHORIZED);
    assert_eq!(iam.calls.lock().unwrap().len(), 1);
}
