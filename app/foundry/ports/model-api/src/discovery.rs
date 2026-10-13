use crate::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftPageRequest {
    pub limit: usize,
    pub after: Option<u64>,
}
impl DraftPageRequest {
    pub fn validate(&self) -> Result<(), ModelError> {
        if self.limit == 0
            || self.limit > MAX_MODEL_ITEMS
            || self
                .after
                .is_some_and(|id| id == 0 || id > MAX_MODEL_VERSION)
        {
            return Err(ModelError::MalformedDocument);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftSummary {
    pub id: u64,
    pub version: u64,
    pub base_generation: u64,
    pub published_generation: Option<u64>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftPage {
    pub drafts: Vec<DraftSummary>,
    pub next: Option<u64>,
}
impl From<&ModelDraft> for DraftSummary {
    fn from(draft: &ModelDraft) -> Self {
        Self {
            id: draft.id,
            version: draft.version,
            base_generation: draft.base_generation,
            published_generation: draft.publication.as_ref().map(|receipt| receipt.generation),
        }
    }
}
