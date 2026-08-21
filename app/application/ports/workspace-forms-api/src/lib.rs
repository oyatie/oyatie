//! Workspace Forms API boundary for submission ingest.
//!
//! This crate owns REST-boundary normalization, idempotent submission-ingest
//! handling, coarse authorization proof checks, submitter binding, and Object
//! Graph route projection around the Workspace Forms kernel. Durable storage,
//! event publication, Object Graph adapter writes, moderation, and public form
//! rendering remain adapter concerns.

use std::collections::BTreeMap;

use data_boundary_kernel::parse_data_class_label;
use workflow_forms_domain::{
    Form, FormAnswer, FormAnswerCreate, FormCreate, FormError, FormField, FormFieldCreate,
    FormFieldKind, FormSubmission, FormSubmissionCreate, workspace_form_data_class_from_legacy,
};

pub const WORKSPACE_FORMS_SUBMISSION_INGEST_SURFACE: &str = "workspace.forms.submission.ingest";
pub const WORKSPACE_FORMS_OPENAPI_CONTRACT: &str =
    "contracts/openapi/workspace/workspace-forms-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceFormsSubmissionIngestApiStatus {
    Accepted,
    BadRequest,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl WorkspaceFormsSubmissionIngestApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceFormsApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    FormIdInvalid,
    SubmissionIdInvalid,
    FormIdMismatch,
    SubmissionIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    SubmitterPermissionDenied,
    IdempotencyKeyReused,
    DataClassInvalid,
    FieldKindInvalid,
    FormNotFound,
    SubmissionAlreadyExists,
    FormInvalidRequest,
}

impl WorkspaceFormsApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "WORKSPACE_FORMS_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "WORKSPACE_FORMS_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "WORKSPACE_FORMS_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "WORKSPACE_FORMS_PRINCIPAL_ID_EMPTY",
            Self::FormIdInvalid => "WORKSPACE_FORMS_FORM_ID_INVALID",
            Self::SubmissionIdInvalid => "WORKSPACE_FORMS_SUBMISSION_ID_INVALID",
            Self::FormIdMismatch => "WORKSPACE_FORMS_FORM_ID_MISMATCH",
            Self::SubmissionIdMismatch => "WORKSPACE_FORMS_SUBMISSION_ID_MISMATCH",
            Self::TenantMismatch => "WORKSPACE_FORMS_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "WORKSPACE_FORMS_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "WORKSPACE_FORMS_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "WORKSPACE_FORMS_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "WORKSPACE_FORMS_AUTHORIZATION_DENIED",
            Self::SubmitterPermissionDenied => "WORKSPACE_FORMS_SUBMITTER_PERMISSION_DENIED",
            Self::IdempotencyKeyReused => "WORKSPACE_FORMS_IDEMPOTENCY_KEY_REUSED",
            Self::DataClassInvalid => "WORKSPACE_FORMS_DATA_CLASS_INVALID",
            Self::FieldKindInvalid => "WORKSPACE_FORMS_FIELD_KIND_INVALID",
            Self::FormNotFound => "WORKSPACE_FORMS_FORM_NOT_FOUND",
            Self::SubmissionAlreadyExists => "WORKSPACE_FORMS_SUBMISSION_ALREADY_EXISTS",
            Self::FormInvalidRequest => "WORKSPACE_FORMS_INVALID_REQUEST",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsIngestBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: PII_IDENTIFYING
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsFieldSeed {
    pub field_id: String,            // data_class: INTERNAL_ONLY
    pub label: String,               // data_class: PII_QUASI_IDENTIFIER
    pub kind: String,                // data_class: INTERNAL_ONLY
    pub required: bool,              // data_class: INTERNAL_ONLY
    pub choice_options: Vec<String>, // data_class: PII_QUASI_IDENTIFIER
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsFormSeed {
    pub form_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub object_graph_route_id: String,        // data_class: INTERNAL_ONLY
    pub title: String,                        // data_class: PII_QUASI_IDENTIFIER
    pub data_class: String,                   // data_class: INTERNAL_ONLY
    pub fields: Vec<WorkspaceFormsFieldSeed>, // data_class: PII_QUASI_IDENTIFIER
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsAnswerRequest {
    pub field_id: String,   // data_class: INTERNAL_ONLY
    pub value_kind: String, // data_class: INTERNAL_ONLY
    pub value: String,      // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsSubmissionIngestRequest {
    pub submission_id: String,                     // data_class: INTERNAL_ONLY
    pub form_id: String,                           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                         // data_class: INTERNAL_ONLY
    pub submitter_ref: String,                     // data_class: PII_IDENTIFYING
    pub answers: Vec<WorkspaceFormsAnswerRequest>, // data_class: PII_IDENTIFYING
    pub data_class: String,                        // data_class: INTERNAL_ONLY
    pub submitted_at_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsSubmissionIngestApiRequest {
    pub path_form_id: String,       // data_class: INTERNAL_ONLY
    pub path_submission_id: String, // data_class: INTERNAL_ONLY
    pub boundary: WorkspaceFormsIngestBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: WorkspaceFormsApiPrincipal, // data_class: PII_IDENTIFYING
    pub authorization: WorkspaceFormsApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: WorkspaceFormsSubmissionIngestRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceFormsSubmissionDirectory {
    forms: BTreeMap<WorkspaceFormsFormKey, Form>, // data_class: INTERNAL_ONLY
    submissions: BTreeMap<WorkspaceFormsSubmissionKey, WorkspaceFormsStoredSubmission>, // data_class: INTERNAL_ONLY
}

impl WorkspaceFormsSubmissionDirectory {
    pub fn form_len(&self) -> usize {
        self.forms.len()
    }

    pub fn submission_len(&self) -> usize {
        self.submissions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.forms.is_empty() && self.submissions.is_empty()
    }

    pub fn submissions(&self) -> impl Iterator<Item = &FormSubmission> {
        self.submissions
            .values()
            .map(|stored_submission| &stored_submission.submission)
    }

    pub fn submission_records(&self) -> impl Iterator<Item = WorkspaceFormsSubmissionRecord> + '_ {
        self.submissions.values().map(|stored_submission| {
            submission_record(
                &stored_submission.submission,
                &stored_submission.object_graph_route_id,
            )
        })
    }

    pub fn insert_form_seed(
        &mut self,
        seed: WorkspaceFormsFormSeed,
    ) -> Result<(), WorkspaceFormsApiError> {
        let form = form_from_seed(seed)?;
        let key = WorkspaceFormsFormKey {
            tenant_id: form.tenant_id.value.clone(),
            form_id: form.id.value.clone(),
        };
        self.forms.insert(key, form);
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkspaceFormsFormKey {
    tenant_id: String, // data_class: INTERNAL_ONLY
    form_id: String,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkspaceFormsSubmissionKey {
    tenant_id: String,     // data_class: INTERNAL_ONLY
    form_id: String,       // data_class: INTERNAL_ONLY
    submission_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceFormsStoredSubmission {
    submission: FormSubmission,    // data_class: INTERNAL_ONLY
    object_graph_route_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceFormsSubmissionIngestIdempotencyLedger {
    entries: BTreeMap<WorkspaceFormsIdempotencyLedgerKey, WorkspaceFormsIngestLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl WorkspaceFormsSubmissionIngestIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkspaceFormsIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: PII_IDENTIFYING
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceFormsIngestLedgerEntry {
    fingerprint: WorkspaceFormsRequestFingerprint, // data_class: INTERNAL_ONLY
    result: WorkspaceFormsSubmissionIngestSuccessResponse, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceFormsRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsSubmissionIngestSuccessResponse {
    pub data: WorkspaceFormsSubmissionRecord, // data_class: INTERNAL_ONLY
    pub metadata: WorkspaceFormsSubmissionMetadata, // data_class: INTERNAL_ONLY
}

impl WorkspaceFormsSubmissionIngestSuccessResponse {
    pub fn accepted(data: WorkspaceFormsSubmissionRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: WorkspaceFormsSubmissionMetadata {
                request_id: request_id.into(),
                surface: WORKSPACE_FORMS_SUBMISSION_INGEST_SURFACE.to_string(),
                openapi_contract: WORKSPACE_FORMS_OPENAPI_CONTRACT.to_string(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsSubmissionMetadata {
    pub request_id: String,       // data_class: INTERNAL_ONLY
    pub surface: String,          // data_class: INTERNAL_ONLY
    pub openapi_contract: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsAnswerRecord {
    pub field_id: String,   // data_class: INTERNAL_ONLY
    pub value_kind: String, // data_class: INTERNAL_ONLY
    pub value: String,      // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsSubmissionRecord {
    pub submission_id: String,                    // data_class: INTERNAL_ONLY
    pub form_id: String,                          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub submitter_ref: String,                    // data_class: PII_IDENTIFYING
    pub answers: Vec<WorkspaceFormsAnswerRecord>, // data_class: PII_IDENTIFYING
    pub data_class: String,                       // data_class: INTERNAL_ONLY
    pub submitted_at_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
    pub object_graph_route_id: String,            // data_class: INTERNAL_ONLY
    pub schema_version: u32,                      // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsApiErrorResponse {
    pub error: WorkspaceFormsApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsApiErrorBody {
    pub code: String,                               // data_class: INTERNAL_ONLY
    pub message: String,                            // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,          // data_class: INTERNAL_ONLY
    pub request_id: String,                         // data_class: INTERNAL_ONLY
    pub details: Vec<WorkspaceFormsApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFormsApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkspaceFormsApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    InvalidFormId {
        form_id: String,
    },
    InvalidSubmissionId {
        submission_id: String,
    },
    FormIdMismatch {
        path_form_id: String,
        body_form_id: String,
    },
    SubmissionIdMismatch {
        path_submission_id: String,
        body_submission_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        authorization_tenant_id: Option<String>,
        body_tenant_id: Option<String>,
        resource_tenant_id: Option<String>,
    },
    EmptyAuthorizationDecisionId,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String,
        principal_tenant_id: String,
    },
    AuthorizationPrincipalMismatch {
        authorization_principal_id: String,
        principal_id: String,
    },
    AuthorizationDenied {
        surface: String,
    },
    SubmitterPermissionDenied {
        principal_id: String,
        submitter_ref: String,
        form_id: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    InvalidFieldKind {
        kind: String,
    },
    FormNotFound {
        tenant_id: String,
        form_id: String,
    },
    SubmissionAlreadyExists {
        tenant_id: String,
        form_id: String,
        submission_id: String,
    },
    Form(FormError),
}

impl WorkspaceFormsApiError {
    pub fn submission_status_code(&self) -> u16 {
        match self.status_kind() {
            WorkspaceFormsApiStatusKind::BadRequest => 400,
            WorkspaceFormsApiStatusKind::Forbidden => 403,
            WorkspaceFormsApiStatusKind::NotFound => 404,
            WorkspaceFormsApiStatusKind::Conflict => 409,
            WorkspaceFormsApiStatusKind::UnprocessableEntity => 422,
        }
    }

    pub fn code(&self) -> WorkspaceFormsApiErrorCode {
        match self {
            Self::EmptyRequestId => WorkspaceFormsApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => WorkspaceFormsApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => WorkspaceFormsApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => WorkspaceFormsApiErrorCode::PrincipalIdEmpty,
            Self::InvalidFormId { .. } => WorkspaceFormsApiErrorCode::FormIdInvalid,
            Self::InvalidSubmissionId { .. } => WorkspaceFormsApiErrorCode::SubmissionIdInvalid,
            Self::FormIdMismatch { .. } => WorkspaceFormsApiErrorCode::FormIdMismatch,
            Self::SubmissionIdMismatch { .. } => WorkspaceFormsApiErrorCode::SubmissionIdMismatch,
            Self::TenantMismatch { .. } => WorkspaceFormsApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                WorkspaceFormsApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                WorkspaceFormsApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                WorkspaceFormsApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => WorkspaceFormsApiErrorCode::AuthorizationDenied,
            Self::SubmitterPermissionDenied { .. } => {
                WorkspaceFormsApiErrorCode::SubmitterPermissionDenied
            }
            Self::IdempotencyKeyReused { .. } => WorkspaceFormsApiErrorCode::IdempotencyKeyReused,
            Self::InvalidDataClassLabel { .. } => WorkspaceFormsApiErrorCode::DataClassInvalid,
            Self::InvalidFieldKind { .. } => WorkspaceFormsApiErrorCode::FieldKindInvalid,
            Self::FormNotFound { .. } => WorkspaceFormsApiErrorCode::FormNotFound,
            Self::SubmissionAlreadyExists { .. } => {
                WorkspaceFormsApiErrorCode::SubmissionAlreadyExists
            }
            Self::Form(_) => WorkspaceFormsApiErrorCode::FormInvalidRequest,
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> WorkspaceFormsApiErrorResponse {
        WorkspaceFormsApiErrorResponse {
            error: WorkspaceFormsApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> WorkspaceFormsApiStatusKind {
        match self {
            Self::TenantMismatch { .. }
            | Self::EmptyPrincipalId
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. }
            | Self::SubmitterPermissionDenied { .. } => WorkspaceFormsApiStatusKind::Forbidden,
            Self::FormNotFound { .. } => WorkspaceFormsApiStatusKind::NotFound,
            Self::SubmissionAlreadyExists { .. } => WorkspaceFormsApiStatusKind::Conflict,
            Self::IdempotencyKeyReused { .. } => WorkspaceFormsApiStatusKind::UnprocessableEntity,
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::InvalidFormId { .. }
            | Self::InvalidSubmissionId { .. }
            | Self::FormIdMismatch { .. }
            | Self::SubmissionIdMismatch { .. }
            | Self::InvalidDataClassLabel { .. }
            | Self::InvalidFieldKind { .. }
            | Self::Form(_) => WorkspaceFormsApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::InvalidFormId { .. } => "Workspace Forms form id is required",
            Self::InvalidSubmissionId { .. } => "Workspace Forms submission id is required",
            Self::FormIdMismatch { .. } => "Path and body form ids must match",
            Self::SubmissionIdMismatch { .. } => "Path and body submission ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal, authorization, body, and resource"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the Workspace Forms submission-ingest surface"
            }
            Self::SubmitterPermissionDenied { .. } => {
                "Submitter must match the authenticated principal"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::InvalidFieldKind { .. } => "Form field kind must be known",
            Self::FormNotFound { .. } => "Workspace Forms form was not found",
            Self::SubmissionAlreadyExists { .. } => "Workspace Forms submission already exists",
            Self::Form(error) => form_error_message(error),
        }
    }

    fn details(&self) -> Vec<WorkspaceFormsApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::InvalidFormId { .. } => vec![detail("path.form_id", "must be non-empty")],
            Self::InvalidSubmissionId { .. } => {
                vec![detail("path.submission_id", "must be non-empty")]
            }
            Self::FormIdMismatch { .. } => vec![detail(
                "form_id",
                "path form_id and body form_id must match",
            )],
            Self::SubmissionIdMismatch { .. } => vec![detail(
                "submission_id",
                "path submission_id and body submission_id must match",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, authorization tenant, body tenant_id, and resource tenant must match",
            )],
            Self::EmptyAuthorizationDecisionId => vec![detail(
                "authorization.decision_id",
                "must be non-empty authorization evidence",
            )],
            Self::AuthorizationTenantMismatch { .. } => vec![detail(
                "authorization.tenant_id",
                "must match the authenticated principal tenant",
            )],
            Self::AuthorizationPrincipalMismatch { .. } => vec![detail(
                "authorization.principal_id",
                "must match the authenticated principal id",
            )],
            Self::AuthorizationDenied { .. } => vec![detail(
                "authorization.allowed_surfaces",
                "must include workspace.forms.submission.ingest",
            )],
            Self::SubmitterPermissionDenied { .. } => vec![detail(
                "body.submitter_ref",
                "submitter_ref must match the authenticated principal",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::InvalidFieldKind { .. } => vec![detail(
                "value_kind",
                "must be one of short_text, long_text, number, boolean, single_choice, multi_choice",
            )],
            Self::FormNotFound { .. } => vec![detail(
                "path.form_id",
                "form schema must exist before submissions can be ingested",
            )],
            Self::SubmissionAlreadyExists { .. } => vec![detail(
                "path.submission_id",
                "submission already exists for the requested tenant form",
            )],
            Self::Form(error) => vec![detail("workspace_forms", form_error_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkspaceFormsApiStatusKind {
    BadRequest,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_workspace_forms_submission_ingest_request(
    request: &WorkspaceFormsSubmissionIngestApiRequest,
) -> Result<(), WorkspaceFormsApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_form_id(&request.path_form_id)?;
    validate_path_submission_id(&request.path_submission_id)?;
    validate_path_body_binding(
        &request.path_form_id,
        &request.body.form_id,
        &request.path_submission_id,
        &request.body.submission_id,
    )?;
    validate_tenant_binding(
        &request.boundary.tenant_id,
        &request.principal,
        &request.authorization,
        Some(&request.body.tenant_id),
        None,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        WORKSPACE_FORMS_SUBMISSION_INGEST_SURFACE,
    )?;
    Ok(())
}

pub fn ingest_workspace_forms_submission_from_api(
    directory: &mut WorkspaceFormsSubmissionDirectory,
    idempotency_ledger: &mut WorkspaceFormsSubmissionIngestIdempotencyLedger,
    request: WorkspaceFormsSubmissionIngestApiRequest,
) -> Result<WorkspaceFormsSubmissionIngestSuccessResponse, WorkspaceFormsApiError> {
    validate_workspace_forms_submission_ingest_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        WORKSPACE_FORMS_SUBMISSION_INGEST_SURFACE,
    );
    let fingerprint = ingest_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(WorkspaceFormsApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let principal_id = request.principal.principal_id.clone();
    let form_id = request.body.form_id.clone();
    let tenant_id = request.body.tenant_id.clone();
    require_submitter(&principal_id, &request.body.submitter_ref, &form_id)?;
    let form_key = WorkspaceFormsFormKey {
        tenant_id: tenant_id.clone(),
        form_id: form_id.clone(),
    };
    let form =
        directory
            .forms
            .get(&form_key)
            .ok_or_else(|| WorkspaceFormsApiError::FormNotFound {
                tenant_id: tenant_id.clone(),
                form_id: form_id.clone(),
            })?;
    let submission = form_submission_from_request(request.body, form)?;
    let submission_key = WorkspaceFormsSubmissionKey {
        tenant_id,
        form_id,
        submission_id: submission.submission_id.value.clone(),
    };
    if directory.submissions.contains_key(&submission_key) {
        return Err(WorkspaceFormsApiError::SubmissionAlreadyExists {
            tenant_id: submission_key.tenant_id,
            form_id: submission_key.form_id,
            submission_id: submission_key.submission_id,
        });
    }

    let object_graph_route_id = form.object_graph_route_id.value.clone();
    let response = WorkspaceFormsSubmissionIngestSuccessResponse::accepted(
        submission_record(&submission, &object_graph_route_id),
        request_id,
    );
    directory.submissions.insert(
        submission_key,
        WorkspaceFormsStoredSubmission {
            submission,
            object_graph_route_id,
        },
    );
    idempotency_ledger.entries.insert(
        key,
        WorkspaceFormsIngestLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

fn validate_boundary(
    boundary: &WorkspaceFormsIngestBoundaryContext,
) -> Result<(), WorkspaceFormsApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(WorkspaceFormsApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(WorkspaceFormsApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(WorkspaceFormsApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_form_id(form_id: &str) -> Result<(), WorkspaceFormsApiError> {
    if form_id.trim().is_empty() {
        return Err(WorkspaceFormsApiError::InvalidFormId {
            form_id: form_id.to_string(),
        });
    }
    Ok(())
}

fn validate_path_submission_id(submission_id: &str) -> Result<(), WorkspaceFormsApiError> {
    if submission_id.trim().is_empty() {
        return Err(WorkspaceFormsApiError::InvalidSubmissionId {
            submission_id: submission_id.to_string(),
        });
    }
    Ok(())
}

fn validate_path_body_binding(
    path_form_id: &str,
    body_form_id: &str,
    path_submission_id: &str,
    body_submission_id: &str,
) -> Result<(), WorkspaceFormsApiError> {
    if path_form_id != body_form_id {
        return Err(WorkspaceFormsApiError::FormIdMismatch {
            path_form_id: path_form_id.to_string(),
            body_form_id: body_form_id.to_string(),
        });
    }
    if path_submission_id != body_submission_id {
        return Err(WorkspaceFormsApiError::SubmissionIdMismatch {
            path_submission_id: path_submission_id.to_string(),
            body_submission_id: body_submission_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    header_tenant_id: &str,
    principal: &WorkspaceFormsApiPrincipal,
    authorization: &WorkspaceFormsApiAuthorization,
    body_tenant_id: Option<&str>,
    resource_tenant_id: Option<&str>,
) -> Result<(), WorkspaceFormsApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(WorkspaceFormsApiError::EmptyPrincipalId);
    }
    if header_tenant_id != principal.tenant_id
        || header_tenant_id != authorization.tenant_id
        || body_tenant_id.is_some_and(|tenant_id| header_tenant_id != tenant_id)
        || resource_tenant_id.is_some_and(|tenant_id| header_tenant_id != tenant_id)
    {
        return Err(WorkspaceFormsApiError::TenantMismatch {
            header_tenant_id: header_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            authorization_tenant_id: Some(authorization.tenant_id.clone()),
            body_tenant_id: body_tenant_id.map(str::to_string),
            resource_tenant_id: resource_tenant_id.map(str::to_string),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &WorkspaceFormsApiPrincipal,
    authorization: &WorkspaceFormsApiAuthorization,
    surface: &str,
) -> Result<(), WorkspaceFormsApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(WorkspaceFormsApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(WorkspaceFormsApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(WorkspaceFormsApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed| allowed == surface)
    {
        return Err(WorkspaceFormsApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn form_from_seed(seed: WorkspaceFormsFormSeed) -> Result<Form, WorkspaceFormsApiError> {
    let data_class = parse_privacy_data_class_label(&seed.data_class)?;
    let fields = seed
        .fields
        .into_iter()
        .map(field_from_seed)
        .collect::<Result<Vec<_>, _>>()?;
    Form::new(FormCreate {
        id: seed.form_id,
        tenant_id: seed.tenant_id,
        region: seed.region,
        cell_id: seed.cell_id,
        object_graph_route_id: seed.object_graph_route_id,
        title: seed.title,
        data_class: Some(data_class),
        fields,
        created_at_epoch_seconds: seed.created_at_epoch_seconds,
    })
    .map_err(WorkspaceFormsApiError::Form)
}

fn field_from_seed(seed: WorkspaceFormsFieldSeed) -> Result<FormField, WorkspaceFormsApiError> {
    FormField::new(FormFieldCreate {
        field_id: seed.field_id,
        label: seed.label,
        kind: field_kind_from_label(&seed.kind)?,
        required: seed.required,
        choice_options: seed.choice_options,
    })
    .map_err(WorkspaceFormsApiError::Form)
}

fn form_submission_from_request(
    request: WorkspaceFormsSubmissionIngestRequest,
    form: &Form,
) -> Result<FormSubmission, WorkspaceFormsApiError> {
    let data_class = parse_privacy_data_class_label(&request.data_class)?;
    let answers = request
        .answers
        .into_iter()
        .map(answer_from_request)
        .collect::<Result<Vec<_>, _>>()?;
    FormSubmission::new(
        FormSubmissionCreate {
            submission_id: request.submission_id,
            form_id: request.form_id,
            tenant_id: request.tenant_id,
            submitter_ref: request.submitter_ref,
            answers,
            data_class: Some(data_class),
            submitted_at_epoch_seconds: request.submitted_at_epoch_seconds,
        },
        form,
    )
    .map_err(WorkspaceFormsApiError::Form)
}

fn answer_from_request(
    answer: WorkspaceFormsAnswerRequest,
) -> Result<FormAnswer, WorkspaceFormsApiError> {
    FormAnswer::new(FormAnswerCreate {
        field_id: answer.field_id,
        value_kind: field_kind_from_label(&answer.value_kind)?,
        value: answer.value,
    })
    .map_err(WorkspaceFormsApiError::Form)
}

fn parse_privacy_data_class_label(
    data_class: &str,
) -> Result<data_boundary_kernel::PrivacyDataClass, WorkspaceFormsApiError> {
    let parsed = parse_data_class_label(data_class).ok_or_else(|| {
        WorkspaceFormsApiError::InvalidDataClassLabel {
            data_class: data_class.to_string(),
        }
    })?;
    workspace_form_data_class_from_legacy(parsed).map_err(|_| {
        WorkspaceFormsApiError::InvalidDataClassLabel {
            data_class: data_class.to_string(),
        }
    })
}

fn field_kind_from_label(kind: &str) -> Result<FormFieldKind, WorkspaceFormsApiError> {
    match kind.trim() {
        "short_text" => Ok(FormFieldKind::ShortText),
        "long_text" => Ok(FormFieldKind::LongText),
        "number" => Ok(FormFieldKind::Number),
        "boolean" => Ok(FormFieldKind::Boolean),
        "single_choice" => Ok(FormFieldKind::SingleChoice),
        "multi_choice" => Ok(FormFieldKind::MultiChoice),
        _ => Err(WorkspaceFormsApiError::InvalidFieldKind {
            kind: kind.to_string(),
        }),
    }
}

fn field_kind_label(kind: FormFieldKind) -> &'static str {
    match kind {
        FormFieldKind::ShortText => "short_text",
        FormFieldKind::LongText => "long_text",
        FormFieldKind::Number => "number",
        FormFieldKind::Boolean => "boolean",
        FormFieldKind::SingleChoice => "single_choice",
        FormFieldKind::MultiChoice => "multi_choice",
    }
}

fn require_submitter(
    principal_id: &str,
    submitter_ref: &str,
    form_id: &str,
) -> Result<(), WorkspaceFormsApiError> {
    if principal_id == submitter_ref {
        return Ok(());
    }
    Err(WorkspaceFormsApiError::SubmitterPermissionDenied {
        principal_id: principal_id.to_string(),
        submitter_ref: submitter_ref.to_string(),
        form_id: form_id.to_string(),
    })
}

fn idempotency_key_for(
    boundary: &WorkspaceFormsIngestBoundaryContext,
    principal: &WorkspaceFormsApiPrincipal,
    surface: &str,
) -> WorkspaceFormsIdempotencyLedgerKey {
    WorkspaceFormsIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn ingest_fingerprint_for(
    request: &WorkspaceFormsSubmissionIngestApiRequest,
) -> WorkspaceFormsRequestFingerprint {
    let answers = request
        .body
        .answers
        .iter()
        .map(|answer| format!("{}:{}:{}", answer.field_id, answer.value_kind, answer.value))
        .collect::<Vec<_>>()
        .join(",");
    WorkspaceFormsRequestFingerprint {
        canonical: [
            format!("path.form_id={}", request.path_form_id),
            format!("path.submission_id={}", request.path_submission_id),
            format!("header.tenant_id={}", request.boundary.tenant_id),
            format!("principal.tenant_id={}", request.principal.tenant_id),
            format!("principal.principal_id={}", request.principal.principal_id),
            format!(
                "authorization.tenant_id={}",
                request.authorization.tenant_id
            ),
            format!(
                "authorization.principal_id={}",
                request.authorization.principal_id
            ),
            format!(
                "authorization.decision_id={}",
                request.authorization.decision_id
            ),
            format!(
                "authorization.allowed_surfaces={}",
                request.authorization.allowed_surfaces.join(",")
            ),
            format!("body.submission_id={}", request.body.submission_id),
            format!("body.form_id={}", request.body.form_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.submitter_ref={}", request.body.submitter_ref),
            format!("body.answers={answers}"),
            format!("body.data_class={}", request.body.data_class),
            format!(
                "body.submitted_at_epoch_seconds={}",
                request.body.submitted_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn submission_record(
    submission: &FormSubmission,
    object_graph_route_id: &str,
) -> WorkspaceFormsSubmissionRecord {
    WorkspaceFormsSubmissionRecord {
        submission_id: submission.submission_id.value.clone(),
        form_id: submission.form_id.value.clone(),
        tenant_id: submission.tenant_id.value.clone(),
        submitter_ref: submission.submitter_ref.value.clone(),
        answers: submission
            .answers
            .value
            .iter()
            .map(|answer| WorkspaceFormsAnswerRecord {
                field_id: answer.field_id.value.clone(),
                value_kind: field_kind_label(answer.value_kind.value).to_string(),
                value: answer.value.value.clone(),
            })
            .collect(),
        data_class: submission.privacy_data_class().label().to_string(),
        submitted_at_epoch_seconds: submission.submitted_at_epoch_seconds.value,
        object_graph_route_id: object_graph_route_id.to_string(),
        schema_version: submission.schema_version.value,
    }
}

fn detail(field: &str, issue: &str) -> WorkspaceFormsApiErrorDetail {
    WorkspaceFormsApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}

fn form_error_message(error: &FormError) -> &'static str {
    match error {
        FormError::InvalidFormId => "Workspace Forms form id is invalid",
        FormError::InvalidTenantId => "Workspace Forms tenant id is invalid",
        FormError::InvalidRegion => "Workspace Forms region is invalid",
        FormError::InvalidCellId => "Workspace Forms cell id is invalid",
        FormError::InvalidObjectGraphRouteId => "Workspace Forms Object Graph route id is invalid",
        FormError::InvalidTitle => "Workspace Forms title is invalid",
        FormError::EmptyFieldSet => "Workspace Forms field set is required",
        FormError::InvalidFieldId => "Workspace Forms field id is invalid",
        FormError::InvalidFieldLabel => "Workspace Forms field label is invalid",
        FormError::DuplicateFieldId => "Workspace Forms field ids must be unique",
        FormError::MissingChoiceOptions => "Workspace Forms choice options are required",
        FormError::UnexpectedChoiceOptions => {
            "Workspace Forms non-choice fields cannot have options"
        }
        FormError::InvalidChoiceOption => "Workspace Forms choice option is invalid",
        FormError::InvalidSubmissionId => "Workspace Forms submission id is invalid",
        FormError::InvalidSubmitterRef => "Workspace Forms submitter ref is invalid",
        FormError::InvalidAnswerValue => "Workspace Forms answer value is invalid",
        FormError::DuplicateAnswerField => "Workspace Forms answers must target fields once",
        FormError::UnknownAnswerField => "Workspace Forms answer targets an unknown field",
        FormError::MissingRequiredAnswer => "Workspace Forms required answer is missing",
        FormError::AnswerKindMismatch => "Workspace Forms answer kind does not match field schema",
        FormError::SubmissionFormMismatch => "Workspace Forms submission form does not match",
        FormError::SubmissionTenantMismatch => "Workspace Forms submission tenant does not match",
        FormError::InvalidDataClass => "Workspace Forms data class is invalid",
    }
}

fn form_error_issue(error: &FormError) -> &'static str {
    match error {
        FormError::InvalidFormId => "form id must be non-empty",
        FormError::InvalidTenantId => "tenant id must be non-empty",
        FormError::InvalidRegion => "region must be non-empty",
        FormError::InvalidCellId => "cell id must be non-empty",
        FormError::InvalidObjectGraphRouteId => "object graph route id must be non-empty",
        FormError::InvalidTitle => "title must be trimmed and printable",
        FormError::EmptyFieldSet => "form fields must be non-empty",
        FormError::InvalidFieldId => "field id must be non-empty",
        FormError::InvalidFieldLabel => "field label must be trimmed and printable",
        FormError::DuplicateFieldId => "field ids must be unique",
        FormError::MissingChoiceOptions => "choice fields require options",
        FormError::UnexpectedChoiceOptions => "non-choice fields cannot include options",
        FormError::InvalidChoiceOption => "choice options must be trimmed and printable",
        FormError::InvalidSubmissionId => "submission id must be non-empty",
        FormError::InvalidSubmitterRef => "submitter ref must be non-empty",
        FormError::InvalidAnswerValue => "answer value must match its declared kind",
        FormError::DuplicateAnswerField => "answers cannot repeat a field id",
        FormError::UnknownAnswerField => "answers must target schema fields",
        FormError::MissingRequiredAnswer => "all required schema fields need answers",
        FormError::AnswerKindMismatch => "answer value_kind must match the form field kind",
        FormError::SubmissionFormMismatch => "submission form_id must match the form schema",
        FormError::SubmissionTenantMismatch => "submission tenant_id must match the form schema",
        FormError::InvalidDataClass => "data class must be a privacy data class",
    }
}
