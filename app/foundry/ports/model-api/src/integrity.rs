use crate::*;

impl ModelLedger {
    pub fn validate(&self) -> Result<(), ModelError> {
        self.history.registry()?;
        if self.next_draft != self.drafts.len() as u64 + 1 || self.next_draft > MAX_MODEL_VERSION {
            return Err(ModelError::CorruptState);
        }
        for (index, draft) in self.drafts.iter().enumerate() {
            if draft.id != index as u64 + 1
                || draft.tenant_id != self.history.binding.tenant_id
                || draft.owner.trim().is_empty()
                || draft.owner.len() > MAX_MODEL_CONTEXT_BYTES
                || draft.version == 0
                || draft.version > MAX_MODEL_VERSION
                || draft.base_generation == 0
                || draft.base_generation > self.history.head()?.generation
            {
                return Err(ModelError::CorruptState);
            }
            draft.document.check_bounds()?;
            if let Some(receipt) = &draft.publication {
                let published = self
                    .history
                    .publications
                    .iter()
                    .find(|row| row.generation == receipt.generation)
                    .ok_or(ModelError::CorruptState)?;
                if published.receipt.as_ref() != Some(receipt)
                    || published.document != draft.document
                    || receipt.draft_id != draft.id
                    || receipt.draft_version != draft.version
                    || receipt.owner != draft.owner
                    || receipt.prior_generation != draft.base_generation
                {
                    return Err(ModelError::CorruptState);
                }
            }
        }
        for published in self.history.publications.iter().skip(1) {
            let receipt = published.receipt.as_ref().ok_or(ModelError::CorruptState)?;
            if !self
                .drafts
                .iter()
                .any(|draft| draft.publication.as_ref() == Some(receipt))
            {
                return Err(ModelError::CorruptState);
            }
        }
        self.history.baseline.check_bounds()?;
        Ok(())
    }
}

impl ModelBaseline {
    pub fn check_bounds(&self) -> Result<(), ModelError> {
        self.document.check_bounds()?;
        for row in &self.prior_entity_revisions {
            let mut remaining = MAX_MODEL_VALUE_NODES;
            row.bounds("baseline.entity_revisions", &mut remaining)?;
        }
        Ok(())
    }
}
