use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use messenger_domain::{AuthorityEvent, Error};
use serde_json::{Value, json};

use crate::state::AppState;

#[derive(Clone)]
pub(crate) struct Caller {
    pub user: String,
    pub device: String,
}

pub(crate) fn matrix_error(status: StatusCode, errcode: &str, error: &str) -> Response {
    (status, Json(json!({ "errcode": errcode, "error": error }))).into_response()
}

pub(crate) fn map_error(error: Error) -> Response {
    match error {
        Error::Invalid(message) => {
            matrix_error(StatusCode::BAD_REQUEST, "M_INVALID_PARAM", &message)
        }
        Error::Denied | Error::ActionRejected | Error::Unencrypted | Error::ArchiveNotReady => {
            matrix_error(StatusCode::FORBIDDEN, "M_FORBIDDEN", &error.to_string())
        }
        Error::Unavailable(message) => {
            matrix_error(StatusCode::SERVICE_UNAVAILABLE, "M_UNKNOWN", &message)
        }
    }
}

pub(crate) async fn caller(state: &AppState, headers: &HeaderMap) -> Result<Caller, Response> {
    let Some(header) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    else {
        return Err(matrix_error(
            StatusCode::UNAUTHORIZED,
            "M_MISSING_TOKEN",
            "missing access token",
        ));
    };
    let Some(token) = header.strip_prefix("Bearer ") else {
        return Err(matrix_error(
            StatusCode::UNAUTHORIZED,
            "M_UNKNOWN_TOKEN",
            "unknown token",
        ));
    };
    state
        .caller(token)
        .await
        .ok_or_else(|| matrix_error(StatusCode::UNAUTHORIZED, "M_UNKNOWN_TOKEN", "unknown token"))
}

pub(crate) fn matrix_event(event: &AuthorityEvent) -> Value {
    let mut value = json!({
        "event_id": event.event_id,
        "room_id": event.room,
        "sender": event.sender,
        "origin_server_ts": event.origin_server_ts,
        "type": event.event_type,
        "content": event.content,
    });
    if let Some(state_key) = &event.state_key {
        value["state_key"] = json!(state_key);
    }
    value
}
