use crate::{ModelBaseline, ModelDocument, PreparedPublication};

pub const MAX_MODEL_CONTEXT_BYTES: usize = 256;

pub const MAX_MODEL_VERSION: u64 = i64::MAX as u64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelError {
    MalformedDocument,
    Limit {
        path: String,
    },
    InvalidContext,
    TenantMismatch,
    OwnerMismatch,
    MissingDraft,
    DraftClosed,
    DraftConflict {
        current_version: u64,
    },
    HeadConflict {
        current_generation: u64,
    },
    VersionOverflow,
    AlreadyInitialized,
    Uninitialized,
    BindingMismatch,
    CorruptState,
    NoChanges,
    NewDefinitionRevision {
        path: String,
    },
    UnsupportedRemoval {
        path: String,
    },
    UnsupportedKeyAdoption {
        entity_type: String,
    },
    UnsupportedPublicationDataClass {
        path: String,
        data_class: String,
    },
    CrossTenantLink {
        link_type: String,
    },
    Native {
        path: String,
        cause: data_ontology_kernel::OntologyEngineError,
    },
    Storage {
        detail: String,
    },
}

model_object!(ModelBinding {
    tenant_id: String,
    store_id: String,
    baseline_id: String,
});

/// The facade must supply independently authenticated context and check grants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelCaller {
    pub tenant_id: String,
    pub owner: String,
}

model_object!(PublicationReceipt {
    generation: u64,
    prior_generation: u64,
    draft_id: u64,
    draft_version: u64,
    owner: String,
});

model_object!(ModelPublication {
    generation: u64,
    document: ModelDocument,
    receipt: Option<PublicationReceipt>,
});

model_object!(ModelDraft {
    id: u64,
    tenant_id: String,
    owner: String,
    base_generation: u64,
    version: u64,
    document: ModelDocument,
    publication: Option<PublicationReceipt>,
});

model_object!(ModelHistory {
    binding: ModelBinding,
    baseline: ModelBaseline,
    publications: Vec<ModelPublication>,
});

pub trait ModelStore {
    fn store_access(&self) -> Option<&foundry_store_access::StoreAccess> {
        None
    }

    fn head(&self, caller: &ModelCaller) -> Result<ModelPublication, ModelError> {
        self.history(caller)?.head().cloned()
    }
    fn owned_drafts(
        &self,
        caller: &ModelCaller,
        page: &crate::DraftPageRequest,
    ) -> Result<crate::DraftPage, ModelError>;
    fn history(&self, caller: &ModelCaller) -> Result<ModelHistory, ModelError>;
    fn create_draft(
        &mut self,
        caller: &ModelCaller,
        expected_generation: u64,
        document: ModelDocument,
    ) -> Result<ModelDraft, ModelError>;
    fn open_draft(&self, caller: &ModelCaller, draft_id: u64) -> Result<ModelDraft, ModelError>;
    fn save_draft(
        &mut self,
        caller: &ModelCaller,
        draft_id: u64,
        expected_version: u64,
        expected_base: u64,
        document: ModelDocument,
    ) -> Result<ModelDraft, ModelError>;
    fn prepare(
        &self,
        caller: &ModelCaller,
        draft_id: u64,
        expected_version: u64,
        expected_base: u64,
    ) -> Result<PreparedPublication, ModelError>;
    /// Commits metadata only. Fresh authorization and full replay precede activation.
    fn publish(
        &mut self,
        caller: &ModelCaller,
        prepared: &PreparedPublication,
    ) -> Result<PublicationReceipt, ModelError>;

    fn receipt(
        &self,
        caller: &ModelCaller,
        id: u64,
        version: u64,
        base: u64,
    ) -> Result<Option<PublicationReceipt>, ModelError> {
        let draft = self.open_draft(caller, id)?;
        if draft.version != version {
            return Err(ModelError::DraftConflict {
                current_version: draft.version,
            });
        }
        if draft.base_generation != base {
            return Err(ModelError::HeadConflict {
                current_generation: draft.base_generation,
            });
        }
        Ok(draft.publication)
    }
}

pub fn next_model_version(current: u64) -> Result<u64, ModelError> {
    current
        .checked_add(1)
        .filter(|v| *v <= MAX_MODEL_VERSION)
        .ok_or(ModelError::VersionOverflow)
}

pub(crate) fn context(binding: &ModelBinding, caller: &ModelCaller) -> Result<(), ModelError> {
    binding.validate()?;
    if caller.owner.trim().is_empty() || caller.owner.len() > MAX_MODEL_CONTEXT_BYTES {
        return Err(ModelError::InvalidContext);
    }
    if caller.tenant_id != binding.tenant_id {
        return Err(ModelError::TenantMismatch);
    }
    Ok(())
}

impl ModelBinding {
    pub fn check_caller(&self, caller: &ModelCaller) -> Result<(), ModelError> {
        context(self, caller)
    }
    pub fn validate(&self) -> Result<(), ModelError> {
        if !self.tenant_id.starts_with("ten_")
            || self.tenant_id.len() <= 4
            || self.tenant_id.len() > MAX_MODEL_CONTEXT_BYTES
            || self.store_id.trim().is_empty()
            || self.baseline_id.trim().is_empty()
            || self.store_id.len() > MAX_MODEL_CONTEXT_BYTES
            || self.baseline_id.len() > MAX_MODEL_CONTEXT_BYTES
        {
            return Err(ModelError::InvalidContext);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn numerical_versions_refuse_overflow_instead_of_wrapping() {
        assert_eq!(
            super::next_model_version(super::MAX_MODEL_VERSION),
            Err(super::ModelError::VersionOverflow)
        );
        assert_eq!(
            super::next_model_version(u64::MAX),
            Err(super::ModelError::VersionOverflow)
        );
        assert_eq!(
            super::next_model_version(super::MAX_MODEL_VERSION - 1),
            Ok(super::MAX_MODEL_VERSION)
        );
    }
}
