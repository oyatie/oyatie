use super::{HumanOntologyCardSource, transport};
use application_ontology_read::*;
use reqwest::{StatusCode, Url, header};
#[path = "model_transport.rs"]
mod model_transport;
use model_transport::unavailable;

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
    async fn draft_base(
        &self,
        tenant: &str,
        id: ModelNumber,
        request: &PrepareModelDraft,
    ) -> Result<ModelDraftBase, ModelSourceError> {
        let mut url = self.url(MODEL_DRAFT_BASE_ROUTE, tenant, Some(id))?;
        url.query_pairs_mut()
            .append_pair(
                "expected_version",
                &request.expected_version.get().to_string(),
            )
            .append_pair("expected_base", &request.expected_base.get().to_string());
        let base = self
            .request(
                url,
                None,
                self.limits.draft_base,
                StatusCode::OK,
                ModelDraftBase::decode,
            )
            .await?;
        if base.draft_id != id
            || base.draft_version != request.expected_version
            || base.base_generation != request.expected_base
        {
            return Err(unavailable());
        }
        Ok(base)
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
