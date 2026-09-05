//! Every returned tuple must match the exact tenant/object/relation query.

use std::cell::RefCell;
use std::collections::BTreeMap;

use policy_cedar_domain::rebac::{
    RebacReadSnapshot, RebacTenantScope, RebacTuple, RebacTuplePage, RebacTupleQuery,
    RebacTupleStore, RebacTupleStoreError, ResolvedRebacSnapshot, SnapshotToken, Zookie,
};
use policy_decision_domain::{DecisionError, decide};
use policy_rebac_domain::ExpansionError;

use crate::decision_scope_support::MustNotDecide;
use crate::join_fixtures::{
    context_entities, eng_candidate, graph, model, object, relation, request, tenant,
};

struct OutOfQueryStore {
    tuple_tenant: RebacTenantScope,
    tuple: &'static str,
    corrupt_on_read: usize,
    queries: RefCell<Vec<RebacTupleQuery>>,
}

impl OutOfQueryStore {
    fn new(tuple_tenant: RebacTenantScope, tuple: &'static str) -> Self {
        Self {
            tuple_tenant,
            tuple,
            corrupt_on_read: 1,
            queries: RefCell::new(Vec::new()),
        }
    }

    fn late(tuple_tenant: RebacTenantScope, tuple: &'static str) -> Self {
        Self {
            tuple_tenant,
            tuple,
            corrupt_on_read: 2,
            queries: RefCell::new(Vec::new()),
        }
    }

    fn assert_only_expected_query(&self) {
        assert_eq!(
            self.queries.borrow().as_slice(),
            &[RebacTupleQuery::object_relation(
                tenant(),
                object("group:eng"),
                relation("member"),
            )]
        );
    }

    fn assert_two_expected_pages(&self) {
        assert_eq!(
            self.queries.borrow().as_slice(),
            &[
                RebacTupleQuery::object_relation(
                    tenant(),
                    object("group:eng"),
                    relation("member"),
                ),
                RebacTupleQuery::object_relation(
                    tenant(),
                    object("group:eng"),
                    relation("member"),
                )
                .at_page(Some("second-page".to_owned())),
            ]
        );
    }
}

impl RebacTupleStore for OutOfQueryStore {
    fn write_tuple(&mut self, _tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError> {
        Err(RebacTupleStoreError::Backend(
            "read-only adversarial store".to_owned(),
        ))
    }

    fn resolve_snapshot(
        &self,
        tenant: &RebacTenantScope,
        _requested: RebacReadSnapshot,
    ) -> Result<ResolvedRebacSnapshot, RebacTupleStoreError> {
        Ok(ResolvedRebacSnapshot::new(
            tenant.clone(),
            SnapshotToken::new("coherent-head").expect("token is valid"),
        ))
    }

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: &ResolvedRebacSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError> {
        let read = {
            let mut queries = self.queries.borrow_mut();
            queries.push(query.clone());
            queries.len()
        };
        if read < self.corrupt_on_read {
            return Ok(RebacTuplePage {
                tuples: vec![
                    RebacTuple::parse(query.tenant.clone(), "group:eng#member@user:bob")
                        .expect("non-granting tuple is valid"),
                ],
                snapshot: snapshot.clone(),
                next_page_token: Some("second-page".to_owned()),
            });
        }
        Ok(RebacTuplePage {
            tuples: vec![
                RebacTuple::parse(self.tuple_tenant.clone(), self.tuple)
                    .expect("adversarial tuple is valid"),
            ],
            snapshot: snapshot.clone(),
            next_page_token: None,
        })
    }
}

#[test]
fn tuple_store_cannot_return_a_tuple_from_another_tenant() {
    let store = OutOfQueryStore::new(
        RebacTenantScope::new("ten_other").expect("other tenant is valid"),
        "group:eng#member@user:alice",
    );
    let namespace = model();
    let candidates = [eng_candidate()];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);
    let engine = MustNotDecide::new();

    let result = decide(
        &engine,
        &inputs,
        &request("alice"),
        BTreeMap::new(),
        context_entities(),
    );

    assert!(matches!(
        result,
        Err(DecisionError::Expansion(ExpansionError::Store(
            RebacTupleStoreError::TupleOutsideQuery { query, tuple }
        ))) if query.tenant.as_str() == "ten_join"
            && tuple.tenant.as_str() == "ten_other"
            && tuple.object == object("group:eng")
            && tuple.relation == relation("member")
    ));
    store.assert_only_expected_query();
    assert!(!engine.was_consulted());
}

#[test]
fn later_tuple_page_cannot_return_an_out_of_query_tuple() {
    let store = OutOfQueryStore::late(tenant(), "group:other#member@user:alice");
    let namespace = model();
    let candidates = [eng_candidate()];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);
    let engine = MustNotDecide::new();

    let result = decide(
        &engine,
        &inputs,
        &request("alice"),
        BTreeMap::new(),
        context_entities(),
    );

    assert!(matches!(
        result,
        Err(DecisionError::Expansion(ExpansionError::Store(
            RebacTupleStoreError::TupleOutsideQuery { query, tuple }
        ))) if query.object.as_ref() == Some(&object("group:eng"))
            && query.relation.as_ref() == Some(&relation("member"))
            && query.page_token.as_deref() == Some("second-page")
            && tuple.object == object("group:other")
    ));
    store.assert_two_expected_pages();
    assert!(!engine.was_consulted());
}

#[test]
fn tuple_store_cannot_return_a_same_tenant_tuple_for_another_object() {
    let store = OutOfQueryStore::new(tenant(), "group:other#member@user:alice");
    let namespace = model();
    let candidates = [eng_candidate()];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);
    let engine = MustNotDecide::new();

    let result = decide(
        &engine,
        &inputs,
        &request("alice"),
        BTreeMap::new(),
        context_entities(),
    );

    assert!(matches!(
        result,
        Err(DecisionError::Expansion(ExpansionError::Store(
            RebacTupleStoreError::TupleOutsideQuery { query, tuple }
        ))) if query.tenant == tuple.tenant
            && query.object.as_ref() == Some(&object("group:eng"))
            && tuple.object == object("group:other")
            && tuple.relation == relation("member")
    ));
    store.assert_only_expected_query();
    assert!(!engine.was_consulted());
}

#[test]
fn tuple_store_cannot_return_a_same_object_tuple_for_another_relation() {
    let store = OutOfQueryStore::new(tenant(), "group:eng#owner@user:alice");
    let namespace = model();
    let candidates = [eng_candidate()];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);
    let engine = MustNotDecide::new();

    let result = decide(
        &engine,
        &inputs,
        &request("alice"),
        BTreeMap::new(),
        context_entities(),
    );

    assert!(matches!(
        result,
        Err(DecisionError::Expansion(ExpansionError::Store(
            RebacTupleStoreError::TupleOutsideQuery { query, tuple }
        ))) if query.tenant == tuple.tenant
            && query.object.as_ref() == Some(&tuple.object)
            && query.relation.as_ref() == Some(&relation("member"))
            && tuple.relation == relation("owner")
    ));
    store.assert_only_expected_query();
    assert!(!engine.was_consulted());
}
