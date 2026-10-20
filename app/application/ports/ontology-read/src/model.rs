use std::future::Future;

pub use foundry_model_api::control::*;
pub use foundry_model_api::{MAX_MODEL_ITEMS, ModelDocument, ModelError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelSourceError {
    Refused(ModelRefusalCode),
    /// A submitted mutation may have committed. Reopen before resubmitting.
    OutcomeUnknown,
}

/// Request-scoped model access. Credentials and authority remain server-owned.
pub trait OntologyModelSource: Send + Sync {
    fn workspace(
        &self,
        tenant: &str,
    ) -> impl Future<Output = Result<ModelWorkspace, ModelSourceError>> + Send;
    fn owned_drafts(
        &self,
        tenant: &str,
        after: Option<ModelNumber>,
        limit: usize,
    ) -> impl Future<Output = Result<ModelDraftPage, ModelSourceError>> + Send;
    fn draft(
        &self,
        tenant: &str,
        id: ModelNumber,
    ) -> impl Future<Output = Result<ModelDraftWire, ModelSourceError>> + Send;
    fn draft_base(
        &self,
        tenant: &str,
        id: ModelNumber,
        request: &PrepareModelDraft,
    ) -> impl Future<Output = Result<ModelDraftBase, ModelSourceError>> + Send;
    fn create_draft(
        &self,
        tenant: &str,
        request: &CreateModelDraft,
    ) -> impl Future<Output = Result<ModelDraftWire, ModelSourceError>> + Send;
    fn save_draft(
        &self,
        tenant: &str,
        id: ModelNumber,
        request: &SaveModelDraft,
    ) -> impl Future<Output = Result<ModelDraftWire, ModelSourceError>> + Send;
    fn validate_draft(
        &self,
        tenant: &str,
        id: ModelNumber,
        request: &PrepareModelDraft,
    ) -> impl Future<Output = Result<ModelValidation, ModelSourceError>> + Send;
}
