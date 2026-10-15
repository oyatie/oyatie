use super::{TenantState, WriteHandles, WriteHandlesRefused};
use data_ontology_kernel::OntologyEngine;
use foundry_projection_draft::ProjectionStore;
use foundry_records_draft::RecordsLog;
use foundry_spine::ProjectionState;

impl TenantState {
    pub fn projection(&self) -> &ProjectionState {
        &self.projection
    }
    pub fn projection_store(&self) -> &dyn ProjectionStore {
        &*self.projection_store
    }
    pub fn action_log(&self) -> &dyn RecordsLog {
        &*self.action_log
    }
    pub fn denial_log(&self) -> &dyn RecordsLog {
        &*self.denial_log
    }
    pub fn observed_model_generation(&self) -> Option<u64> {
        self.model_installation.generation(self)
    }
    pub(crate) fn invalidate_model_installation(&mut self) {
        self.model_installation.invalidate();
    }

    pub fn projection_for_injection(&mut self) -> &mut ProjectionState {
        self.invalidate_model_installation();
        &mut self.projection
    }
    pub fn projection_store_for_injection(&mut self) -> &mut dyn ProjectionStore {
        self.invalidate_model_installation();
        &mut *self.projection_store
    }
    pub fn inject_registry(&mut self, registry: OntologyEngine) {
        self.invalidate_model_installation();
        self.projection.registry_input = registry.clone();
        self.projection.engine = registry;
    }
    pub fn replace_projection_store(
        &mut self,
        store: Box<dyn ProjectionStore + Send>,
    ) -> Box<dyn ProjectionStore + Send> {
        self.invalidate_model_installation();
        std::mem::replace(&mut self.projection_store, store)
    }
    pub fn replace_action_log(
        &mut self,
        log: Box<dyn RecordsLog + Send>,
    ) -> Box<dyn RecordsLog + Send> {
        self.invalidate_model_installation();
        std::mem::replace(&mut self.action_log, log)
    }
    pub fn replace_denial_log(
        &mut self,
        log: Box<dyn RecordsLog + Send>,
    ) -> Box<dyn RecordsLog + Send> {
        self.invalidate_model_installation();
        std::mem::replace(&mut self.denial_log, log)
    }
    /// Explicit native input injection invalidates installed evidence before exposing mutable state.
    pub fn injected_write_handles(&mut self) -> Result<WriteHandles<'_>, WriteHandlesRefused> {
        self.invalidate_model_installation();
        self.write_handles()
    }
}
