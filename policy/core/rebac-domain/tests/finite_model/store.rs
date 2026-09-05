use std::cell::Cell;

use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacReadSnapshot, RebacRelation, RebacSubjectRef, RebacTenantScope,
    RebacTuple, RebacTuplePage, RebacTupleQuery, RebacTupleStore, RebacTupleStoreError,
    ResolvedRebacSnapshot, SnapshotToken, UsersetRewrite, Zookie,
};
use policy_rebac_domain::{NamespaceConfig, ValidatedNamespace};

use super::{Expr, Model, Object, Subject, Tuple};

const UNAVAILABLE: &str = "finite conformance store unavailable";

pub struct Rendered {
    pub store: PagedStore,
    pub namespace: ValidatedNamespace,
    pub tenant: RebacTenantScope,
    pub snapshot: SnapshotToken,
}

pub struct PagedStore {
    tuples: Vec<RebacTuple>,
    resolved: ResolvedRebacSnapshot,
    page_size: usize,
    reads: Cell<usize>,
    continuation_reads: Cell<usize>,
    unavailable: bool,
}

impl PagedStore {
    pub fn reads(&self) -> usize {
        self.reads.get()
    }

    pub fn continuation_reads(&self) -> usize {
        self.continuation_reads.get()
    }
}

pub fn render(model: &Model, page_size: usize) -> Rendered {
    assert!(page_size > 0);
    let tenant = RebacTenantScope::new("ten_finite_membership").expect("tenant is valid");
    let snapshot = SnapshotToken::new("finite_snapshot").expect("snapshot is valid");
    let resolved = ResolvedRebacSnapshot::new(tenant.clone(), snapshot.clone());
    let namespace = model
        .relations
        .iter()
        .fold(NamespaceConfig::new(), |namespace, relation| {
            namespace.define(
                relation.object_type,
                &native_relation(relation.name),
                native_expr(&relation.expr),
            )
        })
        .validated()
        .expect("finite model is stratified");
    Rendered {
        store: PagedStore {
            tuples: model
                .tuples
                .iter()
                .map(|tuple| native_tuple(&tenant, tuple))
                .collect(),
            resolved,
            page_size,
            reads: Cell::new(0),
            continuation_reads: Cell::new(0),
            unavailable: false,
        },
        namespace,
        tenant,
        snapshot,
    }
}

pub fn render_unavailable(model: &Model) -> Rendered {
    let mut rendered = render(model, 32);
    rendered.store.unavailable = true;
    rendered
}

fn native_object(object: &Object) -> RebacObjectRef {
    RebacObjectRef::new(object.object_type, object.object_id).expect("model object is valid")
}

fn native_relation(relation: &str) -> RebacRelation {
    RebacRelation::new(relation).expect("model relation is valid")
}

fn native_subject(subject: &Subject) -> RebacSubjectRef {
    match subject {
        Subject::Concrete(object) => RebacSubjectRef::object(native_object(object)),
        Subject::Userset { object, relation } => {
            RebacSubjectRef::userset(native_object(object), native_relation(relation))
        }
    }
}

fn native_tuple(tenant: &RebacTenantScope, tuple: &Tuple) -> RebacTuple {
    RebacTuple::new(
        tenant.clone(),
        native_object(&tuple.object),
        native_relation(tuple.relation),
        native_subject(&tuple.subject),
    )
}

fn native_expr(expr: &Expr) -> UsersetRewrite {
    match expr {
        Expr::This => UsersetRewrite::this(),
        Expr::Computed(relation) => UsersetRewrite::computed_userset(native_relation(relation)),
        Expr::TupleToUserset { tupleset, computed } => {
            UsersetRewrite::tuple_to_userset(native_relation(tupleset), native_relation(computed))
        }
        Expr::Union(children) => UsersetRewrite::union(children.iter().map(native_expr).collect())
            .expect("finite union has children"),
        Expr::Intersection(children) => {
            UsersetRewrite::intersection(children.iter().map(native_expr).collect())
                .expect("finite intersection has children")
        }
        Expr::Difference(base, subtract) => {
            UsersetRewrite::difference(native_expr(base), native_expr(subtract))
        }
    }
}

impl RebacTupleStore for PagedStore {
    fn write_tuple(&mut self, _tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError> {
        Err(RebacTupleStoreError::Backend(
            "finite conformance store is immutable".to_owned(),
        ))
    }

    fn resolve_snapshot(
        &self,
        _tenant: &RebacTenantScope,
        _requested: RebacReadSnapshot,
    ) -> Result<ResolvedRebacSnapshot, RebacTupleStoreError> {
        if self.unavailable {
            return Err(RebacTupleStoreError::Backend(UNAVAILABLE.to_owned()));
        }
        Ok(self.resolved.clone())
    }

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: &ResolvedRebacSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError> {
        if self.unavailable {
            return Err(RebacTupleStoreError::Backend(UNAVAILABLE.to_owned()));
        }
        if snapshot != &self.resolved {
            return Err(RebacTupleStoreError::InconsistentSnapshot {
                requested: self.resolved.clone(),
                served: snapshot.clone(),
            });
        }
        self.reads.set(self.reads.get() + 1);
        if query.page_token.is_some() {
            self.continuation_reads
                .set(self.continuation_reads.get() + 1);
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
        let start = query.page_token.as_deref().map_or(Ok(0), |token| {
            token.parse::<usize>().map_err(|_| {
                RebacTupleStoreError::Backend(format!("invalid finite page token {token:?}"))
            })
        })?;
        let end = start.saturating_add(self.page_size).min(matched.len());
        let tuples = matched.get(start..end).unwrap_or_default().to_vec();
        Ok(RebacTuplePage {
            tuples,
            snapshot: self.resolved.clone(),
            next_page_token: (end < matched.len()).then(|| end.to_string()),
        })
    }
}
