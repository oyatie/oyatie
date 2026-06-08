#![forbid(unsafe_code)]
//! Common IDs, tenant scopes, data classes, request context, audit event shape,
//! and shared error primitives for the Oya Office public-SaaS office suite.
//!
//! This crate stays provider-neutral and framework-free. It is the innermost
//! source-shaped kernel used by tenant, authz, Drive, format, collaboration, and
//! editor domain crates.

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-kernel";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "platform";

/// Source-shaped architectural layer represented by this crate.
pub const ARCHITECTURE_LAYER: &str = "kernel";

/// Reason an identifier failed validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentifierErrorReason {
    /// The identifier was empty after trimming whitespace.
    Empty,
    /// The identifier was longer than the kernel maximum.
    TooLong,
    /// The identifier contained a character outside the stable public-SaaS set.
    InvalidCharacter,
}

/// Validation error for kernel identifier value objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentifierError {
    kind: &'static str,
    reason: IdentifierErrorReason,
}

impl IdentifierError {
    /// Creates a new identifier validation error.
    #[must_use]
    pub const fn new(kind: &'static str, reason: IdentifierErrorReason) -> Self {
        Self { kind, reason }
    }

    /// Returns the identifier kind that failed validation.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        self.kind
    }

    /// Returns the validation failure reason.
    #[must_use]
    pub const fn reason(&self) -> IdentifierErrorReason {
        self.reason
    }
}

impl core::fmt::Display for IdentifierError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "invalid {}: {:?}", self.kind, self.reason)
    }
}

impl std::error::Error for IdentifierError {}

macro_rules! identifier_type {
    ($(#[$meta:meta])* $name:ident, $kind:literal) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            /// Creates a validated identifier value object.
            pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
                validate_identifier($kind, value).map(Self)
            }

            /// Returns the stable string representation.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
    };
}

identifier_type!(
    /// Tenant-scoped account boundary for every request and artifact.
    TenantId,
    "tenant id"
);
identifier_type!(
    /// Principal identifier scoped to an authenticated tenant subject.
    PrincipalId,
    "principal id"
);
identifier_type!(
    /// Oya Drive object identifier for files, folders, and suite documents.
    ObjectId,
    "object id"
);
identifier_type!(
    /// Request or event correlation identifier.
    RequestId,
    "request id"
);
identifier_type!(
    /// Deployment cell identifier used for tenant routing and isolation.
    CellId,
    "cell id"
);

fn validate_identifier(
    kind: &'static str,
    value: impl Into<String>,
) -> Result<String, IdentifierError> {
    const MAX_IDENTIFIER_LEN: usize = 128;

    let value = value.into();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(IdentifierError::new(kind, IdentifierErrorReason::Empty));
    }
    if trimmed.len() > MAX_IDENTIFIER_LEN {
        return Err(IdentifierError::new(kind, IdentifierErrorReason::TooLong));
    }
    if !trimmed
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
    {
        return Err(IdentifierError::new(
            kind,
            IdentifierErrorReason::InvalidCharacter,
        ));
    }
    Ok(trimmed.to_owned())
}

/// Data classification label carried by Drive objects, audit events, and jobs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DataClass {
    /// Public or intentionally shareable content.
    Public,
    /// Internal tenant content with normal access controls.
    Internal,
    /// Sensitive content requiring stricter audit and sharing checks.
    Confidential,
    /// Highly restricted content requiring explicit policy gates.
    Restricted,
}

/// Result of an authorization, quota, or policy-sensitive action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AuditOutcome {
    /// The action was permitted.
    Allowed,
    /// The action was refused.
    Denied,
}

/// Stable audit action vocabulary for early tenant/security contracts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AuditAction {
    /// Authorization decision for a Drive read.
    DriveRead,
    /// Authorization decision for a Drive write.
    DriveWrite,
    /// Authorization decision for Drive sharing.
    DriveShare,
    /// Authorization decision for Drive export/download.
    DriveExport,
    /// Authorization decision for Drive delete/trash/lifecycle action.
    DriveDelete,
    /// Tenant quota evaluation.
    TenantQuotaEvaluate,
    /// Tenant rate-limit evaluation.
    RateLimitEvaluate,
    /// Generic authorization decision.
    AuthorizationDecision,
}

/// Tenant-scoped request metadata propagated through API, worker, and audit paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestContext {
    request_id: RequestId,
    tenant_id: TenantId,
    principal_id: PrincipalId,
    cell_id: CellId,
}

impl RequestContext {
    /// Creates a new tenant-scoped request context.
    #[must_use]
    pub const fn new(
        request_id: RequestId,
        tenant_id: TenantId,
        principal_id: PrincipalId,
        cell_id: CellId,
    ) -> Self {
        Self {
            request_id,
            tenant_id,
            principal_id,
            cell_id,
        }
    }

    /// Returns the request correlation identifier.
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// Returns the tenant boundary for the request.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the principal performing the request.
    #[must_use]
    pub const fn principal_id(&self) -> &PrincipalId {
        &self.principal_id
    }

    /// Returns the serving or home cell for the request.
    #[must_use]
    pub const fn cell_id(&self) -> &CellId {
        &self.cell_id
    }
}

/// Immutable audit event shape shared by tenant, authz, Drive, and future API contracts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditEvent {
    event_id: RequestId,
    request_id: RequestId,
    tenant_id: TenantId,
    actor: PrincipalId,
    action: AuditAction,
    resource: Option<ObjectId>,
    data_class: DataClass,
    outcome: AuditOutcome,
    reason: Option<String>,
}

impl AuditEvent {
    /// Creates an audit event from already validated kernel value objects.
    #[must_use]
    pub fn new(input: AuditEventInput) -> Self {
        Self {
            event_id: input.event_id,
            request_id: input.request_id,
            tenant_id: input.tenant_id,
            actor: input.actor,
            action: input.action,
            resource: input.resource,
            data_class: input.data_class,
            outcome: input.outcome,
            reason: input.reason,
        }
    }

    /// Returns the durable audit event identifier.
    #[must_use]
    pub const fn event_id(&self) -> &RequestId {
        &self.event_id
    }

    /// Returns the originating request identifier.
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// Returns the tenant associated with this audit event.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the actor associated with this audit event.
    #[must_use]
    pub const fn actor(&self) -> &PrincipalId {
        &self.actor
    }

    /// Returns the action being audited.
    #[must_use]
    pub const fn action(&self) -> AuditAction {
        self.action
    }

    /// Returns the optional Drive object or resource identifier.
    #[must_use]
    pub const fn resource(&self) -> Option<&ObjectId> {
        self.resource.as_ref()
    }

    /// Returns the data class attached to the resource.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }

    /// Returns the policy outcome.
    #[must_use]
    pub const fn outcome(&self) -> AuditOutcome {
        self.outcome
    }

    /// Returns the optional denial or policy reason.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}

/// Input object for creating an [`AuditEvent`] without a long positional argument list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditEventInput {
    /// Durable audit event identifier.
    pub event_id: RequestId,
    /// Originating request identifier.
    pub request_id: RequestId,
    /// Tenant associated with the event.
    pub tenant_id: TenantId,
    /// Actor associated with the event.
    pub actor: PrincipalId,
    /// Audited action.
    pub action: AuditAction,
    /// Optional resource identifier.
    pub resource: Option<ObjectId>,
    /// Data classification label.
    pub data_class: DataClass,
    /// Allowed/denied outcome.
    pub outcome: AuditOutcome,
    /// Optional policy reason.
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        ARCHITECTURE_LAYER, CRATE_NAME, CellId, IdentifierErrorReason, TenantId, VERTICAL_SLICE,
    };

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!ARCHITECTURE_LAYER.is_empty());
        assert!(!CRATE_NAME.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    #[test]
    fn identifiers_reject_blank_or_invalid_values() {
        let empty = TenantId::new("   ").expect_err("blank tenant id must fail");
        let invalid = CellId::new("iad/1").expect_err("slash is not a stable id character");

        assert_eq!(empty.reason(), IdentifierErrorReason::Empty);
        assert_eq!(invalid.reason(), IdentifierErrorReason::InvalidCharacter);
    }
}
