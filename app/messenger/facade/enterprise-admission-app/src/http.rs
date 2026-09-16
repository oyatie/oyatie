use std::sync::Arc;

use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use messenger_domain::Error;
use messenger_policy_api::Policy;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{Admission, AppState};

pub(crate) async fn authenticate<P: Policy + 'static>(
    State(app): State<Arc<AppState<P>>>,
    request: Request,
    next: Next,
) -> Response {
    let Some(token) = request
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
    else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let hash = Sha256::digest(token.as_bytes());
    let differs = hash
        .iter()
        .zip(app.token_hash)
        .fold(0u8, |acc, (left, right)| acc | (left ^ right));
    if differs != 0 {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Ok(_permit) = app.capacity.try_acquire() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    match tokio::time::timeout(std::time::Duration::from_secs(4), next.run(request)).await {
        Ok(response) => response,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

pub(crate) async fn admit<P: Policy + 'static>(
    State(app): State<Arc<AppState<P>>>,
    Json(request): Json<Admission>,
) -> Result<Json<Value>, StatusCode> {
    app.authorize_admission(&request).await.map_err(|reason| {
        eprintln!("enterprise admission refused: {reason}");
        match reason {
            Error::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::FORBIDDEN,
        }
    })?;
    Ok(Json(json!({
        "event_id": request.event_id,
        "allow": true,
        "obligations": []
    })))
}
