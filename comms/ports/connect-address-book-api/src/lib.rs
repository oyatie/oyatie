//! Connect address-book port (capability `comms`, face `ports`).
//!
//! The inbound contact-management API surface for the connect address-book domain. This crate
//! is CLOUD-AGNOSTIC: it owns the typed request/receipt DTOs, the fail-closed authorization
//! context, and the OUTBOUND port traits (`ContactStore`, `DirectoryGrantStore`) that the
//! deferred cloud/persistence/identity adapters implement. No persistence, identity-provider, or
//! transport dependency lives here — clean-architecture ports point INWARD to the domain and are
//! implemented OUTWARD by adapters, so the trait shapes model the owned-stack destination
//! (review litmus: would this trait change at cutover? — no, it speaks tenant/contact/consent,
//! never a concrete store).
//!
//! Fail-closed authz (founder directive, new-HTTP-surfaces doctrine): every contact-management
//! operation flows through an [`AuthorizedContactContext`] that REFUSES to proceed without a
//! verified principal, a tenant-scoped subject, a PDP policy-decision reference, an idempotency
//! key, and an audit-correlation id. The usecase calls `validate()` first; a missing or empty
//! authorization field is a hard `Unauthorized`/`Missing*` error, never a silent allow.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

/// Errors the port surface can raise. Authorization failures are first-class and fail-closed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContactApiError {
    /// A required authorization/identity field was absent or empty.
    MissingPrincipal,
    MissingTenantScope,
    MissingPolicyDecision,
    MissingIdempotencyKey,
    MissingAuditCorrelation,
    /// The request subject does not match the verified principal (cross-principal write).
    PrincipalMismatch,
    /// The request tenant does not match the authorized context tenant (cross-tenant write).
    TenantMismatch,
    /// A required request field was absent or malformed.
    InvalidRequest,
}

/// A fully-authorized contact-management context. Constructing one is not enough — callers MUST
/// invoke [`AuthorizedContactContext::validate`] before acting, and the usecase does so on every
/// path. The fields encode the verified principal + tenant scope + PDP decision + idempotency +
/// audit correlation that a fail-closed control plane requires.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedContactContext {
    /// The tenant the authorization decision was scoped to (e.g. `tenant:acme`).
    pub tenant_ref: String,
    /// The verified caller principal (e.g. `user:ada@example.com`).
    pub principal_ref: String,
    /// Per-request idempotency key (replay-safe writes).
    pub idempotency_key: String,
    /// The PDP (cloud-iam) policy-decision reference that authorized this operation.
    pub policy_decision_ref: String,
    /// The audit-correlation id binding this operation to the tamper-evident log.
    pub audit_correlation_id: String,
}

impl AuthorizedContactContext {
    /// Fail-closed gate: returns `Ok(())` only when EVERY authorization field is present and the
    /// tenant scope is well-formed. Any missing field is a hard error — there is no allow path
    /// that skips this check.
    pub fn validate(&self) -> Result<(), ContactApiError> {
        if self.principal_ref.trim().is_empty() {
            return Err(ContactApiError::MissingPrincipal);
        }
        if self.tenant_ref.trim().is_empty() || !self.tenant_ref.starts_with("tenant:") {
            return Err(ContactApiError::MissingTenantScope);
        }
        if self.idempotency_key.trim().is_empty() {
            return Err(ContactApiError::MissingIdempotencyKey);
        }
        if self.policy_decision_ref.trim().is_empty() {
            return Err(ContactApiError::MissingPolicyDecision);
        }
        if self.audit_correlation_id.trim().is_empty() {
            return Err(ContactApiError::MissingAuditCorrelation);
        }
        Ok(())
    }
}

/// An email entry on a contact-create request (mirrors the domain `ContactEmail` shape without
/// the data-class wrappers — the domain re-classifies on construction).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactEmailInput {
    pub email: String,
    pub label: String,
    pub primary: bool,
}

/// A phone entry on a contact-create request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactPhoneInput {
    pub phone: String,
    pub label: String,
    pub primary: bool,
}

/// Visibility the API caller requests for a contact card.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactVisibilityRequest {
    PrivateUser,
    TenantDirectory,
    CrossTenantDirectory,
}

/// Scope of the target address book.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddressBookScopeRequest {
    UserContacts,
    TenantDirectory,
}

/// Request to provision an address book.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateAddressBookRequest {
    pub book_id: String,
    pub tenant_id: String,
    pub region: String,
    pub cell_id: String,
    pub owner_ref: String,
    pub scope: AddressBookScopeRequest,
    pub created_at_epoch_seconds: u64,
    pub updated_at_epoch_seconds: u64,
}

/// Request to add a contact card to a book.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddContactRequest {
    pub contact_id: String,
    pub book_id: String,
    pub tenant_id: String,
    pub owner_ref: String,
    pub display_name: String,
    pub emails: Vec<ContactEmailInput>,
    pub phones: Vec<ContactPhoneInput>,
    pub organization: Option<String>,
    pub visibility: ContactVisibilityRequest,
    pub consent_receipt_id: Option<String>,
    pub updated_at_epoch_seconds: u64,
}

/// Receipt returned after a contact-management write. Carries the audit/idempotency/PDP
/// references back to the caller so the operation is traceable end-to-end.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactWriteReceipt {
    pub contact_id: String,
    pub book_id: String,
    pub tenant_id: String,
    pub event_type: &'static str,
    pub audit_correlation_id: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
}

/// OUTBOUND persistence port for contact cards. A deferred cloud/persistence adapter (e.g. a
/// Postgres RLS-enforced store) implements this OUTSIDE the core; the usecase depends only on the
/// trait. The trait speaks tenant/book/contact, never a concrete backend — it would not change at
/// owned-stack cutover.
pub trait ContactStore {
    type Error;

    /// Persist a validated contact card. The implementation is responsible for tenant isolation
    /// (fail-closed on the isolation invariant per the durable-store doctrine); the usecase has
    /// already enforced authz + domain invariants before calling.
    fn put_contact(
        &self,
        tenant_id: &str,
        book_id: &str,
        contact_id: &str,
    ) -> Result<(), Self::Error>;
}

/// OUTBOUND persistence port for cross-tenant directory-search grants. Deferred adapter.
pub trait DirectoryGrantStore {
    type Error;

    fn put_grant(
        &self,
        grant_id: &str,
        source_tenant: &str,
        target_tenant: &str,
    ) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> AuthorizedContactContext {
        AuthorizedContactContext {
            tenant_ref: "tenant:acme".into(),
            principal_ref: "user:ada@example.com".into(),
            idempotency_key: "idem-1".into(),
            policy_decision_ref: "pdp:allow-1".into(),
            audit_correlation_id: "audit-1".into(),
        }
    }

    #[test]
    fn authorized_context_accepts_complete_authz() {
        assert_eq!(ctx().validate(), Ok(()));
    }

    #[test]
    fn authorized_context_fails_closed_on_missing_principal() {
        let mut c = ctx();
        c.principal_ref = "   ".into();
        assert_eq!(c.validate(), Err(ContactApiError::MissingPrincipal));
    }

    #[test]
    fn authorized_context_fails_closed_on_non_tenant_scope() {
        let mut c = ctx();
        c.tenant_ref = "person:ada".into();
        assert_eq!(c.validate(), Err(ContactApiError::MissingTenantScope));

        c.tenant_ref = String::new();
        assert_eq!(c.validate(), Err(ContactApiError::MissingTenantScope));
    }

    #[test]
    fn authorized_context_fails_closed_on_missing_pdp_decision() {
        let mut c = ctx();
        c.policy_decision_ref = String::new();
        assert_eq!(c.validate(), Err(ContactApiError::MissingPolicyDecision));
    }

    #[test]
    fn authorized_context_fails_closed_on_missing_idempotency_and_audit() {
        let mut c = ctx();
        c.idempotency_key = String::new();
        assert_eq!(c.validate(), Err(ContactApiError::MissingIdempotencyKey));

        let mut c = ctx();
        c.audit_correlation_id = String::new();
        assert_eq!(c.validate(), Err(ContactApiError::MissingAuditCorrelation));
    }
}
