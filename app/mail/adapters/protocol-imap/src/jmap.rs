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
    // Blocking SQLite and MIME work must not occupy the socket runtime. Keep
    // admission until the worker exits, even if the HTTP caller disconnects.
    state
        .blocking(move |state| {
            let _permit = permit;
            Ok(dispatch(state, &token, headers, body))
        })
        .await
        .unwrap_or_else(IntoResponse::into_response)
}

fn dispatch(state: Jmap, token: &str, headers: HeaderMap, body: Bytes) -> Response {
    if !headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(';')
                .next()
                .is_some_and(|v| v.trim().eq_ignore_ascii_case("application/json"))
        })
    {
        return problem(StatusCode::UNSUPPORTED_MEDIA_TYPE, "notJSON");
    }
    let Ok(body) = serde_json::from_slice::<Value>(&body) else {
        return problem(StatusCode::BAD_REQUEST, "notJSON");
    };
    let (Some(using), Some(calls)) = (body["using"].as_array(), body["methodCalls"].as_array())
    else {
        return problem(StatusCode::BAD_REQUEST, "notRequest");
    };
    if using.iter().any(|c| !c.is_string()) {
        return problem(StatusCode::BAD_REQUEST, "notRequest");
    }
    if using.iter().any(|c| !session::known(c)) {
        return problem(StatusCode::BAD_REQUEST, "unknownCapability");
    }
    if calls.len() > 16 {
        return problem(StatusCode::BAD_REQUEST, "limit");
    }
    if calls.iter().any(|c| {
        !c.is_array()
            || c.as_array().is_none_or(|a| a.len() != 3)
            || !c[0].is_string()
            || !c[1].is_object()
            || !c[2].is_string()
    }) {
        return problem(StatusCode::BAD_REQUEST, "notRequest");
    }
    let mut responses = vec![];
    let mut remaining = limits::MAX_RESPONSE;
    let mut created_ids = std::collections::BTreeMap::new();
    for call in calls {
        let name = call[0].as_str().unwrap_or_default();
        let args = references::resolve(&call[1], &responses);
        let capability = session::capability(name);
        let mut destroy = None;
        // Keep room for the largest bounded mutation response before executing
        // a side effect. Read methods receive the remaining materialization budget.
        let result = if remaining < 2 * MAX_REQUEST {
            Err("limit")
        } else if !using.iter().any(|v| v == capability) {
            Err("unknownMethod")
        } else {
            args.and_then(|mut a| {
                if name != "Core/echo" {
                    references::created(&mut a, &created_ids);
                }
                let result = method(
                    &state.service,
                    token,
                    name,
                    &a,
                    remaining - 2 * MAX_REQUEST,
                    &mut destroy,
                );
                if name == "Email/copy"
                    && let Ok(result) = &result
                {
                    destroy = copy::destroy_args(&a, result);
                }
                result
            })
        };
        let result = result.and_then(|value| {
            if let Err(error) = limits::charge(&value, &mut remaining) {
                remaining = 0;
                // A committed mutation must retain its success receipt. Its
                // bounded response was reserved before execution.
                if !matches!(
                    name,
                    "Identity/set"
                        | "EmailSubmission/set"
                        | "VacationResponse/set"
                        | "Email/set"
                        | "Mailbox/set"
                        | "Email/copy"
                        | "Email/import"
                        | "Blob/copy"
                ) {
                    return Err(error);
                }
            }
            Ok(value)
        });
        if let Ok(value) = &result
            && let Some(objects) = value["created"].as_object()
        {
            for (key, value) in objects {
                if let Some(id) = value["id"].as_str() {
                    created_ids.insert(key.clone(), id.to_owned());
                }
            }
        }
        responses.push(match result {
            Ok(value) => json!([name, value, call[2]]),
            Err(kind) => json!(["error",{"type":kind},call[2]]),
        });
        if let Some(args) = destroy {
            responses.push(
                match method(
                    &state.service,
                    token,
                    "Email/set",
                    &args,
                    remaining,
                    &mut None,
                ) {
                    Ok(value) => {
                        if limits::charge(&value, &mut remaining).is_err() {
                            remaining = 0;
                        }
                        json!(["Email/set", value, call[2]])
                    }
                    Err(kind) => json!(["error", {"type":kind}, call[2]]),
                },
            );
        }
    }
    Json(json!({"methodResponses":responses,"sessionState":"1"})).into_response()
}

mod blob;
mod blocking;
mod changes;
mod compose;
mod copy;
mod email;
mod filter;
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
mod session;
mod set;
mod slots;
mod snippet;
mod submission_envelope;
mod submission_query;
mod submission_read;
mod submission_set;
mod thread;
mod vacation;
use method::method;
