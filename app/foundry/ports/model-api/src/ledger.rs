use crate::store::{context, next_model_version};
use crate::*;

/// Shared native transitions; adapters provide locking and durable transactions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelLedger {
    pub(crate) history: ModelHistory,
    pub(crate) next_draft: u64,
    pub(crate) drafts: Vec<ModelDraft>,
}

impl ModelLedger {
    pub fn initialize(binding: ModelBinding, baseline: ModelBaseline) -> Result<Self, ModelError> {
        let first = ModelPublication {
            generation: 1,
            document: baseline.document.clone(),
            receipt: None,
        };
        let ledger = Self {
            history: ModelHistory {
                binding,
                baseline,
                publications: vec![first],
            },
            next_draft: 1,
            drafts: Vec::new(),
        };
        ledger.validate()?;
        Ok(ledger)
    }

    pub fn restore(
        history: ModelHistory,
        next_draft: u64,
        drafts: Vec<ModelDraft>,
    ) -> Result<Self, ModelError> {
        let ledger = Self {
            history,
            next_draft,
            drafts,
        };
        ledger.validate().map_err(|_| ModelError::CorruptState)?;
        Ok(ledger)
    }

    pub fn history_record(&self) -> &ModelHistory {
        &self.history
    }
    pub fn next_draft_id(&self) -> u64 {
        self.next_draft
    }
    pub fn draft_records(&self) -> &[ModelDraft] {
        &self.drafts
    }

    fn transition<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, ModelError>,
    ) -> Result<T, ModelError> {
        self.validate()?;
        let mut candidate = self.clone();
        let result = operation(&mut candidate)?;
        candidate.validate()?;
        *self = candidate;
        Ok(result)
    }

    fn owned(&self, caller: &ModelCaller, id: u64) -> Result<&ModelDraft, ModelError> {
        context(&self.history.binding, caller)?;
        let draft = self
            .drafts
            .iter()
            .find(|draft| draft.id == id)
            .ok_or(ModelError::MissingDraft)?;
        if draft.owner != caller.owner {
            return Err(ModelError::OwnerMismatch);
        }
        Ok(draft)
    }
}

impl ModelStore for ModelLedger {
    fn owned_drafts(
        &self,
        caller: &ModelCaller,
        page: &DraftPageRequest,
    ) -> Result<DraftPage, ModelError> {
        context(&self.history.binding, caller)?;
        page.validate()?;
        self.validate()?;
        let mut owned = self
            .drafts
            .iter()
            .filter(|draft| draft.owner == caller.owner && draft.id > page.after.unwrap_or(0));
        let drafts: Vec<_> = owned
            .by_ref()
            .take(page.limit)
            .map(DraftSummary::from)
            .collect();
        let next = owned
            .next()
            .and_then(|_| drafts.last().map(|draft| draft.id));
        Ok(DraftPage { drafts, next })
    }
    fn prepare(
        &self,
        caller: &ModelCaller,
        id: u64,
        version: u64,
        base: u64,
    ) -> Result<PreparedPublication, ModelError> {
        self.validate()?;
        self.history
            .prepare_saved(caller, self.owned(caller, id)?, version, base)
    }

    fn history(&self, caller: &ModelCaller) -> Result<ModelHistory, ModelError> {
        context(&self.history.binding, caller)?;
        self.validate()?;
        Ok(self.history.clone())
    }

    fn create_draft(
        &mut self,
        caller: &ModelCaller,
        expected_generation: u64,
        document: ModelDocument,
    ) -> Result<ModelDraft, ModelError> {
        context(&self.history.binding, caller)?;
        document.check_bounds()?;
        self.transition(|candidate| {
            let generation = candidate.history.head()?.generation;
            if expected_generation != generation {
                return Err(ModelError::HeadConflict {
                    current_generation: generation,
                });
            }
            let id = candidate.next_draft;
            candidate.next_draft = next_model_version(id)?;
            let draft =
                ModelDraft::create(&candidate.history.binding, caller, id, generation, document)?;
            candidate.drafts.push(draft.clone());
            Ok(draft)
        })
    }

    fn open_draft(&self, caller: &ModelCaller, draft_id: u64) -> Result<ModelDraft, ModelError> {
        self.validate()?;
        self.owned(caller, draft_id).cloned()
    }

    fn save_draft(
        &mut self,
        caller: &ModelCaller,
        draft_id: u64,
        expected_version: u64,
        expected_base: u64,
        document: ModelDocument,
    ) -> Result<ModelDraft, ModelError> {
        document.check_bounds()?;
        self.owned(caller, draft_id)?;
        self.transition(|candidate| {
            let draft = candidate
                .drafts
                .iter_mut()
                .find(|draft| draft.id == draft_id)
                .ok_or(ModelError::MissingDraft)?;
            *draft = draft.saved(
                &candidate.history.binding,
                caller,
                expected_version,
                expected_base,
                document,
            )?;
            Ok(draft.clone())
        })
    }

    fn publish(
        &mut self,
        caller: &ModelCaller,
        prepared: &PreparedPublication,
    ) -> Result<PublicationReceipt, ModelError> {
        self.validate()?;
        let stored = self.owned(caller, prepared.draft.id)?;
        let receipt = self.history.publication_receipt(caller, stored, prepared)?;
        if stored.publication.is_some() {
            return Ok(receipt);
        }
        self.transition(|candidate| {
            candidate.history.publications.push(ModelPublication {
                generation: receipt.generation,
                document: prepared.draft.document.clone(),
                receipt: Some(receipt.clone()),
            });
            let draft = candidate
                .drafts
                .iter_mut()
                .find(|draft| draft.id == receipt.draft_id)
                .ok_or(ModelError::MissingDraft)?;
            draft.publication = Some(receipt.clone());
            Ok(receipt)
        })
    }
}
