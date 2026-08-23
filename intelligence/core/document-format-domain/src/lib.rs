//! Workspace document-format kernel.
//!
//! Workspace-internal typed kernel records for the export surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. This crate owns the shared
//! export-format vocabulary and fail-closed format compatibility checks for
//! Docs, Sheets, and Slides while leaving renderer, storage, and protocol work
//! to adapters.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use intelligence_collab_runtime_domain::CollabRuntime;

const DOCUMENT_EXPORT_REQUEST_SCHEMA_VERSION: u32 = 1;
const DOCUMENT_EXPORT_PAYLOAD_SCHEMA_VERSION: u32 = 1;
const MIN_EXPORT_BYTES: usize = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentFormatError {
    InvalidRequestId,
    InvalidDocumentId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidActorRef,
    InvalidFilename,
    InvalidStateVectorHash,
    RuntimeDocumentMismatch,
    RuntimeTenantMismatch,
    RuntimeRegionMismatch,
    RuntimeCellMismatch,
    RuntimeStateVectorMismatch,
    UnsupportedExportFormat,
    InvalidMediaType,
    InvalidFileExtension,
    EmptyExportPayload,
    InvalidPayloadHash,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DocumentKind {
    Document,
    Spreadsheet,
    Presentation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExportFormat {
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Hwpx,
    Odt,
    Ods,
    Odp,
    Markdown,
    Csv,
    Tsv,
    Html,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentExportRequestCreate {
    pub request_id: String,                   // data_class: INTERNAL_ONLY
    pub document_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub document_kind: DocumentKind,          // data_class: INTERNAL_ONLY
    pub target_format: ExportFormat,          // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub requested_by_actor_ref: String,       // data_class: PII_IDENTIFYING
    pub preferred_filename: Option<String>,   // data_class: PII_QUASI_IDENTIFIER
    pub source_state_vector_hash: String,     // data_class: INTERNAL_ONLY
    pub requested_at_epoch_millis: u64,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentExportRequest {
    pub request_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub document_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub region: Classified<String>,      // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub document_kind: Classified<DocumentKind>, // data_class: INTERNAL_ONLY
    pub target_format: Classified<ExportFormat>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub requested_by_actor_ref: Classified<String>, // data_class: PII_IDENTIFYING
    pub preferred_filename: Classified<Option<String>>, // data_class: PII_QUASI_IDENTIFIER
    pub source_state_vector_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_millis: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentExportPayloadCreate {
    pub target_format: ExportFormat,          // data_class: INTERNAL_ONLY
    pub media_type: String,                   // data_class: INTERNAL_ONLY
    pub file_extension: String,               // data_class: INTERNAL_ONLY
    pub bytes: Vec<u8>,                       // data_class: PII_IDENTIFYING
    pub sha256: String,                       // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub produced_at_epoch_millis: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentExportPayload {
    pub target_format: Classified<ExportFormat>, // data_class: INTERNAL_ONLY
    pub media_type: Classified<String>,          // data_class: INTERNAL_ONLY
    pub file_extension: Classified<String>,      // data_class: INTERNAL_ONLY
    pub bytes: Classified<Vec<u8>>,              // data_class: PII_IDENTIFYING
    pub sha256: Classified<String>,              // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub produced_at_epoch_millis: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,         // data_class: INTERNAL_ONLY
}

pub trait DocumentExporter {
    fn export(
        &self,
        runtime: &CollabRuntime,
        request: &DocumentExportRequest,
    ) -> Result<DocumentExportPayload, DocumentFormatError>;
}

impl ExportFormat {
    pub const fn canonical_extension(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Docx => "docx",
            Self::Xlsx => "xlsx",
            Self::Pptx => "pptx",
            Self::Hwpx => "hwpx",
            Self::Odt => "odt",
            Self::Ods => "ods",
            Self::Odp => "odp",
            Self::Markdown => "md",
            Self::Csv => "csv",
            Self::Tsv => "tsv",
            Self::Html => "html",
        }
    }

    pub const fn canonical_media_type(self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Self::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
            Self::Hwpx => "application/vnd.hancom.hwpx",
            Self::Odt => "application/vnd.oasis.opendocument.text",
            Self::Ods => "application/vnd.oasis.opendocument.spreadsheet",
            Self::Odp => "application/vnd.oasis.opendocument.presentation",
            Self::Markdown => "text/markdown",
            Self::Csv => "text/csv",
            Self::Tsv => "text/tab-separated-values",
            Self::Html => "text/html",
        }
    }

    pub const fn is_supported_for(self, kind: DocumentKind) -> bool {
        match kind {
            DocumentKind::Document => matches!(
                self,
                Self::Pdf | Self::Docx | Self::Hwpx | Self::Odt | Self::Markdown
            ),
            DocumentKind::Spreadsheet => {
                matches!(
                    self,
                    Self::Pdf | Self::Xlsx | Self::Ods | Self::Csv | Self::Tsv
                )
            }
            DocumentKind::Presentation => {
                matches!(self, Self::Pdf | Self::Pptx | Self::Odp | Self::Html)
            }
        }
    }
}

impl DocumentExportRequest {
    pub fn new(
        input: DocumentExportRequestCreate,
        runtime: &CollabRuntime,
    ) -> Result<Self, DocumentFormatError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_document_export_data_class());
        validate_non_empty(&input.request_id, DocumentFormatError::InvalidRequestId)?;
        validate_non_empty(&input.document_id, DocumentFormatError::InvalidDocumentId)?;
        validate_non_empty(&input.tenant_id, DocumentFormatError::InvalidTenantId)?;
        validate_non_empty(&input.region, DocumentFormatError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, DocumentFormatError::InvalidCellId)?;
        validate_non_empty(
            &input.requested_by_actor_ref,
            DocumentFormatError::InvalidActorRef,
        )?;
        validate_non_empty(
            &input.source_state_vector_hash,
            DocumentFormatError::InvalidStateVectorHash,
        )?;
        validate_filename(input.preferred_filename.as_deref())?;
        validate_runtime_binding(&input, runtime)?;
        if !input.target_format.is_supported_for(input.document_kind) {
            return Err(DocumentFormatError::UnsupportedExportFormat);
        }

        Ok(Self {
            request_id: internal(input.request_id),
            document_id: internal(input.document_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            document_kind: internal(input.document_kind),
            target_format: internal(input.target_format),
            data_class: internal(data_class),
            requested_by_actor_ref: Classified::new(
                input.requested_by_actor_ref,
                actor_data_class(),
            ),
            preferred_filename: Classified::new(input.preferred_filename, filename_data_class()),
            source_state_vector_hash: internal(input.source_state_vector_hash),
            requested_at_epoch_millis: internal(input.requested_at_epoch_millis),
            schema_version: internal(DOCUMENT_EXPORT_REQUEST_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl DocumentExportPayload {
    pub fn new(input: DocumentExportPayloadCreate) -> Result<Self, DocumentFormatError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_document_export_data_class());
        validate_non_empty(&input.media_type, DocumentFormatError::InvalidMediaType)?;
        validate_non_empty(
            &input.file_extension,
            DocumentFormatError::InvalidFileExtension,
        )?;
        validate_non_empty(&input.sha256, DocumentFormatError::InvalidPayloadHash)?;
        if input.bytes.len() < MIN_EXPORT_BYTES {
            return Err(DocumentFormatError::EmptyExportPayload);
        }
        if input.media_type != input.target_format.canonical_media_type() {
            return Err(DocumentFormatError::InvalidMediaType);
        }
        if input.file_extension != input.target_format.canonical_extension() {
            return Err(DocumentFormatError::InvalidFileExtension);
        }

        Ok(Self {
            target_format: internal(input.target_format),
            media_type: internal(input.media_type),
            file_extension: internal(input.file_extension),
            bytes: Classified::new(input.bytes, data_class),
            sha256: internal(input.sha256),
            data_class: internal(data_class),
            produced_at_epoch_millis: internal(input.produced_at_epoch_millis),
            schema_version: internal(DOCUMENT_EXPORT_PAYLOAD_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

pub fn default_workspace_document_export_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn actor_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn filename_data_class() -> PrivacyDataClass {
    // ADR-0083 Tier 1: use kernel's infallible `pii_quasi_identifier()` constructor.
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn workspace_document_export_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, DocumentFormatError> {
    PrivacyDataClass::new(data_class).map_err(|_| DocumentFormatError::InvalidDataClass)
}

fn validate_runtime_binding(
    input: &DocumentExportRequestCreate,
    runtime: &CollabRuntime,
) -> Result<(), DocumentFormatError> {
    if input.document_id != runtime.document_id.value {
        return Err(DocumentFormatError::RuntimeDocumentMismatch);
    }
    if input.tenant_id != runtime.tenant_id.value {
        return Err(DocumentFormatError::RuntimeTenantMismatch);
    }
    if input.region != runtime.region.value {
        return Err(DocumentFormatError::RuntimeRegionMismatch);
    }
    if input.cell_id != runtime.cell_id.value {
        return Err(DocumentFormatError::RuntimeCellMismatch);
    }
    if input.source_state_vector_hash != runtime.state_vector.value.state_vector_hash.value {
        return Err(DocumentFormatError::RuntimeStateVectorMismatch);
    }
    Ok(())
}

fn validate_filename(filename: Option<&str>) -> Result<(), DocumentFormatError> {
    let Some(filename) = filename else {
        return Ok(());
    };
    if filename.trim() != filename
        || filename.is_empty()
        || filename == "."
        || filename == ".."
        || filename.contains('/')
        || filename.contains('\\')
        || filename.chars().any(char::is_control)
    {
        return Err(DocumentFormatError::InvalidFilename);
    }
    Ok(())
}

fn validate_non_empty(value: &str, error: DocumentFormatError) -> Result<(), DocumentFormatError> {
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
    use intelligence_collab_runtime_domain::{
        CollabRuntimeCreate, CollabSnapshotRef, CollabStateVectorRef, CollabSurface,
    };

    fn runtime() -> CollabRuntime {
        CollabRuntime::new(CollabRuntimeCreate {
            document_id: "doc-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            surface: CollabSurface::Docs,
            data_class: None,
            snapshot: CollabSnapshotRef::new(
                "snap-1".into(),
                "tenant-1/docs/doc-1/snap-1".into(),
                "sha256:snapshot".into(),
                "sv:1".into(),
                1,
                7,
                128,
            )
            .unwrap(),
            state_vector: CollabStateVectorRef::new("sv:1".into(), 1, 7, 32).unwrap(),
            active_awareness: Vec::new(),
            created_at_epoch_millis: 1_700_000_000_000,
            updated_at_epoch_millis: 1_700_000_010_000,
        })
        .unwrap()
    }

    fn request_input() -> DocumentExportRequestCreate {
        DocumentExportRequestCreate {
            request_id: "export-1".into(),
            document_id: "doc-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            document_kind: DocumentKind::Document,
            target_format: ExportFormat::Pdf,
            data_class: None,
            requested_by_actor_ref: "user:writer@example.com".into(),
            preferred_filename: Some("workspace-plan.pdf".into()),
            source_state_vector_hash: "sv:1".into(),
            requested_at_epoch_millis: 1_700_000_020_000,
        }
    }

    #[test]
    fn request_defaults_to_identifying_and_classifies_filename_as_quasi() {
        let request = DocumentExportRequest::new(request_input(), &runtime()).unwrap();

        assert_eq!(
            request.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            request.preferred_filename.data_class,
            DataClassification::Privacy(filename_data_class())
        );
        assert_eq!(request.schema_version.value, 1);
    }

    #[test]
    fn export_format_matrix_is_fail_closed_per_document_kind() {
        let mut invalid = request_input();
        invalid.target_format = ExportFormat::Xlsx;
        assert_eq!(
            DocumentExportRequest::new(invalid, &runtime()),
            Err(DocumentFormatError::UnsupportedExportFormat)
        );

        let mut sheet = request_input();
        sheet.document_kind = DocumentKind::Spreadsheet;
        sheet.target_format = ExportFormat::Csv;
        assert!(DocumentExportRequest::new(sheet, &runtime()).is_ok());

        let mut slide = request_input();
        slide.document_kind = DocumentKind::Presentation;
        slide.target_format = ExportFormat::Html;
        assert!(DocumentExportRequest::new(slide, &runtime()).is_ok());
    }

    #[test]
    fn request_must_bind_to_runtime_identity_and_state_vector() {
        let mut invalid = request_input();
        invalid.tenant_id = "tenant-2".into();
        assert_eq!(
            DocumentExportRequest::new(invalid, &runtime()),
            Err(DocumentFormatError::RuntimeTenantMismatch)
        );

        let mut invalid = request_input();
        invalid.source_state_vector_hash = "sv:stale".into();
        assert_eq!(
            DocumentExportRequest::new(invalid, &runtime()),
            Err(DocumentFormatError::RuntimeStateVectorMismatch)
        );

        let mut invalid = request_input();
        invalid.preferred_filename = Some("../secret.pdf".into());
        assert_eq!(
            DocumentExportRequest::new(invalid, &runtime()),
            Err(DocumentFormatError::InvalidFilename)
        );
    }

    #[test]
    fn payload_rejects_empty_or_mislabelled_bytes_and_classifies_content() {
        let empty = DocumentExportPayload::new(DocumentExportPayloadCreate {
            target_format: ExportFormat::Pdf,
            media_type: ExportFormat::Pdf.canonical_media_type().into(),
            file_extension: ExportFormat::Pdf.canonical_extension().into(),
            bytes: Vec::new(),
            sha256: "sha256:payload".into(),
            data_class: None,
            produced_at_epoch_millis: 1_700_000_030_000,
        });
        assert_eq!(empty, Err(DocumentFormatError::EmptyExportPayload));

        let wrong_media_type = DocumentExportPayload::new(DocumentExportPayloadCreate {
            target_format: ExportFormat::Pdf,
            media_type: "application/octet-stream".into(),
            file_extension: ExportFormat::Pdf.canonical_extension().into(),
            bytes: vec![1, 2, 3],
            sha256: "sha256:payload".into(),
            data_class: None,
            produced_at_epoch_millis: 1_700_000_030_000,
        });
        assert_eq!(wrong_media_type, Err(DocumentFormatError::InvalidMediaType));

        let payload = DocumentExportPayload::new(DocumentExportPayloadCreate {
            target_format: ExportFormat::Pdf,
            media_type: ExportFormat::Pdf.canonical_media_type().into(),
            file_extension: ExportFormat::Pdf.canonical_extension().into(),
            bytes: vec![1, 2, 3],
            sha256: "sha256:payload".into(),
            data_class: None,
            produced_at_epoch_millis: 1_700_000_030_000,
        })
        .unwrap();
        assert_eq!(
            payload.bytes.data_class,
            DataClassification::Privacy(default_workspace_document_export_data_class())
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_document_export_data_class_from_legacy(DataClass::Audit),
            Err(DocumentFormatError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}
