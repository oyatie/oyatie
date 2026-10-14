use application_ontology_read::{
    MAX_RELATIONSHIP_RESPONSE_BYTES, OntologyReadError, OntologyReadSource, RelationshipGraph,
    RelationshipQuery,
};
use reqwest::{StatusCode, header};

use super::{HumanOntologyReadSource, Refusal, transport};

impl HumanOntologyReadSource {
    pub(super) async fn relationships(
        &self,
        tenant: &str,
        query: &RelationshipQuery,
    ) -> Result<RelationshipGraph, OntologyReadError> {
        query.validate()?;
        // Catalog and traversal are separate authorized reads, not an atomic snapshot.
        let links = self.link_types(tenant).await.map_err(|error| match error {
            OntologyReadError::AbsentObject
            | OntologyReadError::RevisionConflict
            | OntologyReadError::FilterConflict => OntologyReadError::SourceUnavailable,
            error => error,
        })?;
        let link = links
            .iter()
            .find(|link| link.link_type == query.link_type)
            .ok_or(OntologyReadError::InvalidQuery)?;
        query.validate_for(link)?;
        let bytes = serde_json::to_vec(query).map_err(|_| OntologyReadError::InvalidQuery)?;
        let response = self
            .transport
            .client
            .post(self.url(tenant, &["search-around"])?)
            .header(header::COOKIE, &self.cookie)
            .header(header::ACCEPT, "application/json")
            .header(header::CONTENT_TYPE, "application/json")
            .body(bytes)
            .send()
            .await
            .map_err(|_| OntologyReadError::SourceUnavailable)?;
        match response.status() {
            StatusCode::OK => RelationshipGraph::decode(
                &transport::bytes(response, MAX_RELATIONSHIP_RESPONSE_BYTES).await?,
                query,
                link,
            ),
            StatusCode::UNAUTHORIZED => Err(OntologyReadError::Unauthenticated),
            StatusCode::FORBIDDEN => Err(OntologyReadError::Forbidden),
            StatusCode::PAYLOAD_TOO_LARGE | StatusCode::UNSUPPORTED_MEDIA_TYPE => {
                Err(OntologyReadError::InvalidQuery)
            }
            status @ (StatusCode::BAD_REQUEST | StatusCode::CONFLICT) => {
                let refusal: Refusal = transport::decode(response, 16 * 1024).await?;
                Err(refusal.error(status))
            }
            _ => Err(OntologyReadError::SourceUnavailable),
        }
    }
}
