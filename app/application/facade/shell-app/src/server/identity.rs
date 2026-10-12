use application_ontology_card_http::HumanOntologyRefusal;
use axum::{
    Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::json;
use std::sync::Arc;

mod config;
mod cookie;
mod transport;
pub use config::SignInConfig;
use config::canonical_tenant;

type Configuration = Option<Arc<SignInConfig>>;

pub(super) fn router(config: Option<SignInConfig>) -> Router {
    Router::new()
        .route("/api/identity-config", get(availability))
        .route("/api/identity/{tenant}/passkeys/enroll", post(enroll))
        .route(
            "/api/identity/{tenant}/passkeys/authentication-options",
            post(authentication_options),
        )
        .route(
            "/api/identity/{tenant}/passkeys/authenticate",
            post(authenticate),
        )
        .route("/api/identity/{tenant}/session", get(session))
        .route("/api/identity/{tenant}/session:logout", post(logout))
        .route("/api/ontology-status/{tenant}", get(ontology))
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(middleware::map_response(no_store))
        .with_state(config.map(Arc::new))
}

async fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
        .headers_mut()
        .insert(header::PRAGMA, header::HeaderValue::from_static("no-cache"));
    response
}

async fn availability(State(config): State<Configuration>) -> impl IntoResponse {
    axum::Json(json!({"enabled": config.is_some()}))
}

#[derive(Clone, Copy)]
pub(super) enum Operation {
    Enroll,
    Options,
    Authenticate,
    Session,
    Logout,
}

impl Operation {
    fn path(self) -> &'static str {
        match self {
            Self::Enroll => "passkeys/enroll",
            Self::Options => "passkeys/authentication-options",
            Self::Authenticate => "passkeys/authenticate",
            Self::Session => "session",
            Self::Logout => "session:logout",
        }
    }
    fn mutation(self) -> bool {
        !matches!(self, Self::Session)
    }
}

async fn enroll(
    State(c): State<Configuration>,
    Path(t): Path<String>,
    h: HeaderMap,
    b: Bytes,
) -> Response {
    proxy(c, t, h, b, Operation::Enroll).await
}
async fn authentication_options(
    State(c): State<Configuration>,
    Path(t): Path<String>,
    h: HeaderMap,
    b: Bytes,
) -> Response {
    proxy(c, t, h, b, Operation::Options).await
}
async fn authenticate(
    State(c): State<Configuration>,
    Path(t): Path<String>,
    h: HeaderMap,
    b: Bytes,
) -> Response {
    proxy(c, t, h, b, Operation::Authenticate).await
}
async fn session(State(c): State<Configuration>, Path(t): Path<String>, h: HeaderMap) -> Response {
    proxy(c, t, h, Bytes::new(), Operation::Session).await
}
async fn logout(
    State(c): State<Configuration>,
    Path(t): Path<String>,
    h: HeaderMap,
    b: Bytes,
) -> Response {
    proxy(c, t, h, b, Operation::Logout).await
}

async fn proxy(
    config: Configuration,
    tenant: String,
    headers: HeaderMap,
    body: Bytes,
    op: Operation,
) -> Response {
    let Some(config) = config else {
        return failure(StatusCode::SERVICE_UNAVAILABLE);
    };
    if !canonical_tenant(&tenant) {
        return failure(StatusCode::BAD_REQUEST);
    }
    if op.mutation() {
        let mut origins = headers.get_all(header::ORIGIN).iter();
        if origins.next().and_then(|v| v.to_str().ok()) != Some(&config.public_origin)
            || origins.next().is_some()
        {
            return failure(StatusCode::FORBIDDEN);
        }
        if !matches!(op, Operation::Logout)
            && headers
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .is_none_or(|v| v.split(';').next().map(str::trim) != Some("application/json"))
        {
            return failure(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
    }
    let session = match cookie::session_cookie(&headers) {
        Ok(value) => value,
        Err(()) => return failure(StatusCode::UNAUTHORIZED),
    };
    if matches!(op, Operation::Session) && session.is_none() {
        return failure(StatusCode::UNAUTHORIZED);
    }
    transport::request(&config, &tenant, op, body, session).await
}

async fn ontology(
    State(config): State<Configuration>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
) -> Response {
    let Some(config) = config else {
        return failure(StatusCode::SERVICE_UNAVAILABLE);
    };
    if !canonical_tenant(&tenant) {
        return failure(StatusCode::BAD_REQUEST);
    }
    let Ok(Some(cookie)) = cookie::session_cookie(&headers) else {
        return failure(StatusCode::UNAUTHORIZED);
    };
    match config.ontology.facts(&tenant, &cookie).await {
        Ok(facts) => axum::Json(json!({
            "policyVersion": facts.policy_version,
            "servedTenants": facts.served_tenants,
            "projectionLag": facts.projection_lag,
            "poisonedEntries": facts.poisoned_entries,
        }))
        .into_response(),
        Err(HumanOntologyRefusal::Unauthenticated) => failure(StatusCode::UNAUTHORIZED),
        Err(HumanOntologyRefusal::Forbidden) => failure(StatusCode::FORBIDDEN),
        Err(HumanOntologyRefusal::Unavailable) => failure(StatusCode::SERVICE_UNAVAILABLE),
    }
}

pub(super) fn failure(status: StatusCode) -> Response {
    let error = match status {
        StatusCode::UNAUTHORIZED => "Session unavailable. Sign in again.",
        StatusCode::FORBIDDEN => "Access refused.",
        StatusCode::BAD_REQUEST | StatusCode::UNSUPPORTED_MEDIA_TYPE => "Invalid identity request.",
        _ => "Identity or live status unavailable. Retry when the service is available.",
    };
    (status, axum::Json(json!({"error": error}))).into_response()
}

#[cfg(test)]
mod tests;
