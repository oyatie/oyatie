use policy_cedar_domain::rebac::{
    RebacReadSnapshot, RebacRelation, RebacTenantScope, RebacTupleStore, RebacTupleStoreError,
};
use policy_rebac_domain::{Expander, ExpansionError, ExpansionSession};

use super::evaluator::FiniteEvaluator;
use super::model::{Bounds, Model, Outcome, Query, Refusal, Tuple};
use super::store::{CANCELLED_DETAIL, FiniteStore};

#[derive(Clone, Debug)]
pub struct Execution {
    pub bounds: Bounds,
    pub page_size: usize,
    pub cancel_on_read: Option<usize>,
    pub snapshot_tenant: Option<String>,
}

impl Default for Execution {
    fn default() -> Self {
        Self {
            bounds: Bounds::GENEROUS,
            page_size: 2,
            cancel_on_read: None,
            snapshot_tenant: None,
        }
    }
}

pub fn assert_match(model: &Model, tuples: &[Tuple], query: &Query, execution: &Execution) {
    assert_eq!(
        production_outcome(model, tuples, query, execution),
        oracle_outcome(model, tuples, query),
        "production diverged from the finite reference"
    );
}

pub fn oracle_outcome(model: &Model, tuples: &[Tuple], query: &Query) -> Outcome {
    match FiniteEvaluator::new(model, tuples) {
        Ok(evaluator) => evaluator.evaluate(query),
        Err(refusal) => Outcome::Refuse(refusal),
    }
}

pub fn production_outcome(
    model: &Model,
    tuples: &[Tuple],
    query: &Query,
    execution: &Execution,
) -> Outcome {
    let namespace = match model.native() {
        Ok(namespace) => namespace,
        Err(error) => return normalize(Err(error)),
    };
    let snapshot_tenant = execution
        .snapshot_tenant
        .as_ref()
        .map(|tenant| RebacTenantScope::new(tenant.clone()).expect("snapshot tenant is valid"));
    let store = FiniteStore::new(
        tuples.iter().map(Tuple::native).collect(),
        execution.page_size,
    )
    .cancelling_on_read(execution.cancel_on_read)
    .serving_snapshot_for(snapshot_tenant);
    let tenant = RebacTenantScope::new(query.tenant.clone()).expect("oracle tenant is valid");
    let relation = RebacRelation::new(query.relation.clone()).expect("oracle relation is valid");
    normalize(
        Expander::new(&store, &namespace, tenant, RebacReadSnapshot::latest())
            .with_bounds(execution.bounds.native())
            .check(&query.subject.native(), &relation, &query.object.native()),
    )
}

pub fn production_session_outcomes(
    model: &Model,
    tuples: &[Tuple],
    query: &Query,
    bounds: Bounds,
    checks: usize,
) -> Vec<Outcome> {
    let namespace = model.native().expect("session model is valid");
    let store = FiniteStore::new(tuples.iter().map(Tuple::native).collect(), 2);
    let tenant = RebacTenantScope::new(query.tenant.clone()).expect("oracle tenant is valid");
    let snapshot = store
        .resolve_snapshot(&tenant, RebacReadSnapshot::latest())
        .expect("finite snapshot resolves");
    let relation = RebacRelation::new(query.relation.clone()).expect("oracle relation is valid");
    let mut session = ExpansionSession::new(&store, &namespace, snapshot, bounds.native());
    (0..checks)
        .map(|_| {
            normalize(session.check(&query.subject.native(), &relation, &query.object.native()))
        })
        .collect()
}

fn normalize(result: Result<bool, ExpansionError>) -> Outcome {
    match result {
        Ok(true) => Outcome::Allow,
        Ok(false) => Outcome::Deny,
        Err(ExpansionError::UndefinedRelation {
            object_type,
            relation,
        }) => Outcome::Refuse(Refusal::UnknownRelation {
            object_type,
            relation,
        }),
        Err(ExpansionError::NonStratified {
            object_type,
            relation,
        }) => Outcome::Refuse(Refusal::NonStratified {
            object_type,
            relation,
        }),
        Err(ExpansionError::NegatedCycleInData {
            object_type,
            relation,
        }) => Outcome::Refuse(Refusal::NegatedCycleInData {
            object_type,
            relation,
        }),
        Err(ExpansionError::CandidateBudgetExceeded { limit }) => {
            Outcome::Refuse(Refusal::CandidateBudgetExceeded(limit))
        }
        Err(ExpansionError::DepthExceeded { limit }) => {
            Outcome::Refuse(Refusal::DepthExceeded(limit))
        }
        Err(ExpansionError::TupleBudgetExceeded { limit }) => {
            Outcome::Refuse(Refusal::TupleBudgetExceeded(limit))
        }
        Err(ExpansionError::PageBudgetExceeded { limit }) => {
            Outcome::Refuse(Refusal::PageBudgetExceeded(limit))
        }
        Err(ExpansionError::Store(RebacTupleStoreError::Backend(detail)))
            if detail == CANCELLED_DETAIL =>
        {
            Outcome::Refuse(Refusal::Cancelled)
        }
        Err(ExpansionError::Store(RebacTupleStoreError::SnapshotScopeMismatch { .. })) => {
            Outcome::Refuse(Refusal::TenantScope)
        }
        Err(ExpansionError::Store(error)) => Outcome::Refuse(Refusal::Store(error.to_string())),
    }
}
