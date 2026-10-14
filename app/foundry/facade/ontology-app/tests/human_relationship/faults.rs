use axum::http::StatusCode;

use crate::facade_support::TENANT;
use crate::human_support::{now, view};
use crate::support::{HumanGraph, request};

#[tokio::test]
async fn unreadable_projection_is_unavailable_and_never_successful_empty() {
    let fixture = HumanGraph::new("human-store-fault");
    fixture.state.tenants[TENANT].lock().await.projection_store =
        Box::new(crate::failing_store::AlwaysFailingStore {
            detail: "injected storage failure",
        });
    let refusal = fixture
        .walk(&request("store-fault"), StatusCode::SERVICE_UNAVAILABLE)
        .await;
    assert_eq!(refusal["gate"], "store");
    assert!(refusal.get("nodes").is_none());
}

#[tokio::test]
async fn expiry_after_node_admission_refuses_before_emitting_any_graph() {
    let fixture = HumanGraph::new("human-expiry-emission");
    let mut membership = view();
    membership.expires_at = now() + 1;
    *fixture.sessions.answer.lock().unwrap() = Ok(membership);
    {
        let mut tenant = fixture.state.tenants[TENANT].lock().await;
        let inner = std::mem::replace(
            &mut tenant.projection_store,
            Box::new(foundry_projection_draft::MemoryProjectionStore::default()),
        );
        tenant.projection_store = Box::new(SlowNeighbor {
            inner,
            waited: std::sync::atomic::AtomicBool::new(false),
        });
    }
    let refusal = fixture
        .walk(&request("expiry"), StatusCode::UNAUTHORIZED)
        .await;
    assert_eq!(refusal["gate"], "session");
    assert!(refusal.get("nodes").is_none());
    assert_eq!(
        fixture.state.tenants[TENANT]
            .lock()
            .await
            .queries
            .receipt_count(),
        1,
        "expiry is checked after a completed query and before disclosure"
    );
}

use foundry_projection_draft::{
    AppliedEntry, ApplyReceipt, KeyDesignations, Page, PageRequest, ProjectedLink, ProjectedObject,
    ProjectionStore, ProjectionStoreError, PropertyPredicate,
};
struct SlowNeighbor {
    inner: Box<dyn ProjectionStore + Send>,
    waited: std::sync::atomic::AtomicBool,
}
impl ProjectionStore for SlowNeighbor {
    fn get(&self, t: &str, r: &str) -> Result<Option<ProjectedObject>, ProjectionStoreError> {
        if r == "ent_team" && !self.waited.swap(true, std::sync::atomic::Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(1200));
        }
        self.inner.get(t, r)
    }
    fn apply(
        &mut self,
        e: AppliedEntry,
        k: &KeyDesignations,
    ) -> Result<ApplyReceipt, ProjectionStoreError> {
        self.inner.apply(e, k)
    }
    fn applied_head(&self, t: &str) -> Result<u64, ProjectionStoreError> {
        self.inner.applied_head(t)
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
    fn links_from(&self, t: &str, r: &str) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.inner.links_from(t, r)
    }
    fn links_to(&self, t: &str, r: &str) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.inner.links_to(t, r)
    }
    fn poisoned(&self, t: &str) -> Result<Vec<(u64, String)>, ProjectionStoreError> {
        self.inner.poisoned(t)
    }
}
