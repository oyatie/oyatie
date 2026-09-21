//! Shared fixtures for the E2E suites: boot the REAL service (the same `server::start` path `main` uses) on
//! live sockets and exercise the G005 acceptance contract end to end:
//!
//! - mint a real ES256 workload JWT -> `POST /tokens/validate` -> `200`;
//! - a denied principal -> `403` fail-closed, NEVER a `404` (unknown,
//!   unauthorized, and suspended principals all land on `403`);
//! - the gRPC surface returns the same decisions over tonic;
//! - graceful shutdown drains both servers.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub use std::time::{SystemTime, UNIX_EPOCH};

pub use aws_lc_rs::rand::SystemRandom;
pub use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
pub use base64::Engine as _;
pub use base64::engine::general_purpose::URL_SAFE_NO_PAD;

pub use iam_identity_app::config::Config;
pub use iam_identity_app::server;
pub use iam_identity_workload_rest::grpc::proto;

pub const ISSUER: &str = "https://idp.oyatie.com";
pub const AUDIENCE: &str = "cloud-kms";
pub const KID: &str = "kid-e2e-1";
/// AUTH-005: the decision surfaces (`/authorize-with-token`, `/tokens/validate`,
/// gRPC `Authorize`/`ValidateToken`) require a verified caller. The boot config
/// binds this bearer to `ten_acme`, matching the minted tokens' tenant, so the
/// same-tenant decision gate permits.
pub const LIFECYCLE_BEARER: &str = "e2e-lifecycle-bearer";

pub const CEDAR_POLICIES: &str = r#"
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

pub const PRINCIPAL_SEED: &str = r#"[
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

pub fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn epoch_now() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_secs(),
    )
    .expect("epoch fits i64")
}

/// An ES256 signing fixture: the key pair plus its public JWKS document.
pub struct SigningFixture {
    pub key_pair: EcdsaKeyPair,
    pub rng: SystemRandom,
    pub jwks_document: String,
}

impl SigningFixture {
    pub fn generate() -> Self {
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
    pub fn mint(&self, workload_id: &str, scope: &str) -> String {
        let now = epoch_now();
        let claims = format!(
            r#"{{"iss":"{ISSUER}","aud":"{AUDIENCE}","exp":{},"iat":{now},"tenant_id":"ten_acme","sub":"{workload_id}","owning_capability":"cap.cloud.kms","scope":"{scope}"}}"#,
            now + 300
        );
        let header = format!(r#"{{"alg":"ES256","typ":"JWT","kid":"{KID}"}}"#);
        let signing_input = format!(
            "{}.{}",
            b64url(header.as_bytes()),
            b64url(claims.as_bytes())
        );
        let signature = self
            .key_pair
            .sign(&self.rng, signing_input.as_bytes())
            .expect("sign");
        format!("{signing_input}.{}", b64url(signature.as_ref()))
    }
}

/// Materialize the config fixture files in a unique temp dir and boot the
/// service on ephemeral loopback ports.
pub async fn boot(fixture: &SigningFixture) -> server::ServiceHandle {
    let dir = std::env::temp_dir().join(format!("identity-e2e-{}", uuid::Uuid::new_v4()));
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
        signing_kid: "identity-e2e-k1".into(),
        lifecycle_bearer: LIFECYCLE_BEARER.into(),
        lifecycle_caller_tenant: "ten_acme".into(),
        lifecycle_caller_id: "e2e-control-plane".into(),
    };
    server::start(&config).await.expect("service boots")
}

pub fn authorize_body(token: &str) -> serde_json::Value {
    serde_json::json!({
        "token": token,
        "action": "cloud.kms.Decrypt",
        "resource": {"resourceType": "Secret", "resourceId": "sec_db_creds"}
    })
}
