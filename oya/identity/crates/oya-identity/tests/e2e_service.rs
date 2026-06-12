//! E2E: boot the REAL service (the same `server::start` path `main` uses) on
//! live sockets and exercise the G005 acceptance contract end to end:
//!
//! - mint a real ES256 workload JWT -> `POST /tokens/validate` -> `200`;
//! - a denied principal -> `403` fail-closed, NEVER a `404` (unknown,
//!   unauthorized, and suspended principals all land on `403`);
//! - the gRPC surface returns the same decisions over tonic;
//! - graceful shutdown drains both servers.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::{SystemTime, UNIX_EPOCH};

use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use oya_identity::config::Config;
use oya_identity::server;
use oya_identity_workload_rest::grpc::proto;

const ISSUER: &str = "https://idp.oyatie.com";
const AUDIENCE: &str = "oya-cloud-kms";
const KID: &str = "kid-e2e-1";

const CEDAR_POLICIES: &str = r#"
@id("permit-acme-kms-decrypt")
permit (
  principal is Workload,
  action == Action::"cloud.kms.Decrypt",
  resource is Secret
) when {
  principal.tenant_id == "ten_acme" &&
  principal.scopes.contains("cloud.kms.decrypt")
};

@id("permit-acme-scim-manage")
permit (
  principal is Workload,
  action == Action::"identity.scim.Manage",
  resource is ScimTenant
) when {
  principal.tenant_id == "ten_acme" &&
  principal.scopes.contains("scim.manage")
};
"#;

const PRINCIPAL_SEED: &str = r#"[
    {"tenant_id":"ten_acme","workload_id":"wl_secrets_sync",
     "owning_capability":"cap.cloud.kms","scopes":["cloud.kms.decrypt"]},
    {"tenant_id":"ten_acme","workload_id":"wl_denied",
     "owning_capability":"cap.cloud.kms","scopes":["cloud.kms.encrypt"]},
    {"tenant_id":"ten_acme","workload_id":"wl_suspended",
     "owning_capability":"cap.cloud.kms","scopes":["cloud.kms.decrypt"],
     "state":"suspended"},
    {"tenant_id":"ten_acme","workload_id":"wl_provisioner",
     "owning_capability":"cap.identity.scim","scopes":["scim.manage"]}
]"#;

fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn epoch_now() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_secs(),
    )
    .expect("epoch fits i64")
}

/// An ES256 signing fixture: the key pair plus its public JWKS document.
struct SigningFixture {
    key_pair: EcdsaKeyPair,
    rng: SystemRandom,
    jwks_document: String,
}

impl SigningFixture {
    fn generate() -> Self {
        let rng = SystemRandom::new();
        let pkcs8 =
            EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).expect("pkcs8");
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref())
            .expect("key pair");
        let public = key_pair.public_key().as_ref();
        let (x, y) = (b64url(&public[1..33]), b64url(&public[33..65]));
        let jwks_document = format!(
            r#"{{"keys":[{{"kty":"EC","crv":"P-256","kid":"{KID}","alg":"ES256","x":"{x}","y":"{y}"}}]}}"#
        );
        Self {
            key_pair,
            rng,
            jwks_document,
        }
    }

    /// Mint a 5-minute ES256 workload JWT for `workload_id` with `scope`.
    fn mint(&self, workload_id: &str, scope: &str) -> String {
        let now = epoch_now();
        let claims = format!(
            r#"{{"iss":"{ISSUER}","aud":"{AUDIENCE}","exp":{},"iat":{now},"tenant_id":"ten_acme","sub":"{workload_id}","owning_capability":"cap.cloud.kms","scope":"{scope}"}}"#,
            now + 300
        );
        let header = format!(r#"{{"alg":"ES256","typ":"JWT","kid":"{KID}"}}"#);
        let signing_input = format!("{}.{}", b64url(header.as_bytes()), b64url(claims.as_bytes()));
        let signature = self
            .key_pair
            .sign(&self.rng, signing_input.as_bytes())
            .expect("sign");
        format!("{signing_input}.{}", b64url(signature.as_ref()))
    }
}

/// Materialize the config fixture files in a unique temp dir and boot the
/// service on ephemeral loopback ports.
async fn boot(fixture: &SigningFixture) -> server::ServiceHandle {
    let dir = std::env::temp_dir().join(format!("oya-identity-e2e-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let jwks_path = dir.join("jwks.json");
    let cedar_path = dir.join("policies.cedar");
    let seed_path = dir.join("principals.json");
    std::fs::write(&jwks_path, &fixture.jwks_document).expect("write jwks");
    std::fs::write(&cedar_path, CEDAR_POLICIES).expect("write cedar");
    std::fs::write(&seed_path, PRINCIPAL_SEED).expect("write seed");

    // Issuer signing key: fresh ES256 PKCS#8 mounted the same way a K8s
    // secret would be (custody moves behind the G02 KMS port later).
    let signing_key_path = dir.join("signing-key.p8");
    let signing_pkcs8 =
        EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &SystemRandom::new())
            .expect("issuer pkcs8");
    std::fs::write(&signing_key_path, signing_pkcs8.as_ref()).expect("write signing key");

    let config = Config {
        rest_addr: "127.0.0.1:0".into(),
        grpc_addr: "127.0.0.1:0".into(),
        issuer: ISSUER.into(),
        audience: AUDIENCE.into(),
        jwks_path: jwks_path.to_string_lossy().into_owned(),
        cedar_policy_path: cedar_path.to_string_lossy().into_owned(),
        principals_path: Some(seed_path.to_string_lossy().into_owned()),
        signing_key_path: Some(signing_key_path.to_string_lossy().into_owned()),
        signing_kid: "oya-identity-e2e-k1".into(),
    };
    server::start(&config).await.expect("service boots")
}

fn authorize_body(token: &str) -> serde_json::Value {
    serde_json::json!({
        "token": token,
        "action": "cloud.kms.Decrypt",
        "resource": {"resourceType": "Secret", "resourceId": "sec_db_creds"}
    })
}

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
async fn issuer_discovery_and_jwks_serve_on_the_live_socket() {
    let fixture = SigningFixture::generate();
    let handle = boot(&fixture).await;
    let base = format!("http://{}", handle.rest_addr);
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{base}/.well-known/openid-configuration"))
        .send()
        .await
        .expect("discovery responds");
    assert_eq!(response.status().as_u16(), 200);
    let document: serde_json::Value = response.json().await.expect("discovery json");
    assert_eq!(document["issuer"], ISSUER);
    assert_eq!(document["jwks_uri"], format!("{ISSUER}/oauth/jwks"));

    let response = client
        .get(format!("{base}/oauth/jwks"))
        .send()
        .await
        .expect("jwks responds");
    assert_eq!(response.status().as_u16(), 200);
    let document: serde_json::Value = response.json().await.expect("jwks json");
    let keys = document["keys"].as_array().expect("keys array");
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0]["kid"], "oya-identity-e2e-k1");
    assert_eq!(keys[0]["alg"], "ES256");
    assert_eq!(keys[0]["use"], "sig");

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
    let response = validator
        .validate_token(proto::ValidateTokenRequest {
            token: token.clone(),
        })
        .await
        .expect("validate rpc")
        .into_inner();
    assert!(response.ok, "valid ES256 token must validate over gRPC");

    // AuthorizeWithToken: ALLOW for the permitted principal, DENY (as a
    // response value, not an RPC error) for the scope-denied principal.
    let mut authorizer =
        proto::workload_authorizer_client::WorkloadAuthorizerClient::new(channel);
    let request = |token: String| proto::AuthorizeWithTokenRequest {
        token,
        action: "cloud.kms.Decrypt".into(),
        resource: Some(proto::Resource {
            resource_type: "Secret".into(),
            resource_id: "sec_db_creds".into(),
        }),
        context: Default::default(),
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
