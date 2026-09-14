//! Projection store doubles: one that fails every call, and one that takes
//! reads but refuses every mirror.
#![allow(dead_code)]

use foundry_projection_draft::{
    AppliedEntry, ApplyReceipt, KeyDesignations, MemoryProjectionStore, Page, PageRequest,
    ProjectedLink, ProjectedObject, ProjectionStore, ProjectionStoreError, PropertyPredicate,
};

pub struct AlwaysFailingStore {
    pub detail: &'static str,
}

impl AlwaysFailingStore {
    fn fault(&self) -> ProjectionStoreError {
        ProjectionStoreError::Storage {
            detail: self.detail.to_owned(),
        }
    }
}

impl ProjectionStore for AlwaysFailingStore {
    fn apply(
        &mut self,
        _entry: AppliedEntry,
        _keys: &KeyDesignations,
    ) -> Result<ApplyReceipt, ProjectionStoreError> {
        Err(self.fault())
    }
    fn applied_head(&self, _tenant_id: &str) -> Result<u64, ProjectionStoreError> {
        Err(self.fault())
    }
    fn get(&self, _t: &str, _o: &str) -> Result<Option<ProjectedObject>, ProjectionStoreError> {
        Err(self.fault())
    }
    fn objects_of_type(
        &self,
        _t: &str,
        _e: &str,
        _p: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        Err(self.fault())
    }
    fn filter(
        &self,
        _t: &str,
        _e: &str,
        _q: &PropertyPredicate,
        _p: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        Err(self.fault())
    }
    fn links_from(&self, _t: &str, _o: &str) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        Err(self.fault())
    }
    fn links_to(&self, _t: &str, _o: &str) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        Err(self.fault())
    }
    fn poisoned(&self, _t: &str) -> Result<Vec<(u64, String)>, ProjectionStoreError> {
        Err(self.fault())
    }
}

/// Reads answer from a memory store; the first `refusals` mirrors are
/// refused and the rest are taken. The shape of a store that is reachable
/// but could not take a write.
#[derive(Default)]
pub struct ApplyRefusingStore {
    inner: MemoryProjectionStore,
    refusals: usize,
}

impl ApplyRefusingStore {
    pub fn refusing_the_first(refusals: usize) -> Self {
        Self {
            inner: MemoryProjectionStore::default(),
            refusals,
        }
    }
}

impl ProjectionStore for ApplyRefusingStore {
    fn apply(
        &mut self,
        entry: AppliedEntry,
        keys: &KeyDesignations,
    ) -> Result<ApplyReceipt, ProjectionStoreError> {
        if self.refusals > 0 {
            self.refusals -= 1;
            return Err(ProjectionStoreError::Storage {
                detail: "the store refused this mirror".to_owned(),
            });
        }
        self.inner.apply(entry, keys)
    }
    fn applied_head(&self, tenant_id: &str) -> Result<u64, ProjectionStoreError> {
        self.inner.applied_head(tenant_id)
    }
    fn get(&self, t: &str, o: &str) -> Result<Option<ProjectedObject>, ProjectionStoreError> {
        self.inner.get(t, o)
    }
    fn objects_of_type(
        &self,
        t: &str,
        e: &str,
        p: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.inner.objects_of_type(t, e, p)
    }
    fn filter(
        &self,
        t: &str,
        e: &str,
        q: &PropertyPredicate,
        p: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.inner.filter(t, e, q, p)
    }
    fn links_from(&self, t: &str, o: &str) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.inner.links_from(t, o)
    }
    fn links_to(&self, t: &str, o: &str) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.inner.links_to(t, o)
    }
    fn poisoned(&self, t: &str) -> Result<Vec<(u64, String)>, ProjectionStoreError> {
        self.inner.poisoned(t)
    }
}
