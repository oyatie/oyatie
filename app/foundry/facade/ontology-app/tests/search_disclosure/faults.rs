use axum::http::StatusCode;
use foundry_projection_draft::{
    AppliedEntry, ApplyReceipt, KeyDesignations, MemoryProjectionStore, Page, PageRequest,
    ProjectedLink, ProjectedObject, ProjectionStore, ProjectionStoreError, PropertyPredicate,
};

use crate::facade_support::TENANT;
use crate::support::{GraphFixture, body, links, objects, refusal};

struct ReadFault {
    inner: Box<dyn ProjectionStore + Send>,
    fail_links: bool,
}

fn unavailable() -> ProjectionStoreError {
    ProjectionStoreError::Storage {
        detail: "private fixture storage diagnostic".into(),
    }
}

impl ProjectionStore for ReadFault {
    fn apply(
        &mut self,
        entry: AppliedEntry,
        keys: &KeyDesignations,
    ) -> Result<ApplyReceipt, ProjectionStoreError> {
        self.inner.apply(entry, keys)
    }
    fn applied_head(&self, tenant: &str) -> Result<u64, ProjectionStoreError> {
        self.inner.applied_head(tenant)
    }
    fn get(
        &self,
        tenant: &str,
        object: &str,
    ) -> Result<Option<ProjectedObject>, ProjectionStoreError> {
        if !self.fail_links && object == "ent_team" {
            return Err(unavailable());
        }
        self.inner.get(tenant, object)
    }
    fn objects_of_type(
        &self,
        tenant: &str,
        kind: &str,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.inner.objects_of_type(tenant, kind, page)
    }
    fn filter(
        &self,
        tenant: &str,
        kind: &str,
        predicate: &PropertyPredicate,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.inner.filter(tenant, kind, predicate, page)
    }
    fn links_from(
        &self,
        tenant: &str,
        object: &str,
    ) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        if self.fail_links {
            return Err(unavailable());
        }
        self.inner.links_from(tenant, object)
    }
    fn links_to(
        &self,
        tenant: &str,
        object: &str,
    ) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.inner.links_to(tenant, object)
    }
    fn poisoned(&self, tenant: &str) -> Result<Vec<(u64, String)>, ProjectionStoreError> {
        self.inner.poisoned(tenant)
    }
}

#[tokio::test]
async fn cached_endpoint_or_edge_storage_fault_is_not_an_empty_graph_or_success() {
    for fail_links in [false, true] {
        let fixture = GraphFixture::new("cached-store-fault", objects(), links());
        let request = body("cached-store-fault");
        assert_eq!(fixture.walk(&request).await.0, StatusCode::OK);
        let before = {
            let mut tenant = fixture.state.tenants[TENANT].lock().await;
            let inner = std::mem::replace(
                &mut tenant.projection_store,
                Box::new(MemoryProjectionStore::default()),
            );
            tenant.projection_store = Box::new(ReadFault { inner, fail_links });
            (
                tenant.queries.clone(),
                tenant.query_decisions.clone(),
                tenant.projection.clone(),
            )
        };
        let (status, reply) = fixture.walk(&request).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{reply}");
        refusal(&reply, "store");
        assert!(
            !reply
                .to_string()
                .contains("private fixture storage diagnostic")
        );
        let tenant = fixture.state.tenants[TENANT].lock().await;
        assert_eq!(
            (
                tenant.queries.clone(),
                tenant.query_decisions.clone(),
                tenant.projection.clone()
            ),
            before
        );
        drop(tenant);
        assert_eq!(fixture.counters().await, (1, 1));
    }
}
