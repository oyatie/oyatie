use application_ontology_read::{
    MAX_RELATIONSHIP_REQUEST_BYTES, OntologyReadError, OntologyReadSource, RelationshipQuery,
};
use axum::{
    Router,
    body::Bytes,
    extract::{
        DefaultBodyLimit, Path, State,
        rejection::{BytesRejection, PathRejection},
    },
    http::{HeaderMap, StatusCode, header},
    response::Response,
    routing::post,
};

use super::{Configuration, reader, response};

pub(super) fn router() -> Router<Configuration> {
    Router::new()
        .route("/api/ontology/{tenant}/search-around", post(search_around))
        .layer(DefaultBodyLimit::max(MAX_RELATIONSHIP_REQUEST_BYTES))
}

async fn search_around(
    State(config): State<Configuration>,
    path: Result<Path<String>, PathRejection>,
    headers: HeaderMap,
    bytes: Result<Bytes, BytesRejection>,
) -> Response {
    let guarded = async {
        let tenant = path.map_err(|_| OntologyReadError::InvalidQuery)?.0;
        let origin = config.as_ref().map(|c| c.public_origin.clone());
        let (source, member) = reader(config, &tenant, &headers).await?;
        member
            .ensure_current(super::super::config::now())
            .map_err(|_| OntologyReadError::Unauthenticated)?;
        let mut origins = headers.get_all(header::ORIGIN).iter();
        if origins.next().and_then(|value| value.to_str().ok()) != origin.as_deref()
            || origins.next().is_some()
        {
            return Err(OntologyReadError::Forbidden);
        }
        Ok((tenant, source, member))
    }
    .await;
    let (tenant, source, member) = match guarded {
        Ok(value) => value,
        Err(error) => return response::<()>(Err(error)),
    };
    let mut media = headers.get_all(header::CONTENT_TYPE).iter();
    if media.next().and_then(|value| value.to_str().ok()) != Some("application/json")
        || media.next().is_some()
    {
        return protocol_refusal(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
    let bytes = match bytes {
        Ok(bytes) => bytes,
        Err(error) => {
            return protocol_refusal(if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
                StatusCode::PAYLOAD_TOO_LARGE
            } else {
                StatusCode::BAD_REQUEST
            });
        }
    };
    response(
        async {
            let query = RelationshipQuery::decode(&bytes)?;
            Ok((source.search_around(&tenant, &query).await?, member))
        }
        .await,
    )
}

fn protocol_refusal(status: StatusCode) -> Response {
    let mut result = response::<()>(Err(OntologyReadError::InvalidQuery));
    *result.status_mut() = status;
    result
}
