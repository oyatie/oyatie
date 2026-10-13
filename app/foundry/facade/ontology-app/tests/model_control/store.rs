use super::*;

pub struct CountingStore {
    pub inner: ReferenceModelStore,
    pub calls: Arc<AtomicUsize>,
    pub failure: Arc<AtomicBool>,
    pub oversized: Arc<AtomicBool>,
}
impl CountingStore {
    fn enter(&self) -> Result<(), ModelError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.failure.load(Ordering::SeqCst) {
            Err(ModelError::Storage {
                detail: "private storage metadata must not escape".into(),
            })
        } else {
            Ok(())
        }
    }
}
impl ModelStore for CountingStore {
    fn head(&self, caller: &ModelCaller) -> Result<ModelPublication, ModelError> {
        self.enter()?;
        let mut head = self.inner.head(caller)?;
        if self.oversized.load(Ordering::SeqCst) {
            head.document.entities[0].display_name = "x".repeat(MAX_MODEL_TEXT_BYTES + 1);
        }
        Ok(head)
    }
    fn owned_drafts(
        &self,
        caller: &ModelCaller,
        page: &DraftPageRequest,
    ) -> Result<DraftPage, ModelError> {
        self.enter()?;
        self.inner.owned_drafts(caller, page)
    }
    fn history(&self, caller: &ModelCaller) -> Result<ModelHistory, ModelError> {
        self.enter()?;
        self.inner.history(caller)
    }
    fn create_draft(
        &mut self,
        caller: &ModelCaller,
        generation: u64,
        document: ModelDocument,
    ) -> Result<ModelDraft, ModelError> {
        self.enter()?;
        self.inner.create_draft(caller, generation, document)
    }
    fn open_draft(&self, caller: &ModelCaller, id: u64) -> Result<ModelDraft, ModelError> {
        self.enter()?;
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
        self.enter()?;
        self.inner.save_draft(caller, id, version, base, document)
    }
    fn prepare(
        &self,
        caller: &ModelCaller,
        id: u64,
        version: u64,
        base: u64,
    ) -> Result<PreparedPublication, ModelError> {
        self.enter()?;
        self.inner.prepare(caller, id, version, base)
    }
    fn publish(
        &mut self,
        _caller: &ModelCaller,
        _prepared: &PreparedPublication,
    ) -> Result<PublicationReceipt, ModelError> {
        panic!("human draft routes must never publish")
    }
}
