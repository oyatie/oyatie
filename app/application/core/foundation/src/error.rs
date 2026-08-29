//! The foundation error surface.

use crate::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FoundationError {
    TenantAlreadyExists,
    TenantNotFound,
    UserNotFound,
    CapabilityNotFound,
    CapabilityAlreadyExists,
    CapabilityNotLicensed,
    CapabilityEvalGateNotReady,
    McpAccessDenied,
    McpRateLimited,
    CellBindingImmutable,
    RegionalPackAlreadyExists,
    PolicyVersionAlreadyExists,
    DataUseNotAllowed,
    OutboxRecordNotFound,
    TokenTtlTooLong,
    InvalidInput,
    AutonomyCeilingExceeded,
    CapabilityInvocationUnauthorized,
    CostBudgetNotConfigured,
    CostBudgetExceeded,
    /// ADR-0083 amendment 2026-05-15: `AuditChain::append_classifications`
    /// returns `Result<&AuditEvent, AuditChainError>` — Tier 1 fallible.
    /// The variants of `AuditChainError` (`EmptyTenantId`,
    /// `TenantShardMismatch`, etc.) propagate to this app boundary so callers
    /// can pattern-match the failure mode rather than seeing a silent panic.
    AuditChainAppendFailed(audit_chain_domain::AuditChainError),
}

impl From<audit_chain_domain::AuditChainError> for FoundationError {
    fn from(error: audit_chain_domain::AuditChainError) -> Self {
        Self::AuditChainAppendFailed(error)
    }
}

pub(crate) struct DeniedInvocationRecord<'a> {
    pub(crate) request: &'a CapabilityInvocationRequest,
    pub(crate) tenant: &'a Tenant,
    pub(crate) capability: &'a Capability,
    pub(crate) disposition: RunDisposition,
    pub(crate) evidence_kind: EvidenceKind,
    pub(crate) reason: &'static str,
    pub(crate) audit_event_hash: String,
    pub(crate) extra_fields: BTreeMap<String, String>,
}
