use axum::{
    body::Bytes,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use iam_identity_user_session::SessionView;
use reqwest::Method;

use super::{Operation, SignInConfig, config::now, cookie, failure};

pub(super) async fn request(
    config: &SignInConfig,
    tenant: &str,
    op: Operation,
    body: Bytes,
    session: Option<String>,
) -> Response {
    let mut url = config.iam.clone();
    let Ok(mut segments) = url.path_segments_mut() else {
        return failure(StatusCode::BAD_GATEWAY);
    };
    segments.extend(["identity", "v1", tenant]);
    segments.extend(op.path().split('/'));
    drop(segments);
    let mut request = config
        .client
        .request(
            if op.mutation() {
                Method::POST
            } else {
                Method::GET
            },
            url,
        )
        .header(header::ACCEPT, "application/json");
    if op.mutation() {
        request = request
            .header(header::ORIGIN, &config.public_origin)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body);
    }
    if matches!(op, Operation::Session | Operation::Logout)
        && let Some(cookie) = session
    {
        request = request.header(header::COOKIE, cookie);
    }
    let mut upstream = match request.send().await {
        Ok(response) => response,
        Err(_) => return failure(StatusCode::SERVICE_UNAVAILABLE),
    };
    let status = upstream.status();
    if !status.is_success() {
        return failure(match status {
            StatusCode::BAD_REQUEST
            | StatusCode::UNAUTHORIZED
            | StatusCode::FORBIDDEN
            | StatusCode::CONFLICT
            | StatusCode::TOO_MANY_REQUESTS => status,
            _ => StatusCode::SERVICE_UNAVAILABLE,
        });
    }
    let issued = match op {
        Operation::Authenticate | Operation::Logout => {
            match cookie::checked_set_cookie(upstream.headers(), matches!(op, Operation::Logout)) {
                Ok(cookie) => Some(cookie),
                Err(()) => return failure(StatusCode::BAD_GATEWAY),
            }
        }
        _ => None,
    };
    let mut body = Vec::new();
    loop {
        match upstream.chunk().await {
            Ok(Some(chunk)) if body.len() + chunk.len() <= 64 * 1024 => {
                body.extend_from_slice(&chunk)
            }
            Ok(None) => break,
            _ => return failure(StatusCode::SERVICE_UNAVAILABLE),
        }
    }
    let mut response = if matches!(op, Operation::Logout | Operation::Enroll) {
        let expected = if matches!(op, Operation::Enroll) {
            StatusCode::CREATED
        } else {
            StatusCode::NO_CONTENT
        };
        if status != expected || !body.is_empty() {
            return failure(StatusCode::BAD_GATEWAY);
        }
        status.into_response()
    } else if matches!(op, Operation::Session | Operation::Authenticate) {
        let view = match serde_json::from_slice::<SessionView>(&body) {
            Ok(view) => view,
            Err(_) => return failure(StatusCode::BAD_GATEWAY),
        };
        let member = match view.into_active_membership(tenant, now()) {
            Ok(member) => member,
            Err(_) => return failure(StatusCode::UNAUTHORIZED),
        };
        axum::Json(SessionView::from(&member)).into_response()
    } else {
        let value = match serde_json::from_slice::<serde_json::Value>(&body) {
            Ok(value) => value,
            Err(_) => return failure(StatusCode::BAD_GATEWAY),
        };
        (status, axum::Json(value)).into_response()
    };
    if let Some(cookie) = issued {
        response.headers_mut().insert(header::SET_COOKIE, cookie);
    }
    response
}

pub(super) async fn verified_session(
    config: &SignInConfig,
    tenant: &str,
    cookie: String,
) -> Result<
    iam_identity_user_session::ActiveUserMembership,
    application_ontology_read::OntologyReadError,
> {
    use application_ontology_read::OntologyReadError;
    let response = request(
        config,
        tenant,
        Operation::Session,
        Bytes::new(),
        Some(cookie),
    )
    .await;
    if response.status() != StatusCode::OK {
        return Err(match response.status() {
            StatusCode::UNAUTHORIZED => OntologyReadError::Unauthenticated,
            StatusCode::FORBIDDEN => OntologyReadError::Forbidden,
            _ => OntologyReadError::SourceUnavailable,
        });
    }
    let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .map_err(|_| OntologyReadError::SourceUnavailable)?;
    let view: SessionView =
        serde_json::from_slice(&body).map_err(|_| OntologyReadError::SourceUnavailable)?;
    view.into_active_membership(tenant, now())
        .map_err(|_| OntologyReadError::Unauthenticated)
}
