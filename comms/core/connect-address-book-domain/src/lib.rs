//! Workspace address-book kernel.
//!
//! Typed kernel records for the W-Workspace-GA Org Address Book adjunct surface
//! named by `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns
//! per-tenant and per-user contact-card validation plus consent-gated
//! cross-tenant directory exposure without owning CardDAV, identity lookup, or
//! search indexing adapters.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const ADDRESS_BOOK_SCHEMA_VERSION: u32 = 1;
const CONTACT_CARD_SCHEMA_VERSION: u32 = 1;
const DIRECTORY_GRANT_SCHEMA_VERSION: u32 = 1;
const CROSS_TENANT_PURPOSE_ID: &str = "cross_tenant_individual";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AddressBookError {
    InvalidBookId,
    InvalidContactId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidOwnerRef,
    InvalidDisplayName,
    InvalidEmail,
    InvalidPhone,
    InvalidLabel,
    EmptyEmailSet,
    DuplicateEmail,
    MissingPrimaryEmail,
    MultiplePrimaryEmails,
    DuplicatePhone,
    MultiplePrimaryPhones,
    InvalidOrganization,
    ContactBookMismatch,
    ContactTenantMismatch,
    ContactOwnerMismatch,
    InvalidVisibilityForBook,
    MissingConsentReceipt,
    UnexpectedConsentReceipt,
    InvalidGrantId,
    InvalidPurposeId,
    InvalidGrantExpiry,
    InvalidTimeOrder,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AddressBookScope {
    UserContacts,
    TenantDirectory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ContactVisibility {
    PrivateUser,
    TenantDirectory,
    CrossTenantDirectory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressBookCreate {
    pub book_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub owner_ref: String,                    // data_class: PII_IDENTIFYING
    pub scope: AddressBookScope,              // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressBook {
    pub book_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<String>,          // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub owner_ref: Classified<String>,       // data_class: PII_IDENTIFYING
    pub scope: Classified<AddressBookScope>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactCardCreate {
    pub contact_id: String,                   // data_class: INTERNAL_ONLY
    pub book_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub owner_ref: String,                    // data_class: PII_IDENTIFYING
    pub display_name: String,                 // data_class: PII_IDENTIFYING
    pub emails: Vec<ContactEmail>,            // data_class: PII_IDENTIFYING
    pub phones: Vec<ContactPhone>,            // data_class: PII_IDENTIFYING
    pub organization: Option<String>,         // data_class: PII_QUASI_IDENTIFIER
    pub visibility: ContactVisibility,        // data_class: INTERNAL_ONLY
    pub consent_receipt_id: Option<String>,   // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactCard {
    pub contact_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub book_id: Classified<String>,      // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub owner_ref: Classified<String>,    // data_class: PII_IDENTIFYING
    pub display_name: Classified<String>, // data_class: PII_IDENTIFYING
    pub emails: Classified<Vec<ContactEmail>>, // data_class: PII_IDENTIFYING
    pub phones: Classified<Vec<ContactPhone>>, // data_class: PII_IDENTIFYING
    pub organization: Classified<Option<String>>, // data_class: PII_QUASI_IDENTIFIER
    pub visibility: Classified<ContactVisibility>, // data_class: INTERNAL_ONLY
    pub consent_receipt_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ContactEmail {
    pub email: Classified<String>, // data_class: PII_IDENTIFYING
    pub label: Classified<String>, // data_class: INTERNAL_ONLY
    pub primary: Classified<bool>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ContactPhone {
    pub phone: Classified<String>, // data_class: PII_IDENTIFYING
    pub label: Classified<String>, // data_class: INTERNAL_ONLY
    pub primary: Classified<bool>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectorySearchGrantCreate {
    pub grant_id: String,              // data_class: INTERNAL_ONLY
    pub source_tenant_id: String,      // data_class: INTERNAL_ONLY
    pub target_tenant_id: String,      // data_class: INTERNAL_ONLY
    pub consent_receipt_id: String,    // data_class: INTERNAL_ONLY
    pub purpose_id: String,            // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectorySearchGrant {
    pub grant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub source_tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub target_tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub consent_receipt_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub purpose_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

pub trait AddressBookReader {
    fn read_contact(
        &self,
        tenant_id: &str,
        book_id: &str,
        contact_id: &str,
    ) -> Result<Option<ContactCard>, AddressBookError>;
}

impl AddressBook {
    pub fn new(input: AddressBookCreate) -> Result<Self, AddressBookError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_address_book_data_class());
        validate_non_empty(&input.book_id, AddressBookError::InvalidBookId)?;
        validate_non_empty(&input.tenant_id, AddressBookError::InvalidTenantId)?;
        validate_non_empty(&input.region, AddressBookError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, AddressBookError::InvalidCellId)?;
        validate_non_empty(&input.owner_ref, AddressBookError::InvalidOwnerRef)?;
        validate_time_order(
            input.created_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;
        Ok(Self {
            book_id: internal(input.book_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            owner_ref: Classified::new(input.owner_ref, contact_data_class()),
            scope: internal(input.scope),
            data_class: internal(data_class),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(ADDRESS_BOOK_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl ContactCard {
    pub fn new(input: ContactCardCreate, book: &AddressBook) -> Result<Self, AddressBookError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_address_book_data_class());
        validate_non_empty(&input.contact_id, AddressBookError::InvalidContactId)?;
        validate_contact_book_binding(&input, book)?;
        validate_text(&input.display_name, AddressBookError::InvalidDisplayName)?;
        validate_emails(&input.emails)?;
        validate_phones(&input.phones)?;
        validate_optional_text(
            input.organization.as_deref(),
            AddressBookError::InvalidOrganization,
        )?;
        validate_visibility(
            input.visibility,
            input.consent_receipt_id.as_deref(),
            book.scope.value,
        )?;
        Ok(Self {
            contact_id: internal(input.contact_id),
            book_id: internal(input.book_id),
            tenant_id: internal(input.tenant_id),
            owner_ref: Classified::new(input.owner_ref, contact_data_class()),
            display_name: Classified::new(input.display_name, contact_data_class()),
            emails: Classified::new(input.emails, contact_data_class()),
            phones: Classified::new(input.phones, contact_data_class()),
            organization: Classified::new(input.organization, contact_metadata_data_class()),
            visibility: internal(input.visibility),
            consent_receipt_id: internal(input.consent_receipt_id),
            data_class: internal(data_class),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(CONTACT_CARD_SCHEMA_VERSION),
        })
    }

    pub fn primary_email(&self) -> &str {
        // ADR-0083 Tier 1: `ContactCard::new` validation guarantees exactly one
        // primary email, but the lint disallows `.expect()` on an `Option`. Fall
        // back to the first email's address (always present per validation),
        // then to an empty string for the static-borrow case. The
        // first-element fallback preserves observable behavior for the
        // (validation-guaranteed-unreachable) no-primary case.
        if let Some(primary) = self.emails.value.iter().find(|email| email.primary.value) {
            return primary.email.value.as_str();
        }
        self.emails
            .value
            .first()
            .map(|email| email.email.value.as_str())
            .unwrap_or("")
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl ContactEmail {
    pub fn new(email: String, label: String, primary: bool) -> Result<Self, AddressBookError> {
        validate_email(&email)?;
        validate_non_empty(&label, AddressBookError::InvalidLabel)?;
        Ok(Self {
            email: Classified::new(email, contact_data_class()),
            label: internal(label),
            primary: internal(primary),
        })
    }
}

impl ContactPhone {
    pub fn new(phone: String, label: String, primary: bool) -> Result<Self, AddressBookError> {
        validate_phone(&phone)?;
        validate_non_empty(&label, AddressBookError::InvalidLabel)?;
        Ok(Self {
            phone: Classified::new(phone, contact_data_class()),
            label: internal(label),
            primary: internal(primary),
        })
    }
}

impl DirectorySearchGrant {
    pub fn new(input: DirectorySearchGrantCreate) -> Result<Self, AddressBookError> {
        validate_non_empty(&input.grant_id, AddressBookError::InvalidGrantId)?;
        validate_non_empty(&input.source_tenant_id, AddressBookError::InvalidTenantId)?;
        validate_non_empty(&input.target_tenant_id, AddressBookError::InvalidTenantId)?;
        if input.source_tenant_id == input.target_tenant_id {
            return Err(AddressBookError::InvalidTenantId);
        }
        validate_non_empty(
            &input.consent_receipt_id,
            AddressBookError::MissingConsentReceipt,
        )?;
        if input.purpose_id != CROSS_TENANT_PURPOSE_ID {
            return Err(AddressBookError::InvalidPurposeId);
        }
        if input.expires_at_epoch_seconds == 0 {
            return Err(AddressBookError::InvalidGrantExpiry);
        }
        Ok(Self {
            grant_id: internal(input.grant_id),
            source_tenant_id: internal(input.source_tenant_id),
            target_tenant_id: internal(input.target_tenant_id),
            consent_receipt_id: internal(input.consent_receipt_id),
            purpose_id: internal(input.purpose_id),
            expires_at_epoch_seconds: internal(input.expires_at_epoch_seconds),
            schema_version: internal(DIRECTORY_GRANT_SCHEMA_VERSION),
        })
    }

    pub fn is_active_at(&self, now_epoch_seconds: u64) -> bool {
        now_epoch_seconds < self.expires_at_epoch_seconds.value
    }
}

pub fn default_workspace_address_book_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn contact_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn contact_metadata_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn workspace_address_book_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, AddressBookError> {
    PrivacyDataClass::new(data_class).map_err(|_| AddressBookError::InvalidDataClass)
}

fn validate_contact_book_binding(
    input: &ContactCardCreate,
    book: &AddressBook,
) -> Result<(), AddressBookError> {
    validate_non_empty(&input.book_id, AddressBookError::InvalidBookId)?;
    validate_non_empty(&input.tenant_id, AddressBookError::InvalidTenantId)?;
    validate_non_empty(&input.owner_ref, AddressBookError::InvalidOwnerRef)?;
    if input.book_id != book.book_id.value {
        return Err(AddressBookError::ContactBookMismatch);
    }
    if input.tenant_id != book.tenant_id.value {
        return Err(AddressBookError::ContactTenantMismatch);
    }
    if input.owner_ref != book.owner_ref.value {
        return Err(AddressBookError::ContactOwnerMismatch);
    }
    Ok(())
}

fn validate_visibility(
    visibility: ContactVisibility,
    consent_receipt_id: Option<&str>,
    book_scope: AddressBookScope,
) -> Result<(), AddressBookError> {
    match visibility {
        ContactVisibility::PrivateUser => {
            if book_scope != AddressBookScope::UserContacts {
                return Err(AddressBookError::InvalidVisibilityForBook);
            }
            if consent_receipt_id.is_some() {
                return Err(AddressBookError::UnexpectedConsentReceipt);
            }
        }
        ContactVisibility::TenantDirectory => {
            if book_scope != AddressBookScope::TenantDirectory {
                return Err(AddressBookError::InvalidVisibilityForBook);
            }
            if consent_receipt_id.is_some() {
                return Err(AddressBookError::UnexpectedConsentReceipt);
            }
        }
        ContactVisibility::CrossTenantDirectory => {
            if book_scope != AddressBookScope::TenantDirectory {
                return Err(AddressBookError::InvalidVisibilityForBook);
            }
            let Some(consent_receipt_id) = consent_receipt_id else {
                return Err(AddressBookError::MissingConsentReceipt);
            };
            validate_non_empty(consent_receipt_id, AddressBookError::MissingConsentReceipt)?;
        }
    }
    Ok(())
}

fn validate_emails(emails: &[ContactEmail]) -> Result<(), AddressBookError> {
    if emails.is_empty() {
        return Err(AddressBookError::EmptyEmailSet);
    }
    let mut seen = BTreeSet::new();
    let mut primary_count = 0_u32;
    for email in emails {
        validate_email(&email.email.value)?;
        validate_non_empty(&email.label.value, AddressBookError::InvalidLabel)?;
        if !seen.insert(email.email.value.to_ascii_lowercase()) {
            return Err(AddressBookError::DuplicateEmail);
        }
        if email.primary.value {
            primary_count += 1;
        }
    }
    match primary_count {
        0 => Err(AddressBookError::MissingPrimaryEmail),
        1 => Ok(()),
        _ => Err(AddressBookError::MultiplePrimaryEmails),
    }
}

fn validate_phones(phones: &[ContactPhone]) -> Result<(), AddressBookError> {
    let mut seen = BTreeSet::new();
    let mut primary_count = 0_u32;
    for phone in phones {
        validate_phone(&phone.phone.value)?;
        validate_non_empty(&phone.label.value, AddressBookError::InvalidLabel)?;
        if !seen.insert(phone.phone.value.clone()) {
            return Err(AddressBookError::DuplicatePhone);
        }
        if phone.primary.value {
            primary_count += 1;
        }
    }
    if primary_count > 1 {
        Err(AddressBookError::MultiplePrimaryPhones)
    } else {
        Ok(())
    }
}

fn validate_email(email: &str) -> Result<(), AddressBookError> {
    if email.trim() != email || email.chars().any(char::is_control) {
        return Err(AddressBookError::InvalidEmail);
    }
    let Some((local, domain)) = email.split_once('@') else {
        return Err(AddressBookError::InvalidEmail);
    };
    if local.is_empty()
        || domain.is_empty()
        || domain.contains('@')
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
    {
        return Err(AddressBookError::InvalidEmail);
    }
    Ok(())
}

fn validate_phone(phone: &str) -> Result<(), AddressBookError> {
    if phone.trim() != phone
        || phone.is_empty()
        || phone.chars().any(char::is_control)
        || !phone
            .chars()
            .all(|character| character.is_ascii_digit() || matches!(character, '+' | '-' | ' '))
    {
        Err(AddressBookError::InvalidPhone)
    } else {
        Ok(())
    }
}

fn validate_time_order(created_at: u64, updated_at: u64) -> Result<(), AddressBookError> {
    if updated_at < created_at {
        Err(AddressBookError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_optional_text(
    value: Option<&str>,
    error: AddressBookError,
) -> Result<(), AddressBookError> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_text(value, error)
}

fn validate_text(value: &str, error: AddressBookError) -> Result<(), AddressBookError> {
    if value.trim() != value || value.is_empty() || value.chars().any(char::is_control) {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: AddressBookError) -> Result<(), AddressBookError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn book(scope: AddressBookScope) -> AddressBook {
        AddressBook::new(AddressBookCreate {
            book_id: "book-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            owner_ref: "user:owner@example.com".into(),
            scope,
            data_class: None,
            created_at_epoch_seconds: 1_700_000_000,
            updated_at_epoch_seconds: 1_700_000_010,
        })
        .unwrap()
    }

    fn email(address: &str, primary: bool) -> ContactEmail {
        ContactEmail::new(address.into(), "work".into(), primary).unwrap()
    }

    fn phone(number: &str, primary: bool) -> ContactPhone {
        ContactPhone::new(number.into(), "mobile".into(), primary).unwrap()
    }

    fn contact_input(visibility: ContactVisibility) -> ContactCardCreate {
        ContactCardCreate {
            contact_id: "contact-1".into(),
            book_id: "book-1".into(),
            tenant_id: "tenant-1".into(),
            owner_ref: "user:owner@example.com".into(),
            display_name: "Ada Lovelace".into(),
            emails: vec![email("ada@example.com", true)],
            phones: vec![phone("+1-555-0100", true)],
            organization: Some("Analytical Engines".into()),
            visibility,
            consent_receipt_id: None,
            data_class: None,
            updated_at_epoch_seconds: 1_700_000_020,
        }
    }

    #[test]
    fn address_book_and_contact_classify_identifying_fields() {
        let book = book(AddressBookScope::UserContacts);
        let contact =
            ContactCard::new(contact_input(ContactVisibility::PrivateUser), &book).unwrap();

        assert_eq!(
            book.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            contact.display_name.data_class,
            DataClassification::Privacy(contact_data_class())
        );
        assert_eq!(contact.primary_email(), "ada@example.com");
        assert_eq!(contact.schema_version.value, 1);
    }

    #[test]
    fn contact_requires_valid_unique_email_and_single_primary() {
        assert_eq!(
            ContactEmail::new("not-an-email".into(), "work".into(), true),
            Err(AddressBookError::InvalidEmail)
        );

        let book = book(AddressBookScope::UserContacts);
        let mut duplicate = contact_input(ContactVisibility::PrivateUser);
        duplicate.emails = vec![
            email("ada@example.com", true),
            email("ADA@example.com", false),
        ];
        assert_eq!(
            ContactCard::new(duplicate, &book),
            Err(AddressBookError::DuplicateEmail)
        );

        let mut no_primary = contact_input(ContactVisibility::PrivateUser);
        no_primary.emails = vec![email("ada@example.com", false)];
        assert_eq!(
            ContactCard::new(no_primary, &book),
            Err(AddressBookError::MissingPrimaryEmail)
        );
    }

    #[test]
    fn cross_tenant_directory_requires_tenant_book_and_consent() {
        let user_book = book(AddressBookScope::UserContacts);
        let mut cross = contact_input(ContactVisibility::CrossTenantDirectory);
        cross.consent_receipt_id = Some("consent-1".into());
        assert_eq!(
            ContactCard::new(cross.clone(), &user_book),
            Err(AddressBookError::InvalidVisibilityForBook)
        );

        let tenant_book = book(AddressBookScope::TenantDirectory);
        cross.consent_receipt_id = None;
        assert_eq!(
            ContactCard::new(cross.clone(), &tenant_book),
            Err(AddressBookError::MissingConsentReceipt)
        );

        cross.consent_receipt_id = Some("consent-1".into());
        assert!(ContactCard::new(cross, &tenant_book).is_ok());

        let mut private_with_consent = contact_input(ContactVisibility::PrivateUser);
        private_with_consent.consent_receipt_id = Some("consent-1".into());
        assert_eq!(
            ContactCard::new(private_with_consent, &user_book),
            Err(AddressBookError::UnexpectedConsentReceipt)
        );
    }

    #[test]
    fn directory_search_grant_requires_cross_tenant_consent_purpose() {
        let grant = DirectorySearchGrant::new(DirectorySearchGrantCreate {
            grant_id: "grant-1".into(),
            source_tenant_id: "tenant-1".into(),
            target_tenant_id: "tenant-2".into(),
            consent_receipt_id: "consent-1".into(),
            purpose_id: CROSS_TENANT_PURPOSE_ID.into(),
            expires_at_epoch_seconds: 1_800_000_000,
        })
        .unwrap();
        assert!(grant.is_active_at(1_700_000_000));
        assert!(!grant.is_active_at(1_800_000_000));

        assert_eq!(
            DirectorySearchGrant::new(DirectorySearchGrantCreate {
                grant_id: "grant-2".into(),
                source_tenant_id: "tenant-1".into(),
                target_tenant_id: "tenant-1".into(),
                consent_receipt_id: "consent-1".into(),
                purpose_id: CROSS_TENANT_PURPOSE_ID.into(),
                expires_at_epoch_seconds: 1_800_000_000,
            }),
            Err(AddressBookError::InvalidTenantId)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_address_book_data_class_from_legacy(DataClass::Audit),
            Err(AddressBookError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}

// ---------------------------------------------------------------------------
// M03-P06-IP — workspace.address-book STAGING surface markers (SPEC §4 rows).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressBookSurfaceStaging {
    pub entry_id: Classified<String>,      // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub vcard_version: Classified<String>, // data_class: INTERNAL_ONLY
}

impl AddressBookSurfaceStaging {
    pub fn new(entry_id: String, tenant_id: String, vcard_version: String) -> Self {
        Self {
            entry_id: Classified::new(entry_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            vcard_version: Classified::new(vcard_version, DataClass::InternalOnly),
        }
    }
}

#[cfg(test)]
mod m03_p06_tests {
    use super::*;

    fn sample() -> AddressBookSurfaceStaging {
        AddressBookSurfaceStaging::new(
            "address-book-1".into(),
            "address-book-1".into(),
            "address-book-1".into(),
        )
    }

    #[test]
    fn surface_staging_constructor_sets_internal_only() {
        let s = sample();
        assert_eq!(s.entry_id.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn surface_staging_round_trip_equality() {
        assert_eq!(sample(), sample());
    }
}
