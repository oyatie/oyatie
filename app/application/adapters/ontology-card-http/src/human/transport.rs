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
    response: Response,
    maximum: usize,
) -> Result<T, OntologyReadError> {
    serde_json::from_slice(&bytes(response, maximum).await?)
        .map_err(|_| OntologyReadError::SourceUnavailable)
}
pub(super) async fn bytes(
    mut response: Response,
    maximum: usize,
) -> Result<Vec<u8>, OntologyReadError> {
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
    Ok(body)
}

pub(super) fn cookie(session_cookie: &str) -> Result<header::HeaderValue, OntologyReadError> {
    let token = session_cookie
        .strip_prefix("__Host-oya_session=v1.")
        .ok_or(OntologyReadError::Unauthenticated)?;
    if token.len() != 43
        || !token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        || !token
            .as_bytes()
            .last()
            .is_some_and(|b| b"AEIMQUYcgkosw048".contains(b))
    {
        return Err(OntologyReadError::Unauthenticated);
    }
    let mut cookie = header::HeaderValue::from_str(session_cookie)
        .map_err(|_| OntologyReadError::Unauthenticated)?;
    cookie.set_sensitive(true);
    Ok(cookie)
}
