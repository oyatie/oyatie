use super::*;

use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use iam_scim_server_kernel::{InMemoryGroupStore, InMemoryUserStore, User};
use tower::ServiceExt as _;

use iam_identity_workload_app::{
    InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository, WorkloadPrincipalRepository,
};
use iam_identity_workload_authz_cedar::CedarWorkloadAuthorizer;
use iam_identity_workload_domain::{WorkloadPrincipal, WorkloadState};
use iam_identity_workload_oidc::Jwk;
use iam_identity_workload_rest::{BearerCallerVerifier, InMemoryAuditSink, WorkloadAuthzState};

use crate::decision_authz::TenantScopedDecisionAuthorizer;
use crate::lifecycle_authz::TenantScopedLifecycleAuthorizer;

const ISSUER: &str = "https://idp.oyatie.com";
const AUDIENCE: &str = "identity-scim";
const KID: &str = "kid-scim-1";
const NOW: i64 = 1_700_000_000;

struct Fixture {
    router: Router,
    token: String,
    admin_token: String,
    cedar_denied_admin_token: String,
    wrong_scope_token: String,
    unknown_principal_token: String,
    /// Registered + scoped principal in `ten_other` — passes every guard
    /// pre-check, then Cedar default-denies (the policy permits `ten_acme`
    /// only): the pure policy-deny path.
    cedar_denied_token: String,
    /// Inspection handle onto the SAME audit log the state emits to
    /// (clones share the underlying append-only store).
    audit: InMemoryAuditSink,
}

fn b64(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn mint(
    key_pair: &EcdsaKeyPair,
    rng: &SystemRandom,
    tenant: &str,
    workload: &str,
    scope: &str,
) -> String {
    let claims = format!(
        r#"{{"iss":"{ISSUER}","aud":"{AUDIENCE}","exp":{},"iat":{NOW},"tenant_id":"{tenant}","sub":"{workload}","owning_capability":"cap.identity.scim","scope":"{scope}"}}"#,
        NOW + 300
    );
    let header = format!(r#"{{"alg":"ES256","typ":"JWT","kid":"{KID}"}}"#);
    let input = format!("{}.{}", b64(header.as_bytes()), b64(claims.as_bytes()));
    let sig = key_pair.sign(rng, input.as_bytes()).expect("sign");
    format!("{input}.{}", b64(sig.as_ref()))
}

fn fixture() -> Fixture {
    let rng = SystemRandom::new();
    let pkcs8 =
        EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).expect("pkcs8");
    let key_pair =
        EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref()).expect("key");
    let public = key_pair.public_key().as_ref();
    let jwks = Jwks::new().add_key(Jwk::ec_p256(KID, b64(&public[1..33]), b64(&public[33..65])));
    let mut repository = InMemoryWorkloadPrincipalRepository::new();
    for (tenant, workload) in [
        ("ten_acme", "wl_provisioner"),
        ("ten_acme", "wl_native_admin"),
        ("ten_other", "wl_other_provisioner"),
    ] {
        let mut provisioner =
            WorkloadPrincipal::provision(tenant, workload, "cap.identity.scim").expect("principal");
        provisioner
            .transition_to(WorkloadState::Active)
            .expect("activate");
        repository.save(&provisioner).expect("seed principal");
    }
    let authorizer = CedarWorkloadAuthorizer::from_cedar_policies(
        r#"
        @id("permit-scim-manage")
        permit(
          principal,
          action == Action::"identity.scim.Manage",
          resource is ScimTenant
        ) when {
          principal.tenant_id == "ten_acme"
        };
        @id("permit-native-user-admin")
        permit(
          principal == Workload::"wl_native_admin",
          action == Action::"identity.user.Administer",
          resource is ScimTenant
        ) when {
          principal.tenant_id == "ten_acme"
        };
        "#,
    )
    .expect("policy");
    let audit = InMemoryAuditSink::new();
    let authz = Arc::new(WorkloadAuthzState::with_clock(
        repository,
        InMemoryRevocationDenylist::new(),
        authorizer,
        jwks.clone(),
        ValidationConfig::new(ISSUER, AUDIENCE),
        audit.clone(),
        Arc::new(BearerCallerVerifier::new(
            "scim-test-lifecycle-bearer",
            "ten_acme",
            "scim-test-control-plane",
        )),
        Arc::new(TenantScopedLifecycleAuthorizer::new()),
        Arc::new(TenantScopedDecisionAuthorizer::new()),
        || NOW,
    ));
    let state = Arc::new(ScimSurfaceState::new(
        format!("{ISSUER}{SCIM_BASE}"),
        jwks,
        ValidationConfig::new(ISSUER, AUDIENCE),
        || NOW,
        authz,
        InMemoryUserStore::default(),
        InMemoryGroupStore::default(),
    ));
    Fixture {
        router: build_scim_router(state),
        token: mint(
            &key_pair,
            &rng,
            "ten_acme",
            "wl_provisioner",
            SCIM_MANAGE_SCOPE,
        ),
        admin_token: mint(
            &key_pair,
            &rng,
            "ten_acme",
            "wl_native_admin",
            "scim.manage identity.user.administer",
        ),
        cedar_denied_admin_token: mint(
            &key_pair,
            &rng,
            "ten_acme",
            "wl_provisioner",
            "scim.manage identity.user.administer",
        ),
        wrong_scope_token: mint(
            &key_pair,
            &rng,
            "ten_acme",
            "wl_provisioner",
            "cloud.kms.decrypt",
        ),
        unknown_principal_token: mint(
            &key_pair,
            &rng,
            "ten_acme",
            "wl_not_seeded",
            SCIM_MANAGE_SCOPE,
        ),
        cedar_denied_token: mint(
            &key_pair,
            &rng,
            "ten_other",
            "wl_other_provisioner",
            SCIM_MANAGE_SCOPE,
        ),
        audit,
    }
}

async fn send(
    router: &Router,
    method: &str,
    uri: &str,
    bearer: Option<&str>,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let mut request = axum::http::Request::builder().method(method).uri(uri);
    if let Some(token) = bearer {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let request = match body {
        Some(json) => request
            .header("content-type", "application/json")
            .body(axum::body::Body::from(json.to_string())),
        None => request.body(axum::body::Body::empty()),
    }
    .expect("request");
    let response = router.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
        .await
        .expect("body");
    let value = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("json body")
    };
    (status, value)
}

/// RFC 7644 camelCase wire shape, absent members omitted — what a real
/// SCIM client (Okta/Entra) sends. The legacy snake_case alias shape is
/// covered by the kernel tests and the service E2E.
fn new_user_payload(user_name: &str) -> serde_json::Value {
    serde_json::json!({
        "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
        "userName": user_name,
        "displayName": "Provisioned User",
        "active": true,
    })
}

mod audit;
mod credentials;
mod groups;
mod native_authority;
mod users;
