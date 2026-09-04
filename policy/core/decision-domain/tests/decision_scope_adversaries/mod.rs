//! Fail-closed conformance for malicious tuple-store responses and principal
//! types outside the policy-owned identity mapping.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use policy_cedar_domain::rebac::{
    RebacReadSnapshot, RebacTenantScope, RebacTuple, RebacTuplePage, RebacTupleQuery,
    RebacTupleStore, RebacTupleStoreError, ResolvedRebacSnapshot, SnapshotToken, Zookie,
};
use policy_decision_domain::{DecisionError, IdentityMappingError, decide};
use policy_rebac_domain::ExpansionError;

use super::decision_scope_support::MustNotDecide;
use super::join_fixtures::{context_entities, eng_candidate, graph, model, request};

mod page_snapshot_refusals;
mod tuple_query_refusals;

struct TenantSubstitutionStore {
    resolutions: AtomicUsize,
    reads: AtomicUsize,
}

impl TenantSubstitutionStore {
    fn new() -> Self {
        Self {
            resolutions: AtomicUsize::new(0),
            reads: AtomicUsize::new(0),
        }
    }
}

impl RebacTupleStore for TenantSubstitutionStore {
    fn write_tuple(&mut self, _tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError> {
        Err(RebacTupleStoreError::Backend(
            "read-only adversarial store".to_owned(),
        ))
    }

    fn resolve_snapshot(
        &self,
        _tenant: &RebacTenantScope,
        _requested: RebacReadSnapshot,
    ) -> Result<ResolvedRebacSnapshot, RebacTupleStoreError> {
        self.resolutions.fetch_add(1, Ordering::SeqCst);
        Ok(ResolvedRebacSnapshot::new(
            RebacTenantScope::new("ten_other").expect("other tenant is valid"),
            SnapshotToken::new("substituted-head").expect("token is valid"),
        ))
    }

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: &ResolvedRebacSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(RebacTuplePage {
            tuples: vec![
                RebacTuple::parse(query.tenant.clone(), "group:eng#member@user:alice")
                    .expect("adversarial tuple is valid"),
            ],
            snapshot: snapshot.clone(),
            next_page_token: None,
        })
    }
}

struct SnapshotSubstitutionStore {
    reads: AtomicUsize,
}

impl SnapshotSubstitutionStore {
    fn new() -> Self {
        Self {
            reads: AtomicUsize::new(0),
        }
    }
}

impl RebacTupleStore for SnapshotSubstitutionStore {
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
            SnapshotToken::new("after").expect("substituted token is valid"),
        ))
    }

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: &ResolvedRebacSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(RebacTuplePage {
            tuples: vec![
                RebacTuple::parse(query.tenant.clone(), "group:eng#member@user:alice")
                    .expect("later grant is valid"),
            ],
            snapshot: snapshot.clone(),
            next_page_token: None,
        })
    }
}

#[test]
fn resolver_cannot_substitute_the_request_tenant() {
    let store = TenantSubstitutionStore::new();
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
            RebacTupleStoreError::SnapshotScopeMismatch {
                query_tenant,
                snapshot_tenant,
            }
        ))) if query_tenant.as_str() == "ten_join"
            && snapshot_tenant.as_str() == "ten_other"
    ));
    assert_eq!(store.resolutions.load(Ordering::SeqCst), 1);
    assert_eq!(store.reads.load(Ordering::SeqCst), 0);
    assert!(!engine.was_consulted());
}

#[test]
fn resolver_cannot_substitute_an_explicit_snapshot_token() {
    let store = SnapshotSubstitutionStore::new();
    let namespace = model();
    let candidates = [eng_candidate()];
    let requested =
        RebacReadSnapshot::at(SnapshotToken::new("before").expect("requested token is valid"));
    let inputs = graph(&store, &namespace, requested, &candidates);
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
            RebacTupleStoreError::InconsistentSnapshot { requested, served }
        ))) if requested.as_str() == "before" && served.as_str() == "after"
    ));
    assert_eq!(store.reads.load(Ordering::SeqCst), 0);
    assert!(!engine.was_consulted());
}

#[test]
fn unmapped_principal_type_is_refused_before_store_or_cedar() {
    let store = TenantSubstitutionStore::new();
    let namespace = model();
    let candidates = [eng_candidate()];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);
    let mut unmapped = request("alice");
    unmapped.principal.entity_type = "Service".to_owned();
    let engine = MustNotDecide::new();

    let result = decide(&engine, &inputs, &unmapped, BTreeMap::new(), Vec::new());

    assert!(matches!(
        result,
        Err(DecisionError::Identity(
            IdentityMappingError::UnmappedPrincipalType { .. }
        ))
    ));
    assert_eq!(store.resolutions.load(Ordering::SeqCst), 0);
    assert_eq!(store.reads.load(Ordering::SeqCst), 0);
    assert!(!engine.was_consulted());
}
