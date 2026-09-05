//! The tuple-store port: tenant-scoped queries, paged reads, and the
//! fail-closed error surface adapters must satisfy.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::RebacTupleValidationError;
use super::token::{RebacReadSnapshot, ResolvedRebacSnapshot, SnapshotToken, Zookie};
use super::tuple::{RebacObjectRef, RebacRelation, RebacSubjectRef, RebacTenantScope, RebacTuple};

/// Tenant-scoped tuple-store query. Any `None` field is a wildcard within the tenant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RebacTupleQuery {
    pub tenant: RebacTenantScope,         // data_class: TENANT_SCOPED
    pub object: Option<RebacObjectRef>,   // data_class: TENANT_SCOPED
    pub relation: Option<RebacRelation>,  // data_class: INTERNAL_ONLY
    pub subject: Option<RebacSubjectRef>, // data_class: TENANT_SCOPED
    /// Continuation from a prior [`RebacTuplePage::next_page_token`]. `None`
    /// starts at the first page. A reader that ignores this reads only the
    /// first page of a tupleset, which for an authorization walk is not a
    /// truncated answer but a wrong one.
    #[serde(default)]
    pub page_token: Option<String>, // data_class: INTERNAL_ONLY
}

impl RebacTupleQuery {
    #[must_use]
    pub fn new(tenant: RebacTenantScope) -> Self {
        Self {
            tenant,
            object: None,
            relation: None,
            subject: None,
            page_token: None,
        }
    }

    #[must_use]
    pub fn object_relation(
        tenant: RebacTenantScope,
        object: RebacObjectRef,
        relation: RebacRelation,
    ) -> Self {
        Self {
            tenant,
            object: Some(object),
            relation: Some(relation),
            subject: None,
            page_token: None,
        }
    }

    /// The same query, resumed at `page_token`.
    #[must_use]
    pub fn at_page(mut self, page_token: Option<String>) -> Self {
        self.page_token = page_token;
        self
    }

    #[must_use]
    pub fn matches(&self, tuple: &RebacTuple) -> bool {
        self.tenant == tuple.tenant
            && self
                .object
                .as_ref()
                .is_none_or(|object| object == &tuple.object)
            && self
                .relation
                .as_ref()
                .is_none_or(|relation| relation == &tuple.relation)
            && self
                .subject
                .as_ref()
                .is_none_or(|subject| subject == &tuple.subject)
    }
}

/// One page returned by the tuple-store port.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RebacTuplePage {
    pub tuples: Vec<RebacTuple>,         // data_class: TENANT_SCOPED
    pub snapshot: ResolvedRebacSnapshot, // data_class: INTERNAL_ONLY
    pub next_page_token: Option<String>, // data_class: INTERNAL_ONLY
}

/// Tuple-store port: adapters own persistence; the domain crate owns the contract.
pub trait RebacTupleStore {
    fn write_tuple(&mut self, tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError>;

    /// Resolve a tenant-scoped request into one immutable store snapshot.
    fn resolve_snapshot(
        &self,
        tenant: &RebacTenantScope,
        requested: RebacReadSnapshot,
    ) -> Result<ResolvedRebacSnapshot, RebacTupleStoreError>;

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: &ResolvedRebacSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError>;
}

/// Resolve one snapshot request and reject adapter substitution.
///
/// `Latest` deliberately accepts the store-issued token for the current
/// tenant. An explicit `At` request is already store-issued and must resolve
/// to that exact tenant and token; silently advancing it would authorize
/// against a different graph than the caller requested.
pub fn resolve_snapshot<S: RebacTupleStore + ?Sized>(
    store: &S,
    tenant: &RebacTenantScope,
    requested: RebacReadSnapshot,
) -> Result<ResolvedRebacSnapshot, RebacTupleStoreError> {
    let requested_token = match &requested {
        RebacReadSnapshot::Latest => None,
        RebacReadSnapshot::At { snapshot } => Some(snapshot.clone()),
    };
    let resolved = store.resolve_snapshot(tenant, requested)?;
    if resolved.tenant() != tenant {
        return Err(RebacTupleStoreError::SnapshotScopeMismatch {
            query_tenant: tenant.clone(),
            snapshot_tenant: resolved.tenant().clone(),
        });
    }
    if let Some(snapshot) = requested_token {
        let expected = ResolvedRebacSnapshot::new(tenant.clone(), snapshot);
        if resolved != expected {
            return Err(RebacTupleStoreError::InconsistentSnapshot {
                requested: expected,
                served: resolved,
            });
        }
    }
    Ok(resolved)
}

/// Fail-closed tuple-store port errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebacTupleStoreError {
    InvalidTuple(RebacTupleValidationError),
    InvalidZookie(RebacTupleValidationError),
    StaleSnapshot {
        requested: SnapshotToken,
        current: SnapshotToken,
    },
    SnapshotScopeMismatch {
        query_tenant: RebacTenantScope,
        snapshot_tenant: RebacTenantScope,
    },
    InconsistentSnapshot {
        requested: ResolvedRebacSnapshot,
        served: ResolvedRebacSnapshot,
    },
    TupleOutsideQuery {
        query: Box<RebacTupleQuery>,
        tuple: Box<RebacTuple>,
    },
    Backend(String),
}

impl fmt::Display for RebacTupleStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTuple(err) => write!(f, "invalid ReBAC tuple: {err}"),
            Self::InvalidZookie(err) => write!(f, "invalid ReBAC zookie: {err}"),
            Self::StaleSnapshot { requested, current } => write!(
                f,
                "stale ReBAC snapshot: requested {} but current is {}",
                requested.as_str(),
                current.as_str()
            ),
            Self::SnapshotScopeMismatch {
                query_tenant,
                snapshot_tenant,
            } => write!(
                f,
                "ReBAC snapshot tenant {} does not match query tenant {}",
                snapshot_tenant.as_str(),
                query_tenant.as_str()
            ),
            Self::InconsistentSnapshot { requested, served } => write!(
                f,
                "ReBAC tuple store served tenant {} snapshot {} for requested tenant {} snapshot {}",
                served.tenant().as_str(),
                served.as_str(),
                requested.tenant().as_str(),
                requested.as_str()
            ),
            Self::TupleOutsideQuery { query, tuple } => write!(
                f,
                "ReBAC tuple store served tenant {} tuple {} outside tenant {} query",
                tuple.tenant().as_str(),
                tuple.to_canonical_string(),
                query.tenant.as_str()
            ),
            Self::Backend(detail) => write!(f, "ReBAC tuple-store backend error: {detail}"),
        }
    }
}

impl std::error::Error for RebacTupleStoreError {}
