use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use messenger_domain::Error;
use messenger_policy_api::Policy;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::state::AppState;
use crate::translate::Admission;

pub(crate) async fn authenticate<P: Policy + 'static>(
    State(app): State<Arc<AppState<P>>>,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let response = authenticate_request(State(app.clone()), request, next).await;
    app.metrics.requests.fetch_add(1, Ordering::Relaxed);
    app.metrics.elapsed_micros.fetch_add(
        u64::try_from(start.elapsed().as_micros().min(u64::MAX as u128)).unwrap_or(u64::MAX),
        Ordering::Relaxed,
    );
    if response.status().is_client_error() {
        app.metrics.denials.fetch_add(1, Ordering::Relaxed);
    }
    if response.status().is_server_error() {
        app.metrics.unavailable.fetch_add(1, Ordering::Relaxed);
    }
    response
}

async fn authenticate_request<P: Policy + 'static>(
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
    let result = app.authorize_admission(&request).await;
    app.metrics.policy_unavailable.store(
        matches!(result, Err(Error::Unavailable(_))),
        Ordering::Relaxed,
    );
    result.map_err(|reason| {
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

pub(crate) async fn metrics<P: Policy + 'static>(State(app): State<Arc<AppState<P>>>) -> String {
    let requests = app.metrics.requests.load(Ordering::Relaxed);
    let denials = app.metrics.denials.load(Ordering::Relaxed);
    let unavailable = app.metrics.unavailable.load(Ordering::Relaxed);
    let elapsed = app.metrics.elapsed_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0;
    let last = u8::from(app.metrics.policy_unavailable.load(Ordering::Relaxed));
    format!(
        "messenger_admission_requests_total {requests}\n\
         messenger_admission_denials_total {denials}\n\
         messenger_admission_unavailable_total {unavailable}\n\
         messenger_admission_duration_seconds_sum {elapsed}\n\
         messenger_admission_duration_seconds_count {requests}\n\
         messenger_admission_last_policy_unavailable {last}\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Tenant};
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use std::collections::BTreeMap;
    use tower::ServiceExt;

    struct Grant;

    impl Policy for Grant {
        async fn authorize(
            &self,
            _: &str,
            _: &str,
            _: messenger_policy_api::Action,
            _: &str,
        ) -> Result<(), Error> {
            Ok(())
        }
    }

    fn app() -> Arc<AppState<Grant>> {
        let mut subjects = BTreeMap::new();
        subjects.insert("@alice:local".into(), "usr_alice".into());
        subjects.insert("@bot:local".into(), "svc_archive".into());
        let mut tenants = BTreeMap::new();
        tenants.insert(
            "acme".into(),
            Tenant {
                audit_bot: "@bot:local".into(),
                subjects,
            },
        );
        AppState::bind(
            Config {
                listen: "127.0.0.1:0".parse().unwrap(),
                policy_origin: "http://127.0.0.1:1".into(),
                policy_version: "v1".into(),
                workload_identity_file: None,
                tenants,
            },
            &"t".repeat(32),
            Grant,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn saturation_is_unavailable_without_a_queue() {
        let state = app();
        let _held: Vec<_> = (0..64)
            .map(|_| state.capacity.clone().try_acquire_owned().unwrap())
            .collect();
        let response = crate::router(state)
            .oneshot(
                HttpRequest::post("/v1/messenger/admit")
                    .header("authorization", format!("Bearer {}", "t".repeat(32)))
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
