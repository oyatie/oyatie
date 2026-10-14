use axum::{
    body::{Body, to_bytes},
    http::{HeaderMap, StatusCode, header},
    response::Response,
};

use super::{HumanReadState, invalid};
use crate::reads::refuse;

pub(super) async fn request_body(
    state: &HumanReadState,
    headers: &HeaderMap,
    body: Body,
) -> Result<String, Response> {
    let types: Vec<_> = headers.get_all(header::CONTENT_TYPE).iter().collect();
    if types.len() != 1 || types[0].as_bytes() != b"application/json" {
        state.app.metrics.read_refused();
        return Err(refuse(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "surface",
            "a relationship read requires application/json",
        ));
    }
    let lengths: Vec<_> = headers.get_all(header::CONTENT_LENGTH).iter().collect();
    if lengths.len() > 1 {
        return Err(invalid(state));
    }
    if let Some(length) = lengths.first() {
        let Ok(raw) = length.to_str() else {
            return Err(invalid(state));
        };
        if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(invalid(state));
        }
        let Ok(length) = raw.parse::<u64>() else {
            return Err(invalid(state));
        };
        if length > 4096 {
            return Err(too_large(state));
        }
    }
    let bytes = to_bytes(body, 4096).await.map_err(|error| {
        if std::error::Error::source(&error)
            .is_some_and(|cause| cause.is::<http_body_util::LengthLimitError>())
        {
            too_large(state)
        } else {
            invalid(state)
        }
    })?;
    String::from_utf8(bytes.to_vec()).map_err(|_| invalid(state))
}
fn too_large(state: &HumanReadState) -> Response {
    state.app.metrics.read_refused();
    refuse(
        StatusCode::PAYLOAD_TOO_LARGE,
        "surface",
        "a relationship read body may hold at most 4096 bytes",
    )
}
