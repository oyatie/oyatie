//! Reserved-namespace guard usecase — refuses tenant/sub-scope names colliding
//! with the platform-owner binding from `/specs/platform-owner-binding.json`.
//!
//! Wave 15-IMPL-truth-up scaffold; full implementation lands in IP-017 execution.
//! Per `feedback_oyatie_is_a_tenant_doctrine` (ADR-0242), oyatie is a reserved
//! namespace tenant — no carve-outs; Unicode confusable handling required.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(dead_code)]

/// Decision returned by the guard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamespaceDecision {
    Allow,
    DenyReserved,
    DenyConfusable,
    DenyMalformed,
}

/// Inputs evaluated by the guard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceCandidate {
    pub candidate: String,
    pub principal: String,
    pub action: NamespaceAction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamespaceAction {
    CreateTenant,
    RenameTenant,
    CreateSubScope,
}

/// Sealed port reading the platform-owner reservation list.
pub trait ReservedNamespaceSource {
    fn reserved(&self) -> Vec<String>;
}

/// Sealed port evaluating Cedar action-authorization.
pub trait NamespaceActionAuthorizer {
    fn authorize(&self, input: &NamespaceCandidate) -> Result<bool, NamespaceUsecaseError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamespaceUsecaseError {
    SourceUnavailable,
    CedarEvaluationFailed,
}

/// Stub entry-point; full implementation in IP-017 execution.
pub fn evaluate<S: ReservedNamespaceSource, A: NamespaceActionAuthorizer>(
    _source: &S,
    _authorizer: &A,
    _candidate: &NamespaceCandidate,
) -> Result<NamespaceDecision, NamespaceUsecaseError> {
    Ok(NamespaceDecision::Allow)
}
