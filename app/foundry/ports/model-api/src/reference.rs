use crate::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ReferenceModelStore {
    ledger: Arc<Mutex<ModelLedger>>,
}

impl ReferenceModelStore {
    pub fn initialize(binding: ModelBinding, baseline: ModelBaseline) -> Result<Self, ModelError> {
        Ok(Self {
            ledger: Arc::new(Mutex::new(ModelLedger::initialize(binding, baseline)?)),
        })
    }
    pub fn open_existing(
        &self,
        binding: &ModelBinding,
        baseline: &ModelBaseline,
    ) -> Result<Self, ModelError> {
        let ledger = self.ledger.lock().map_err(|_| ModelError::CorruptState)?;
        ledger.validate()?;
        if &ledger.history_record().binding != binding
            || &ledger.history_record().baseline != baseline
        {
            return Err(ModelError::BindingMismatch);
        }
        Ok(self.clone())
    }
}

impl ModelStore for ReferenceModelStore {
    fn owned_drafts(
        &self,
        caller: &ModelCaller,
        page: &DraftPageRequest,
    ) -> Result<DraftPage, ModelError> {
        self.ledger
            .lock()
            .map_err(|_| ModelError::CorruptState)?
            .owned_drafts(caller, page)
    }
    fn prepare(
        &self,
        caller: &ModelCaller,
        id: u64,
        version: u64,
        base: u64,
    ) -> Result<PreparedPublication, ModelError> {
        self.ledger
            .lock()
            .map_err(|_| ModelError::CorruptState)?
            .prepare(caller, id, version, base)
    }
    fn history(&self, caller: &ModelCaller) -> Result<ModelHistory, ModelError> {
        self.ledger
            .lock()
            .map_err(|_| ModelError::CorruptState)?
            .history(caller)
    }
    fn create_draft(
        &mut self,
        caller: &ModelCaller,
        generation: u64,
        document: ModelDocument,
    ) -> Result<ModelDraft, ModelError> {
        self.ledger
            .lock()
            .map_err(|_| ModelError::CorruptState)?
            .create_draft(caller, generation, document)
    }
    fn open_draft(&self, caller: &ModelCaller, id: u64) -> Result<ModelDraft, ModelError> {
        self.ledger
            .lock()
            .map_err(|_| ModelError::CorruptState)?
            .open_draft(caller, id)
    }
    fn save_draft(
        &mut self,
        caller: &ModelCaller,
        id: u64,
        version: u64,
        base: u64,
        document: ModelDocument,
    ) -> Result<ModelDraft, ModelError> {
        self.ledger
            .lock()
            .map_err(|_| ModelError::CorruptState)?
            .save_draft(caller, id, version, base, document)
    }
    fn publish(
        &mut self,
        caller: &ModelCaller,
        prepared: &PreparedPublication,
    ) -> Result<PublicationReceipt, ModelError> {
        self.ledger
            .lock()
            .map_err(|_| ModelError::CorruptState)?
            .publish(caller, prepared)
    }
}
