use super::support::*;
use axum::http::{Method, StatusCode, header};

#[tokio::test]
async fn sessions_are_native_tenant_bound_fresh_and_expiry_checked() {
    let iam = Fixture::new(&session()).await;
    let foundry = Fixture::new(FACTS).await;
    let response = call(
        iam.shell(&foundry),
        Method::GET,
        "/api/identity/ten_acme/session",
        headers(),
        "",
    )
    .await;
    expect_no_store(&response, StatusCode::OK);
    assert_eq!(json(response).await["userId"], "usr_alice");
    let response = call(
        iam.shell(&foundry),
        Method::GET,
        "/api/identity/ten_other/session",
        headers(),
        "",
    )
    .await;
    expect_no_store(&response, StatusCode::UNAUTHORIZED);
    for body in [
        session().replace("usr_alice", "workload"),
        session().replace("foundry-operator", " "),
        r#"{"tenantId":"ten_acme","userId":"usr_alice","expiresAt":1,"roles":[]}"#.into(),
    ] {
        *iam.reply.lock().unwrap() = Reply::json(&body);
        let response = call(
            iam.shell(&foundry),
            Method::GET,
            "/api/identity/ten_acme/session",
            headers(),
            "",
        )
        .await;
        expect_no_store(&response, StatusCode::UNAUTHORIZED);
    }
    *iam.reply.lock().unwrap() = Reply::json(&session().replace("[\"foundry-operator\"]", "[]"));
    let response = call(
        iam.shell(&foundry),
        Method::GET,
        "/api/identity/ten_acme/session",
        headers(),
        "",
    )
    .await;
    expect_no_store(&response, StatusCode::OK);
    assert_eq!(json(response).await["roles"], serde_json::json!([]));
    assert_eq!(iam.calls.lock().unwrap().len(), 6);
}

#[tokio::test]
async fn only_authenticate_and_logout_can_forward_protected_native_cookie() {
    let iam = Fixture::new(&session()).await;
    let foundry = Fixture::new(FACTS).await;
    iam.reply
        .lock()
        .unwrap()
        .headers
        .insert(header::SET_COOKIE, issued().parse().unwrap());
    let response = call(
        iam.shell(&foundry),
        Method::POST,
        "/api/identity/ten_acme/passkeys/authenticate",
        headers(),
        "{}",
    )
    .await;
    expect_no_store(&response, StatusCode::OK);
    assert_eq!(response.headers()[header::SET_COOKIE], issued());
    for op in ["passkeys/authentication-options", "session"] {
        let method = if op == "session" {
            Method::GET
        } else {
            Method::POST
        };
        let response = call(
            iam.shell(&foundry),
            method,
            &format!("/api/identity/ten_acme/{op}"),
            headers(),
            "{}",
        )
        .await;
        expect_no_store(&response, StatusCode::OK);
        assert!(!response.headers().contains_key(header::SET_COOKIE));
    }
    let mut enrollment = Reply::json("");
    enrollment.status = StatusCode::CREATED;
    enrollment
        .headers
        .insert(header::SET_COOKIE, issued().parse().unwrap());
    *iam.reply.lock().unwrap() = enrollment;
    let response = call(
        iam.shell(&foundry),
        Method::POST,
        "/api/identity/ten_acme/passkeys/enroll",
        headers(),
        "{}",
    )
    .await;
    expect_no_store(&response, StatusCode::CREATED);
    assert!(!response.headers().contains_key(header::SET_COOKIE));
    assert!(
        axum::body::to_bytes(response.into_body(), 1)
            .await
            .unwrap()
            .is_empty()
    );
    *iam.reply.lock().unwrap() = Reply::json(&session());
    iam.reply.lock().unwrap().headers.insert(
        header::SET_COOKIE,
        issued().replace("; Secure", "").parse().unwrap(),
    );
    let response = call(
        iam.shell(&foundry),
        Method::POST,
        "/api/identity/ten_acme/passkeys/authenticate",
        headers(),
        "{}",
    )
    .await;
    expect_no_store(&response, StatusCode::BAD_GATEWAY);
    let mut reply = Reply::json("");
    reply.status = StatusCode::NO_CONTENT;
    reply.headers.insert(
        header::SET_COOKIE,
        "__Host-oya_session=; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=0"
            .parse()
            .unwrap(),
    );
    *iam.reply.lock().unwrap() = reply;
    let response = call(
        iam.shell(&foundry),
        Method::POST,
        "/api/identity/ten_acme/session:logout",
        headers(),
        "{}",
    )
    .await;
    expect_no_store(&response, StatusCode::NO_CONTENT);
    assert!(
        response.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .contains("Max-Age=0")
    );
}

#[tokio::test]
async fn live_status_reads_human_endpoint_again_and_preserves_refusals_and_unavailable() {
    let iam = Fixture::new(&session()).await;
    let foundry = Fixture::new(FACTS).await;
    for lag in [7, 8] {
        *foundry.reply.lock().unwrap() = Reply::json(&FACTS.replace(":7", &format!(":{lag}")));
        let response = call(
            iam.shell(&foundry),
            Method::GET,
            "/api/ontology-status/ten_acme",
            headers(),
            "",
        )
        .await;
        expect_no_store(&response, StatusCode::OK);
        assert_eq!(json(response).await["projectionLag"], lag);
    }
    for status in [
        StatusCode::UNAUTHORIZED,
        StatusCode::FORBIDDEN,
        StatusCode::SERVICE_UNAVAILABLE,
    ] {
        foundry.reply.lock().unwrap().status = status;
        let response = call(
            iam.shell(&foundry),
            Method::GET,
            "/api/ontology-status/ten_acme",
            headers(),
            "",
        )
        .await;
        expect_no_store(&response, status);
        assert!(json(response).await.get("projectionLag").is_none());
    }
    let calls = foundry.calls.lock().unwrap();
    assert_eq!(calls.len(), 5);
    for (path, h, _) in calls.iter() {
        assert_eq!(path, "/v1/tenants/ten_acme/status");
        assert_eq!(h[header::COOKIE], cookie());
        assert!(!h.contains_key(header::AUTHORIZATION));
    }
    assert!(iam.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn redirects_oversized_malformed_and_slow_responses_never_become_live_data() {
    let iam = Fixture::new("{}").await;
    let foundry = Fixture::new(FACTS).await;
    for fixture in [&iam, &foundry] {
        let mut reply = Reply::json("{}");
        reply.status = StatusCode::TEMPORARY_REDIRECT;
        reply.headers.insert(
            header::LOCATION,
            format!("{}/trap", fixture.url).parse().unwrap(),
        );
        *fixture.reply.lock().unwrap() = reply;
    }
    for path in [
        "/api/identity/ten_acme/session",
        "/api/ontology-status/ten_acme",
    ] {
        let response = call(iam.shell(&foundry), Method::GET, path, headers(), "").await;
        expect_no_store(&response, StatusCode::SERVICE_UNAVAILABLE);
    }
    assert_eq!(iam.calls.lock().unwrap().len(), 1);
    assert_eq!(foundry.calls.lock().unwrap().len(), 1);
    for body in ["not-json".into(), "x".repeat(65537)] {
        *iam.reply.lock().unwrap() = Reply::json(&body);
        let response = call(
            iam.shell(&foundry),
            Method::GET,
            "/api/identity/ten_acme/session",
            headers(),
            "",
        )
        .await;
        assert!(matches!(
            response.status(),
            StatusCode::BAD_GATEWAY | StatusCode::SERVICE_UNAVAILABLE
        ));
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    }
    *foundry.reply.lock().unwrap() = Reply::json(&"x".repeat(16385));
    let response = call(
        iam.shell(&foundry),
        Method::GET,
        "/api/ontology-status/ten_acme",
        headers(),
        "",
    )
    .await;
    expect_no_store(&response, StatusCode::SERVICE_UNAVAILABLE);
    *iam.reply.lock().unwrap() = Reply::json(&session());
    iam.reply.lock().unwrap().delay_ms = 2400;
    let response = call(
        iam.shell(&foundry),
        Method::GET,
        "/api/identity/ten_acme/session",
        headers(),
        "",
    )
    .await;
    expect_no_store(&response, StatusCode::SERVICE_UNAVAILABLE);
}
