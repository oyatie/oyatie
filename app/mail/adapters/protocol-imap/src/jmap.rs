use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, FromRequest, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use mail_service::MailService;
use serde_json::{Value, json};
use session::MAX_REQUEST;
use std::sync::Arc;

#[derive(Clone)]
struct Jmap {
    service: Arc<MailService>,
    base: String,
    uploads: Arc<slots::Slots<1>>,
    requests: Arc<slots::Slots<4>>,
    workers: Arc<tokio::sync::Semaphore>,
}

pub fn jmap_router(service: Arc<MailService>, base: String) -> Router {
    Router::new()
        .route("/.well-known/jmap", get(session))
        .route("/jmap", post(request))
        .route("/download/{account}/{blob}/{name}", get(blob::download))
        .layer(DefaultBodyLimit::max(MAX_REQUEST))
        .route(
            "/upload/{account}",
            post(blob::upload).layer(DefaultBodyLimit::max(mail_kernel::MAX_MESSAGE_BYTES)),
        )
        .with_state(Jmap {
            service,
            base,
            uploads: Arc::default(),
            requests: Arc::default(),
            workers: Arc::new(tokio::sync::Semaphore::new(128)),
        })
}

fn token(headers: &HeaderMap) -> Result<&str, StatusCode> {
    headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)
}

async fn session(State(state): State<Jmap>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    let token = token(&headers)?.to_owned();
    state
        .blocking(move |state| session::response(state, &token))
        .await
}

fn problem(status: StatusCode, kind: &str) -> Response {
    (
        status,
        Json(json!({"type":format!("urn:ietf:params:jmap:error:{kind}"),"status":status.as_u16()})),
    )
        .into_response()
}

async fn request(State(state): State<Jmap>, request: Request) -> Response {
    let headers = request.headers();
    let Ok(token) = token(headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let token = token.to_owned();
    let (token, permit) = match state
        .blocking(move |state| {
            let principal = state
                .service
                .principal(&token)
                .map_err(|_| StatusCode::UNAUTHORIZED)?;
            let permit = state.requests.acquire(&principal)?;
            Ok((token, permit))
        })
        .await
    {
        Ok(admitted) => admitted,
        Err(StatusCode::TOO_MANY_REQUESTS) => {
            return problem(StatusCode::TOO_MANY_REQUESTS, "limit");
        }
        Err(status) => return status.into_response(),
    };
    let headers = headers.clone();
    let body = match tokio::time::timeout(
        std::time::Duration::from_secs(30),
        Bytes::from_request(request, &state),
    )
    .await
    {
        Ok(Ok(body)) => body,
        Ok(Err(error)) => return error.into_response(),
        Err(_) => return StatusCode::REQUEST_TIMEOUT.into_response(),
    };
    // Blocking SQLite and MIME work must not occupy the socket runtime. The
    // call loop runs as its own task that keeps admission until it exits,
    // even if the HTTP caller disconnects; the guard only stops busy waits.
    let parsed = state
        .blocking(move |_| Ok(calls::parse(&headers, &body)))
        .await;
    let run = match parsed {
        Ok(Ok(run)) => run,
        Ok(Err((status, kind))) => return problem(status, kind),
        Err(status) => return status.into_response(),
    };
    let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let _guard = retry::Guard(cancelled.clone());
    let task = tokio::spawn(async move {
        let _permit = permit;
        calls::run(state, Arc::from(token), run, cancelled).await
    });
    match task.await {
        Ok(Ok(response)) => response,
        Ok(Err(status)) => status.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

mod blob;
mod blocking;
mod calls;
mod changes;
mod compose;
mod copy;
mod email;
mod filter;
mod filter_message;
mod headers;
mod identity;
mod inspect;
mod limits;
mod mailbox;
mod mailbox_query;
mod message;
mod method;
mod query;
mod references;
mod retry;
mod session;
mod set;
mod slots;
mod snippet;
mod sort;
mod submission_envelope;
mod submission_query;
mod submission_read;
mod submission_set;
mod thread;
mod vacation;
