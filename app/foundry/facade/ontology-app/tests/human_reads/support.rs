use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use foundry_ontology_app::AppState;
use foundry_projection_draft::{
    AppliedEntry, ApplyReceipt, KeyDesignations, Page, PageRequest, ProjectedLink, ProjectedObject,
    ProjectionStore, ProjectionStoreError, PropertyPredicate,
};
use iam_identity_user_session::UserSessionAuthorizer;

use crate::human_support::{TrackingPdp, instrument};

pub const SUFFIXES: [&str; 8] = [
    "/types",
    "/schemas",
    "/type-labels",
    "/link-types",
    "/objects?type=ety_record&revision=1",
    "/objects/ent_alpha?revision=1",
    "/objects/ent_alpha/history",
    "/audit",
];

pub fn instrument_reads(
    mut state: AppState,
    sessions: Arc<dyn UserSessionAuthorizer>,
) -> (Router, Arc<AtomicUsize>, Arc<AtomicUsize>, Arc<TrackingPdp>) {
    let store_reads = Arc::new(AtomicUsize::new(0));
    for tenant in state.tenants.values_mut() {
        let tenant = tenant.get_mut();
        let inner = std::mem::replace(
            &mut tenant.projection_store,
            Box::new(foundry_projection_draft::MemoryProjectionStore::default()),
        );
        tenant.projection_store = Box::new(CountingStore {
            inner,
            reads: store_reads.clone(),
        });
    }
    let (router, log_reads, pdp) = instrument(state, sessions);
    (router, log_reads, store_reads, pdp)
}

struct CountingStore {
    inner: Box<dyn ProjectionStore + Send>,
    reads: Arc<AtomicUsize>,
}

impl ProjectionStore for CountingStore {
    fn apply(
        &mut self,
        entry: AppliedEntry,
        keys: &KeyDesignations,
    ) -> Result<ApplyReceipt, ProjectionStoreError> {
        self.inner.apply(entry, keys)
    }
    fn applied_head(&self, tenant: &str) -> Result<u64, ProjectionStoreError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.inner.applied_head(tenant)
    }
    fn get(
        &self,
        tenant: &str,
        object: &str,
    ) -> Result<Option<ProjectedObject>, ProjectionStoreError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.inner.get(tenant, object)
    }
    fn objects_of_type(
        &self,
        tenant: &str,
        entity: &str,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.inner.objects_of_type(tenant, entity, page)
    }
    fn filter(
        &self,
        tenant: &str,
        entity: &str,
        predicate: &PropertyPredicate,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.inner.filter(tenant, entity, predicate, page)
    }
    fn links_from(
        &self,
        tenant: &str,
        object: &str,
    ) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.inner.links_from(tenant, object)
    }
    fn links_to(
        &self,
        tenant: &str,
        object: &str,
    ) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.inner.links_to(tenant, object)
    }
    fn poisoned(&self, tenant: &str) -> Result<Vec<(u64, String)>, ProjectionStoreError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.inner.poisoned(tenant)
    }
}
