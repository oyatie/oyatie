//! Shared fixtures for the join conformance suite: the inline Cedar bundle
//! whose only grant is a group membership, the nested-membership model, and
//! the request/entity constructors.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacReadSnapshot, RebacRelation, RebacSubjectRef, RebacTenantScope,
    RebacTuple, RebacTupleStore, UsersetRewrite, Zookie,
};
use policy_decision_domain::{
    DecisionError, ExpansionInputs, MembershipCandidate, PolicyDecisionPoint, decide,
    materialize_parents,
};
use policy_pdp_cedar::CedarPdp;
use policy_pdp_kernel::{EntityRecord, EntitySlice, PdpError, PdpOutcome, PolicyBundle};
use policy_rebac_domain::{NamespaceConfig, ValidatedNamespace};
use policy_tuple_store_inmemory::InMemoryTupleStore;
use shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, Decision, EntityRef, PolicyVersion,
};
use shared_ulid_id_kernel::SeededIdGenerator;

pub const SCHEMA: &str = r#"
entity Group in [Group];
entity User in [Group];
entity Doc;
action "read" appliesTo { principal: [User], resource: [Doc] };
"#;

/// Membership decides everything: the only grant is `principal in
/// Group::"eng"`, so an allow can only come from a parent edge — and the
/// only source of parent edges is the graph walk.
pub const POLICIES: &str = r#"
permit(principal in Group::"eng", action == Action::"read", resource);
"#;

pub fn pdp() -> CedarPdp {
    let bundle = PolicyBundle {
        version: PolicyVersion::new("psv-join-000001").expect("valid version token"),
        schema_src: SCHEMA.to_owned(),
        policies_src: POLICIES.to_owned(),
        tenant_policies: BTreeMap::new(),
        templates: Vec::new(),
        template_links: Vec::new(),
        action_map: BTreeMap::from([("read".to_owned(), "Action::\"read\"".to_owned())]),
    };
    CedarPdp::load(&bundle, Arc::new(SeededIdGenerator::default()), 16)
        .expect("inline join bundle must load")
}

pub fn tenant() -> RebacTenantScope {
    RebacTenantScope::new("ten_join").expect("tenant scope is valid")
}

pub fn model() -> ValidatedNamespace {
    NamespaceConfig::new()
        .define("group", &relation("member"), UsersetRewrite::this())
        .validated()
        .expect("the membership model is stratified")
}

pub fn relation(name: &str) -> RebacRelation {
    RebacRelation::new(name).expect("relation is valid")
}

pub fn object(reference: &str) -> RebacObjectRef {
    RebacObjectRef::parse(reference).expect("object reference is valid")
}

pub fn write(store: &mut InMemoryTupleStore, tuple: &str) -> Zookie {
    let parsed = RebacTuple::parse(tenant(), tuple).expect("canonical tuple parses");
    store.write_tuple(parsed).expect("write succeeds")
}

pub fn entity(entity_type: &str, entity_id: &str) -> EntityRef {
    EntityRef {
        entity_type: entity_type.to_owned(),
        entity_id: entity_id.to_owned(),
    }
}

pub fn request(principal_id: &str) -> AuthorizationRequest {
    AuthorizationRequest {
        request_id: format!("req-join-{principal_id}"),
        tenant_id: "ten_join".to_owned(),
        principal: entity("User", principal_id),
        action: "read".to_owned(),
        resource: entity("Doc", "spec"),
        context: BTreeMap::new(),
        min_policy_version: None,
    }
}

pub fn eng_candidate() -> MembershipCandidate {
    MembershipCandidate {
        object: object("group:eng"),
        relation: relation("member"),
        parent: entity("Group", "eng"),
    }
}

pub fn context_entities() -> Vec<EntityRecord> {
    let bare = |uid: EntityRef| EntityRecord {
        uid,
        attributes: BTreeMap::new(),
        parents: Vec::new(),
    };
    vec![bare(entity("Doc", "spec")), bare(entity("Group", "eng"))]
}

pub fn graph<'a>(
    store: &'a InMemoryTupleStore,
    namespace: &'a ValidatedNamespace,
    snapshot: RebacReadSnapshot,
    subject: &'a RebacSubjectRef,
    candidates: &'a [MembershipCandidate],
) -> ExpansionInputs<'a, InMemoryTupleStore> {
    ExpansionInputs {
        store,
        namespace,
        tenant: tenant(),
        snapshot,
        subject,
        candidates,
    }
}
