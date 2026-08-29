//! The tuple-store port: tenant-scoped queries, paged reads, and the
//! fail-closed error surface adapters must satisfy.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::RebacTupleValidationError;
use super::token::{RebacReadSnapshot, SnapshotToken, Zookie};
use super::tuple::{RebacObjectRef, RebacRelation, RebacSubjectRef, RebacTenantScope, RebacTuple};

/// Tenant-scoped tuple-store query. Any `None` field is a wildcard within the tenant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RebacTupleQuery {
    pub tenant: RebacTenantScope,         // data_class: TENANT_SCOPED
    pub object: Option<RebacObjectRef>,   // data_class: TENANT_SCOPED
    pub relation: Option<RebacRelation>,  // data_class: INTERNAL_ONLY
    pub subject: Option<RebacSubjectRef>, // data_class: TENANT_SCOPED
}

impl RebacTupleQuery {
    #[must_use]
    pub fn new(tenant: RebacTenantScope) -> Self {
        Self {
            tenant,
            object: None,
            relation: None,
            subject: None,
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
        }
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
    pub snapshot: SnapshotToken,         // data_class: INTERNAL_ONLY
    pub next_page_token: Option<String>, // data_class: INTERNAL_ONLY
}

/// Tuple-store port: adapters own persistence; the domain crate owns the contract.
pub trait RebacTupleStore {
    fn write_tuple(&mut self, tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError>;

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: RebacReadSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError>;
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
            Self::Backend(detail) => write!(f, "ReBAC tuple-store backend error: {detail}"),
        }
    }
}

impl std::error::Error for RebacTupleStoreError {}
