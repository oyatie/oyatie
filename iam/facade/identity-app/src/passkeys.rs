//! Durable human passkey ceremonies and tenant-scoped, revocable sessions.
//! Enrollment is an IAM administrator action over an existing active SCIM
//! binding. Authentication never creates a user or grants a role.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use aws_lc_rs::digest::{Context, SHA256, digest};
use aws_lc_rs::rand::{SecureRandom, SystemRandom};
use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ciborium::Value as CborValue;
use iam_identity_scim_store_postgres::PgScimUserStore;
use iam_identity_scim_store_postgres::passkeys::{
    ChallengeWindow, PasskeyStoreError, PgPasskeyStore,
};
use iam_identity_user_session::{
    AuthenticatedSubject, MembershipSessionAuthorizer, SessionRefusal, UserCredentialVerifier,
    UserMembershipStore, UserSessionAuthorizer,
};
use passkey_auth::{
    AuthenticationResponse, AuthenticationState, CredentialId, PasskeyCredential,
    RegistrationResponse, RegistrationState, Webauthn,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;

pub use iam_identity_user_session::SessionView;

const SESSION_COOKIE: &str = "__Host-oya_session";
const CHALLENGE_SECONDS: i64 = 300;
const SESSION_SECONDS: i64 = 3600;

pub(crate) type AdminGuard =
    Arc<dyn Fn(&HeaderMap, &str) -> Result<(), Box<Response>> + Send + Sync>;

/// Exact browser origin, from deployment configuration rather than request
/// headers. HTTP is admitted only on the loopback WebAuthn development origin.
pub struct PasskeyOrigin {
    origin: String,
    rp_id: String,
}

impl PasskeyOrigin {
    pub fn parse(origin: &str) -> Result<Self, &'static str> {
        let uri: Uri = origin.parse().map_err(|_| "invalid passkey origin")?;
        let scheme = uri.scheme_str().ok_or("passkey origin needs a scheme")?;
        let authority = uri.authority().ok_or("passkey origin needs a host")?;
        let host = uri.host().ok_or("passkey origin needs a host")?;
        if uri.path() != "/"
            || uri.query().is_some()
            || origin != format!("{scheme}://{authority}")
            || (scheme != "https"
                && !(scheme == "http" && matches!(host, "localhost" | "127.0.0.1")))
        {
            return Err("passkey origin must be an exact secure origin");
        }
        Ok(Self {
            origin: origin.to_owned(),
            rp_id: host.to_owned(),
        })
    }
}

#[derive(Clone)]
struct PgSessionVerifier {
    store: PgPasskeyStore,
}

impl UserCredentialVerifier for PgSessionVerifier {
    fn verify<'a>(
        &'a self,
        presented: Option<&'a str>,
        expected_tenant: &'a str,
        now_epoch_seconds: i64,
    ) -> Pin<Box<dyn Future<Output = Result<AuthenticatedSubject, SessionRefusal>> + Send + 'a>>
    {
        Box::pin(async move {
            let token = presented.ok_or(SessionRefusal::Unauthenticated)?;
            let session_hash = session_hash(token).ok_or(SessionRefusal::InvalidCredential)?;
            let stored = self
                .store
                .session(expected_tenant, &session_hash, now_epoch_seconds)
                .await
                .map_err(|_| SessionRefusal::MembershipUnavailable)?
                .ok_or(SessionRefusal::InvalidCredential)?;
            AuthenticatedSubject::new(
                stored.user_id,
                expected_tenant,
                stored.expires_at_epoch_seconds,
            )
        })
    }
}

struct PasskeyState {
    store: PgPasskeyStore,
    members: PgScimUserStore,
    sessions: Arc<dyn UserSessionAuthorizer>,
    webauthn: Webauthn,
    origin: String,
    admin: AdminGuard,
    now: fn() -> i64,
}

pub(crate) fn build_router(
    pool: PgPool,
    origin: PasskeyOrigin,
    admin: AdminGuard,
    now: fn() -> i64,
) -> Router {
    let store = PgPasskeyStore::from_pool(pool.clone());
    let members = PgScimUserStore::from_pool(pool);
    let sessions: Arc<dyn UserSessionAuthorizer> = Arc::new(MembershipSessionAuthorizer::new(
        PgSessionVerifier {
            store: store.clone(),
        },
        members.clone(),
    ));
    let state = Arc::new(PasskeyState {
        store,
        members,
        sessions,
        webauthn: Webauthn::new(&origin.rp_id, "Oyatie", &origin.origin)
            .strict_base64(true)
            .require_user_verification(true),
        origin: origin.origin,
        admin,
        now,
    });
    Router::new()
        .route(
            "/identity/v1/{tenant}/passkeys/enrollment-options",
            post(enrollment_options),
        )
        .route("/identity/v1/{tenant}/passkeys/enroll", post(enroll))
        .route(
            "/identity/v1/{tenant}/passkeys/authentication-options",
            post(authentication_options),
        )
        .route(
            "/identity/v1/{tenant}/passkeys/authenticate",
            post(authenticate),
        )
        .route("/identity/v1/{tenant}/session", get(current_session))
        .route("/identity/v1/{tenant}/session:logout", post(logout))
        .with_state(state)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserStart {
    user_id: String,
}

#[derive(Deserialize)]
struct BrowserCredential<T> {
    id: String,
    #[serde(rename = "rawId")]
    raw_id: String,
    #[serde(rename = "type")]
    kind: String,
    response: T,
}

#[derive(Deserialize)]
struct RegistrationFields {
    #[serde(rename = "attestationObject")]
    attestation_object: String,
    #[serde(rename = "clientDataJSON")]
    client_data_json: String,
    #[serde(default)]
    transports: Vec<String>,
}

#[derive(Deserialize)]
struct AuthenticationFields {
    #[serde(rename = "authenticatorData")]
    authenticator_data: String,
    signature: String,
    #[serde(rename = "clientDataJSON")]
    client_data_json: String,
    #[serde(rename = "userHandle")]
    user_handle: Option<String>,
}

#[derive(Deserialize)]
struct Finish<T> {
    challenge: String,
    credential: BrowserCredential<T>,
}

fn reply(status: StatusCode, message: &'static str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

fn refused(status: StatusCode, message: &'static str) -> Box<Response> {
    Box::new(reply(status, message))
}

fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
}

fn store_error(_: PasskeyStoreError) -> Response {
    reply(
        StatusCode::SERVICE_UNAVAILABLE,
        "identity store unavailable",
    )
}

fn origin_matches(headers: &HeaderMap, expected: &str) -> bool {
    headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        == Some(expected)
}

async fn active_account(
    state: &PasskeyState,
    tenant: &str,
    user_id: &str,
) -> Result<(), Box<Response>> {
    AuthenticatedSubject::new(user_id, tenant, (state.now)() + SESSION_SECONDS)
        .map_err(|_| refused(StatusCode::BAD_REQUEST, "invalid account"))?;
    let records = state
        .members
        .candidates(tenant, user_id)
        .await
        .map_err(|_| {
            refused(
                StatusCode::SERVICE_UNAVAILABLE,
                "identity store unavailable",
            )
        })?;
    let [record] = records.as_slice() else {
        return Err(refused(StatusCode::UNAUTHORIZED, "account unavailable"));
    };
    if !record.active || record.tenant_id != tenant || record.user_id != user_id {
        return Err(refused(StatusCode::UNAUTHORIZED, "account unavailable"));
    }
    Ok(())
}

mod authentication;
mod checks;
mod enrollment;
mod sessions;
use authentication::*;
use checks::*;
use enrollment::*;
use sessions::*;

#[cfg(test)]
mod tests;
