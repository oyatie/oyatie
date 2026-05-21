//! Sub-scope registry kernel — pure hierarchy types + validators + ports.
//!
//! Wave 15-IMPL-truth-up scaffold; full implementation lands in IP-016 execution.
//! Enforces cycle refusal, max depth, tenant-boundary preservation, immutable root,
//! and namespace normalization. Persistence ports live here; adapters in IP-023.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(dead_code)]

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SubScopeId(pub String);

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
    pub id: SubScopeId,
    pub tenant_id: String,
    pub kind: SubScopeKind,
    pub parent: Option<SubScopeId>,
    pub path: SubScopePath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubScopePath(pub Vec<String>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyEdge {
    pub parent: SubScopeId,
    pub child: SubScopeId,
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
