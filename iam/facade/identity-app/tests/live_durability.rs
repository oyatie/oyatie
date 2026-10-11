//! The durable SCIM store survives a service rebuild against live Postgres.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "service_support/mod.rs"]
mod service_support;

use aws_lc_rs::digest::{SHA256, digest};
use iam_identity_scim_store_postgres::{PgScimUserStore, passkeys::PgPasskeyStore};
use iam_scim_server_kernel::{OyatieExtension, TenantId, User, UserStore};
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
    passkey_origin: Option<&str>,
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
    match passkey_origin {
        Some(origin) => server::start_with_passkey_origin(&config, scim_url, origin.to_owned())
            .await
            .expect("passkey service boots"),
        None => server::start_with_scim_url(&config, scim_url)
            .await
            .expect("service boots"),
    }
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
#[path = "live_durability/scim.rs"]
mod scim;
#[path = "live_durability/sessions.rs"]
mod sessions;
