use std::sync::Arc;
use std::time::Instant;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use foundry_submission_draft::{ActionSubmitter, SubmitError};

use crate::auth::bearer_token;
use crate::composition::AppState;
use crate::dto::{RefusalBody, SubmitRequest};

pub async fn submit_action(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let started = Instant::now();
    let Some(token) = bearer_token(
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok()),
    ) else {
        state.metrics.submit_refused();
        return refuse(
            StatusCode::UNAUTHORIZED,
            "credential",
            "no bearer credential",
        );
    };
    // Preserve the HTTP credential-before-JSON refusal order. This probe
    // conveys no authority: the port verifies the credential again itself.
    if state.verifier.verify(Some(token)).is_none() {
        state.metrics.submit_refused();
        return failure(SubmitError::Credential);
    }
    let Ok(request) = serde_json::from_str::<SubmitRequest>(&body) else {
        state.metrics.submit_refused();
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "the request body is not a well-formed submission",
        );
    };
    let submitter = crate::submission::TimedSubmission {
        state: &state,
        started,
    };
    match submitter.submit(token, request).await {
        Ok(response) => Json(response).into_response(),
        Err(error) => failure(error),
    }
}

fn failure(error: SubmitError) -> Response {
    match error {
        SubmitError::Credential => refuse(
            StatusCode::UNAUTHORIZED,
            "credential",
            "the presented credential is not recognized",
        ),
        SubmitError::UnservedTenant => refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the credential names a tenant this process does not serve",
        ),
        SubmitError::Authorization => refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the policy decision point refused this invocation",
        ),
        SubmitError::TenantMismatch => refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the credential does not belong to the expected tenant",
        ),
        SubmitError::Surface { cause } => refuse(StatusCode::BAD_REQUEST, "surface", cause),
        SubmitError::Refused { gate, cause } => refuse(StatusCode::FORBIDDEN, &gate, cause),
        SubmitError::Conflict => refuse(
            StatusCode::CONFLICT,
            "log",
            "this idempotency key is already spent on different content",
        ),
        SubmitError::Unavailable => refuse(
            StatusCode::SERVICE_UNAVAILABLE,
            "log",
            "the action log could not be written; the submission was not accepted",
        ),
    }
}

fn refuse(status: StatusCode, gate: &str, cause: &str) -> Response {
    (
        status,
        Json(RefusalBody {
            gate: gate.to_owned(),
            cause: cause.to_owned(),
        }),
    )
        .into_response()
}
