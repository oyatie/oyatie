use super::{
    Configuration,
    config::{canonical_tenant, now},
    cookie,
};
use application_ontology_card_http::HumanOntologyModelSource;
use application_ontology_read::*;
use axum::{
    Router,
    body::Bytes,
    extract::{
        DefaultBodyLimit, Path, RawQuery, State,
        rejection::{BytesRejection, PathRejection},
    },
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use iam_identity_user_session::ActiveUserMembership;
use serde::Serialize;
#[path = "ontology_model_reads.rs"]
mod reads;

pub(super) fn router() -> Router<Configuration> {
    let maximum = model_wire_limits()
        .map(|l| l.create.max(l.save))
        .unwrap_or(0);
    Router::new()
        .route("/api/ontology/{tenant}/model", get(workspace))
        .route(
            "/api/ontology/{tenant}/model/drafts",
            get(reads::drafts).post(create),
        )
        .route("/api/ontology/{tenant}/model/drafts/{draft_id}", get(draft))
        .route(
            "/api/ontology/{tenant}/model/drafts/{draft_id}/base",
            get(reads::base),
        )
        .route(
            "/api/ontology/{tenant}/model/drafts/{draft_id}/save",
            post(save),
        )
        .route(
            "/api/ontology/{tenant}/model/drafts/{draft_id}/validate",
            post(validate),
        )
        .layer(DefaultBodyLimit::max(maximum))
}
type ModelPath<T> = Result<Path<T>, PathRejection>;
fn extract_path<T>(path: ModelPath<T>) -> Result<T, ModelSourceError> {
    path.map(|Path(value)| value)
        .map_err(|_| refused(ModelRefusalCode::MalformedRequest))
}
fn extract_body(body: Result<Bytes, BytesRejection>) -> Result<Bytes, ModelSourceError> {
    body.map_err(|error| {
        refused(if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ModelRefusalCode::BodyTooLarge
        } else {
            ModelRefusalCode::MalformedRequest
        })
    })
}
fn refused(code: ModelRefusalCode) -> ModelSourceError {
    ModelSourceError::Refused(code)
}
async fn source(
    c: Configuration,
    tenant: &str,
    headers: &HeaderMap,
    mutation: bool,
) -> Result<(HumanOntologyModelSource, ActiveUserMembership), ModelSourceError> {
    let c = c.ok_or(refused(ModelRefusalCode::ModelUnavailable))?;
    if !canonical_tenant(tenant) {
        return Err(refused(ModelRefusalCode::MalformedRequest));
    }
    let cookie = cookie::session_cookie(headers)
        .map_err(|_| refused(ModelRefusalCode::Unauthenticated))?
        .ok_or(refused(ModelRefusalCode::Unauthenticated))?;
    let member = super::transport::verified_session(&c, tenant, cookie.clone())
        .await
        .map_err(|e| {
            refused(match e {
                OntologyReadError::Unauthenticated => ModelRefusalCode::Unauthenticated,
                OntologyReadError::Forbidden => ModelRefusalCode::Forbidden,
                _ => ModelRefusalCode::ModelUnavailable,
            })
        })?;
    member
        .ensure_current(now())
        .map_err(|_| refused(ModelRefusalCode::Unauthenticated))?;
    if mutation {
        let mut origins = headers.get_all(header::ORIGIN).iter();
        if origins.next().and_then(|v| v.to_str().ok()) != Some(&c.public_origin)
            || origins.next().is_some()
        {
            return Err(refused(ModelRefusalCode::Forbidden));
        }
        let mut types = headers.get_all(header::CONTENT_TYPE).iter();
        if !matches!(
            types.next().and_then(|v| v.to_str().ok()),
            Some("application/json")
        ) || types.next().is_some()
        {
            return Err(refused(ModelRefusalCode::UnsupportedContentType));
        }
    }
    Ok((c.ontology.model_source(&cookie, &c.public_origin)?, member))
}
fn response<T: Serialize>(
    result: Result<(T, ActiveUserMembership), ModelSourceError>,
    success: StatusCode,
) -> Response {
    match result {
        Ok((value, member)) => {
            if member.ensure_current(now()).is_err() {
                return response::<T>(Err(refused(ModelRefusalCode::Unauthenticated)), success);
            }
            let mut response = (success, axum::Json(value)).into_response();
            for (name, value) in [
                ("x-oyatie-tenant", member.tenant_id().to_owned()),
                ("x-oyatie-user", member.user_id().to_owned()),
                (
                    "x-oyatie-session-expires",
                    member.expires_at_epoch_seconds().to_string(),
                ),
            ] {
                let Ok(value) = value.parse() else {
                    return self::response::<T>(
                        Err(refused(ModelRefusalCode::ModelUnavailable)),
                        success,
                    );
                };
                response.headers_mut().insert(name, value);
            }
            response
        }
        Err(ModelSourceError::OutcomeUnknown) => (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({"error":"outcome_unknown"})),
        )
            .into_response(),
        Err(ModelSourceError::Refused(error)) => {
            let status = match error {
                ModelRefusalCode::Unauthenticated => StatusCode::UNAUTHORIZED,
                ModelRefusalCode::Forbidden => StatusCode::FORBIDDEN,
                ModelRefusalCode::DraftNotFound => StatusCode::NOT_FOUND,
                ModelRefusalCode::DraftConflict
                | ModelRefusalCode::HeadConflict
                | ModelRefusalCode::DraftClosed => StatusCode::CONFLICT,
                ModelRefusalCode::MalformedRequest => StatusCode::BAD_REQUEST,
                ModelRefusalCode::BodyTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
                ModelRefusalCode::UnsupportedContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
                ModelRefusalCode::ModelUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            };
            (status, axum::Json(ModelRefusal { error })).into_response()
        }
    }
}
fn malformed(error: ModelError) -> ModelSourceError {
    refused(match error {
        ModelError::Limit { .. } => ModelRefusalCode::BodyTooLarge,
        _ => ModelRefusalCode::MalformedRequest,
    })
}
fn number(value: &str) -> Result<ModelNumber, ModelSourceError> {
    ModelNumber::parse(value).map_err(|_| refused(ModelRefusalCode::MalformedRequest))
}
fn own(
    draft: ModelDraftWire,
    member: &ActiveUserMembership,
) -> Result<ModelDraftWire, ModelSourceError> {
    if draft.owner != member.user_id() || draft.tenant_id != member.tenant_id() {
        return Err(refused(ModelRefusalCode::ModelUnavailable));
    }
    Ok(draft)
}
async fn workspace(
    State(c): State<Configuration>,
    path: ModelPath<String>,
    h: HeaderMap,
) -> Response {
    response(
        async {
            let t = extract_path(path)?;
            let (s, m) = source(c, &t, &h, false).await?;
            Ok((s.workspace(&t).await?, m))
        }
        .await,
        StatusCode::OK,
    )
}
async fn draft(
    State(c): State<Configuration>,
    path: ModelPath<(String, String)>,
    h: HeaderMap,
) -> Response {
    response(
        async {
            let (t, id) = extract_path(path)?;
            let (s, m) = source(c, &t, &h, false).await?;
            Ok((own(s.draft(&t, number(&id)?).await?, &m)?, m))
        }
        .await,
        StatusCode::OK,
    )
}
async fn create(
    State(c): State<Configuration>,
    path: ModelPath<String>,
    h: HeaderMap,
    b: Result<Bytes, BytesRejection>,
) -> Response {
    response(
        async {
            let t = extract_path(path)?;
            let (s, m) = source(c, &t, &h, true).await?;
            let b = extract_body(b)?;
            let request = CreateModelDraft::decode(&b).map_err(malformed)?;
            Ok((
                own(s.create_draft(&t, &request).await?, &m)
                    .map_err(|_| ModelSourceError::OutcomeUnknown)?,
                m,
            ))
        }
        .await,
        StatusCode::CREATED,
    )
}
async fn save(
    State(c): State<Configuration>,
    path: ModelPath<(String, String)>,
    h: HeaderMap,
    b: Result<Bytes, BytesRejection>,
) -> Response {
    response(
        async {
            let (t, id) = extract_path(path)?;
            let (s, m) = source(c, &t, &h, true).await?;
            let b = extract_body(b)?;
            let request = SaveModelDraft::decode(&b).map_err(malformed)?;
            Ok((
                own(s.save_draft(&t, number(&id)?, &request).await?, &m)
                    .map_err(|_| ModelSourceError::OutcomeUnknown)?,
                m,
            ))
        }
        .await,
        StatusCode::OK,
    )
}
async fn validate(
    State(c): State<Configuration>,
    path: ModelPath<(String, String)>,
    h: HeaderMap,
    b: Result<Bytes, BytesRejection>,
) -> Response {
    response(
        async {
            let (t, id) = extract_path(path)?;
            let (s, m) = source(c, &t, &h, true).await?;
            let b = extract_body(b)?;
            let request = PrepareModelDraft::decode(&b).map_err(malformed)?;
            Ok((s.validate_draft(&t, number(&id)?, &request).await?, m))
        }
        .await,
        StatusCode::OK,
    )
}
