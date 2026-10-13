use super::support::*;
use axum::{
    body::Body,
    http::{Method, StatusCode, header},
};

const PATH: &str = "/api/ontology/ten_acme/type-labels";

#[tokio::test]
async fn type_label_forwarding_preserves_typed_rows_and_verifies_fresh_identity() {
    let wire =
        serde_json::json!([{"entity_type":"ety_기록","revision":0,"display_name":"기록 <script>"}]);
    let iam = Fixture::new(&session()).await;
    let foundry = Fixture::new(&wire.to_string()).await;
    let shell = iam.shell(&foundry);
    let response = call(shell.clone(), Method::GET, PATH, headers(), Body::empty()).await;
    expect_no_store(&response, StatusCode::OK);
    assert_eq!(response.headers()["x-oyatie-tenant"], "ten_acme");
    assert_eq!(response.headers()["x-oyatie-user"], "usr_alice");
    assert_eq!(json(response).await, wire);
    {
        let calls = foundry.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "/v1/tenants/ten_acme/type-labels");
        assert_eq!(calls[0].1[header::COOKIE], cookie());
        assert!(!calls[0].1.contains_key(header::AUTHORIZATION));
        assert!(calls[0].2.is_empty());
    }
    iam.reply.lock().unwrap().body = session().replace("ten_acme", "ten_other");
    let response = call(shell, Method::GET, PATH, headers(), Body::empty()).await;
    expect_no_store(&response, StatusCode::UNAUTHORIZED);
    assert_eq!(json(response).await["error"], "unauthenticated");
    assert_eq!(foundry.calls.lock().unwrap().len(), 1);
    assert_eq!(iam.calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn malformed_type_labels_are_unavailable_without_empty_or_sample_fallback() {
    let iam = Fixture::new(&session()).await;
    let foundry =
        Fixture::new(r#"[{"entity_type":"ety_record","revision":1,"display_name":"\u2003"}]"#)
            .await;
    let response = call(
        iam.shell(&foundry),
        Method::GET,
        PATH,
        headers(),
        Body::empty(),
    )
    .await;
    expect_no_store(&response, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        json(response).await,
        serde_json::json!({"error":"source_unavailable"})
    );
}
