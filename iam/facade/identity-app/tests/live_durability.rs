//! The durable SCIM store survives a service rebuild against live Postgres.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "service_support/mod.rs"]
mod service_support;

use service_support::*;

// ============================================================
// G005 SLICE-2 — durable Postgres SCIM store wiring (composition root)
// ============================================================

/// Env that turns the live-Postgres tier ON (mirrors the durable adapter's
/// `tests/live_rls.rs` gate and the tenancy facade acceptance test). Default
/// `buck2 test` leaves it unset, so the live test below skips cleanly (the
/// DB-free lane stays the default).
const LIVE_ENV: &str = "OYATIE_BACKBONE_LIVE_POSTGRES";

/// The NON-SUPERUSER app-role Postgres URL for the live durability test.
///
/// Convention (mirrors `iam/adapters/identity-scim-store-postgres/tests/live_rls.rs`):
///   - `OYATIE_BACKBONE_POSTGRES_URL`     = superuser / setup URL (CREATE ROLE, migrations)
///   - `OYATIE_BACKBONE_POSTGRES_APP_URL` = non-superuser `identity_scim_runtime` role URL
///
/// The RLS-enforceability guard (`assert_rls_enforceable`) rejects superusers
/// (`rolsuper = true`), so the live durability test MUST boot the service with
/// the APP URL, not the setup URL. Using `start_with_scim_url` also avoids
/// writing to the process-global env (`std::env::set_var`), which races with
/// other parallel tokio tests.
const LIVE_APP_URL_ENV: &str = "OYATIE_BACKBONE_POSTGRES_APP_URL";

/// Truthy-gate identical to the adapter's live tests.
fn live_enabled() -> bool {
    std::env::var(LIVE_ENV)
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn require_live_app_url() -> String {
    assert!(
        live_enabled(),
        "live test requires {LIVE_ENV}=1 (nextest --profile live --run-ignored only)"
    );
    std::env::var(LIVE_APP_URL_ENV)
        .ok()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| panic!("{LIVE_ENV} is set but {LIVE_APP_URL_ENV} is missing or empty"))
}

/// Boot the service with an explicit SCIM database URL (bypasses env read).
/// Used by the live durability test to pass the app-role URL without racing the
/// process-global env with other parallel tests.
async fn boot_with_url(
    fixture: &SigningFixture,
    scim_url: Option<String>,
) -> server::ServiceHandle {
    let dir = std::env::temp_dir().join(format!("identity-e2e-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let jwks_path = dir.join("jwks.json");
    let cedar_path = dir.join("policies.cedar");
    let seed_path = dir.join("principals.json");
    std::fs::write(&jwks_path, &fixture.jwks_document).expect("write jwks");
    std::fs::write(&cedar_path, CEDAR_POLICIES).expect("write cedar");
    std::fs::write(&seed_path, PRINCIPAL_SEED).expect("write seed");

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
    server::start_with_scim_url(&config, scim_url)
        .await
        .expect("service boots")
}

/// LIVE durability proof (env-gated): boot the REAL service via
/// `server::start_with_scim_url` (the same composition path `start` delegates
/// to) with the APP-ROLE Postgres URL (`OYATIE_BACKBONE_POSTGRES_APP_URL`) so the
/// DURABLE Postgres SCIM stores are composed; POST a SCIM user; then boot a
/// FRESH service over the SAME url and GET the user back — proving the write
/// survived a full service rebuild (the property the in-memory store CANNOT
/// provide). Also asserts the REAL facade-layer SCIM PEP cross-tenant deny: a
/// verified `ten_acme` token hitting `/scim/v2/ten_other/Users` receives
/// **403 FORBIDDEN** (tenant-mismatch) before the store is ever consulted.
/// Store-layer RLS cross-tenant denial (unset-GUC deny-all, cross-tenant
/// INSERT/SELECT) is proven separately in the adapter's `tests/live_rls.rs` —
/// not duplicated here.
///
/// The test uses `OYATIE_BACKBONE_POSTGRES_APP_URL` (non-superuser role) rather
/// than `OYATIE_BACKBONE_POSTGRES_URL` (superuser / setup URL) because the
/// RLS-enforceability guard rejects superusers — booting with a superuser URL
/// would cause `start_with_scim_url` to return `Err(StartError::Store(...))`.
///
/// `#[ignore]` so default nextest does not count a skip as a pass. The live
/// job runs this with `--run-ignored only`. Missing env is a hard failure.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "live postgres"]
async fn live_durable_scim_store_persists_across_service_rebuild() {
    let app_url = require_live_app_url();

    // ONE signing fixture across both boots so the JWKS is identical: a token
    // minted now must validate against the freshly-rebuilt service.
    let fixture = SigningFixture::generate();

    // A unique userName per run so repeated live runs against the same database
    // do not collide on the durable UNIQUE (tenant_id, user_name).
    let user_name = format!("durable-{}@acme.example", std::process::id());
    let payload = serde_json::json!({
        "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
        "userName": user_name,
        "displayName": "Durable Provisioned User",
        "active": true,
    });
    let token = fixture.mint("wl_provisioner", "scim.manage");

    // Service #1: provision the user (durable store, born in Postgres).
    let handle = boot_with_url(&fixture, Some(app_url.clone())).await;
    let base = format!("http://{}", handle.rest_addr);
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/scim/v2/ten_acme/Users"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("scim create responds");
    assert_eq!(
        response.status().as_u16(),
        201,
        "durable SCIM create must succeed"
    );
    let created: serde_json::Value = response.json().await.expect("created json");
    let id = created["id"].as_str().expect("assigned id").to_owned();
    assert_eq!(created["userName"], user_name.as_str());
    handle.shutdown().await;

    // Service #2: a FRESH composition over the SAME backend. The earlier write
    // is only observable here if it was durably persisted (the in-memory store,
    // being per-process state, would 404).
    let rebuilt = boot_with_url(&fixture, Some(app_url)).await;
    let base2 = format!("http://{}", rebuilt.rest_addr);

    let response = client
        .get(format!("{base2}/scim/v2/ten_acme/Users/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("scim get responds");
    assert_eq!(
        response.status().as_u16(),
        200,
        "durable SCIM user must survive a service rebuild"
    );
    let fetched: serde_json::Value = response.json().await.expect("fetched json");
    assert_eq!(fetched["userName"], user_name.as_str());

    // Facade-layer SCIM PEP cross-tenant deny: the `ten_acme` token hitting a
    // `ten_other` path is denied 403 (tenant-mismatch) BEFORE the store is
    // consulted. This is the PEP guard, NOT a store-layer RLS test.
    let cross = client
        .get(format!("{base2}/scim/v2/ten_other/Users/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("cross-tenant scim responds");
    assert_eq!(
        cross.status().as_u16(),
        403,
        "cross-tenant SCIM access must be denied 403 by the PEP before the store"
    );

    rebuilt.shutdown().await;
}
