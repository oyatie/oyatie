use super::*;
use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};

pub(super) type Refusal = Box<Response>;

pub(super) fn refuse(status: StatusCode, error: ModelRefusalCode) -> Refusal {
    Box::new((status, axum::Json(ModelRefusal { error })).into_response())
}

pub(super) fn unavailable() -> Refusal {
    refuse(
        StatusCode::SERVICE_UNAVAILABLE,
        ModelRefusalCode::ModelUnavailable,
    )
}

pub(super) fn malformed() -> Refusal {
    refuse(StatusCode::BAD_REQUEST, ModelRefusalCode::MalformedRequest)
}

pub(super) fn source_error(error: ModelError) -> Refusal {
    let (status, code) = match error {
        ModelError::TenantMismatch | ModelError::OwnerMismatch => {
            (StatusCode::FORBIDDEN, ModelRefusalCode::Forbidden)
        }
        ModelError::MissingDraft => (StatusCode::NOT_FOUND, ModelRefusalCode::DraftNotFound),
        ModelError::DraftConflict { .. } => (StatusCode::CONFLICT, ModelRefusalCode::DraftConflict),
        ModelError::HeadConflict { .. } => (StatusCode::CONFLICT, ModelRefusalCode::HeadConflict),
        ModelError::DraftClosed => (StatusCode::CONFLICT, ModelRefusalCode::DraftClosed),
        ModelError::MalformedDocument | ModelError::InvalidContext => {
            (StatusCode::BAD_REQUEST, ModelRefusalCode::MalformedRequest)
        }
        ModelError::Limit { .. } => (
            StatusCode::PAYLOAD_TOO_LARGE,
            ModelRefusalCode::BodyTooLarge,
        ),
        _ => (
            StatusCode::SERVICE_UNAVAILABLE,
            ModelRefusalCode::ModelUnavailable,
        ),
    };
    refuse(status, code)
}

pub(super) fn encoded(
    status: StatusCode,
    bytes: Result<Vec<u8>, ModelError>,
) -> Result<Response, Refusal> {
    let bytes = bytes.map_err(|_| unavailable())?;
    Ok((status, [(header::CONTENT_TYPE, "application/json")], bytes).into_response())
}

pub(super) async fn mutation_body(
    state: &ModelState,
    headers: &HeaderMap,
    body: Body,
    bound: usize,
) -> Result<axum::body::Bytes, Refusal> {
    let origin = state
        .origin
        .as_ref()
        .ok_or_else(|| refuse(StatusCode::FORBIDDEN, ModelRefusalCode::Forbidden))?;
    let origins: Vec<_> = headers.get_all(header::ORIGIN).iter().collect();
    if origins.len() != 1 || origins[0].as_bytes() != origin.0.as_bytes() {
        return Err(refuse(StatusCode::FORBIDDEN, ModelRefusalCode::Forbidden));
    }
    let types: Vec<_> = headers.get_all(header::CONTENT_TYPE).iter().collect();
    if types.len() != 1 || types[0].as_bytes() != b"application/json" {
        return Err(refuse(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ModelRefusalCode::UnsupportedContentType,
        ));
    }
    let lengths: Vec<_> = headers.get_all(header::CONTENT_LENGTH).iter().collect();
    if lengths.len() > 1 {
        return Err(malformed());
    }
    if let Some(length) = lengths.first() {
        let raw = length.to_str().map_err(|_| malformed())?;
        if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(malformed());
        }
        let length: u64 = raw.parse().map_err(|_| malformed())?;
        if length > bound as u64 {
            return Err(refuse(
                StatusCode::PAYLOAD_TOO_LARGE,
                ModelRefusalCode::BodyTooLarge,
            ));
        }
    }
    to_bytes(body, bound).await.map_err(|error| {
        if std::error::Error::source(&error)
            .is_some_and(|cause| cause.is::<http_body_util::LengthLimitError>())
        {
            refuse(
                StatusCode::PAYLOAD_TOO_LARGE,
                ModelRefusalCode::BodyTooLarge,
            )
        } else {
            malformed()
        }
    })
}
