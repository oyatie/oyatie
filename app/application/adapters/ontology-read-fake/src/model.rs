use application_ontology_read::{
    CreateModelDraft, ModelCapabilities, ModelDiagnostic, ModelDraftPage, ModelDraftWire,
    ModelNumber, ModelRefusalCode, ModelSourceError, ModelValidation, ModelWorkspace,
    OntologyModelSource, PrepareModelDraft, SaveModelDraft,
};
use foundry_model_api::{
    DraftPageRequest, ModelCaller, ModelError, ModelStore, ReferenceModelStore,
};

/// Explicit fixture access; no production role resolver or sample fallback.
#[derive(Clone)]
pub struct FixtureOntologyModelSource {
    store: ReferenceModelStore,
    caller: ModelCaller,
    readable: bool,
    capabilities: ModelCapabilities,
}
impl FixtureOntologyModelSource {
    pub fn new(
        store: ReferenceModelStore,
        caller: ModelCaller,
        readable: bool,
        capabilities: ModelCapabilities,
    ) -> Self {
        Self {
            store,
            caller,
            readable,
            capabilities,
        }
    }
    fn check(&self, tenant: &str, author: bool) -> Result<(), ModelSourceError> {
        if tenant != self.caller.tenant_id || !self.readable || author && !self.capabilities.author
        {
            return Err(ModelSourceError::Refused(ModelRefusalCode::Forbidden));
        }
        Ok(())
    }
}
fn refusal(error: ModelError) -> ModelSourceError {
    ModelSourceError::Refused(match error {
        ModelError::TenantMismatch | ModelError::OwnerMismatch => ModelRefusalCode::Forbidden,
        ModelError::MissingDraft => ModelRefusalCode::DraftNotFound,
        ModelError::DraftConflict { .. } => ModelRefusalCode::DraftConflict,
        ModelError::HeadConflict { .. } => ModelRefusalCode::HeadConflict,
        ModelError::DraftClosed => ModelRefusalCode::DraftClosed,
        ModelError::MalformedDocument | ModelError::InvalidContext | ModelError::Limit { .. } => {
            ModelRefusalCode::MalformedRequest
        }
        _ => ModelRefusalCode::ModelUnavailable,
    })
}
impl OntologyModelSource for FixtureOntologyModelSource {
    async fn workspace(&self, tenant: &str) -> Result<ModelWorkspace, ModelSourceError> {
        self.check(tenant, false)?;
        let head = self.store.head(&self.caller).map_err(refusal)?;
        Ok(ModelWorkspace {
            generation: ModelNumber::new(head.generation).map_err(refusal)?,
            document: head.document,
            capabilities: self.capabilities.clone(),
        })
    }
    async fn owned_drafts(
        &self,
        tenant: &str,
        after: Option<ModelNumber>,
        limit: usize,
    ) -> Result<ModelDraftPage, ModelSourceError> {
        self.check(tenant, false)?;
        ModelDraftPage::try_from(
            self.store
                .owned_drafts(
                    &self.caller,
                    &DraftPageRequest {
                        after: after.map(ModelNumber::get),
                        limit,
                    },
                )
                .map_err(refusal)?,
        )
        .map_err(refusal)
    }
    async fn draft(
        &self,
        tenant: &str,
        id: ModelNumber,
    ) -> Result<ModelDraftWire, ModelSourceError> {
        self.check(tenant, false)?;
        ModelDraftWire::try_from(
            &self
                .store
                .open_draft(&self.caller, id.get())
                .map_err(refusal)?,
        )
        .map_err(refusal)
    }
    async fn create_draft(
        &self,
        tenant: &str,
        request: &CreateModelDraft,
    ) -> Result<ModelDraftWire, ModelSourceError> {
        self.check(tenant, true)?;
        request.check_bounds().map_err(refusal)?;
        ModelDraftWire::try_from(
            &self
                .store
                .clone()
                .create_draft(
                    &self.caller,
                    request.expected_generation.get(),
                    request.document.clone(),
                )
                .map_err(refusal)?,
        )
        .map_err(refusal)
    }
    async fn save_draft(
        &self,
        tenant: &str,
        id: ModelNumber,
        request: &SaveModelDraft,
    ) -> Result<ModelDraftWire, ModelSourceError> {
        self.check(tenant, true)?;
        request.check_bounds().map_err(refusal)?;
        ModelDraftWire::try_from(
            &self
                .store
                .clone()
                .save_draft(
                    &self.caller,
                    id.get(),
                    request.expected_version.get(),
                    request.expected_base.get(),
                    request.document.clone(),
                )
                .map_err(refusal)?,
        )
        .map_err(refusal)
    }
    async fn validate_draft(
        &self,
        tenant: &str,
        id: ModelNumber,
        request: &PrepareModelDraft,
    ) -> Result<ModelValidation, ModelSourceError> {
        self.check(tenant, false)?;
        if !self.capabilities.author && !self.capabilities.publish {
            return Err(ModelSourceError::Refused(ModelRefusalCode::Forbidden));
        }
        let diagnostics = match self.store.prepare(
            &self.caller,
            id.get(),
            request.expected_version.get(),
            request.expected_base.get(),
        ) {
            Ok(_) => vec![],
            Err(
                ModelError::NoChanges
                | ModelError::MalformedDocument
                | ModelError::NewDefinitionRevision { .. }
                | ModelError::UnsupportedRemoval { .. }
                | ModelError::UnsupportedKeyAdoption { .. }
                | ModelError::UnsupportedPublicationDataClass { .. }
                | ModelError::CrossTenantLink { .. }
                | ModelError::Native { .. },
            ) => vec![ModelDiagnostic {
                code: "metadata_incompatible".into(),
                path: "document".into(),
                message: "Native metadata preparation refused this saved document.".into(),
            }],
            Err(error) => return Err(refusal(error)),
        };
        Ok(ModelValidation {
            draft_id: id,
            draft_version: request.expected_version,
            base_generation: request.expected_base,
            valid: diagnostics.is_empty(),
            diagnostics,
        })
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
