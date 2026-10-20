use super::support::*;
use axum::{
    body::Body,
    http::{HeaderMap, Method, StatusCode},
};

#[tokio::test]
async fn exact_base_proxy_preserves_raw_query_and_decoded_path_with_fresh_identity() {
    let iam = Fixture::new(&session()).await;
    let value = serde_json::json!({"draft_id":"9223372036854775807","draft_version":"9007199254740993","base_generation":"9223372036854775807","document":{"entities":[],"links":[]}});
    let foundry = Fixture::new(&value.to_string()).await;
    let shell = iam.shell(&foundry);
    let prefix = "/api/ontology/ten_acme/model/drafts/9223372036854775807/base";
    let query = "expected_version=9007199254740993&expected_base=9223372036854775807";
    let path = format!("{prefix}?{query}");
    expect_no_store(
        &call(
            shell.clone(),
            Method::GET,
            &path,
            HeaderMap::new(),
            Body::empty(),
        )
        .await,
        StatusCode::UNAUTHORIZED,
    );
    assert!(foundry.calls.lock().unwrap().is_empty());
    for path in [
        path.clone(),
        format!("{prefix}?expected_base=9223372036854775807&expected_version=9007199254740993"),
        path.replace("/9223372036854775807/", "/%39223372036854775807/"),
    ] {
        let response = call(shell.clone(), Method::GET, &path, headers(), Body::empty()).await;
        expect_no_store(&response, StatusCode::OK);
        assert_eq!(response.headers()["x-oyatie-tenant"], "ten_acme");
        assert_eq!(response.headers()["x-oyatie-user"], "usr_alice");
        assert!(response.headers().contains_key("x-oyatie-session-expires"));
        assert_eq!(json(response).await, value);
    }
    assert_eq!(
        foundry.calls.lock().unwrap()[0].0,
        format!("/v1/tenants/ten_acme/model/drafts/9223372036854775807/base?{query}")
    );
    for query in [
        "",
        "expected_version=1",
        "expected_base=1",
        "expected_version=1&expected_base=1&extra=1",
        "expected_version=1&expected_version=1&expected_base=1",
        "expected_version=1&expected_base=1&expected_base=1",
        "expected_version=01&expected_base=1",
        "expected_version=0&expected_base=1",
        "expected_version=9223372036854775808&expected_base=1",
        "expected_version=%31&expected_base=1",
        "expected_%76ersion=1&expected_base=1",
        "expected_version=1&expected_base=%31",
        "expected_version=1&expected_base=1&",
        "expected_version=1&expected_base=1=2",
    ] {
        expect_no_store(
            &call(
                shell.clone(),
                Method::GET,
                &format!("{prefix}?{query}"),
                headers(),
                Body::empty(),
            )
            .await,
            StatusCode::BAD_REQUEST,
        );
    }
    for id in ["0", "01", "%FF", "9223372036854775808"] {
        expect_no_store(
            &call(
                shell.clone(),
                Method::GET,
                &format!("/api/ontology/ten_acme/model/drafts/{id}/base?{query}"),
                headers(),
                Body::empty(),
            )
            .await,
            StatusCode::BAD_REQUEST,
        );
    }
    assert_eq!(
        foundry.calls.lock().unwrap().len(),
        3,
        "malformed path/query never reaches upstream"
    );
    iam.reply.lock().unwrap().body = serde_json::json!({"tenantId":"ten_other","userId":"usr_alice","expiresAt":9999999999_u64,"roles":[]}).to_string();
    expect_no_store(
        &call(shell.clone(), Method::GET, &path, headers(), Body::empty()).await,
        StatusCode::UNAUTHORIZED,
    );
    iam.reply.lock().unwrap().body =
        serde_json::json!({"tenantId":"ten_acme","userId":"usr_alice","expiresAt":1,"roles":[]})
            .to_string();
    expect_no_store(
        &call(shell, Method::GET, &path, headers(), Body::empty()).await,
        StatusCode::UNAUTHORIZED,
    );
    assert_eq!(foundry.calls.lock().unwrap().len(), 3);
}
