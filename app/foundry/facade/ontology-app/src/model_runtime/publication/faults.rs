use foundry_model_api::*;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Fault {
    None,
    RejectCommit,
    CommittedError,
    CommitAndLoseReadback,
    RejectAndLoseReadback,
    WrongAcknowledgement,
    ContradictoryHead,
    CrashAfterCommit,
}
pub(super) struct FaultModel {
    pub inner: Box<dyn ModelStore + Send>,
    pub fault: Arc<Mutex<Fault>>,
    pub attempted: AtomicBool,
    pub writes: Arc<Mutex<WriteAttempts>>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct WriteAttempts {
    pub creates: usize,
    pub saves: usize,
    pub publishes: usize,
}
fn failure() -> ModelError {
    ModelError::Storage {
        detail: "injected publication storage failure".into(),
    }
}
impl FaultModel {
    fn read(&self) -> Result<(), ModelError> {
        if self.attempted.load(Ordering::SeqCst)
            && matches!(
                *self.fault.lock().unwrap(),
                Fault::CommitAndLoseReadback | Fault::RejectAndLoseReadback
            )
        {
            return Err(failure());
        }
        Ok(())
    }
}
impl ModelStore for FaultModel {
    fn store_access(&self) -> Option<&foundry_store_access::StoreAccess> {
        self.inner.store_access()
    }
    fn owned_drafts(
        &self,
        caller: &ModelCaller,
        page: &DraftPageRequest,
    ) -> Result<DraftPage, ModelError> {
        self.read()?;
        self.inner.owned_drafts(caller, page)
    }
    fn history(&self, caller: &ModelCaller) -> Result<ModelHistory, ModelError> {
        self.read()?;
        self.inner.history(caller)
    }
    fn head(&self, caller: &ModelCaller) -> Result<ModelPublication, ModelError> {
        self.read()?;
        let mut head = self.inner.head(caller)?;
        if self.attempted.load(Ordering::SeqCst)
            && *self.fault.lock().unwrap() == Fault::ContradictoryHead
        {
            head.generation += 1;
        }
        Ok(head)
    }
    fn create_draft(
        &mut self,
        caller: &ModelCaller,
        base: u64,
        document: ModelDocument,
    ) -> Result<ModelDraft, ModelError> {
        self.writes.lock().unwrap().creates += 1;
        self.inner.create_draft(caller, base, document)
    }
    fn open_draft(&self, caller: &ModelCaller, id: u64) -> Result<ModelDraft, ModelError> {
        self.read()?;
        self.inner.open_draft(caller, id)
    }
    fn save_draft(
        &mut self,
        caller: &ModelCaller,
        id: u64,
        version: u64,
        base: u64,
        document: ModelDocument,
    ) -> Result<ModelDraft, ModelError> {
        self.writes.lock().unwrap().saves += 1;
        self.inner.save_draft(caller, id, version, base, document)
    }
    fn prepare(
        &self,
        caller: &ModelCaller,
        id: u64,
        version: u64,
        base: u64,
    ) -> Result<PreparedPublication, ModelError> {
        self.read()?;
        self.inner.prepare(caller, id, version, base)
    }
    fn receipt(
        &self,
        caller: &ModelCaller,
        id: u64,
        version: u64,
        base: u64,
    ) -> Result<Option<PublicationReceipt>, ModelError> {
        self.read()?;
        self.inner.receipt(caller, id, version, base)
    }
    fn publish(
        &mut self,
        caller: &ModelCaller,
        prepared: &PreparedPublication,
    ) -> Result<PublicationReceipt, ModelError> {
        self.writes.lock().unwrap().publishes += 1;
        self.attempted.store(true, Ordering::SeqCst);
        let fault = *self.fault.lock().unwrap();
        if matches!(fault, Fault::RejectCommit | Fault::RejectAndLoseReadback) {
            return Err(failure());
        }
        let mut receipt = self.inner.publish(caller, prepared)?;
        if fault == Fault::CrashAfterCommit {
            std::process::exit(72);
        }
        if matches!(
            fault,
            Fault::CommittedError | Fault::CommitAndLoseReadback | Fault::ContradictoryHead
        ) {
            return Err(failure());
        }
        if fault == Fault::WrongAcknowledgement {
            receipt.draft_id += 1;
        }
        Ok(receipt)
    }
}
