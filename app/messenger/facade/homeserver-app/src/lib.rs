#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod http;
mod read;
mod session;
mod state;
mod write;

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::{get, post, put};

pub use state::{AppState, compose};

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/_matrix/client/v3/login", post(session::login))
        .route("/_matrix/client/v3/createRoom", post(session::create_room))
        .route("/_matrix/client/v3/sync", get(read::sync))
        .route(
            "/_matrix/client/v3/rooms/{room}/send/{event_type}/{txn}",
            put(write::send),
        )
        .route("/_matrix/client/v3/rooms/{room}/join", post(write::join))
        .route("/_matrix/client/v3/rooms/{room}/leave", post(write::leave))
        .route(
            "/_matrix/client/v3/rooms/{room}/state/{event_type}/{state_key}",
            get(read::state),
        )
        .route(
            "/_matrix/client/v3/rooms/{room}/state/{event_type}",
            get(read::state_empty_key),
        )
        .with_state(state)
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok\n")
}
