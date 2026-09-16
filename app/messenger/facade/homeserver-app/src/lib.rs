#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod cs;

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use messenger_authority_memory::MemoryAuthority;
use messenger_domain::{AuthorityEvent, Error};
use serde_json::{Value, json};
use tokio::sync::Mutex;

pub(crate) struct AppState {
    authority: Arc<MemoryAuthority>,
    server_name: String,
    sessions: Mutex<BTreeMap<String, Caller>>,
    next_token: Mutex<u64>,
}

#[derive(Clone)]
pub(crate) struct Caller {
    user: String,
    device: String,
}

pub fn router(server_name: impl Into<String>) -> Router {
    let server_name = server_name.into();
    Router::new()
        .route("/healthz", get(healthz))
        .route("/_matrix/client/v3/login", post(cs::login))
        .route("/_matrix/client/v3/createRoom", post(cs::create_room))
        .route("/_matrix/client/v3/sync", get(cs::sync))
        .route(
            "/_matrix/client/v3/rooms/{room}/send/{event_type}/{txn}",
            put(cs::send),
        )
        .route("/_matrix/client/v3/rooms/{room}/join", post(cs::join))
        .route("/_matrix/client/v3/rooms/{room}/leave", post(cs::leave))
        .with_state(Arc::new(AppState {
            authority: MemoryAuthority::new(server_name.clone()),
            server_name,
            sessions: Mutex::new(BTreeMap::new()),
            next_token: Mutex::new(1),
        }))
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok\n")
}

impl AppState {
    pub(crate) async fn issue(&self, user: String, device: String) -> String {
        let mut seq = self.next_token.lock().await;
        let token = format!("s{seq}");
        *seq = seq.saturating_add(1);
        drop(seq);
        self.sessions
            .lock()
            .await
            .insert(token.clone(), Caller { user, device });
        token
    }

    async fn lookup(&self, token: &str) -> Option<Caller> {
        self.sessions.lock().await.get(token).cloned()
    }
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

pub(crate) async fn caller(state: &AppState, headers: &HeaderMap) -> Result<Caller, Box<Response>> {
    let Some(header) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    else {
        return Err(Box::new(matrix_error(
            StatusCode::UNAUTHORIZED,
            "M_MISSING_TOKEN",
            "missing access token",
        )));
    };
    let Some(token) = header.strip_prefix("Bearer ") else {
        return Err(Box::new(matrix_error(
            StatusCode::UNAUTHORIZED,
            "M_UNKNOWN_TOKEN",
            "unknown token",
        )));
    };
    state.lookup(token).await.ok_or_else(|| {
        Box::new(matrix_error(
            StatusCode::UNAUTHORIZED,
            "M_UNKNOWN_TOKEN",
            "unknown token",
        ))
    })
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

pub(crate) fn parse_json(body: &str) -> Result<Value, Box<Response>> {
    serde_json::from_str(body).map_err(|_| {
        Box::new(matrix_error(
            StatusCode::BAD_REQUEST,
            "M_NOT_JSON",
            "not json",
        ))
    })
}
