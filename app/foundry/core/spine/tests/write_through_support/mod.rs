//! Test doubles for the write-through law.

use foundry_projection_draft::{
    AppliedEntry, ApplyReceipt, KeyDesignations, MemoryProjectionStore, Page, PageRequest,
    ProjectedLink, ProjectedObject, ProjectionStore, ProjectionStoreError, PropertyPredicate,
};

/// A store that refuses the Nth apply — an outage, not a poison.
pub(crate) struct FailsAt {
    pub(crate) inner: MemoryProjectionStore,
    pub(crate) fail_on_ordinal: u64,
    /// The head itself is unreadable — the disk is there, the answer is
    /// not. A caller must refuse rather than read that as "empty".
    pub(crate) fail_head: bool,
    /// The poison ledger is unreadable. Same law, different read: an
    /// unreadable ledger must never be treated as poison-free.
    pub(crate) fail_poisoned: bool,
}

impl ProjectionStore for FailsAt {
    fn apply(
        &mut self,
        entry: AppliedEntry,
        keys: &KeyDesignations,
    ) -> Result<ApplyReceipt, ProjectionStoreError> {
        if entry.ordinal == self.fail_on_ordinal {
            return Err(ProjectionStoreError::Storage {
                detail: "disk gone".to_owned(),
            });
        }
        self.inner.apply(entry, keys)
    }

    fn applied_head(&self, tenant_id: &str) -> Result<u64, ProjectionStoreError> {
        if self.fail_head {
            return Err(ProjectionStoreError::Storage {
                detail: "head unreadable".to_owned(),
            });
        }
        self.inner.applied_head(tenant_id)
    }

    fn get(
        &self,
        tenant_id: &str,
        object_ref: &str,
    ) -> Result<Option<ProjectedObject>, ProjectionStoreError> {
        self.inner.get(tenant_id, object_ref)
    }

    fn objects_of_type(
        &self,
        tenant_id: &str,
        entity_type: &str,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.inner.objects_of_type(tenant_id, entity_type, page)
    }

    fn filter(
        &self,
        tenant_id: &str,
        entity_type: &str,
        predicate: &PropertyPredicate,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.inner.filter(tenant_id, entity_type, predicate, page)
    }

    fn links_from(
        &self,
        tenant_id: &str,
        object_ref: &str,
    ) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.inner.links_from(tenant_id, object_ref)
    }

    fn links_to(
        &self,
        tenant_id: &str,
        object_ref: &str,
    ) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.inner.links_to(tenant_id, object_ref)
    }

    fn poisoned(&self, tenant_id: &str) -> Result<Vec<(u64, String)>, ProjectionStoreError> {
        if self.fail_poisoned {
            return Err(ProjectionStoreError::Storage {
                detail: "poison ledger unreadable".to_owned(),
            });
        }
        self.inner.poisoned(tenant_id)
    }
}
