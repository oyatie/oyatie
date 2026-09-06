use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacReadSnapshot, RebacRelation, RebacSubjectRef, RebacTenantScope,
    RebacTuple, RebacTuplePage, RebacTupleQuery, RebacTupleStore, RebacTupleStoreError,
    ResolvedRebacSnapshot, SnapshotToken, Zookie,
};
use policy_rebac_domain::{Expander, ValidatedNamespace};

struct ImmutableStore {
    tuples: Vec<RebacTuple>,
    snapshot: ResolvedRebacSnapshot,
}

pub fn assert_inherited_membership(namespace: &ValidatedNamespace) {
    let tenant = RebacTenantScope::new("ten_compilation").unwrap();
    let token = SnapshotToken::new("compiled_fixture").unwrap();
    let document = RebacObjectRef::new("document", "spec").unwrap();
    let folder = RebacObjectRef::new("folder", "team").unwrap();
    let alice = RebacSubjectRef::object(RebacObjectRef::new("user", "alice").unwrap());
    let bob = RebacSubjectRef::object(RebacObjectRef::new("user", "bob").unwrap());
    let store = ImmutableStore {
        tuples: vec![
            RebacTuple::new(
                tenant.clone(),
                document.clone(),
                RebacRelation::new("parent").unwrap(),
                RebacSubjectRef::object(folder.clone()),
            ),
            RebacTuple::new(
                tenant.clone(),
                folder,
                RebacRelation::new("member").unwrap(),
                alice.clone(),
            ),
        ],
        snapshot: ResolvedRebacSnapshot::new(tenant.clone(), token.clone()),
    };
    let expander = Expander::new(&store, namespace, tenant, RebacReadSnapshot::at(token));
    let viewer = RebacRelation::new("viewer").unwrap();
    assert_eq!(expander.check(&alice, &viewer, &document), Ok(true));
    assert_eq!(expander.check(&bob, &viewer, &document), Ok(false));
}

impl RebacTupleStore for ImmutableStore {
    fn write_tuple(&mut self, _tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError> {
        Err(RebacTupleStoreError::Backend(
            "compilation fixture is immutable".into(),
        ))
    }

    fn resolve_snapshot(
        &self,
        tenant: &RebacTenantScope,
        requested: RebacReadSnapshot,
    ) -> Result<ResolvedRebacSnapshot, RebacTupleStoreError> {
        assert_eq!(tenant, self.snapshot.tenant());
        assert_eq!(
            requested,
            RebacReadSnapshot::at(self.snapshot.token().clone())
        );
        Ok(self.snapshot.clone())
    }

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: &ResolvedRebacSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError> {
        assert_eq!(snapshot, &self.snapshot);
        assert!(query.page_token.is_none());
        Ok(RebacTuplePage {
            tuples: self
                .tuples
                .iter()
                .filter(|tuple| query.matches(tuple))
                .cloned()
                .collect(),
            snapshot: self.snapshot.clone(),
            next_page_token: None,
        })
    }
}
