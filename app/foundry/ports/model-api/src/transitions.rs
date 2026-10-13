use crate::store::{context, next_model_version};
use crate::*;

impl ModelDraft {
    pub fn create(
        binding: &ModelBinding,
        caller: &ModelCaller,
        id: u64,
        generation: u64,
        document: ModelDocument,
    ) -> Result<Self, ModelError> {
        context(binding, caller)?;
        document.check_bounds()?;
        if id == 0 || id > MAX_MODEL_VERSION || generation == 0 || generation > MAX_MODEL_VERSION {
            return Err(ModelError::CorruptState);
        }
        Ok(Self {
            id,
            tenant_id: caller.tenant_id.clone(),
            owner: caller.owner.clone(),
            base_generation: generation,
            version: 1,
            document,
            publication: None,
        })
    }

    pub fn check_owner(
        &self,
        binding: &ModelBinding,
        caller: &ModelCaller,
    ) -> Result<(), ModelError> {
        context(binding, caller)?;
        if self.tenant_id != binding.tenant_id {
            return Err(ModelError::CorruptState);
        }
        if self.owner != caller.owner {
            return Err(ModelError::OwnerMismatch);
        }
        Ok(())
    }

    pub fn saved(
        &self,
        binding: &ModelBinding,
        caller: &ModelCaller,
        version: u64,
        base: u64,
        document: ModelDocument,
    ) -> Result<Self, ModelError> {
        self.check_owner(binding, caller)?;
        document.check_bounds()?;
        if self.publication.is_some() {
            return Err(ModelError::DraftClosed);
        }
        if self.version != version {
            return Err(ModelError::DraftConflict {
                current_version: self.version,
            });
        }
        if self.base_generation != base {
            return Err(ModelError::HeadConflict {
                current_generation: self.base_generation,
            });
        }
        Ok(Self {
            version: next_model_version(self.version)?,
            document,
            ..self.clone()
        })
    }

    pub fn check_record(&self, history: &ModelHistory) -> Result<(), ModelError> {
        if self.id == 0
            || self.id > MAX_MODEL_VERSION
            || self.tenant_id != history.binding.tenant_id
            || self.owner.trim().is_empty()
            || self.owner.len() > MAX_MODEL_CONTEXT_BYTES
            || self.version == 0
            || self.version > MAX_MODEL_VERSION
            || self.base_generation == 0
            || self.base_generation > history.head()?.generation
        {
            return Err(ModelError::CorruptState);
        }
        self.document.check_bounds()?;
        if let Some(receipt) = &self.publication {
            let publication = history
                .publications
                .iter()
                .find(|row| row.generation == receipt.generation)
                .ok_or(ModelError::CorruptState)?;
            if publication.receipt.as_ref() != Some(receipt)
                || publication.document != self.document
                || receipt.draft_id != self.id
                || receipt.draft_version != self.version
                || receipt.owner != self.owner
                || receipt.prior_generation != self.base_generation
            {
                return Err(ModelError::CorruptState);
            }
        }
        Ok(())
    }
}

impl ModelHistory {
    pub fn prepare_saved(
        &self,
        caller: &ModelCaller,
        draft: &ModelDraft,
        version: u64,
        base: u64,
    ) -> Result<PreparedPublication, ModelError> {
        draft.check_owner(&self.binding, caller)?;
        draft.check_record(self)?;
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
        crate::admission::prepare_publication(draft, self)
    }

    pub fn publication_receipt(
        &self,
        caller: &ModelCaller,
        stored: &ModelDraft,
        prepared: &PreparedPublication,
    ) -> Result<PublicationReceipt, ModelError> {
        self.registry()?;
        stored.check_record(self)?;
        stored.check_owner(&self.binding, caller)?;
        if stored.version != prepared.draft.version {
            return Err(ModelError::DraftConflict {
                current_version: stored.version,
            });
        }
        if stored.id != prepared.draft.id
            || stored.document != prepared.draft.document
            || stored.base_generation != prepared.draft.base_generation
            || stored.owner != prepared.draft.owner
            || stored.tenant_id != prepared.draft.tenant_id
            || prepared.base.binding != self.binding
            || prepared.base.baseline != self.baseline
        {
            return Err(ModelError::BindingMismatch);
        }
        if let Some(receipt) = &stored.publication {
            if prepared.base.publications != self.publications[..receipt.prior_generation as usize]
            {
                return Err(ModelError::BindingMismatch);
            }
            return Ok(receipt.clone());
        }
        if &prepared.base != self {
            return Err(ModelError::HeadConflict {
                current_generation: self.head()?.generation,
            });
        }
        let prior_generation = self.head()?.generation;
        Ok(PublicationReceipt {
            generation: next_model_version(prior_generation)?,
            prior_generation,
            draft_id: stored.id,
            draft_version: stored.version,
            owner: caller.owner.clone(),
        })
    }
}
