//! The join: relationship expansion materialised into the entity slice the
//! embedded PDP decides against.
//!
//! A `Check` has two halves. The relationship graph answers "which usersets
//! does this principal belong to, at this snapshot"; the policy engine
//! answers "given those memberships, may it act". This crate is the seam:
//! it runs the bounded userset-rewrite walk for each candidate membership,
//! turns the ones that hold into [`EntityRecord::parents`] edges, and calls
//! [`PolicyDecisionPoint::authorize`] on the result. Cedar's `in` operator
//! closes the hierarchy from there.
//!
//! Expansion strictly precedes evaluation. The embedded-PDP doctrine says
//! the PDP evaluates against exactly the slice it is given and never
//! reaches out at decision time — so the graph is consulted here, before
//! the slice exists, never lazily from inside the engine. The signature
//! enforces the order: there is no path to `authorize` that does not pass
//! through materialisation first.
//!
//! Every error on either half is a fail-closed denial. An
//! [`ExpansionError`] means the graph was not fully walked; it must never
//! be read as "not a member".

use std::collections::BTreeMap;

use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacReadSnapshot, RebacRelation, RebacSubjectRef, RebacTenantScope,
    RebacTupleStore,
};
use policy_pdp_kernel::{EntityRecord, EntitySlice, PdpError, PdpOutcome};
use policy_rebac_domain::{Expander, ExpansionError, ValidatedNamespace};
use shared_platform_contracts_kernel::ContractViolation;
use shared_platform_contracts_kernel::pdp::{AuthorizationRequest, EntityRef};

pub use policy_pdp_kernel::PolicyDecisionPoint;

/// One userset whose membership the graph is asked about, and the Cedar
/// entity that membership materialises as.
///
/// The mapping between relationship-graph object types and Cedar entity
/// types is naming policy owned by the caller. The graph gates only
/// WHETHER each candidate's parent is emitted - `parent` itself is
/// caller-supplied and never inspected by the walk, so a caller that maps
/// a userset to the wrong Cedar entity has granted whatever that entity
/// grants. This crate guarantees exactly one thing: no candidate's parent
/// is emitted unless its membership held in the graph at one pinned
/// snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct MembershipCandidate {
    /// The userset object, e.g. `group:eng`.
    pub object: RebacObjectRef,
    /// The membership relation on it, e.g. `member`.
    pub relation: RebacRelation,
    /// The Cedar entity the principal gains as a parent when membership
    /// holds, e.g. `Group::"eng"`.
    pub parent: EntityRef,
}

/// The graph half of a decision: where to read, which model to walk, and
/// which memberships to ask about.
pub struct ExpansionInputs<'a, S: RebacTupleStore> {
    pub store: &'a S,
    pub namespace: &'a ValidatedNamespace,
    pub tenant: RebacTenantScope,
    pub snapshot: RebacReadSnapshot,
    /// The principal as the relationship graph names it.
    pub subject: &'a RebacSubjectRef,
    pub candidates: &'a [MembershipCandidate],
}

/// Why a joined decision was refused. Every variant is a denial; none may
/// be read as "deny was decided" — the decision was never reached.
#[derive(Debug)]
pub enum DecisionError {
    /// The graph was not fully walked (depth, budget, cycle through a
    /// subtraction, stale snapshot, undefined relation).
    Expansion(ExpansionError),
    /// The materialised slice violated the entity contract.
    InvalidSlice(Vec<ContractViolation>),
    /// The engine refused (stale policy pin, unknown action, evaluation).
    Pdp(PdpError),
}

/// Walk every candidate membership for the subject and return the parent
/// edges for the ones that hold, all at one pinned snapshot.
///
/// # Errors
/// Any [`ExpansionError`] aborts the whole materialisation: a partial
/// parent set is an answer about a graph that was never fully consulted.
pub fn materialize_parents<S: RebacTupleStore>(
    graph: &ExpansionInputs<'_, S>,
) -> Result<Vec<EntityRef>, ExpansionError> {
    let expander = Expander::new(
        graph.store,
        graph.namespace,
        graph.tenant.clone(),
        graph.snapshot.clone(),
    );
    let mut parents = Vec::new();
    for candidate in graph.candidates {
        if expander.check(graph.subject, &candidate.relation, &candidate.object)? {
            parents.push(candidate.parent.clone());
        }
    }
    Ok(parents)
}

/// Decide one request: expand, materialise, evaluate — in that order.
///
/// The principal's [`EntityRecord`] is built here so its `parents` can only
/// come from the graph; `context_entities` carries the resource and any
/// other records the policy references, exactly as a PEP would supply them.
///
/// # Errors
/// Every variant of [`DecisionError`] is fail-closed: the caller must treat
/// it as deny, and must not conflate it with a decided
/// [`Decision::Deny`](shared_platform_contracts_kernel::pdp::Decision).
pub fn decide<S: RebacTupleStore>(
    pdp: &dyn PolicyDecisionPoint,
    graph: &ExpansionInputs<'_, S>,
    request: &AuthorizationRequest,
    principal_attributes: BTreeMap<String, serde_json::Value>,
    context_entities: Vec<EntityRecord>,
) -> Result<PdpOutcome, DecisionError> {
    let parents = materialize_parents(graph).map_err(DecisionError::Expansion)?;
    let principal = EntityRecord {
        uid: request.principal.clone(),
        attributes: principal_attributes,
        parents,
    };
    let mut entities = Vec::with_capacity(1 + context_entities.len());
    entities.push(principal);
    entities.extend(context_entities);
    let slice = EntitySlice { entities };
    slice.validate().map_err(DecisionError::InvalidSlice)?;
    pdp.authorize(request, &slice).map_err(DecisionError::Pdp)
}
