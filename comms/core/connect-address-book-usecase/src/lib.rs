//! Connect address-book contact-management usecase (capability `comms`, face `core`).
//!
//! Application layer wiring the pure `connect-address-book-domain` kernel through the
//! `connect-address-book-api` port. It is CLOUD-AGNOSTIC: it depends only on the domain + the port
//! (no persistence, identity provider, or transport). Persistence/identity/cloud adapters are
//! DEFERRED behind the port's `ContactStore`/`DirectoryGrantStore` traits (clean architecture —
//! the usecase orchestrates; adapters are injected later).
//!
//! Every entry point is FAIL-CLOSED: it calls [`AuthorizedContactContext::validate`] FIRST and
//! enforces principal/tenant binding before touching the domain, so a missing authorization field
//! or a cross-principal/cross-tenant write is rejected before any domain construction.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use connect_address_book_api::{
    AddContactRequest, AddressBookScopeRequest, AuthorizedContactContext, ContactApiError,
    ContactEmailInput, ContactPhoneInput, ContactVisibilityRequest, ContactWriteReceipt,
    CreateAddressBookRequest,
};
use connect_address_book_domain::{
    AddressBook, AddressBookCreate, AddressBookError, AddressBookScope, ContactCard,
    ContactCardCreate, ContactEmail, ContactPhone, ContactVisibility,
};

/// Errors the usecase can raise: an authorization/port failure or a domain-invariant failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContactUsecaseError {
    Api(ContactApiError),
    Domain(AddressBookError),
}

/// Provision an address book under the authorized tenant. Fail-closed: validates authz, then binds
/// the book's tenant + owner to the verified context before delegating to the domain constructor.
pub fn create_address_book(
    ctx: &AuthorizedContactContext,
    req: CreateAddressBookRequest,
) -> Result<AddressBook, ContactUsecaseError> {
    ctx.validate().map_err(ContactUsecaseError::Api)?;
    require_tenant_match(ctx, &req.tenant_id)?;
    require_principal_owner(ctx, &req.owner_ref)?;
    AddressBook::new(AddressBookCreate {
        book_id: req.book_id,
        tenant_id: req.tenant_id,
        region: req.region,
        cell_id: req.cell_id,
        owner_ref: req.owner_ref,
        scope: map_scope(req.scope),
        data_class: None,
        created_at_epoch_seconds: req.created_at_epoch_seconds,
        updated_at_epoch_seconds: req.updated_at_epoch_seconds,
    })
    .map_err(ContactUsecaseError::Domain)
}

/// Add a contact card to an existing book. Fail-closed: validates authz + principal/tenant binding,
/// constructs the validated domain card (which enforces email/phone/visibility/consent invariants),
/// and returns a traceable receipt. Persistence is DEFERRED — a caller wiring a `ContactStore`
/// adapter persists the returned card; the usecase itself stays cloud-agnostic.
pub fn add_contact(
    ctx: &AuthorizedContactContext,
    book: &AddressBook,
    req: AddContactRequest,
) -> Result<(ContactCard, ContactWriteReceipt), ContactUsecaseError> {
    ctx.validate().map_err(ContactUsecaseError::Api)?;
    require_tenant_match(ctx, &req.tenant_id)?;
    require_principal_owner(ctx, &req.owner_ref)?;

    let emails = map_emails(req.emails)?;
    let phones = map_phones(req.phones)?;

    let card = ContactCard::new(
        ContactCardCreate {
            contact_id: req.contact_id,
            book_id: req.book_id,
            tenant_id: req.tenant_id,
            owner_ref: req.owner_ref,
            display_name: req.display_name,
            emails,
            phones,
            organization: req.organization,
            visibility: map_visibility(req.visibility),
            consent_receipt_id: req.consent_receipt_id,
            data_class: None,
            updated_at_epoch_seconds: req.updated_at_epoch_seconds,
        },
        book,
    )
    .map_err(ContactUsecaseError::Domain)?;

    let receipt = ContactWriteReceipt {
        contact_id: card.contact_id.value.clone(),
        book_id: card.book_id.value.clone(),
        tenant_id: card.tenant_id.value.clone(),
        event_type: "connect.address_book.contact.added",
        audit_correlation_id: ctx.audit_correlation_id.clone(),
        idempotency_key: ctx.idempotency_key.clone(),
        policy_decision_ref: ctx.policy_decision_ref.clone(),
    };
    Ok((card, receipt))
}

fn require_tenant_match(
    ctx: &AuthorizedContactContext,
    tenant_id: &str,
) -> Result<(), ContactUsecaseError> {
    // The authorized context carries a `tenant:<id>` scope; the request carries a bare tenant id.
    // Bind them so a request can never act on a tenant the PDP did not authorize.
    let scoped = ctx
        .tenant_ref
        .strip_prefix("tenant:")
        .unwrap_or(ctx.tenant_ref.as_str());
    if tenant_id.trim().is_empty() {
        return Err(ContactUsecaseError::Api(ContactApiError::InvalidRequest));
    }
    if scoped != tenant_id {
        return Err(ContactUsecaseError::Api(ContactApiError::TenantMismatch));
    }
    Ok(())
}

fn require_principal_owner(
    ctx: &AuthorizedContactContext,
    owner_ref: &str,
) -> Result<(), ContactUsecaseError> {
    if owner_ref.trim().is_empty() {
        return Err(ContactUsecaseError::Api(ContactApiError::InvalidRequest));
    }
    if owner_ref != ctx.principal_ref {
        return Err(ContactUsecaseError::Api(ContactApiError::PrincipalMismatch));
    }
    Ok(())
}

fn map_emails(inputs: Vec<ContactEmailInput>) -> Result<Vec<ContactEmail>, ContactUsecaseError> {
    inputs
        .into_iter()
        .map(|e| {
            ContactEmail::new(e.email, e.label, e.primary).map_err(ContactUsecaseError::Domain)
        })
        .collect()
}

fn map_phones(inputs: Vec<ContactPhoneInput>) -> Result<Vec<ContactPhone>, ContactUsecaseError> {
    inputs
        .into_iter()
        .map(|p| {
            ContactPhone::new(p.phone, p.label, p.primary).map_err(ContactUsecaseError::Domain)
        })
        .collect()
}

fn map_scope(scope: AddressBookScopeRequest) -> AddressBookScope {
    match scope {
        AddressBookScopeRequest::UserContacts => AddressBookScope::UserContacts,
        AddressBookScopeRequest::TenantDirectory => AddressBookScope::TenantDirectory,
    }
}

fn map_visibility(visibility: ContactVisibilityRequest) -> ContactVisibility {
    match visibility {
        ContactVisibilityRequest::PrivateUser => ContactVisibility::PrivateUser,
        ContactVisibilityRequest::TenantDirectory => ContactVisibility::TenantDirectory,
        ContactVisibilityRequest::CrossTenantDirectory => ContactVisibility::CrossTenantDirectory,
    }
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

    fn book_req() -> CreateAddressBookRequest {
        CreateAddressBookRequest {
            book_id: "book-1".into(),
            tenant_id: "acme".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            owner_ref: "user:ada@example.com".into(),
            scope: AddressBookScopeRequest::UserContacts,
            created_at_epoch_seconds: 1_700_000_000,
            updated_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn contact_req() -> AddContactRequest {
        AddContactRequest {
            contact_id: "contact-1".into(),
            book_id: "book-1".into(),
            tenant_id: "acme".into(),
            owner_ref: "user:ada@example.com".into(),
            display_name: "Grace Hopper".into(),
            emails: vec![ContactEmailInput {
                email: "grace@example.com".into(),
                label: "work".into(),
                primary: true,
            }],
            phones: vec![ContactPhoneInput {
                phone: "+1-555-0100".into(),
                label: "mobile".into(),
                primary: true,
            }],
            organization: Some("Navy".into()),
            visibility: ContactVisibilityRequest::PrivateUser,
            consent_receipt_id: None,
            updated_at_epoch_seconds: 1_700_000_020,
        }
    }

    #[test]
    fn create_book_then_add_contact_happy_path() {
        let book = create_address_book(&ctx(), book_req()).unwrap();
        let (card, receipt) = add_contact(&ctx(), &book, contact_req()).unwrap();
        assert_eq!(card.primary_email(), "grace@example.com");
        assert_eq!(receipt.event_type, "connect.address_book.contact.added");
        assert_eq!(receipt.policy_decision_ref, "pdp:allow-1");
        assert_eq!(receipt.tenant_id, "acme");
    }

    #[test]
    fn add_contact_fails_closed_without_authz() {
        let book = create_address_book(&ctx(), book_req()).unwrap();
        let mut unauth = ctx();
        unauth.policy_decision_ref = String::new();
        assert_eq!(
            add_contact(&unauth, &book, contact_req()),
            Err(ContactUsecaseError::Api(
                ContactApiError::MissingPolicyDecision
            ))
        );
    }

    #[test]
    fn add_contact_rejects_cross_tenant_write() {
        let book = create_address_book(&ctx(), book_req()).unwrap();
        let mut req = contact_req();
        req.tenant_id = "evil-corp".into();
        assert_eq!(
            add_contact(&ctx(), &book, req),
            Err(ContactUsecaseError::Api(ContactApiError::TenantMismatch))
        );
    }

    #[test]
    fn add_contact_rejects_cross_principal_write() {
        let book = create_address_book(&ctx(), book_req()).unwrap();
        let mut req = contact_req();
        req.owner_ref = "user:mallory@example.com".into();
        assert_eq!(
            add_contact(&ctx(), &book, req),
            Err(ContactUsecaseError::Api(ContactApiError::PrincipalMismatch))
        );
    }

    #[test]
    fn create_book_rejects_cross_tenant() {
        let mut req = book_req();
        req.tenant_id = "evil-corp".into();
        assert_eq!(
            create_address_book(&ctx(), req),
            Err(ContactUsecaseError::Api(ContactApiError::TenantMismatch))
        );
    }

    #[test]
    fn add_contact_surfaces_domain_invariant_failure() {
        let book = create_address_book(&ctx(), book_req()).unwrap();
        let mut req = contact_req();
        // Two primary emails violate the domain single-primary invariant.
        req.emails = vec![
            ContactEmailInput {
                email: "grace@example.com".into(),
                label: "work".into(),
                primary: true,
            },
            ContactEmailInput {
                email: "grace2@example.com".into(),
                label: "home".into(),
                primary: true,
            },
        ];
        assert_eq!(
            add_contact(&ctx(), &book, req),
            Err(ContactUsecaseError::Domain(
                AddressBookError::MultiplePrimaryEmails
            ))
        );
    }
}
