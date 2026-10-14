use super::{ontology_native::NativeFixture, support::*};
use axum::{
    body::Body,
    http::{Method, StatusCode, header},
};

#[tokio::test]
async fn point_proxy_refuses_missing_ambiguous_lossy_and_oversized_context_before_foundry() {
    let iam = Fixture::new(&session()).await;
    let foundry = Fixture::new("{}").await;
    let shell = iam.shell(&foundry);
    for query in [
        "",
        "revision=1",
        "type=ety_record",
        "type=&revision=1",
        "type=ety_record&revision=-1",
        "type=ety_record&revision=%2B1",
        "type=ety_record&revision=4294967296",
        "type=ety_a&revision=1&revision=2",
        "type=ety_a&type=ety_b&revision=1",
        "type=ety_a&revision=1&role=foundry-operator",
        "%74ype=ety_a&revision=1",
        "type=ety_%61&revision=1",
        "type=ety_%2b&revision=1",
        "type=ety_%&revision=1",
        "type=ety_%FF&revision=1",
        "type=ety_%C0%AF&revision=1",
        "type=ety_a%20b&revision=1",
        "type=ety_a%00&revision=1",
        "type=ety_a+&revision=1",
        &format!("type=ety_{}&revision=1", "x".repeat(4096)),
    ] {
        let response = call(
            shell.clone(),
            Method::GET,
            &format!("/api/ontology/ten_acme/objects/ent_alpha?{query}"),
            headers(),
            Body::empty(),
        )
        .await;
        expect_no_store(&response, StatusCode::BAD_REQUEST);
        assert_eq!(json(response).await["error"], "invalid_query", "{query}");
    }
    assert!(foundry.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn point_proxy_keeps_exact_captured_type_and_pin_with_guarded_cookie_transport() {
    let iam = Fixture::new(&session()).await;
    let foundry = Fixture::new(r#"{"object_ref":"ent_alpha","written_revision":1,"upcast_state":"current","properties":{}}"#).await;
    let shell = iam.shell(&foundry);
    for entity_type in [
        "ety_record",
        "ety_+%&=#/?",
        "ety_가",
        "ety_a b",
        "ety_\u{feff}",
    ] {
        let mut url =
            reqwest::Url::parse("https://shell.invalid/api/ontology/ten_acme/objects/ent_alpha")
                .unwrap();
        url.query_pairs_mut()
            .append_pair("revision", "4294967295")
            .append_pair("type", entity_type);
        let response = call(
            shell.clone(),
            Method::GET,
            &format!("{}?{}", url.path(), url.query().unwrap()),
            headers(),
            Body::empty(),
        )
        .await;
        expect_no_store(&response, StatusCode::OK);
        assert_eq!(response.headers()["x-oyatie-tenant"], "ten_acme");
        assert_eq!(json(response).await["requested_revision"], u32::MAX);
        let mut expected =
            reqwest::Url::parse("https://foundry.invalid/v1/tenants/ten_acme/objects/ent_alpha")
                .unwrap();
        expected
            .query_pairs_mut()
            .append_pair("type", entity_type)
            .append_pair("revision", "4294967295");
        let calls = foundry.calls.lock().unwrap();
        let (path, sent, _) = calls.last().unwrap();
        assert_eq!(
            path,
            &format!("{}?{}", expected.path(), expected.query().unwrap())
        );
        assert_eq!(sent[header::COOKIE], cookie());
        assert!(!sent.contains_key(header::AUTHORIZATION));
    }
}

#[tokio::test]
async fn real_human_point_proxy_requires_the_captured_type_before_returning_properties() {
    let iam = Fixture::new(&session()).await;
    let native = NativeFixture::new(&iam).await;
    let shell = native.shell(&iam);
    let response = call(
        shell.clone(),
        Method::GET,
        "/api/ontology/ten_acme/objects/ent_alpha?type=ety_record&revision=1",
        headers(),
        Body::empty(),
    )
    .await;
    expect_no_store(&response, StatusCode::OK);
    assert_eq!(
        json(response).await["object"]["properties"]["name"]["value"],
        "Ada"
    );
    for query in ["type=ety_other&revision=1", "type=ety_other&revision=99"] {
        let response = call(
            shell.clone(),
            Method::GET,
            &format!("/api/ontology/ten_acme/objects/ent_alpha?{query}"),
            headers(),
            Body::empty(),
        )
        .await;
        expect_no_store(&response, StatusCode::NOT_FOUND);
        assert_eq!(
            json(response).await,
            serde_json::json!({"error":"absent_object"})
        );
    }
    let response = call(
        shell,
        Method::GET,
        "/api/ontology/ten_acme/objects/ent_alpha?type=ety_record&revision=99",
        headers(),
        Body::empty(),
    )
    .await;
    expect_no_store(&response, StatusCode::CONFLICT);
    assert_eq!(json(response).await["error"], "revision_conflict");
}
