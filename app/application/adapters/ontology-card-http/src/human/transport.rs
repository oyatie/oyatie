use application_ontology_read::OntologyReadError;
use reqwest::{Response, Url, header};
use serde::de::DeserializeOwned;

use super::HumanOntologyCardSource;

impl HumanOntologyCardSource {
    pub(super) async fn request(
        &self,
        url: Url,
        cookie: &header::HeaderValue,
    ) -> Result<Response, OntologyReadError> {
        self.client
            .get(url)
            .header(header::COOKIE, cookie)
            .header(header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|_| OntologyReadError::SourceUnavailable)
    }
}

pub(super) async fn decode<T: DeserializeOwned>(
    mut response: Response,
    maximum: usize,
) -> Result<T, OntologyReadError> {
    if response
        .content_length()
        .is_some_and(|size| size > maximum as u64)
    {
        return Err(OntologyReadError::SourceUnavailable);
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| OntologyReadError::SourceUnavailable)?
    {
        if chunk.len() > maximum - body.len() {
            return Err(OntologyReadError::SourceUnavailable);
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(|_| OntologyReadError::SourceUnavailable)
}
