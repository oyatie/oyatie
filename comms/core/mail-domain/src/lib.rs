//! Workspace mail kernel.
//!
//! Typed kernel records for the W-Workspace-Preview mail surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. The kernel keeps the first
//! vertical slice deliberately small: mailbox and message aggregates, data-class
//! defaults, and the read seam Foundry/Search consumers will build on.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod dkim_canonicalization;
pub mod dkim_signing_input;
pub mod governance;
pub mod sending_domain_authentication;
pub mod spf_alignment;
pub mod thread_grouping;
pub mod thread_state;
pub use dkim_canonicalization::{
    DkimCanonicalizationAlgorithm, RawHeader, canonicalize_body, canonicalize_header,
};
pub use dkim_signing_input::{
    DkimSigningInputError, DkimSigningInputMaterial, DkimSigningInputRequest,
    build_dkim_signing_input,
};
pub use governance::*;
pub use sending_domain_authentication::*;
pub use spf_alignment::{SpfAlignmentMode, SpfAlignmentVerdict, evaluate_spf_alignment};
pub use thread_grouping::{
    ThreadAssignment, ThreadTransitionError, group_into_thread, transition_thread_status,
};
pub use thread_state::{MailboxKind, ThreadStatus};

use data_boundary_kernel::{Classified, DataClass, DataClassification, PrivacyDataClass};

const MAILBOX_SCHEMA_VERSION: u32 = 1;
const MESSAGE_SCHEMA_VERSION: u32 = 1;
const DEFAULT_BODY_CONTENT_TYPE: &str = "text/plain; charset=utf-8";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailError {
    InvalidMailboxId,
    InvalidMessageId,
    InvalidThreadId,
    InvalidTenantId,
    InvalidRegion,
    InvalidAddress,
    InvalidRetentionPolicy,
    InvalidLegalHoldId,
    InvalidFolderId,
    InvalidFolderName,
    InvalidHeaderName,
    InvalidContentType,
    EmptyBody,
    EmptyAttachmentKey,
    EmptyAttachmentName,
    ZeroQuotaBytes,
    InvalidDataClass,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailboxCreate {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: INTERNAL_ONLY
    pub address: String,                      // data_class: PII_IDENTIFYING
    pub quota_bytes: u64,                     // data_class: INTERNAL_ONLY
    pub retention_policy_id: String,          // data_class: INTERNAL_ONLY
    pub legal_hold_id: Option<String>,        // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageCreate {
    pub id: String,                                // data_class: INTERNAL_ONLY
    pub mailbox_id: String,                        // data_class: INTERNAL_ONLY
    pub thread_id: String,                         // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>,      // data_class: INTERNAL_ONLY
    pub headers: Vec<MailHeader>,                  // data_class: PII_QUASI_IDENTIFIER
    pub body: MimeBody,                            // data_class: PII_IDENTIFYING
    pub attachments: Vec<AttachmentRef>,           // data_class: PII_IDENTIFYING
    pub classifications: Vec<ClassificationLabel>, // data_class: INTERNAL_ONLY
    pub received_at_epoch_seconds: u64,            // data_class: INTERNAL_ONLY
    pub indexed_at_epoch_seconds: Option<u64>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mailbox {
    pub id: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub region: Classified<String>,                // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub address: Classified<String>,               // data_class: PII_IDENTIFYING
    pub quota_bytes: Classified<u64>,              // data_class: INTERNAL_ONLY
    pub retention_policy_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub legal_hold_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub id: Classified<String>,                   // data_class: INTERNAL_ONLY
    pub mailbox_id: Classified<String>,           // data_class: INTERNAL_ONLY
    pub thread_id: Classified<String>,            // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub headers: Classified<Vec<MailHeader>>,     // data_class: PII_QUASI_IDENTIFIER
    pub body: MimeBody,                           // data_class: PII_IDENTIFYING
    pub attachments: Classified<Vec<AttachmentRef>>, // data_class: PII_IDENTIFYING
    pub classifications: Classified<Vec<ClassificationLabel>>, // data_class: INTERNAL_ONLY
    pub received_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub indexed_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailHeader {
    pub name: Classified<String>,  // data_class: PII_QUASI_IDENTIFIER
    pub value: Classified<String>, // data_class: PII_QUASI_IDENTIFIER
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MimeBody {
    pub content_type: Classified<String>, // data_class: INTERNAL_ONLY
    pub bytes: Classified<Vec<u8>>,       // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentRef {
    pub object_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub file_name: Classified<String>,  // data_class: PII_IDENTIFYING
    pub size_bytes: Classified<u64>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassificationLabel {
    pub label: Classified<String>, // data_class: INTERNAL_ONLY
    pub applied_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Folder {
    pub id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub mailbox_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub name: Classified<String>,       // data_class: INTERNAL_ONLY
    pub data_class_override: Classified<Option<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

pub trait MailReader {
    fn read_message(
        &self,
        tenant_id: &str,
        mailbox_id: &str,
        message_id: &str,
    ) -> Result<Option<Message>, MailError>;
}

impl Mailbox {
    pub fn new(input: MailboxCreate) -> Result<Self, MailError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_mail_data_class());
        validate_non_empty(&input.id, MailError::InvalidMailboxId)?;
        validate_non_empty(&input.tenant_id, MailError::InvalidTenantId)?;
        validate_non_empty(&input.region, MailError::InvalidRegion)?;
        validate_mail_address(&input.address)?;
        validate_non_empty(
            &input.retention_policy_id,
            MailError::InvalidRetentionPolicy,
        )?;
        if let Some(legal_hold_id) = input.legal_hold_id.as_deref() {
            validate_non_empty(legal_hold_id, MailError::InvalidLegalHoldId)?;
        }
        if input.quota_bytes == 0 {
            return Err(MailError::ZeroQuotaBytes);
        }

        Ok(Self {
            id: internal(input.id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            data_class: internal(data_class),
            address: Classified::new(input.address, data_class),
            quota_bytes: internal(input.quota_bytes),
            retention_policy_id: internal(input.retention_policy_id),
            legal_hold_id: internal(input.legal_hold_id),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: internal(MAILBOX_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl Message {
    pub fn new(input: MessageCreate) -> Result<Self, MailError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_mail_data_class());
        validate_non_empty(&input.id, MailError::InvalidMessageId)?;
        validate_non_empty(&input.mailbox_id, MailError::InvalidMailboxId)?;
        validate_non_empty(&input.thread_id, MailError::InvalidThreadId)?;
        if input.body.bytes.value.is_empty() {
            return Err(MailError::EmptyBody);
        }
        if input.body.bytes.data_class != DataClassification::Privacy(data_class) {
            return Err(MailError::InvalidDataClass);
        }
        for header in &input.headers {
            validate_non_empty(&header.name.value, MailError::InvalidHeaderName)?;
        }
        for attachment in &input.attachments {
            validate_non_empty(&attachment.object_key.value, MailError::EmptyAttachmentKey)?;
            validate_non_empty(&attachment.file_name.value, MailError::EmptyAttachmentName)?;
        }

        Ok(Self {
            id: internal(input.id),
            mailbox_id: internal(input.mailbox_id),
            thread_id: internal(input.thread_id),
            data_class: internal(data_class),
            headers: Classified::new(
                input.headers,
                PrivacyDataClass::new(DataClass::PiiQuasiIdentifier)
                    .map_err(|_| MailError::InvalidDataClass)?,
            ),
            body: input.body,
            attachments: Classified::new(input.attachments, data_class),
            classifications: internal(input.classifications),
            received_at_epoch_seconds: internal(input.received_at_epoch_seconds),
            indexed_at_epoch_seconds: internal(input.indexed_at_epoch_seconds),
            schema_version: internal(MESSAGE_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl MailHeader {
    pub fn new(name: String, value: String) -> Result<Self, MailError> {
        validate_non_empty(&name, MailError::InvalidHeaderName)?;
        Ok(Self {
            name: Classified::new(
                name,
                PrivacyDataClass::new(DataClass::PiiQuasiIdentifier)
                    .map_err(|_| MailError::InvalidDataClass)?,
            ),
            value: Classified::new(
                value,
                PrivacyDataClass::new(DataClass::PiiQuasiIdentifier)
                    .map_err(|_| MailError::InvalidDataClass)?,
            ),
        })
    }
}

impl MimeBody {
    pub fn plain_text(bytes: Vec<u8>) -> Result<Self, MailError> {
        Self::new(
            DEFAULT_BODY_CONTENT_TYPE.to_string(),
            bytes,
            default_workspace_mail_data_class(),
        )
    }

    pub fn new(
        content_type: String,
        bytes: Vec<u8>,
        data_class: PrivacyDataClass,
    ) -> Result<Self, MailError> {
        validate_non_empty(&content_type, MailError::InvalidContentType)?;
        if bytes.is_empty() {
            return Err(MailError::EmptyBody);
        }
        Ok(Self {
            content_type: internal(content_type),
            bytes: Classified::new(bytes, data_class),
        })
    }
}

impl AttachmentRef {
    pub fn new(object_key: String, file_name: String, size_bytes: u64) -> Result<Self, MailError> {
        validate_non_empty(&object_key, MailError::EmptyAttachmentKey)?;
        validate_non_empty(&file_name, MailError::EmptyAttachmentName)?;
        Ok(Self {
            object_key: internal(object_key),
            file_name: Classified::new(file_name, default_workspace_mail_data_class()),
            size_bytes: internal(size_bytes),
        })
    }
}

impl ClassificationLabel {
    pub fn new(label: String, applied_at_epoch_seconds: u64) -> Result<Self, MailError> {
        validate_non_empty(&label, MailError::InvalidDataClass)?;
        Ok(Self {
            label: internal(label),
            applied_at_epoch_seconds: internal(applied_at_epoch_seconds),
        })
    }
}

impl Folder {
    pub fn new(
        id: String,
        mailbox_id: String,
        name: String,
        data_class_override: Option<PrivacyDataClass>,
    ) -> Result<Self, MailError> {
        validate_non_empty(&id, MailError::InvalidFolderId)?;
        validate_non_empty(&mailbox_id, MailError::InvalidMailboxId)?;
        validate_non_empty(&name, MailError::InvalidFolderName)?;
        Ok(Self {
            id: internal(id),
            mailbox_id: internal(mailbox_id),
            name: internal(name),
            data_class_override: internal(data_class_override),
            schema_version: internal(MAILBOX_SCHEMA_VERSION),
        })
    }
}

pub fn default_workspace_mail_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_mail_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, MailError> {
    PrivacyDataClass::new(data_class).map_err(|_| MailError::InvalidDataClass)
}

fn validate_mail_address(address: &str) -> Result<(), MailError> {
    let trimmed = address.trim();
    if trimmed != address || trimmed.chars().any(char::is_whitespace) {
        return Err(MailError::InvalidAddress);
    }
    let Some((local, domain)) = trimmed.split_once('@') else {
        return Err(MailError::InvalidAddress);
    };
    if local.is_empty()
        || domain.is_empty()
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
    {
        return Err(MailError::InvalidAddress);
    }
    Ok(())
}

fn validate_non_empty(value: &str, error: MailError) -> Result<(), MailError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

// ---------------------------------------------------------------------------
// M03-P06-IP-001 — workspace.mail.{smtp,imap,jmap} STAGING surface markers
// (SPEC §4 rows; RFC 5321 / 3501 / 8620 compliance).
// ---------------------------------------------------------------------------

const SMTP_RFC: u32 = 5321;
const IMAP_RFC: u32 = 3501;
const JMAP_RFC: u32 = 8620;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MailSurfaceProtocol {
    SmtpReceive,
    Imap,
    Jmap,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSurfaceStaging {
    pub mailbox_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub protocol: Classified<MailSurfaceProtocol>, // data_class: INTERNAL_ONLY
    pub rfc_number: Classified<u32>,    // data_class: INTERNAL_ONLY
    pub phishing_dlp_classify_before_store: Classified<bool>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

impl MailSurfaceStaging {
    pub fn new(
        mailbox_id: String,
        tenant_id: String,
        protocol: MailSurfaceProtocol,
    ) -> Result<Self, MailError> {
        validate_non_empty(&mailbox_id, MailError::InvalidMailboxId)?;
        validate_non_empty(&tenant_id, MailError::InvalidTenantId)?;
        let rfc = match protocol {
            MailSurfaceProtocol::SmtpReceive => SMTP_RFC,
            MailSurfaceProtocol::Imap => IMAP_RFC,
            MailSurfaceProtocol::Jmap => JMAP_RFC,
        };
        // SPEC §4: SMTP receive must run phishing+DLP+classify before store.
        let must_classify_before_store = matches!(protocol, MailSurfaceProtocol::SmtpReceive);
        Ok(Self {
            mailbox_id: internal(mailbox_id),
            tenant_id: internal(tenant_id),
            protocol: internal(protocol),
            rfc_number: internal(rfc),
            phishing_dlp_classify_before_store: internal(must_classify_before_store),
            schema_version: internal(MAILBOX_SCHEMA_VERSION),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_boundary_kernel::OperationalDataClass;

    fn valid_mailbox_input() -> MailboxCreate {
        MailboxCreate {
            id: "mailbox-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            address: "employee@example.com".into(),
            quota_bytes: 10_000,
            retention_policy_id: "retain-default".into(),
            legal_hold_id: None,
            data_class: None,
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    fn valid_message_input() -> MessageCreate {
        MessageCreate {
            id: "message-1".into(),
            mailbox_id: "mailbox-1".into(),
            thread_id: "thread-1".into(),
            data_class: None,
            headers: vec![MailHeader::new("Subject".into(), "Quarter close".into()).unwrap()],
            body: MimeBody::plain_text(b"hello".to_vec()).unwrap(),
            attachments: vec![],
            classifications: vec![ClassificationLabel::new("clean".into(), 1_700_000_001).unwrap()],
            received_at_epoch_seconds: 1_700_000_001,
            indexed_at_epoch_seconds: None,
        }
    }

    #[test]
    fn mailbox_defaults_to_pii_identifying_and_classifies_address() {
        let mailbox = Mailbox::new(valid_mailbox_input()).unwrap();

        assert_eq!(
            mailbox.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            mailbox.address.data_class,
            DataClassification::Privacy(default_workspace_mail_data_class())
        );
        assert_eq!(mailbox.schema_version.value, 1);
    }

    #[test]
    fn mailbox_rejects_invalid_address_and_zero_quota() {
        let mut invalid = valid_mailbox_input();
        invalid.address = "employee".into();
        assert_eq!(Mailbox::new(invalid), Err(MailError::InvalidAddress));

        let mut invalid = valid_mailbox_input();
        invalid.quota_bytes = 0;
        assert_eq!(Mailbox::new(invalid), Err(MailError::ZeroQuotaBytes));
    }

    #[test]
    fn message_body_uses_message_privacy_class_and_headers_are_quasi_identifiers() {
        let message = Message::new(valid_message_input()).unwrap();

        assert_eq!(
            message.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            message.body.bytes.data_class,
            DataClassification::Privacy(default_workspace_mail_data_class())
        );
        assert_eq!(
            message.headers.value[0].name.data_class,
            DataClassification::Privacy(PrivacyDataClass::pii_quasi_identifier())
        );
    }

    #[test]
    fn message_rejects_body_class_that_does_not_match_message_class() {
        let mut input = valid_message_input();
        input.data_class = Some(PrivacyDataClass::internal_only());

        assert_eq!(Message::new(input), Err(MailError::InvalidDataClass));
    }

    #[test]
    fn surface_staging_maps_protocol_to_rfc_and_smtp_requires_classify() {
        let smtp = MailSurfaceStaging::new(
            "mailbox-1".into(),
            "tenant-1".into(),
            MailSurfaceProtocol::SmtpReceive,
        )
        .unwrap();
        assert_eq!(smtp.rfc_number.value, 5321);
        assert!(smtp.phishing_dlp_classify_before_store.value);

        let imap = MailSurfaceStaging::new(
            "mailbox-1".into(),
            "tenant-1".into(),
            MailSurfaceProtocol::Imap,
        )
        .unwrap();
        assert_eq!(imap.rfc_number.value, 3501);
        assert!(!imap.phishing_dlp_classify_before_store.value);

        let jmap = MailSurfaceStaging::new(
            "mailbox-1".into(),
            "tenant-1".into(),
            MailSurfaceProtocol::Jmap,
        )
        .unwrap();
        assert_eq!(jmap.rfc_number.value, 8620);
    }

    #[test]
    fn surface_staging_rejects_empty_identifiers() {
        assert_eq!(
            MailSurfaceStaging::new("".into(), "t".into(), MailSurfaceProtocol::Imap),
            Err(MailError::InvalidMailboxId)
        );
        assert_eq!(
            MailSurfaceStaging::new("m".into(), "".into(), MailSurfaceProtocol::Imap),
            Err(MailError::InvalidTenantId)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_mail_data_class_from_legacy(DataClass::Audit),
            Err(MailError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}
