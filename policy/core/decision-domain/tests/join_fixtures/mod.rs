//! Shared fixtures for the join conformance suite: the inline Cedar bundle
//! whose only grant is a group membership, the nested-membership model, and
//! the request/entity constructors.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacReadSnapshot, RebacRelation, RebacTenantScope, RebacTuple, RebacTuplePage,
    RebacTupleQuery, RebacTupleStore, RebacTupleStoreError, ResolvedRebacSnapshot, SnapshotToken,
    UsersetRewrite, Zookie,
};
use policy_decision_domain::{
    DecisionError, DecisionInputs, MembershipCandidate, PolicyDecisionPoint, PrincipalMapping,
    decide, materialize_parents,
};
use policy_pdp_cedar::CedarPdp;
use policy_pdp_kernel::{EntityRecord, EntitySlice, PdpError, PdpOutcome, PolicyBundle};
use policy_rebac_domain::{ExpansionBounds, NamespaceConfig, ValidatedNamespace};
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
    candidate("eng")
}

pub fn candidate(group: &str) -> MembershipCandidate {
    MembershipCandidate {
        object: object(&format!("group:{group}")),
        relation: relation("member"),
        parent: entity("Group", group),
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

pub fn principal_mapping() -> PrincipalMapping {
    PrincipalMapping::new("User", "user").expect("principal mapping is valid")
}

pub fn graph<'a, S: RebacTupleStore>(
    store: &'a S,
    namespace: &'a ValidatedNamespace,
    snapshot: RebacReadSnapshot,
    candidates: &'a [MembershipCandidate],
) -> DecisionInputs<'a, S> {
    bounded_graph(
        store,
        namespace,
        snapshot,
        candidates,
        ExpansionBounds::DEFAULT,
    )
}

pub fn bounded_graph<'a, S: RebacTupleStore>(
    store: &'a S,
    namespace: &'a ValidatedNamespace,
    snapshot: RebacReadSnapshot,
    candidates: &'a [MembershipCandidate],
    bounds: ExpansionBounds,
) -> DecisionInputs<'a, S> {
    DecisionInputs::new(
        store,
        namespace,
        principal_mapping(),
        snapshot,
        candidates,
        bounds,
    )
}

/// A legal shared-backend adapter with two successive `Latest` worlds.
pub struct AdvancingLatestStore {
    resolutions: AtomicUsize,
}

impl AdvancingLatestStore {
    pub fn new() -> Self {
        Self {
            resolutions: AtomicUsize::new(0),
        }
    }

    pub fn resolution_count(&self) -> usize {
        self.resolutions.load(Ordering::SeqCst)
    }
}

impl RebacTupleStore for AdvancingLatestStore {
    fn write_tuple(&mut self, _tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError> {
        Err(RebacTupleStoreError::Backend(
            "read-only conformance store".to_owned(),
        ))
    }

    fn resolve_snapshot(
        &self,
        tenant: &RebacTenantScope,
        requested: RebacReadSnapshot,
    ) -> Result<ResolvedRebacSnapshot, RebacTupleStoreError> {
        let version = match requested {
            RebacReadSnapshot::Latest => 5 + self.resolutions.fetch_add(1, Ordering::SeqCst),
            RebacReadSnapshot::At { snapshot } => snapshot
                .as_str()
                .parse::<usize>()
                .map_err(|error| RebacTupleStoreError::Backend(error.to_string()))?,
        };
        let token =
            SnapshotToken::new(version.to_string()).map_err(RebacTupleStoreError::InvalidZookie)?;
        Ok(ResolvedRebacSnapshot::new(tenant.clone(), token))
    }

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: &ResolvedRebacSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError> {
        let object_id = query.object.as_ref().expect("object query").object_id();
        let held = matches!((snapshot.as_str(), object_id), ("5", "a") | ("6", "b"));
        let tuples = held
            .then(|| {
                RebacTuple::parse(
                    query.tenant.clone(),
                    &format!("group:{object_id}#member@user:alice"),
                )
                .expect("test tuple is valid")
            })
            .into_iter()
            .collect();
        Ok(RebacTuplePage {
            tuples,
            snapshot: snapshot.clone(),
            next_page_token: None,
        })
    }
}

/// Returns the configured tuple count for every candidate object.
pub struct WideCandidateStore {
    tuples_per_candidate: usize,
}

impl WideCandidateStore {
    pub fn new(tuples_per_candidate: usize) -> Self {
        Self {
            tuples_per_candidate,
        }
    }
}

impl RebacTupleStore for WideCandidateStore {
    fn write_tuple(&mut self, _tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError> {
        Err(RebacTupleStoreError::Backend(
            "read-only conformance store".to_owned(),
        ))
    }

    fn resolve_snapshot(
        &self,
        tenant: &RebacTenantScope,
        _requested: RebacReadSnapshot,
    ) -> Result<ResolvedRebacSnapshot, RebacTupleStoreError> {
        let token = SnapshotToken::new("5").map_err(RebacTupleStoreError::InvalidZookie)?;
        Ok(ResolvedRebacSnapshot::new(tenant.clone(), token))
    }

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: &ResolvedRebacSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError> {
        let object_id = query.object.as_ref().expect("object query").object_id();
        let tuples = (0..self.tuples_per_candidate)
            .map(|index| {
                let subject = if index + 1 == self.tuples_per_candidate {
                    "alice".to_owned()
                } else {
                    format!("filler-{object_id}-{index}")
                };
                RebacTuple::parse(
                    query.tenant.clone(),
                    &format!("group:{object_id}#member@user:{subject}"),
                )
                .expect("test tuple is valid")
            })
            .collect();
        Ok(RebacTuplePage {
            tuples,
            snapshot: snapshot.clone(),
            next_page_token: None,
        })
    }
}
