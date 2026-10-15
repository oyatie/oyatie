use super::*;

#[derive(Clone, Copy)]
pub enum HistoryFault {
    Rebound,
    MissingBase,
    Duplicate,
    InvalidDocument,
    Oversized,
    Failure,
    Append,
}

pub struct CountingStore {
    pub inner: ReferenceModelStore,
    pub calls: Arc<AtomicUsize>,
    pub failure: Arc<AtomicBool>,
    pub oversized: Arc<AtomicBool>,
    pub history_fault: Arc<Mutex<Option<HistoryFault>>>,
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
        let fault = *self.history_fault.lock().unwrap();
        if matches!(fault, Some(HistoryFault::Append)) {
            publish_next(&mut self.inner.clone(), caller);
        }
        let mut history = self.inner.history(caller)?;
        match fault {
            Some(HistoryFault::Rebound) => history.binding.store_id.push_str("-wrong"),
            Some(HistoryFault::MissingBase) => {
                history.publications.remove(0);
            }
            Some(HistoryFault::Duplicate) => {
                history.publications.push(history.publications[0].clone())
            }
            Some(HistoryFault::InvalidDocument | HistoryFault::Oversized) => {
                history.baseline.document.entities[0].display_name =
                    if matches!(fault, Some(HistoryFault::Oversized)) {
                        "x".repeat(MAX_MODEL_TEXT_BYTES + 1)
                    } else {
                        String::new()
                    };
                history.publications[0].document = history.baseline.document.clone();
            }
            Some(HistoryFault::Failure) => return Err(ModelError::MalformedDocument),
            _ => {}
        }
        Ok(history)
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

pub fn publish_next(store: &mut dyn ModelStore, caller: &ModelCaller) {
    let head = store.head(caller).unwrap();
    let mut document = head.document;
    document.entities[0].revision += 1;
    document.entities[0].display_name.push_str(" next");
    let draft = store
        .create_draft(caller, head.generation, document)
        .unwrap();
    let prepared = store
        .prepare(caller, draft.id, draft.version, draft.base_generation)
        .unwrap();
    store.publish(caller, &prepared).unwrap();
}
