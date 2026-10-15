use foundry_ontology_query_usecase::OntologyQueryExecutionUsecase;
use foundry_projection_draft::{ProjectionStore, ProjectionStoreError};
use foundry_records_draft::{RecordsLog, RecordsLogError, SealedEnvelope};
use foundry_spine::{ProjectionState, SyncStatus, store_sync_status};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteHandlesRefused {
    PublicationUnknown,
}
pub type WriteHandles<'a> = (
    &'a mut dyn RecordsLog,
    &'a mut dyn RecordsLog,
    &'a mut ProjectionState,
    &'a mut dyn ProjectionStore,
);

pub struct TenantState {
    pub(crate) projection: ProjectionState,
    /// Query executions, held per tenant because the usecase recognises a
    /// replay by the keys it has already answered.
    pub queries: OntologyQueryExecutionUsecase,
    /// The policy decision each (principal, idempotency key) was FIRST
    /// authorized under.
    ///
    /// The usecase fingerprints an intent with its decision id, so that a
    /// replay may return a cached receipt without re-checking policy. A policy
    /// engine mints a fresh decision per call, so without this every honest
    /// retry would read as a different intent. Every attempt is still
    /// authorized on its own before reaching here; this only keeps the evidence
    /// the first attempt was granted, so the intent is the same intent.
    pub query_decisions: BTreeMap<(String, String), String>, // data_class: INTERNAL_ONLY
    /// Human seed metadata is not in the domain fingerprint; retain it for the same receipt lifetime.
    pub(crate) human_query_intents:
        BTreeMap<(String, String), crate::human_session::relationships::HumanRelationshipIntent>, // data_class: INTERNAL_ONLY
    /// Held behind the PORT so a test can install a log that fails on demand.
    pub(crate) action_log: Box<dyn RecordsLog + Send>,
    pub(crate) denial_log: Box<dyn RecordsLog + Send>,
    /// The durable mirror of `projection`; the sync status is read here.
    pub(crate) projection_store: Box<dyn ProjectionStore + Send>,
    pub(crate) model_installation: crate::model_runtime::RuntimeInstallation,
    pub(crate) publication_unknown: Option<foundry_model_api::PublicationReceipt>,
    pub(crate) model_store: Option<crate::model_control::EstablishedModelStore>,
}

/// Why a tenant's sync status could not be read: either side may fail, and
/// an unreadable store is never lag zero.
#[derive(Debug)]
pub enum SyncError {
    PublicationUnknown,
    ModelInstallationUnobserved,
    ModelGenerationMismatch,
    Model(foundry_model_api::ModelError),
    Log(RecordsLogError),
    Store(ProjectionStoreError),
}

impl std::fmt::Debug for TenantState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TenantState")
            .field("applied_ordinal", &self.projection.applied_ordinal)
            .finish_non_exhaustive()
    }
}

impl TenantState {
    /// The four handles a write-through requires, borrowed together so the
    /// borrow checker enforces what the write path already assumes.
    pub(crate) fn write_handles(&mut self) -> Result<WriteHandles<'_>, WriteHandlesRefused> {
        if self.publication_unknown.is_some() {
            return Err(WriteHandlesRefused::PublicationUnknown);
        }
        Ok((
            &mut *self.action_log,
            &mut *self.denial_log,
            &mut self.projection,
            &mut *self.projection_store,
        ))
    }
}

impl TenantState {
    /// The two handles a query execution needs, borrowed together so the borrow
    /// checker enforces what the read path already assumes: the usecase is
    /// mutated (it remembers the keys it has answered) while the store it reads
    /// is not.
    pub fn query_handles(&mut self) -> (&mut OntologyQueryExecutionUsecase, &dyn ProjectionStore) {
        (&mut self.queries, &*self.projection_store)
    }
}

impl TenantState {
    /// Costs a full replay — O(tenant log), unbounded — and callers hold the
    /// tenant mutex across it, so the observation in `metrics` degrades while
    /// it runs.
    ///
    /// The `Applied` disposition `audit_view` reports is sound only while
    /// this process is the sole writer: an entry appended below our
    /// `applied_ordinal` by another writer would pass its filter and be
    /// labelled applied by a process that never folded it.
    pub fn entries_now(&self, tenant_id: &str) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        self.action_log.replay(tenant_id, 1)
    }

    /// The DURABLE store's position against the log head, never the
    /// in-memory fold's: what survives a restart is what an operator asks.
    /// The two positions agree while this process is the sole writer of its
    /// log and every mirror was taken, which is the same assumption
    /// `entries_now` states.
    pub fn sync_status(&self, tenant_id: &str) -> Result<SyncStatus, SyncError> {
        if self.publication_unknown.is_some() {
            return Err(SyncError::PublicationUnknown);
        }
        if self.model_installation.requires_observation() {
            let observed = self
                .observed_model_generation()
                .ok_or(SyncError::ModelInstallationUnobserved)?;
            let model = self
                .model_store
                .as_ref()
                .ok_or(SyncError::ModelInstallationUnobserved)?;
            let desired = model
                .store
                .head(&foundry_model_api::ModelCaller {
                    tenant_id: tenant_id.into(),
                    owner: "model-observation".into(),
                })
                .map_err(SyncError::Model)?;
            if desired.generation != observed {
                return Err(SyncError::ModelGenerationMismatch);
            }
        }
        let head = self.action_log.head(tenant_id).map_err(SyncError::Log)?;
        store_sync_status(&*self.projection_store, tenant_id, head).map_err(SyncError::Store)
    }
}
