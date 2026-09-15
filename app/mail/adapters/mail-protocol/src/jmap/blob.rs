use super::{Jmap, token};
use axum::{
    Json,
    body::Bytes,
    extract::{FromRequest, Path, Query, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use mail_kernel::Error;
use mail_service::MailService;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;

fn status(error: Error) -> StatusCode {
    match error {
        Error::NotFound | Error::Forbidden => StatusCode::NOT_FOUND,
        Error::OverQuota => StatusCode::PAYLOAD_TOO_LARGE,
        Error::Invalid | Error::Conflict => StatusCode::BAD_REQUEST,
        Error::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub(super) fn media_type(value: &str) -> bool {
    value
        .split(';')
        .next()
        .and_then(|v| v.trim().split_once('/'))
        .is_some_and(|(a, b)| {
            [a, b].iter().all(|s| {
                !s.is_empty()
                    && s.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
            })
        })
        && HeaderValue::from_str(value).is_ok()
}

pub(super) async fn upload(
    State(state): State<Jmap>,
    Path(account): Path<String>,
    request: Request,
) -> Response {
    let headers = request.headers();
    let Ok(token) = token(headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let token = token.to_owned();
    let target = account.clone();
    let (token, permit) = match state
        .blocking(move |state| {
            let principal = state
                .service
                .principal(&token)
                .map_err(|_| StatusCode::UNAUTHORIZED)?;
            state
                .service
                .authorize(&token, &target, mail_api::Action::Write)
                .map_err(status)?;
            let permit = state.uploads.acquire(&principal)?;
            Ok((token, permit))
        })
        .await
    {
        Ok(admitted) => admitted,
        Err(status) => return status.into_response(),
    };
    let typ = match headers.get(header::CONTENT_TYPE) {
        Some(value) => match value.to_str() {
            Ok(v) if media_type(v) => v,
            _ => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => "application/octet-stream",
    }
    .to_owned();
    let body = match tokio::time::timeout(
        Duration::from_secs(120),
        Bytes::from_request(request, &state),
    )
    .await
    {
        Ok(Ok(body)) => body,
        Ok(Err(error)) => return error.into_response(),
        Err(_) => return StatusCode::REQUEST_TIMEOUT.into_response(),
    };
    state
        .blocking(move |state| {
            let _permit = permit;
            match state.service.upload(&token, &account, &body) {
                Ok(id) => Ok(Json(
                    json!({"accountId":account,"blobId":id,"type":typ,"size":body.len()}),
                )
                .into_response()),
                Err(error) => Err(status(error)),
            }
        })
        .await
        .unwrap_or_else(IntoResponse::into_response)
}

pub(super) async fn download(
    State(state): State<Jmap>,
    Path((account, id, name)): Path<(String, String, String)>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let Ok(token) = token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let token = token.to_owned();
    state
        .blocking(move |state| Ok(download_response(state, &token, account, id, name, query)))
        .await
        .unwrap_or_else(IntoResponse::into_response)
}

fn download_response(
    state: Jmap,
    token: &str,
    account: String,
    id: String,
    name: String,
    query: HashMap<String, String>,
) -> Response {
    if state.service.principal(token).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let typ = query
        .get("type")
        .map_or("application/octet-stream", String::as_str);
    if !media_type(typ) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    match super::message::blob(&state.service, token, &account, &id) {
        Ok(data) => {
            let encoded = name
                .bytes()
                .map(|b| format!("%{b:02X}"))
                .collect::<String>();
            (
                [
                    (header::CONTENT_TYPE, typ.to_owned()),
                    (
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename*=UTF-8''{encoded}"),
                    ),
                    (header::CACHE_CONTROL, "private, no-store".into()),
                    (header::X_CONTENT_TYPE_OPTIONS, "nosniff".into()),
                ],
                data,
            )
                .into_response()
        }
        Err(error) => status(error).into_response(),
    }
}

pub(super) fn copy(
    service: &MailService,
    token: &str,
    args: &Value,
) -> Result<Value, &'static str> {
    let source = args["fromAccountId"].as_str().ok_or("invalidArguments")?;
    let target = args["accountId"].as_str().ok_or("invalidArguments")?;
    if source == target {
        return Err("invalidArguments");
    }
    let ids = args["blobIds"].as_array().ok_or("invalidArguments")?;
    if ids.iter().any(|id| !id.is_string()) {
        return Err("invalidArguments");
    }
    if ids.len() > 256 {
        return Err("tooManyObjects");
    }
    service
        .read(token, source)
        .map_err(|_| "fromAccountNotFound")?;
    service.read(token, target).map_err(|_| "accountNotFound")?;
    let mut result = json!({"fromAccountId":source,"accountId":target,"copied":{},"notCopied":{}});
    for id in ids {
        let id = id.as_str().ok_or("invalidArguments")?;
        match super::message::blob(service, token, source, id)
            .and_then(|raw| service.upload(token, target, &raw))
        {
            Ok(copied) => result["copied"][id] = json!(copied),
            Err(error) => {
                result["notCopied"][id] = json!({"type": if error == Error::NotFound {"blobNotFound"} else {super::method::error(error)}})
            }
        }
    }
    Ok(result)
}
