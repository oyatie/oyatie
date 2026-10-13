use crate::{SqliteModelStore, encode, load, storage};
use foundry_model_api::*;
use rusqlite::{Connection, params};

impl SqliteModelStore {
    fn write<T>(
        &mut self,
        operation: impl FnOnce(&Connection, &load::State) -> Result<T, ModelError>,
    ) -> Result<T, ModelError> {
        let transaction = self
            .connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage)?;
        let state = load::state(&transaction, &self.binding, &self.baseline)?;
        let result = operation(&transaction, &state)?;
        transaction.commit().map_err(storage)?;
        Ok(result)
    }
}

impl ModelStore for SqliteModelStore {
    fn owned_drafts(
        &self,
        caller: &ModelCaller,
        page: &DraftPageRequest,
    ) -> Result<DraftPage, ModelError> {
        self.binding.check_caller(caller)?;
        page.validate()?;
        self.read(|connection, state| {
            // shortcut: scan until limit+1 owned rows; add a native owner index when measured discovery latency requires it.
            let mut statement = connection
                .prepare("SELECT id FROM ontology_model_draft WHERE id > ?1 ORDER BY id")
                .map_err(storage)?;
            let rows = statement
                .query_map(params![page.after.unwrap_or(0)], |row| row.get::<_, u64>(0))
                .map_err(storage)?;
            let mut drafts = Vec::new();
            let mut next = None;
            for row in rows {
                let draft = load::draft(connection, &state.history, row.map_err(storage)?)?;
                if draft.owner != caller.owner {
                    continue;
                }
                if drafts.len() == page.limit {
                    next = drafts.last().map(|draft: &DraftSummary| draft.id);
                    break;
                }
                drafts.push(DraftSummary::from(&draft));
            }
            Ok(DraftPage { drafts, next })
        })
    }
    fn history(&self, caller: &ModelCaller) -> Result<ModelHistory, ModelError> {
        self.binding.check_caller(caller)?;
        self.read(|_, state| Ok(state.history.clone()))
    }
    fn open_draft(&self, caller: &ModelCaller, id: u64) -> Result<ModelDraft, ModelError> {
        self.binding.check_caller(caller)?;
        self.read(|connection, state| {
            let draft = load::draft(connection, &state.history, id)?;
            draft.check_owner(&state.history.binding, caller)?;
            Ok(draft)
        })
    }
    fn prepare(
        &self,
        caller: &ModelCaller,
        id: u64,
        version: u64,
        base: u64,
    ) -> Result<PreparedPublication, ModelError> {
        self.binding.check_caller(caller)?;
        self.read(|connection, state| {
            state.history.prepare_saved(
                caller,
                &load::draft(connection, &state.history, id)?,
                version,
                base,
            )
        })
    }
    fn create_draft(
        &mut self,
        caller: &ModelCaller,
        expected_generation: u64,
        document: ModelDocument,
    ) -> Result<ModelDraft, ModelError> {
        self.binding.check_caller(caller)?;
        self.write(|connection, state| {
            let generation = state.history.head()?.generation;
            if expected_generation != generation {
                return Err(ModelError::HeadConflict {
                    current_generation: generation,
                });
            }
            let next = next_model_version(state.next_draft)?;
            let draft = ModelDraft::create(
                &state.history.binding,
                caller,
                state.next_draft,
                generation,
                document,
            )?;
            connection
                .execute(
                    "INSERT INTO ontology_model_draft (id, record) VALUES (?1, ?2)",
                    params![draft.id, encode(&draft)?],
                )
                .map_err(storage)?;
            connection
                .execute(
                    "UPDATE ontology_model_binding SET next_draft = ?1 WHERE singleton = 1",
                    params![next],
                )
                .map_err(storage)?;
            Ok(draft)
        })
    }
    fn save_draft(
        &mut self,
        caller: &ModelCaller,
        id: u64,
        version: u64,
        base: u64,
        document: ModelDocument,
    ) -> Result<ModelDraft, ModelError> {
        self.binding.check_caller(caller)?;
        self.write(|connection, state| {
            let stored = load::draft(connection, &state.history, id)?;
            let saved = stored.saved(&state.history.binding, caller, version, base, document)?;
            connection
                .execute(
                    "UPDATE ontology_model_draft SET record = ?1 WHERE id = ?2",
                    params![encode(&saved)?, id],
                )
                .map_err(storage)?;
            Ok(saved)
        })
    }
    fn publish(
        &mut self,
        caller: &ModelCaller,
        prepared: &PreparedPublication,
    ) -> Result<PublicationReceipt, ModelError> {
        self.binding.check_caller(caller)?;
        self.write(|connection, state| {
            let stored = load::draft(connection, &state.history, prepared.draft().id)?;
            let receipt = state.history.publication_receipt(caller, &stored, prepared)?;
            if stored.publication.is_some() { return Ok(receipt); }
            let publication = ModelPublication { generation: receipt.generation,
                document: stored.document.clone(), receipt: Some(receipt.clone()) };
            connection.execute("INSERT INTO ontology_model_publication (generation, draft_id, record) VALUES (?1, ?2, ?3)",
                params![receipt.generation, stored.id, encode(&publication)?]).map_err(storage)?;
            let changed = connection.execute("UPDATE ontology_model_binding SET head = ?1 WHERE singleton = 1 AND head = ?2",
                params![receipt.generation, receipt.prior_generation]).map_err(storage)?;
            if changed != 1 { return Err(ModelError::CorruptState); }
            let closed = ModelDraft { publication: Some(receipt.clone()), ..stored };
            connection.execute("UPDATE ontology_model_draft SET record = ?1 WHERE id = ?2", params![encode(&closed)?, closed.id]).map_err(storage)?;
            Ok(receipt)
        })
    }
}
