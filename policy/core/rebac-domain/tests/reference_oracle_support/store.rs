use std::cell::Cell;

use policy_cedar_domain::rebac::{
    RebacReadSnapshot, RebacTenantScope, RebacTuple, RebacTuplePage, RebacTupleQuery,
    RebacTupleStore, RebacTupleStoreError, ResolvedRebacSnapshot, SnapshotToken, Zookie,
};

pub const CANCELLED_DETAIL: &str = "reference-oracle injected cancellation";

pub struct FiniteStore {
    tuples: Vec<RebacTuple>,
    page_size: usize,
    cancel_on_read: Option<usize>,
    snapshot_tenant: Option<RebacTenantScope>,
    reads: Cell<usize>,
}

impl FiniteStore {
    pub fn new(tuples: Vec<RebacTuple>, page_size: usize) -> Self {
        assert!(page_size > 0, "finite store pages must make progress");
        Self {
            tuples,
            page_size,
            cancel_on_read: None,
            snapshot_tenant: None,
            reads: Cell::new(0),
        }
    }

    pub fn cancelling_on_read(mut self, read: Option<usize>) -> Self {
        self.cancel_on_read = read;
        self
    }

    pub fn serving_snapshot_for(mut self, tenant: Option<RebacTenantScope>) -> Self {
        self.snapshot_tenant = tenant;
        self
    }

    fn snapshot(tenant: RebacTenantScope) -> Result<ResolvedRebacSnapshot, RebacTupleStoreError> {
        let token = SnapshotToken::new("finite").map_err(RebacTupleStoreError::InvalidZookie)?;
        Ok(ResolvedRebacSnapshot::new(tenant, token))
    }
}

impl RebacTupleStore for FiniteStore {
    fn write_tuple(&mut self, tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError> {
        self.tuples.push(tuple);
        Zookie::new("finite").map_err(RebacTupleStoreError::InvalidZookie)
    }

    fn resolve_snapshot(
        &self,
        tenant: &RebacTenantScope,
        _requested: RebacReadSnapshot,
    ) -> Result<ResolvedRebacSnapshot, RebacTupleStoreError> {
        Self::snapshot(
            self.snapshot_tenant
                .clone()
                .unwrap_or_else(|| tenant.clone()),
        )
    }

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: &ResolvedRebacSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError> {
        if &query.tenant != snapshot.tenant() {
            return Err(RebacTupleStoreError::SnapshotScopeMismatch {
                query_tenant: query.tenant.clone(),
                snapshot_tenant: snapshot.tenant().clone(),
            });
        }
        let read = self.reads.get().saturating_add(1);
        self.reads.set(read);
        if self.cancel_on_read == Some(read) {
            return Err(RebacTupleStoreError::Backend(CANCELLED_DETAIL.to_owned()));
        }
        let matched: Vec<RebacTuple> = self
            .tuples
            .iter()
            .filter(|tuple| {
                tuple.tenant == query.tenant
                    && query
                        .object
                        .as_ref()
                        .is_none_or(|object| object == &tuple.object)
                    && query
                        .relation
                        .as_ref()
                        .is_none_or(|relation| relation == &tuple.relation)
                    && query
                        .subject
                        .as_ref()
                        .is_none_or(|subject| subject == &tuple.subject)
            })
            .cloned()
            .collect();
        let start = query
            .page_token
            .as_deref()
            .unwrap_or("0")
            .parse::<usize>()
            .map_err(|_| RebacTupleStoreError::Backend("invalid finite page token".to_owned()))?;
        let end = start.saturating_add(self.page_size).min(matched.len());
        Ok(RebacTuplePage {
            tuples: matched.get(start..end).unwrap_or_default().to_vec(),
            snapshot: snapshot.clone(),
            next_page_token: (end < matched.len()).then(|| end.to_string()),
        })
    }
}
