use crate::*;

mod codec;
mod lifecycle;
mod limits;
mod number;
pub use lifecycle::*;
pub use limits::{ModelWireLimits, model_wire_limits};
pub use number::ModelNumber;

pub const MODEL_WORKSPACE_ROUTE: &str = "/v1/tenants/{tenant}/model";
pub const MODEL_DRAFTS_ROUTE: &str = "/v1/tenants/{tenant}/model/drafts";
pub const MODEL_DRAFT_ROUTE: &str = "/v1/tenants/{tenant}/model/drafts/{draft_id}";
pub const MODEL_SAVE_ROUTE: &str = "/v1/tenants/{tenant}/model/drafts/{draft_id}/save";
pub const MODEL_VALIDATE_ROUTE: &str = "/v1/tenants/{tenant}/model/drafts/{draft_id}/validate";
pub const MODEL_PUBLISH_ROUTE: &str = "/v1/tenants/{tenant}/model/drafts/{draft_id}/publish";
pub const MODEL_LIFECYCLE_ROUTE: &str = "/v1/tenants/{tenant}/model/lifecycle";

pub const MAX_MODEL_DIAGNOSTICS: usize = 1;
pub const MAX_MODEL_DIAGNOSTIC_CODE_BYTES: usize = 64;
pub const MAX_MODEL_DIAGNOSTIC_PATH_BYTES: usize = MAX_MODEL_TEXT_BYTES * 2 + 64;
pub const MAX_MODEL_DIAGNOSTIC_MESSAGE_BYTES: usize = 256;

pub(crate) fn required_field<'de, D: serde::Deserializer<'de>, T: serde::Deserialize<'de>>(
    decoder: D,
) -> Result<T, D::Error> {
    T::deserialize(decoder)
}

model_object!(ModelCapabilities {
    author: bool,
    publish: bool
});
model_object!(ModelWorkspace {
    generation: ModelNumber,
    document: ModelDocument,
    capabilities: ModelCapabilities
});
model_object!(ModelDraftSummary {
    id: ModelNumber,
    version: ModelNumber,
    base_generation: ModelNumber,
    #[serde(deserialize_with = "required_field")]
    published_generation: Option<ModelNumber>,
});
model_object!(ModelDraftPage {
    drafts: Vec<ModelDraftSummary>,
    #[serde(deserialize_with = "required_field")]
    next: Option<ModelNumber>,
});
model_object!(ModelReceiptWire {
    generation: ModelNumber,
    prior_generation: ModelNumber,
    draft_id: ModelNumber,
    draft_version: ModelNumber,
    owner: String,
});
model_object!(ModelDraftWire {
    id: ModelNumber,
    tenant_id: String,
    owner: String,
    base_generation: ModelNumber,
    version: ModelNumber,
    document: ModelDocument,
    #[serde(deserialize_with = "required_field")]
    publication: Option<ModelReceiptWire>,
});
model_object!(CreateModelDraft {
    expected_generation: ModelNumber,
    document: ModelDocument
});
model_object!(SaveModelDraft {
    expected_version: ModelNumber,
    expected_base: ModelNumber,
    document: ModelDocument
});
model_object!(PrepareModelDraft {
    expected_version: ModelNumber,
    expected_base: ModelNumber
});
model_object!(ModelDiagnostic {
    code: String,
    path: String,
    message: String
});
model_object!(ModelValidation {
    draft_id: ModelNumber,
    draft_version: ModelNumber,
    base_generation: ModelNumber,
    valid: bool,
    diagnostics: Vec<ModelDiagnostic>,
});
model_object!(ModelRefusal {
    error: ModelRefusalCode
});

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRefusalCode {
    Unauthenticated,
    Forbidden,
    DraftNotFound,
    DraftConflict,
    HeadConflict,
    DraftClosed,
    MalformedRequest,
    BodyTooLarge,
    UnsupportedContentType,
    ModelUnavailable,
}

impl TryFrom<&PublicationReceipt> for ModelReceiptWire {
    type Error = ModelError;
    fn try_from(row: &PublicationReceipt) -> Result<Self, Self::Error> {
        Ok(Self {
            generation: ModelNumber::new(row.generation)?,
            prior_generation: ModelNumber::new(row.prior_generation)?,
            draft_id: ModelNumber::new(row.draft_id)?,
            draft_version: ModelNumber::new(row.draft_version)?,
            owner: row.owner.clone(),
        })
    }
}
impl TryFrom<&ModelDraft> for ModelDraftWire {
    type Error = ModelError;
    fn try_from(row: &ModelDraft) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ModelNumber::new(row.id)?,
            tenant_id: row.tenant_id.clone(),
            owner: row.owner.clone(),
            base_generation: ModelNumber::new(row.base_generation)?,
            version: ModelNumber::new(row.version)?,
            document: row.document.clone(),
            publication: row
                .publication
                .as_ref()
                .map(ModelReceiptWire::try_from)
                .transpose()?,
        })
    }
}
impl TryFrom<DraftPage> for ModelDraftPage {
    type Error = ModelError;
    fn try_from(row: DraftPage) -> Result<Self, Self::Error> {
        Ok(Self {
            drafts: row
                .drafts
                .into_iter()
                .map(|draft| {
                    Ok(ModelDraftSummary {
                        id: ModelNumber::new(draft.id)?,
                        version: ModelNumber::new(draft.version)?,
                        base_generation: ModelNumber::new(draft.base_generation)?,
                        published_generation: draft
                            .published_generation
                            .map(ModelNumber::new)
                            .transpose()?,
                    })
                })
                .collect::<Result<Vec<_>, ModelError>>()?,
            next: row.next.map(ModelNumber::new).transpose()?,
        })
    }
}
