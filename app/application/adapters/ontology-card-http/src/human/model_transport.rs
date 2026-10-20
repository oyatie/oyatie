use super::{HumanOntologyModelSource, transport};
use application_ontology_read::*;
use reqwest::{StatusCode, Url, header};

pub(super) fn unavailable() -> ModelSourceError {
    ModelSourceError::Refused(ModelRefusalCode::ModelUnavailable)
}
fn status(code: ModelRefusalCode) -> StatusCode {
    match code {
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
    }
}
impl HumanOntologyModelSource {
    pub(super) fn url(
        &self,
        route: &str,
        tenant: &str,
        id: Option<ModelNumber>,
    ) -> Result<Url, ModelSourceError> {
        if !tenant.starts_with("ten_")
            || tenant.len() <= 4
            || tenant.len() > 256
            || !tenant
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
        {
            return Err(ModelSourceError::Refused(
                ModelRefusalCode::MalformedRequest,
            ));
        }
        let mut url = self.transport.base.clone();
        let id = id.map(|number| number.get().to_string());
        url.path_segments_mut().map_err(|_| unavailable())?.extend(
            route
                .trim_start_matches('/')
                .split('/')
                .map(|part| match part {
                    "{tenant}" => tenant,
                    "{draft_id}" => id.as_deref().unwrap_or_default(),
                    value => value,
                }),
        );
        Ok(url)
    }
    pub(super) async fn request<T, E>(
        &self,
        url: Url,
        body: Option<Vec<u8>>,
        maximum: usize,
        expected: StatusCode,
        decode: fn(&[u8]) -> Result<T, E>,
    ) -> Result<T, ModelSourceError> {
        let mutation = body.is_some();
        let uncertain = || {
            if mutation {
                ModelSourceError::OutcomeUnknown
            } else {
                unavailable()
            }
        };
        let mut request = self
            .transport
            .client
            .request(
                if mutation {
                    reqwest::Method::POST
                } else {
                    reqwest::Method::GET
                },
                url,
            )
            .header(header::COOKIE, &self.cookie)
            .header(header::ACCEPT, "application/json");
        if let Some(body) = body {
            request = request
                .header(header::ORIGIN, &self.origin)
                .header(header::CONTENT_TYPE, "application/json")
                .body(body);
        }
        let response = request.send().await.map_err(|_| uncertain())?;
        let received = response.status();
        if received == expected {
            let bytes = transport::bytes(response, maximum)
                .await
                .map_err(|_| uncertain())?;
            return decode(&bytes).map_err(|_| uncertain());
        }
        let bytes = transport::bytes(response, self.limits.refusal)
            .await
            .map_err(|_| uncertain())?;
        let refusal = ModelRefusal::decode(&bytes).map_err(|_| uncertain())?;
        if status(refusal.error) != received {
            return Err(uncertain());
        }
        if mutation && refusal.error == ModelRefusalCode::ModelUnavailable {
            return Err(ModelSourceError::OutcomeUnknown);
        }
        Err(ModelSourceError::Refused(refusal.error))
    }
}
