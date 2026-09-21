//! The REST surface validates, fails closed, and serves JWKS and SCIM on live sockets.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "service_support/mod.rs"]
mod service_support;

use service_support::*;

#[tokio::test(flavor = "multi_thread")]
async fn rest_validate_authorize_and_fail_closed_contract() {
    let fixture = SigningFixture::generate();
    let handle = boot(&fixture).await;
    let base = format!("http://{}", handle.rest_addr);
    let client = reqwest::Client::new();

    // Health surface.
    for probe in ["/healthz", "/livez", "/readyz"] {
        let status = client
            .get(format!("{base}{probe}"))
            .send()
            .await
            .expect("probe responds")
            .status();
        assert_eq!(status.as_u16(), 200, "{probe} must be 200");
    }

    // Acceptance: mint ES256 JWT -> validate -> 200.
    let token = fixture.mint("wl_secrets_sync", "cloud.kms.decrypt");
    let response = client
        .post(format!("{base}/tokens/validate"))
        .bearer_auth(LIFECYCLE_BEARER)
        .json(&serde_json::json!({"token": token}))
        .send()
        .await
        .expect("validate responds");
    assert_eq!(response.status().as_u16(), 200);
    let principal: serde_json::Value = response.json().await.expect("principal json");
    assert_eq!(principal["workloadId"], "wl_secrets_sync");
    assert_eq!(principal["trustDomain"], "spiffe://ten_acme");

    // Permitted principal authorizes: 200 ALLOW.
    let response = client
        .post(format!("{base}/authorize-with-token"))
        .bearer_auth(LIFECYCLE_BEARER)
        .json(&authorize_body(&token))
        .send()
        .await
        .expect("authorize responds");
    assert_eq!(response.status().as_u16(), 200);
    let decision: serde_json::Value = response.json().await.expect("decision json");
    assert_eq!(decision["effect"], "ALLOW");

    // Acceptance: denied principal -> 403 fail-closed, NEVER 404.
    // (1) Known principal without the required scope.
    let denied = fixture.mint("wl_denied", "cloud.kms.encrypt");
    // (2) Validly-signed token for a principal that does not exist.
    let unknown = fixture.mint("wl_ghost", "cloud.kms.decrypt");
    // (3) Suspended principal (revocation denylist).
    let suspended = fixture.mint("wl_suspended", "cloud.kms.decrypt");
    for (label, token) in [
        ("scope-denied", &denied),
        ("unknown-principal", &unknown),
        ("suspended-principal", &suspended),
    ] {
        let status = client
            .post(format!("{base}/authorize-with-token"))
            .bearer_auth(LIFECYCLE_BEARER)
            .json(&authorize_body(token))
            .send()
            .await
            .expect("authorize responds")
            .status()
            .as_u16();
        assert_ne!(status, 404, "{label}: a deny must never be a 404");
        assert_eq!(status, 403, "{label}: deny must fail closed as 403");
    }

    // A garbage credential is a token-validation failure (422), not a 5xx and
    // never an allow.
    let status = client
        .post(format!("{base}/tokens/validate"))
        .bearer_auth(LIFECYCLE_BEARER)
        .json(&serde_json::json!({"token": "not-a-jwt"}))
        .send()
        .await
        .expect("validate responds")
        .status()
        .as_u16();
    assert_eq!(status, 422);

    handle.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn issuer_discovery_is_unmounted_while_jwks_serves_on_the_live_socket() {
    let fixture = SigningFixture::generate();
    let handle = boot(&fixture).await;
    let base = format!("http://{}", handle.rest_addr);
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{base}/.well-known/openid-configuration"))
        .send()
        .await
        .expect("discovery responds");
    assert_eq!(response.status().as_u16(), 404);

    let response = client
        .get(format!("{base}/oauth/v2/keys"))
        .send()
        .await
        .expect("canonical jwks responds");
    assert_eq!(response.status().as_u16(), 200);
    let document: serde_json::Value = response.json().await.expect("jwks json");
    let keys = document["keys"].as_array().expect("keys array");
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0]["kid"], "identity-e2e-k1");
    assert_eq!(keys[0]["alg"], "ES256");
    assert_eq!(keys[0]["use"], "sig");

    let legacy_response = client
        .get(format!("{base}/oauth/jwks"))
        .send()
        .await
        .expect("legacy jwks responds");
    assert_eq!(
        legacy_response.status().as_u16(),
        200,
        "legacy JWKS alias remains mounted for migration compatibility"
    );

    handle.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn scim_surface_guards_and_provisions_on_the_live_socket() {
    let fixture = SigningFixture::generate();
    let handle = boot(&fixture).await;
    let base = format!("http://{}", handle.rest_addr);
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "user_name": "amara@acme.example",
        "external_id": null,
        "name": null,
        "display_name": "Provisioned User",
        "active": true,
        "emails": [],
        "enterprise": null,
        "oyatie": null,
    });

    // Unauthenticated provisioning is refused fail-closed.
    let status = client
        .post(format!("{base}/scim/v2/ten_acme/Users"))
        .json(&payload)
        .send()
        .await
        .expect("scim responds")
        .status()
        .as_u16();
    assert_eq!(status, 401);

    // A workload token carrying scim.manage provisions a user.
    let token = fixture.mint("wl_provisioner", "scim.manage");
    let response = client
        .post(format!("{base}/scim/v2/ten_acme/Users"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("scim responds");
    assert_eq!(response.status().as_u16(), 201);
    let created: serde_json::Value = response.json().await.expect("created json");
    assert_eq!(created["userName"], "amara@acme.example");

    handle.shutdown().await;
}
