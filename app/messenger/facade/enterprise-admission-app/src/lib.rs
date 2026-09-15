#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod config;
mod http;
mod state;
mod translate;

use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::middleware;
use axum::routing::{get, post};
use messenger_policy_api::Policy;

pub use config::{Config, Tenant};
pub use state::{AppState, compose};
pub use translate::Admission;

pub fn router<P: Policy + 'static>(state: Arc<AppState<P>>) -> Router {
    Router::new()
        .route("/v1/messenger/admit", post(http::admit::<P>))
        .layer(DefaultBodyLimit::max(1_048_576))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            http::authenticate::<P>,
        ))
        .route("/healthz", get(ok))
        .route("/readyz", get(ok))
        .route("/metrics", get(http::metrics::<P>))
        .with_state(state)
}

async fn ok() -> StatusCode {
    StatusCode::OK
}
