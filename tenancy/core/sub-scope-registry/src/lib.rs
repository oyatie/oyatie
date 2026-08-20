//! Sub-scope registry kernel — pure hierarchy types + validators + ports.
//!
//! Wave 15-IMPL-truth-up scaffold; full implementation lands in IP-016 execution.
//! Enforces cycle refusal, max depth, tenant-boundary preservation, immutable root,
//! and namespace normalization. Persistence ports live here; adapters in IP-023.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(dead_code)]

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SubScopeId(pub String); // data_class: INTERNAL_ONLY

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubScopeKind {
    BusinessUnit,
    Workspace,
    Engagement,
    Project,
    Investigation,
    Counterparty,
    Custom,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubScope {
    pub id: SubScopeId,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub kind: SubScopeKind,         // data_class: INTERNAL_ONLY
    pub parent: Option<SubScopeId>, // data_class: INTERNAL_ONLY
    pub path: SubScopePath,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubScopePath(pub Vec<String>); // data_class: INTERNAL_ONLY

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyEdge {
    pub parent: SubScopeId, // data_class: INTERNAL_ONLY
    pub child: SubScopeId,  // data_class: INTERNAL_ONLY
}

pub trait SubScopeRegistryPort {
    fn insert(&self, scope: &SubScope) -> Result<(), SubScopeKernelError>;
    fn get(&self, id: &SubScopeId) -> Result<Option<SubScope>, SubScopeKernelError>;
}

pub trait SubScopeHierarchyReadPort {
    fn ancestors(&self, id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError>;
    fn descendants(&self, id: &SubScopeId) -> Result<Vec<SubScopeId>, SubScopeKernelError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubScopeKernelError {
    CycleRefused,
    DepthExceeded,
    TenantBoundaryViolation,
    RootImmutable,
    NamespaceMalformed,
    NotFound,
    PersistenceUnavailable,
}
