use super::{HumanOntologyCardSource, transport};
use application_ontology_read::*;
use reqwest::{StatusCode, Url, header};

/// Cookie and mutation Origin are fixed by the guarded shell, never browser data.
pub struct HumanOntologyModelSource {
    transport: HumanOntologyCardSource,
    cookie: header::HeaderValue, // data_class: SECRET
    origin: header::HeaderValue,
    limits: ModelWireLimits,
}
impl HumanOntologyCardSource {
    pub fn model_source(
        &self,
        session_cookie: &str,
        origin: &str,
    ) -> Result<HumanOntologyModelSource, ModelSourceError> {
        let cookie = transport::cookie(session_cookie)
            .map_err(|_| ModelSourceError::Refused(ModelRefusalCode::Unauthenticated))?;
        let url = Url::parse(origin).map_err(|_| unavailable())?;
        if url.origin().ascii_serialization() != origin
            || !url.username().is_empty()
            || url.password().is_some()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
            || !(url.scheme() == "https"
                || self.base.scheme() == "http"
                    && url.scheme() == "http"
                    && matches!(url.host_str(), Some("localhost" | "127.0.0.1")))
        {
            return Err(unavailable());
        }
        Ok(HumanOntologyModelSource {
            transport: self.clone(),
            cookie,
            origin: header::HeaderValue::from_str(origin).map_err(|_| unavailable())?,
            limits: model_wire_limits().map_err(|_| unavailable())?,
        })
    }
}
fn unavailable() -> ModelSourceError {
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
    fn url(
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
    async fn request<T, E>(
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

impl OntologyModelSource for HumanOntologyModelSource {
    async fn workspace(&self, tenant: &str) -> Result<ModelWorkspace, ModelSourceError> {
        self.request(
            self.url(MODEL_WORKSPACE_ROUTE, tenant, None)?,
            None,
            self.limits.workspace,
            StatusCode::OK,
            ModelWorkspace::decode,
        )
        .await
    }
    async fn owned_drafts(
        &self,
        tenant: &str,
        after: Option<ModelNumber>,
        limit: usize,
    ) -> Result<ModelDraftPage, ModelSourceError> {
        if limit == 0 || limit > MAX_MODEL_ITEMS {
            return Err(ModelSourceError::Refused(
                ModelRefusalCode::MalformedRequest,
            ));
        }
        let mut url = self.url(MODEL_DRAFTS_ROUTE, tenant, None)?;
        url.query_pairs_mut()
            .append_pair("limit", &limit.to_string());
        if let Some(after) = after {
            url.query_pairs_mut()
                .append_pair("after", &after.get().to_string());
        }
        let page = self
            .request(
                url,
                None,
                self.limits.draft_page,
                StatusCode::OK,
                ModelDraftPage::decode,
            )
            .await?;
        if page.drafts.len() > limit
            || page
                .drafts
                .first()
                .is_some_and(|d| after.is_some_and(|a| d.id <= a))
        {
            return Err(unavailable());
        }
        Ok(page)
    }
    async fn draft(
        &self,
        tenant: &str,
        id: ModelNumber,
    ) -> Result<ModelDraftWire, ModelSourceError> {
        let draft = self
            .request(
                self.url(MODEL_DRAFT_ROUTE, tenant, Some(id))?,
                None,
                self.limits.draft,
                StatusCode::OK,
                ModelDraftWire::decode,
            )
            .await?;
        if draft.id != id || draft.tenant_id != tenant {
            return Err(unavailable());
        }
        Ok(draft)
    }
    async fn create_draft(
        &self,
        tenant: &str,
        request: &CreateModelDraft,
    ) -> Result<ModelDraftWire, ModelSourceError> {
        let body = request
            .encode()
            .map_err(|_| ModelSourceError::Refused(ModelRefusalCode::MalformedRequest))?;
        let draft = self
            .request(
                self.url(MODEL_DRAFTS_ROUTE, tenant, None)?,
                Some(body),
                self.limits.draft,
                StatusCode::CREATED,
                ModelDraftWire::decode,
            )
            .await?;
        if draft.tenant_id != tenant
            || draft.base_generation != request.expected_generation
            || draft.version.get() != 1
            || draft.document != request.document
            || draft.publication.is_some()
        {
            return Err(ModelSourceError::OutcomeUnknown);
        }
        Ok(draft)
    }
    async fn save_draft(
        &self,
        tenant: &str,
        id: ModelNumber,
        request: &SaveModelDraft,
    ) -> Result<ModelDraftWire, ModelSourceError> {
        let body = request
            .encode()
            .map_err(|_| ModelSourceError::Refused(ModelRefusalCode::MalformedRequest))?;
        let draft = self
            .request(
                self.url(MODEL_SAVE_ROUTE, tenant, Some(id))?,
                Some(body),
                self.limits.draft,
                StatusCode::OK,
                ModelDraftWire::decode,
            )
            .await?;
        if draft.id != id
            || draft.tenant_id != tenant
            || draft.base_generation != request.expected_base
            || request.expected_version.get().checked_add(1) != Some(draft.version.get())
            || draft.document != request.document
            || draft.publication.is_some()
        {
            return Err(ModelSourceError::OutcomeUnknown);
        }
        Ok(draft)
    }
    async fn validate_draft(
        &self,
        tenant: &str,
        id: ModelNumber,
        request: &PrepareModelDraft,
    ) -> Result<ModelValidation, ModelSourceError> {
        let body = request
            .encode()
            .map_err(|_| ModelSourceError::Refused(ModelRefusalCode::MalformedRequest))?;
        let result = self
            .request(
                self.url(MODEL_VALIDATE_ROUTE, tenant, Some(id))?,
                Some(body),
                self.limits.validation,
                StatusCode::OK,
                ModelValidation::decode,
            )
            .await?;
        if result.draft_id != id
            || result.draft_version != request.expected_version
            || result.base_generation != request.expected_base
        {
            return Err(unavailable());
        }
        Ok(result)
    }
}
