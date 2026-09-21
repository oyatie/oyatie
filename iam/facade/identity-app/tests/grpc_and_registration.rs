//! Account registration provisions through SCIM, and gRPC returns identical decisions.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "service_support/mod.rs"]
mod service_support;

use service_support::*;

#[tokio::test(flavor = "multi_thread")]
async fn account_registration_wrapper_provisions_on_live_socket_and_reads_through_scim() {
    let fixture = SigningFixture::generate();
    let handle = boot(&fixture).await;
    let base = format!("http://{}", handle.rest_addr);
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
        "userName": "api-register@acme.example",
        "displayName": "API Registered User",
        "active": true,
    });

    let status = client
        .post(format!("{base}/identity/v1/ten_acme/account-registrations"))
        .json(&payload)
        .send()
        .await
        .expect("account registration responds")
        .status()
        .as_u16();
    assert_eq!(status, 401);

    let token = fixture.mint("wl_provisioner", "scim.manage");
    let response = client
        .post(format!("{base}/identity/v1/ten_acme/account-registrations"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("account registration responds");
    assert_eq!(response.status().as_u16(), 201);
    let created: serde_json::Value = response.json().await.expect("created json");
    let id = created["id"].as_str().expect("assigned id").to_owned();
    assert_eq!(created["userName"], "api-register@acme.example");

    let fetched = client
        .get(format!("{base}/scim/v2/ten_acme/Users/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("scim read responds");
    assert_eq!(fetched.status().as_u16(), 200);
    let fetched: serde_json::Value = fetched.json().await.expect("fetched json");
    assert_eq!(fetched["id"], id);
    assert_eq!(fetched["userName"], "api-register@acme.example");

    handle.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn grpc_surface_returns_identical_decisions() {
    let fixture = SigningFixture::generate();
    let handle = boot(&fixture).await;
    let endpoint = format!("http://{}", handle.grpc_addr);

    let channel = tonic::transport::Endpoint::from_shared(endpoint)
        .expect("endpoint")
        .connect()
        .await
        .expect("grpc connects");

    // ValidateToken: ok + projected principal.
    let token = fixture.mint("wl_secrets_sync", "cloud.kms.decrypt");
    let mut validator =
        proto::workload_token_validator_client::WorkloadTokenValidatorClient::new(channel.clone());
    let mut validate_req = tonic::Request::new(proto::ValidateTokenRequest {
        token: token.clone(),
    });
    validate_req.metadata_mut().insert(
        "authorization",
        format!("Bearer {LIFECYCLE_BEARER}")
            .parse()
            .expect("ascii bearer"),
    );
    let response = validator
        .validate_token(validate_req)
        .await
        .expect("validate rpc")
        .into_inner();
    assert!(response.ok, "valid ES256 token must validate over gRPC");

    // AuthorizeWithToken: ALLOW for the permitted principal, DENY (as a
    // response value, not an RPC error) for the scope-denied principal.
    let mut authorizer = proto::workload_authorizer_client::WorkloadAuthorizerClient::new(channel);
    let request = |token: String| {
        let mut req = tonic::Request::new(proto::AuthorizeWithTokenRequest {
            token,
            action: "cloud.kms.Decrypt".into(),
            resource: Some(proto::Resource {
                resource_type: "Secret".into(),
                resource_id: "sec_db_creds".into(),
                attributes: Default::default(),
            }),
            context: Default::default(),
        });
        req.metadata_mut().insert(
            "authorization",
            format!("Bearer {LIFECYCLE_BEARER}")
                .parse()
                .expect("ascii bearer"),
        );
        req
    };
    let allow = authorizer
        .authorize_with_token(request(token))
        .await
        .expect("authorize rpc")
        .into_inner();
    assert_eq!(allow.effect, proto::DecisionEffect::Allow as i32);

    let deny = authorizer
        .authorize_with_token(request(fixture.mint("wl_denied", "cloud.kms.encrypt")))
        .await
        .expect("deny is a response value, never an RPC error")
        .into_inner();
    assert_eq!(deny.effect, proto::DecisionEffect::Deny as i32);

    handle.shutdown().await;
}
