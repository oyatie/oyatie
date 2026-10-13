use super::{Configuration, config::canonical_tenant, cookie};
use application_ontology_card_http::HumanOntologyReadSource;
use application_ontology_read::{OntologyReadError, OntologyReadSource};
use axum::{
    Router,
    extract::{Path, RawQuery, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use iam_identity_user_session::ActiveUserMembership;
use serde::Serialize;
mod query;

pub(super) fn router() -> Router<Configuration> {
    Router::new()
        .route("/api/ontology/{tenant}/types", get(types))
        .route("/api/ontology/{tenant}/schemas", get(schemas))
        .route("/api/ontology/{tenant}/link-types", get(link_types))
        .route("/api/ontology/{tenant}/objects", get(objects))
        .route("/api/ontology/{tenant}/objects/{object_ref}", get(object))
        .route(
            "/api/ontology/{tenant}/objects/{object_ref}/history",
            get(history),
        )
        .route("/api/ontology/{tenant}/audit", get(audit))
}

async fn reader(
    config: Configuration,
    tenant: &str,
    headers: &HeaderMap,
) -> Result<(HumanOntologyReadSource, ActiveUserMembership), OntologyReadError> {
    let config = config.ok_or(OntologyReadError::SourceUnavailable)?;
    if !canonical_tenant(tenant) {
        return Err(OntologyReadError::InvalidQuery);
    }
    let cookie = cookie::session_cookie(headers)
        .map_err(|()| OntologyReadError::Unauthenticated)?
        .ok_or(OntologyReadError::Unauthenticated)?;
    let member = super::transport::verified_session(&config, tenant, cookie.clone()).await?;
    Ok((config.ontology.read_source(&cookie)?, member))
}

fn response<T: Serialize>(
    result: Result<(T, ActiveUserMembership), OntologyReadError>,
) -> Response {
    match result {
        Ok((value, member)) => {
            if member.ensure_current(super::config::now()).is_err() {
                return response::<T>(Err(OntologyReadError::Unauthenticated));
            }
            let mut response = axum::Json(value).into_response();
            for (name, value) in [
                ("x-oyatie-tenant", member.tenant_id().to_owned()),
                ("x-oyatie-user", member.user_id().to_owned()),
                (
                    "x-oyatie-session-expires",
                    member.expires_at_epoch_seconds().to_string(),
                ),
            ] {
                let Ok(value) = value.parse() else {
                    return self::response::<T>(Err(OntologyReadError::SourceUnavailable));
                };
                response.headers_mut().insert(name, value);
            }
            response
        }
        Err(error) => {
            let status = match error {
                OntologyReadError::Unauthenticated => StatusCode::UNAUTHORIZED,
                OntologyReadError::Forbidden => StatusCode::FORBIDDEN,
                OntologyReadError::InvalidQuery => StatusCode::BAD_REQUEST,
                OntologyReadError::AbsentObject => StatusCode::NOT_FOUND,
                OntologyReadError::RevisionConflict | OntologyReadError::FilterConflict => {
                    StatusCode::CONFLICT
                }
                OntologyReadError::SourceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            };
            (status, axum::Json(serde_json::json!({"error":error}))).into_response()
        }
    }
}

async fn types(State(c): State<Configuration>, Path(t): Path<String>, h: HeaderMap) -> Response {
    response(
        async {
            let (source, member) = reader(c, &t, &h).await?;
            Ok((source.types(&t).await?, member))
        }
        .await,
    )
}
async fn schemas(State(c): State<Configuration>, Path(t): Path<String>, h: HeaderMap) -> Response {
    response(
        async {
            let (source, member) = reader(c, &t, &h).await?;
            Ok((source.schemas(&t).await?, member))
        }
        .await,
    )
}
async fn link_types(
    State(c): State<Configuration>,
    Path(t): Path<String>,
    h: HeaderMap,
) -> Response {
    response(
        async {
            let (source, member) = reader(c, &t, &h).await?;
            Ok((source.link_types(&t).await?, member))
        }
        .await,
    )
}
async fn objects(
    State(c): State<Configuration>,
    Path(t): Path<String>,
    RawQuery(q): RawQuery,
    h: HeaderMap,
) -> Response {
    response(
        async {
            let (source, member) = reader(c, &t, &h).await?;
            let query = query::listing(q.as_deref())?;
            Ok((source.objects(&t, &query).await?, member))
        }
        .await,
    )
}
async fn object(
    State(c): State<Configuration>,
    Path((t, o)): Path<(String, String)>,
    RawQuery(q): RawQuery,
    h: HeaderMap,
) -> Response {
    response(
        async {
            let (source, member) = reader(c, &t, &h).await?;
            Ok((
                source
                    .object(&t, &o, query::revision(q.as_deref())?)
                    .await?,
                member,
            ))
        }
        .await,
    )
}
async fn history(
    State(c): State<Configuration>,
    Path((t, o)): Path<(String, String)>,
    h: HeaderMap,
) -> Response {
    response(
        async {
            let (source, member) = reader(c, &t, &h).await?;
            Ok((source.history(&t, &o).await?, member))
        }
        .await,
    )
}
async fn audit(State(c): State<Configuration>, Path(t): Path<String>, h: HeaderMap) -> Response {
    response(
        async {
            let (source, member) = reader(c, &t, &h).await?;
            Ok((source.audit(&t).await?, member))
        }
        .await,
    )
}
