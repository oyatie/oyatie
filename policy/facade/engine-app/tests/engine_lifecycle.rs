#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod support;

use policy_bundle_content::PolicySource;
use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacReadSnapshot, RebacRelation, RebacTenantScope, RebacTuple,
    RebacTupleStore, UsersetRewrite,
};
use policy_decision_domain::{DecisionInputs, MembershipCandidate, PrincipalMapping};
use policy_decision_service_kernel::BundleStoreError;
use policy_engine_app::{EngineLoadError, PolicyEngine};
use policy_pdp_bundle_file::FilePolicyBundleStore;
use policy_pdp_kernel::PolicyDecisionPoint;
use policy_rebac_domain::{ExpansionBounds, NamespaceConfig};
use policy_tuple_store_inmemory::InMemoryTupleStore;
use shared_platform_contracts_kernel::pdp::Decision;
use std::collections::BTreeMap;
use support::*;

#[test]
fn authored_cases_publish_load_and_serve_the_exact_qualified_version() {
    let fixture = Fixture::new();
    let authored = project();
    let expected_obligations = authored.cases[0].expected.obligations.clone();
    let prepared = authored.prepare(ids()).unwrap();
    prepared
        .publish(
            &fixture.store,
            &fixture.signer,
            &fixture.signer.public_key_bytes(),
        )
        .unwrap();
    let engine = PolicyEngine::load(&fixture.store, ids(), 16).unwrap();
    let first = engine
        .authorize(&request("alice"), &entities("alice"))
        .unwrap();
    let warm = engine
        .authorize(&request("alice"), &entities("alice"))
        .unwrap();
    assert!(!first.cache_hit);
    assert!(warm.cache_hit);
    assert_eq!(warm.response.decision, Decision::Allow);
    assert_eq!(warm.response.policy_version, prepared.bundle().version);
    assert_eq!(warm.response.obligations, expected_obligations);
    assert_eq!(warm.audit.policy_version, prepared.bundle().version);
}

#[test]
fn unavailable_reload_trust_preserves_loaded_content_obligations_and_warm_cache() {
    let fixture = Fixture::new();
    let authored = project();
    let expected_obligations = authored.cases[0].expected.obligations.clone();
    let prepared = authored.prepare(ids()).unwrap();
    prepared
        .publish(
            &fixture.store,
            &fixture.signer,
            &fixture.signer.public_key_bytes(),
        )
        .unwrap();
    let engine = PolicyEngine::load(&fixture.store, ids(), 16).unwrap();
    let first = engine
        .authorize(&request("alice"), &entities("alice"))
        .unwrap();
    let warm = engine
        .authorize(&request("alice"), &entities("alice"))
        .unwrap();
    assert!(!first.cache_hit);
    assert!(warm.cache_hit);

    let unavailable = FilePolicyBundleStore::new(
        fixture.store.path(),
        fixture.root.join("unavailable-trust-directory"),
    );
    assert!(matches!(
        engine.reload(&unavailable),
        Err(EngineLoadError::Store(BundleStoreError::Unavailable { .. }))
    ));
    assert_eq!(engine.loaded_policy_version(), prepared.bundle().version);
    let preserved = engine
        .authorize(&request("alice"), &entities("alice"))
        .unwrap();
    assert!(preserved.cache_hit);
    assert_eq!(preserved.response.decision, Decision::Allow);
    assert_eq!(preserved.response.obligations, expected_obligations);

    assert_eq!(
        engine.reload(&fixture.store).unwrap(),
        prepared.bundle().version
    );
    let reloaded = engine
        .authorize(&request("alice"), &entities("alice"))
        .unwrap();
    assert!(reloaded.cache_hit);
    assert_ne!(
        reloaded.response.decision_id,
        preserved.response.decision_id
    );
    assert_eq!(reloaded.response.decision, preserved.response.decision);
    assert_eq!(
        reloaded.response.policy_version,
        preserved.response.policy_version
    );
    assert_eq!(
        reloaded.response.determining_policy_ids,
        preserved.response.determining_policy_ids
    );
    assert_eq!(
        reloaded.response.obligations,
        preserved.response.obligations
    );
}

#[test]
fn reload_uses_verified_source_and_refusal_preserves_serving_decisions() {
    let fixture = Fixture::new();
    let prepared = project().prepare(ids()).unwrap();
    prepared
        .publish(
            &fixture.store,
            &fixture.signer,
            &fixture.signer.public_key_bytes(),
        )
        .unwrap();
    let engine = PolicyEngine::load(&fixture.store, ids(), 16).unwrap();
    let mut next = project();
    next.source.policies_src.clear();
    next.cases[0].expected = next.cases[1].expected.clone();
    let next = next.prepare(ids()).unwrap();
    next.publish(
        &fixture.store,
        &fixture.signer,
        &fixture.signer.public_key_bytes(),
    )
    .unwrap();
    assert_eq!(
        engine.reload(&fixture.store).unwrap(),
        next.bundle().version
    );
    let denied = engine
        .authorize(&request("alice"), &entities("alice"))
        .unwrap();
    assert_eq!(denied.response.decision, Decision::Deny);
    assert!(!denied.cache_hit);
    std::fs::write(fixture.store.path(), "unavailable bundle document").unwrap();
    assert!(matches!(
        engine.reload(&fixture.store),
        Err(EngineLoadError::Store(_))
    ));
    assert_eq!(engine.loaded_policy_version(), next.bundle().version);
    assert_eq!(
        engine
            .authorize(&request("alice"), &entities("alice"))
            .unwrap()
            .response
            .decision,
        Decision::Deny
    );
    let mut invalid = next.bundle().clone();
    invalid.schema_src = "invalid schema".into();
    invalid.version = PolicySource::from(&invalid).content_version().unwrap();
    fixture
        .store
        .write_signed_bundle(
            &invalid,
            &fixture.signer,
            &fixture.signer.public_key_bytes(),
        )
        .unwrap();
    assert!(matches!(
        engine.reload(&fixture.store),
        Err(EngineLoadError::Admission(_))
    ));
    assert_eq!(engine.loaded_policy_version(), next.bundle().version);
}

#[test]
fn verified_engine_join_preserves_policy_obligations_and_graph_snapshot() {
    let fixture = Fixture::new();
    let prepared = project().prepare(ids()).unwrap();
    prepared
        .publish(
            &fixture.store,
            &fixture.signer,
            &fixture.signer.public_key_bytes(),
        )
        .unwrap();
    let engine = PolicyEngine::load(&fixture.store, ids(), 16).unwrap();
    let tenant = RebacTenantScope::new("tenant").unwrap();
    let mut tuples = InMemoryTupleStore::new();
    let zookie = tuples
        .write_tuple(RebacTuple::parse(tenant.clone(), "group:eng#member@user:alice").unwrap())
        .unwrap();
    let relation = RebacRelation::new("member").unwrap();
    let namespace = NamespaceConfig::new()
        .define("group", &relation, UsersetRewrite::this())
        .validated()
        .unwrap();
    let candidates = [MembershipCandidate {
        object: RebacObjectRef::parse("group:eng").unwrap(),
        relation,
        parent: entity("Group", "eng"),
    }];
    let inputs = DecisionInputs::new(
        &tuples,
        &namespace,
        PrincipalMapping::new("User", "user").unwrap(),
        RebacReadSnapshot::at_zookie(zookie.clone()),
        &candidates,
        ExpansionBounds::DEFAULT,
    );
    let joined = engine
        .decide(
            &inputs,
            &request("alice"),
            BTreeMap::new(),
            context_entities(),
        )
        .unwrap();
    assert_eq!(joined.outcome.response.decision, Decision::Allow);
    assert_eq!(
        joined.outcome.response.policy_version,
        prepared.bundle().version
    );
    assert_eq!(
        joined.outcome.response.obligations[0].obligation_id,
        "record-access"
    );
    assert_eq!(joined.relationship_snapshot.tenant(), &tenant);
    assert_eq!(joined.relationship_snapshot.as_str(), zookie.as_str());
}
