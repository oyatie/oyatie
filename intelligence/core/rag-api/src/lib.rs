//! Foundry RAG retrieval API boundary.
//!
//! This crate owns the stable `foundry.rag.retrieve` REST boundary: tenant
//! scoping, Foundry authorization evidence, purpose-bound consent receipts,
//! data-class allowlists, idempotent replay, and citation-shaped responses
//! before an eventual search-axis adapter is called.

use std::collections::{BTreeMap, BTreeSet};

use data_boundary_kernel::{
    ConsentScope, PrivacyDataClass, Purpose, is_hard_denied_classification, parse_data_class_label,
    parse_purpose_pascal_label,
};

const FOUNDRY_RAG_RETRIEVE_SCHEMA_VERSION: u32 = 1;
const FOUNDRY_RAG_MAX_TOP_K: u32 = 50;

pub const FOUNDRY_RAG_RETRIEVE_SURFACE: &str = "foundry.rag.retrieve";
pub const FOUNDRY_RAG_OPENAPI_CONTRACT: &str = "contracts/openapi/foundry/rag-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoundryRagRetrieveApiStatus {
    Ok,
    BadRequest,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
}

impl FoundryRagRetrieveApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagConsentReceiptRequest {
    pub receipt_id: String,           // data_class: INTERNAL_ONLY
    pub purpose: String,              // data_class: INTERNAL_ONLY
    pub data_class: String,           // data_class: INTERNAL_ONLY
    pub subject_id: String,           // data_class: PII_QUASI_IDENTIFIER
    pub issued_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagRetrieveRequest {
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub namespace: String,                 // data_class: INTERNAL_ONLY
    pub capability_id: String,             // data_class: INTERNAL_ONLY
    pub query: String,                     // data_class: SEARCH_QUERY
    pub top_k: u32,                        // data_class: INTERNAL_ONLY
    pub data_use_purpose: String,          // data_class: INTERNAL_ONLY
    pub allowed_data_classes: Vec<String>, // data_class: INTERNAL_ONLY
    pub consent_receipts: Vec<FoundryRagConsentReceiptRequest>, // data_class: INTERNAL_ONLY
    pub search_index_id: String,           // data_class: INTERNAL_ONLY
    pub index_tenant_id: String,           // data_class: INTERNAL_ONLY
    pub index_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
    pub retrieved_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagRetrieveApiRequest {
    pub path_namespace: String,                    // data_class: INTERNAL_ONLY
    pub boundary: FoundryRagApiBoundaryContext,    // data_class: INTERNAL_ONLY
    pub principal: FoundryRagApiPrincipal,         // data_class: INTERNAL_ONLY
    pub authorization: FoundryRagApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: FoundryRagRetrieveRequest,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagIndexedDocument {
    pub document_id: String,                // data_class: INTERNAL_ONLY
    pub chunk_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub namespace: String,                  // data_class: INTERNAL_ONLY
    pub title: String,                      // data_class: INTERNAL_ONLY
    pub uri: String,                        // data_class: INTERNAL_ONLY
    pub excerpt: String,                    // data_class: INTERNAL_ONLY
    pub data_class: String,                 // data_class: INTERNAL_ONLY
    pub consent_receipt_id: Option<String>, // data_class: INTERNAL_ONLY
    pub indexed_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FoundryRagRetrieveDirectory {
    documents: BTreeMap<(String, String), FoundryRagIndexedDocument>, // data_class: INTERNAL_ONLY
}

impl FoundryRagRetrieveDirectory {
    pub fn register_document(
        &mut self,
        document: FoundryRagIndexedDocument,
    ) -> Result<(), FoundryRagRetrieveApiError> {
        require_non_empty(&document.document_id, "document.document_id")?;
        require_non_empty(&document.chunk_id, "document.chunk_id")?;
        require_non_empty(&document.tenant_id, "document.tenant_id")?;
        require_non_empty(&document.namespace, "document.namespace")?;
        require_non_empty(&document.excerpt, "document.excerpt")?;
        parse_privacy_data_class(&document.data_class)?;
        self.documents.insert(
            (document.document_id.clone(), document.chunk_id.clone()),
            document,
        );
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    fn retrieve(
        &self,
        request: &FoundryRagRetrieveRequest,
        purpose: Purpose,
        allowed_classes: &BTreeSet<PrivacyDataClass>,
        consent_scope: &ConsentScope,
    ) -> Vec<FoundryRagCitationRecord> {
        let query = request.query.trim().to_lowercase();
        let terms: Vec<&str> = query.split_whitespace().collect();
        let mut matches = Vec::new();

        for document in self.documents.values() {
            if document.tenant_id != request.tenant_id
                || document.namespace != request.namespace
                || !document_matches_query(document, &query, &terms)
            {
                continue;
            }

            let Ok(document_class) = parse_privacy_data_class(&document.data_class) else {
                continue;
            };
            if !allowed_classes.contains(&document_class) {
                continue;
            }
            if !consent_scope.allows(purpose, document_class) {
                continue;
            }

            matches.push((score_document(document, &query, &terms), document));
        }

        matches.sort_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| left.1.document_id.cmp(&right.1.document_id))
                .then_with(|| left.1.chunk_id.cmp(&right.1.chunk_id))
        });

        matches
            .into_iter()
            .take(request.top_k as usize)
            .map(|(score_millis, document)| FoundryRagCitationRecord {
                document_id: document.document_id.clone(),
                chunk_id: document.chunk_id.clone(),
                tenant_id: document.tenant_id.clone(),
                namespace: document.namespace.clone(),
                title: document.title.clone(),
                uri: document.uri.clone(),
                excerpt: document.excerpt.clone(),
                data_class: document.data_class.clone(),
                consent_receipt_id: document.consent_receipt_id.clone(),
                score_millis,
                indexed_at_epoch_seconds: document.indexed_at_epoch_seconds,
            })
            .collect()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FoundryRagRetrieveIdempotencyLedger {
    entries: BTreeMap<FoundryRagRetrieveIdempotencyKey, FoundryRagRetrieveIdempotencyEntry>, // data_class: INTERNAL_ONLY
}

impl FoundryRagRetrieveIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct FoundryRagRetrieveIdempotencyKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundryRagRetrieveIdempotencyEntry {
    fingerprint: FoundryRagRetrieveFingerprint, // data_class: INTERNAL_ONLY
    result: FoundryRagRetrieveApiResult,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundryRagRetrieveFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type FoundryRagRetrieveApiResult =
    Result<FoundryRagRetrieveSuccessResponse, FoundryRagRetrieveApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagRetrieveSuccessResponse {
    pub data: FoundryRagRetrieveRecord, // data_class: INTERNAL_ONLY
    pub metadata: FoundryRagRetrieveMetadata, // data_class: INTERNAL_ONLY
}

impl FoundryRagRetrieveSuccessResponse {
    pub const fn status_code(&self) -> u16 {
        FoundryRagRetrieveApiStatus::Ok.code()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagRetrieveRecord {
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub namespace: String,                        // data_class: INTERNAL_ONLY
    pub capability_id: String,                    // data_class: INTERNAL_ONLY
    pub search_index_id: String,                  // data_class: INTERNAL_ONLY
    pub query_hash: String,                       // data_class: SEARCH_QUERY
    pub data_use_purpose: String,                 // data_class: INTERNAL_ONLY
    pub data_classes: Vec<String>,                // data_class: INTERNAL_ONLY
    pub citations: Vec<FoundryRagCitationRecord>, // data_class: INTERNAL_ONLY
    pub retrieved_at_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
    pub schema_version: u32,                      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagCitationRecord {
    pub document_id: String,                // data_class: INTERNAL_ONLY
    pub chunk_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub namespace: String,                  // data_class: INTERNAL_ONLY
    pub title: String,                      // data_class: INTERNAL_ONLY
    pub uri: String,                        // data_class: INTERNAL_ONLY
    pub excerpt: String,                    // data_class: INTERNAL_ONLY
    pub data_class: String,                 // data_class: INTERNAL_ONLY
    pub consent_receipt_id: Option<String>, // data_class: INTERNAL_ONLY
    pub score_millis: u32,                  // data_class: INTERNAL_ONLY
    pub indexed_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagRetrieveMetadata {
    pub request_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String,  // data_class: INTERNAL_ONLY
    pub surface: String,          // data_class: INTERNAL_ONLY
    pub openapi_contract: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagRetrieveApiErrorResponse {
    pub error: FoundryRagRetrieveApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagRetrieveApiErrorBody {
    pub code: String,                                   // data_class: INTERNAL_ONLY
    pub message: String,                                // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,              // data_class: INTERNAL_ONLY
    pub request_id: String,                             // data_class: INTERNAL_ONLY
    pub details: Vec<FoundryRagRetrieveApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryRagRetrieveApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FoundryRagRetrieveApiError {
    EmptyPathNamespace,
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalTenantId,
    EmptyPrincipalId,
    EmptyAuthorizationDecisionId,
    NamespaceMismatch {
        path_namespace: String, // data_class: INTERNAL_ONLY
        body_namespace: String, // data_class: INTERNAL_ONLY
    },
    TenantMismatch {
        header_tenant_id: String,        // data_class: INTERNAL_ONLY
        principal_tenant_id: String,     // data_class: INTERNAL_ONLY
        authorization_tenant_id: String, // data_class: INTERNAL_ONLY
        body_tenant_id: String,          // data_class: INTERNAL_ONLY
        index_tenant_id: String,         // data_class: INTERNAL_ONLY
    },
    AuthorizationPrincipalMismatch {
        principal_tenant_id: String,        // data_class: INTERNAL_ONLY
        principal_id: String,               // data_class: INTERNAL_ONLY
        authorization_tenant_id: String,    // data_class: INTERNAL_ONLY
        authorization_principal_id: String, // data_class: INTERNAL_ONLY
    },
    AuthorizationSurfaceDenied {
        decision_id: String, // data_class: INTERNAL_ONLY
        surface: String,     // data_class: INTERNAL_ONLY
    },
    EmptyCapabilityId,
    EmptySearchIndexId,
    EmptyQuery,
    InvalidTopK {
        top_k: u32,     // data_class: INTERNAL_ONLY
        max_top_k: u32, // data_class: INTERNAL_ONLY
    },
    InvalidPurpose {
        purpose: String, // data_class: INTERNAL_ONLY
    },
    EmptyAllowedDataClasses,
    InvalidDataClassLabel {
        data_class: String, // data_class: INTERNAL_ONLY
    },
    DataClassHardDenied {
        purpose: String,    // data_class: INTERNAL_ONLY
        data_class: String, // data_class: INTERNAL_ONLY
    },
    MissingConsentReceipt {
        purpose: String,    // data_class: INTERNAL_ONLY
        data_class: String, // data_class: INTERNAL_ONLY
    },
    InvalidConsentReceipt {
        receipt_id: String, // data_class: INTERNAL_ONLY
        reason: String,     // data_class: INTERNAL_ONLY
    },
    IdempotencyKeyReused {
        idempotency_key: String, // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoundryRagRetrieveApiErrorCode {
    EmptyPathNamespace,
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalTenantId,
    EmptyPrincipalId,
    EmptyAuthorizationDecisionId,
    NamespaceMismatch,
    TenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationSurfaceDenied,
    EmptyCapabilityId,
    EmptySearchIndexId,
    EmptyQuery,
    InvalidTopK,
    InvalidPurpose,
    EmptyAllowedDataClasses,
    InvalidDataClassLabel,
    DataClassHardDenied,
    MissingConsentReceipt,
    InvalidConsentReceipt,
    IdempotencyKeyReused,
}

impl FoundryRagRetrieveApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyPathNamespace => "FOUNDRY_RAG_PATH_NAMESPACE_EMPTY",
            Self::EmptyRequestId => "FOUNDRY_RAG_REQUEST_ID_EMPTY",
            Self::EmptyTenantHeader => "FOUNDRY_RAG_TENANT_HEADER_EMPTY",
            Self::EmptyIdempotencyKey => "FOUNDRY_RAG_IDEMPOTENCY_KEY_EMPTY",
            Self::EmptyPrincipalTenantId => "FOUNDRY_RAG_PRINCIPAL_TENANT_ID_EMPTY",
            Self::EmptyPrincipalId => "FOUNDRY_RAG_PRINCIPAL_ID_EMPTY",
            Self::EmptyAuthorizationDecisionId => "FOUNDRY_RAG_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::NamespaceMismatch => "FOUNDRY_RAG_NAMESPACE_MISMATCH",
            Self::TenantMismatch => "FOUNDRY_RAG_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "FOUNDRY_RAG_AUTHORIZATION_PRINCIPAL_MISMATCH",
            Self::AuthorizationSurfaceDenied => "FOUNDRY_RAG_AUTHORIZATION_SURFACE_DENIED",
            Self::EmptyCapabilityId => "FOUNDRY_RAG_CAPABILITY_ID_EMPTY",
            Self::EmptySearchIndexId => "FOUNDRY_RAG_SEARCH_INDEX_ID_EMPTY",
            Self::EmptyQuery => "FOUNDRY_RAG_QUERY_EMPTY",
            Self::InvalidTopK => "FOUNDRY_RAG_TOP_K_INVALID",
            Self::InvalidPurpose => "FOUNDRY_RAG_PURPOSE_INVALID",
            Self::EmptyAllowedDataClasses => "FOUNDRY_RAG_ALLOWED_DATA_CLASSES_EMPTY",
            Self::InvalidDataClassLabel => "FOUNDRY_RAG_DATA_CLASS_INVALID",
            Self::DataClassHardDenied => "FOUNDRY_RAG_DATA_CLASS_HARD_DENIED",
            Self::MissingConsentReceipt => "FOUNDRY_RAG_CONSENT_RECEIPT_MISSING",
            Self::InvalidConsentReceipt => "FOUNDRY_RAG_CONSENT_RECEIPT_INVALID",
            Self::IdempotencyKeyReused => "FOUNDRY_RAG_IDEMPOTENCY_KEY_REUSED",
        }
    }
}

impl FoundryRagRetrieveApiError {
    pub fn status(&self) -> FoundryRagRetrieveApiStatus {
        match self {
            Self::EmptyPrincipalTenantId | Self::EmptyPrincipalId => {
                FoundryRagRetrieveApiStatus::Unauthorized
            }
            Self::TenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationSurfaceDenied { .. }
            | Self::MissingConsentReceipt { .. } => FoundryRagRetrieveApiStatus::Forbidden,
            Self::InvalidPurpose { .. }
            | Self::EmptyAllowedDataClasses
            | Self::InvalidDataClassLabel { .. }
            | Self::DataClassHardDenied { .. }
            | Self::InvalidConsentReceipt { .. }
            | Self::IdempotencyKeyReused { .. } => FoundryRagRetrieveApiStatus::UnprocessableEntity,
            Self::EmptyPathNamespace
            | Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyAuthorizationDecisionId
            | Self::NamespaceMismatch { .. }
            | Self::EmptyCapabilityId
            | Self::EmptySearchIndexId
            | Self::EmptyQuery
            | Self::InvalidTopK { .. } => FoundryRagRetrieveApiStatus::BadRequest,
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status().code()
    }

    pub fn code(&self) -> FoundryRagRetrieveApiErrorCode {
        match self {
            Self::EmptyPathNamespace => FoundryRagRetrieveApiErrorCode::EmptyPathNamespace,
            Self::EmptyRequestId => FoundryRagRetrieveApiErrorCode::EmptyRequestId,
            Self::EmptyTenantHeader => FoundryRagRetrieveApiErrorCode::EmptyTenantHeader,
            Self::EmptyIdempotencyKey => FoundryRagRetrieveApiErrorCode::EmptyIdempotencyKey,
            Self::EmptyPrincipalTenantId => FoundryRagRetrieveApiErrorCode::EmptyPrincipalTenantId,
            Self::EmptyPrincipalId => FoundryRagRetrieveApiErrorCode::EmptyPrincipalId,
            Self::EmptyAuthorizationDecisionId => {
                FoundryRagRetrieveApiErrorCode::EmptyAuthorizationDecisionId
            }
            Self::NamespaceMismatch { .. } => FoundryRagRetrieveApiErrorCode::NamespaceMismatch,
            Self::TenantMismatch { .. } => FoundryRagRetrieveApiErrorCode::TenantMismatch,
            Self::AuthorizationPrincipalMismatch { .. } => {
                FoundryRagRetrieveApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationSurfaceDenied { .. } => {
                FoundryRagRetrieveApiErrorCode::AuthorizationSurfaceDenied
            }
            Self::EmptyCapabilityId => FoundryRagRetrieveApiErrorCode::EmptyCapabilityId,
            Self::EmptySearchIndexId => FoundryRagRetrieveApiErrorCode::EmptySearchIndexId,
            Self::EmptyQuery => FoundryRagRetrieveApiErrorCode::EmptyQuery,
            Self::InvalidTopK { .. } => FoundryRagRetrieveApiErrorCode::InvalidTopK,
            Self::InvalidPurpose { .. } => FoundryRagRetrieveApiErrorCode::InvalidPurpose,
            Self::EmptyAllowedDataClasses => {
                FoundryRagRetrieveApiErrorCode::EmptyAllowedDataClasses
            }
            Self::InvalidDataClassLabel { .. } => {
                FoundryRagRetrieveApiErrorCode::InvalidDataClassLabel
            }
            Self::DataClassHardDenied { .. } => FoundryRagRetrieveApiErrorCode::DataClassHardDenied,
            Self::MissingConsentReceipt { .. } => {
                FoundryRagRetrieveApiErrorCode::MissingConsentReceipt
            }
            Self::InvalidConsentReceipt { .. } => {
                FoundryRagRetrieveApiErrorCode::InvalidConsentReceipt
            }
            Self::IdempotencyKeyReused { .. } => {
                FoundryRagRetrieveApiErrorCode::IdempotencyKeyReused
            }
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::EmptyPathNamespace => "path namespace is required",
            Self::EmptyRequestId => "request id is required",
            Self::EmptyTenantHeader => "tenant header is required",
            Self::EmptyIdempotencyKey => "idempotency key is required",
            Self::EmptyPrincipalTenantId => "principal tenant id is required",
            Self::EmptyPrincipalId => "principal id is required",
            Self::EmptyAuthorizationDecisionId => "authorization decision id is required",
            Self::NamespaceMismatch { .. } => "path namespace must match body namespace",
            Self::TenantMismatch { .. } => {
                "tenant, principal, authorization, body, and index tenant must match"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "authorization principal must match the request principal"
            }
            Self::AuthorizationSurfaceDenied { .. } => {
                "authorization did not allow Foundry RAG retrieval"
            }
            Self::EmptyCapabilityId => "capability id is required",
            Self::EmptySearchIndexId => "search index id is required",
            Self::EmptyQuery => "query is required",
            Self::InvalidTopK { .. } => "top_k must be between 1 and 50",
            Self::InvalidPurpose { .. } => {
                "data-use purpose must be SearchIndex, SearchIndexPrivate, or SearchIndexPublic"
            }
            Self::EmptyAllowedDataClasses => "at least one allowed data class is required",
            Self::InvalidDataClassLabel { .. } => "data class label is not a privacy-program class",
            Self::DataClassHardDenied { .. } => {
                "data class is hard-denied for the requested RAG purpose"
            }
            Self::MissingConsentReceipt { .. } => {
                "purpose-bound consent receipt is required for the data class"
            }
            Self::InvalidConsentReceipt { .. } => "consent receipt is invalid",
            Self::IdempotencyKeyReused { .. } => {
                "idempotency key was reused with a different RAG request"
            }
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> FoundryRagRetrieveApiErrorResponse {
        FoundryRagRetrieveApiErrorResponse {
            error: FoundryRagRetrieveApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn details(&self) -> Vec<FoundryRagRetrieveApiErrorDetail> {
        match self {
            Self::NamespaceMismatch {
                path_namespace,
                body_namespace,
            } => vec![FoundryRagRetrieveApiErrorDetail {
                field: "namespace".to_string(),
                issue: format!(
                    "path namespace `{path_namespace}` did not match body namespace `{body_namespace}`"
                ),
            }],
            Self::TenantMismatch {
                header_tenant_id,
                principal_tenant_id,
                authorization_tenant_id,
                body_tenant_id,
                index_tenant_id,
            } => vec![FoundryRagRetrieveApiErrorDetail {
                field: "tenant_id".to_string(),
                issue: format!(
                    "header `{header_tenant_id}`, principal `{principal_tenant_id}`, authorization `{authorization_tenant_id}`, body `{body_tenant_id}`, and index `{index_tenant_id}` must match"
                ),
            }],
            Self::AuthorizationPrincipalMismatch {
                principal_tenant_id,
                principal_id,
                authorization_tenant_id,
                authorization_principal_id,
            } => vec![FoundryRagRetrieveApiErrorDetail {
                field: "authorization.principal_id".to_string(),
                issue: format!(
                    "principal `{principal_tenant_id}/{principal_id}` did not match authorization `{authorization_tenant_id}/{authorization_principal_id}`"
                ),
            }],
            Self::AuthorizationSurfaceDenied {
                decision_id,
                surface,
            } => {
                vec![FoundryRagRetrieveApiErrorDetail {
                    field: "authorization.allowed_surfaces".to_string(),
                    issue: format!("decision `{decision_id}` did not allow `{surface}`"),
                }]
            }
            Self::InvalidTopK { top_k, max_top_k } => vec![FoundryRagRetrieveApiErrorDetail {
                field: "top_k".to_string(),
                issue: format!("received `{top_k}`, maximum is `{max_top_k}`"),
            }],
            Self::InvalidPurpose { purpose } => vec![FoundryRagRetrieveApiErrorDetail {
                field: "data_use_purpose".to_string(),
                issue: format!("`{purpose}` is not an accepted RAG retrieval purpose"),
            }],
            Self::InvalidDataClassLabel { data_class } => vec![FoundryRagRetrieveApiErrorDetail {
                field: "data_class".to_string(),
                issue: format!("`{data_class}` is not a privacy-program data class"),
            }],
            Self::DataClassHardDenied {
                purpose,
                data_class,
            } => {
                vec![FoundryRagRetrieveApiErrorDetail {
                    field: "allowed_data_classes".to_string(),
                    issue: format!("`{data_class}` is hard-denied for `{purpose}`"),
                }]
            }
            Self::MissingConsentReceipt {
                purpose,
                data_class,
            } => {
                vec![FoundryRagRetrieveApiErrorDetail {
                    field: "consent_receipts".to_string(),
                    issue: format!("`{data_class}` needs a consent receipt for `{purpose}`"),
                }]
            }
            Self::InvalidConsentReceipt { receipt_id, reason } => {
                vec![FoundryRagRetrieveApiErrorDetail {
                    field: "consent_receipts".to_string(),
                    issue: format!("receipt `{receipt_id}` is invalid: {reason}"),
                }]
            }
            Self::IdempotencyKeyReused { idempotency_key } => {
                vec![FoundryRagRetrieveApiErrorDetail {
                    field: "Idempotency-Key".to_string(),
                    issue: format!("`{idempotency_key}` already protects a different request"),
                }]
            }
            _ => Vec::new(),
        }
    }
}

pub fn retrieve_foundry_rag_from_api(
    request: FoundryRagRetrieveApiRequest,
    directory: &FoundryRagRetrieveDirectory,
    ledger: &mut FoundryRagRetrieveIdempotencyLedger,
) -> FoundryRagRetrieveApiResult {
    validate_request(&request)?;

    let key = FoundryRagRetrieveIdempotencyKey {
        tenant_id: request.boundary.tenant_id.clone(),
        principal_id: request.principal.principal_id.clone(),
        surface: FOUNDRY_RAG_RETRIEVE_SURFACE.to_string(),
        idempotency_key: request.boundary.idempotency_key.clone(),
    };
    let fingerprint = FoundryRagRetrieveFingerprint::from_request(&request);
    if let Some(entry) = ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(FoundryRagRetrieveApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let result = retrieve_validated(request, directory);
    ledger.entries.insert(
        key,
        FoundryRagRetrieveIdempotencyEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn retrieve_validated(
    request: FoundryRagRetrieveApiRequest,
    directory: &FoundryRagRetrieveDirectory,
) -> FoundryRagRetrieveApiResult {
    let purpose = parse_retrieval_purpose(&request.body.data_use_purpose)?;
    let allowed_classes =
        validate_allowed_data_classes(&request.body.allowed_data_classes, purpose)?;
    let consent_scope = validate_consent_scope(&request.body.consent_receipts, purpose)?;
    require_consents_for_allowed_classes(&allowed_classes, &consent_scope, purpose)?;

    let citations = directory.retrieve(&request.body, purpose, &allowed_classes, &consent_scope);
    let mut data_classes: Vec<String> = allowed_classes
        .iter()
        .map(|data_class| data_class.label().to_string())
        .collect();
    data_classes.sort();
    data_classes.dedup();

    Ok(FoundryRagRetrieveSuccessResponse {
        data: FoundryRagRetrieveRecord {
            tenant_id: request.body.tenant_id.clone(),
            namespace: request.body.namespace.clone(),
            capability_id: request.body.capability_id.clone(),
            search_index_id: request.body.search_index_id.clone(),
            query_hash: stable_query_hash(&request.body.query),
            data_use_purpose: purpose.pascal_label().to_string(),
            data_classes,
            citations,
            retrieved_at_epoch_seconds: request.body.retrieved_at_epoch_seconds,
            schema_version: FOUNDRY_RAG_RETRIEVE_SCHEMA_VERSION,
        },
        metadata: FoundryRagRetrieveMetadata {
            request_id: request.boundary.request_id,
            idempotency_key: request.boundary.idempotency_key,
            surface: FOUNDRY_RAG_RETRIEVE_SURFACE.to_string(),
            openapi_contract: FOUNDRY_RAG_OPENAPI_CONTRACT.to_string(),
        },
    })
}

fn validate_request(
    request: &FoundryRagRetrieveApiRequest,
) -> Result<(), FoundryRagRetrieveApiError> {
    if request.path_namespace.trim().is_empty() {
        return Err(FoundryRagRetrieveApiError::EmptyPathNamespace);
    }
    if request.boundary.request_id.trim().is_empty() {
        return Err(FoundryRagRetrieveApiError::EmptyRequestId);
    }
    if request.boundary.tenant_id.trim().is_empty() {
        return Err(FoundryRagRetrieveApiError::EmptyTenantHeader);
    }
    if request.boundary.idempotency_key.trim().is_empty() {
        return Err(FoundryRagRetrieveApiError::EmptyIdempotencyKey);
    }
    if request.principal.tenant_id.trim().is_empty() {
        return Err(FoundryRagRetrieveApiError::EmptyPrincipalTenantId);
    }
    if request.principal.principal_id.trim().is_empty() {
        return Err(FoundryRagRetrieveApiError::EmptyPrincipalId);
    }
    if request.authorization.decision_id.trim().is_empty() {
        return Err(FoundryRagRetrieveApiError::EmptyAuthorizationDecisionId);
    }
    if request.path_namespace != request.body.namespace {
        return Err(FoundryRagRetrieveApiError::NamespaceMismatch {
            path_namespace: request.path_namespace.clone(),
            body_namespace: request.body.namespace.clone(),
        });
    }
    if request.boundary.tenant_id != request.principal.tenant_id
        || request.boundary.tenant_id != request.authorization.tenant_id
        || request.boundary.tenant_id != request.body.tenant_id
        || request.boundary.tenant_id != request.body.index_tenant_id
    {
        return Err(FoundryRagRetrieveApiError::TenantMismatch {
            header_tenant_id: request.boundary.tenant_id.clone(),
            principal_tenant_id: request.principal.tenant_id.clone(),
            authorization_tenant_id: request.authorization.tenant_id.clone(),
            body_tenant_id: request.body.tenant_id.clone(),
            index_tenant_id: request.body.index_tenant_id.clone(),
        });
    }
    if request.authorization.tenant_id != request.principal.tenant_id
        || request.authorization.principal_id != request.principal.principal_id
    {
        return Err(FoundryRagRetrieveApiError::AuthorizationPrincipalMismatch {
            principal_tenant_id: request.principal.tenant_id.clone(),
            principal_id: request.principal.principal_id.clone(),
            authorization_tenant_id: request.authorization.tenant_id.clone(),
            authorization_principal_id: request.authorization.principal_id.clone(),
        });
    }
    if !request
        .authorization
        .allowed_surfaces
        .iter()
        .any(|surface| surface == FOUNDRY_RAG_RETRIEVE_SURFACE)
    {
        return Err(FoundryRagRetrieveApiError::AuthorizationSurfaceDenied {
            decision_id: request.authorization.decision_id.clone(),
            surface: FOUNDRY_RAG_RETRIEVE_SURFACE.to_string(),
        });
    }
    require_non_empty(&request.body.capability_id, "capability_id")?;
    require_non_empty(&request.body.search_index_id, "search_index_id")?;
    if request.body.query.trim().is_empty() {
        return Err(FoundryRagRetrieveApiError::EmptyQuery);
    }
    if request.body.top_k == 0 || request.body.top_k > FOUNDRY_RAG_MAX_TOP_K {
        return Err(FoundryRagRetrieveApiError::InvalidTopK {
            top_k: request.body.top_k,
            max_top_k: FOUNDRY_RAG_MAX_TOP_K,
        });
    }
    Ok(())
}

fn require_non_empty(value: &str, field: &str) -> Result<(), FoundryRagRetrieveApiError> {
    if !value.trim().is_empty() {
        return Ok(());
    }
    match field {
        "capability_id" => Err(FoundryRagRetrieveApiError::EmptyCapabilityId),
        "search_index_id" => Err(FoundryRagRetrieveApiError::EmptySearchIndexId),
        _ => Err(FoundryRagRetrieveApiError::InvalidConsentReceipt {
            receipt_id: String::new(),
            reason: format!("{field} is required"),
        }),
    }
}

fn parse_retrieval_purpose(purpose: &str) -> Result<Purpose, FoundryRagRetrieveApiError> {
    let trimmed = purpose.trim();
    let parsed = match trimmed {
        "search_index" => Some(Purpose::SearchIndex),
        "search_index_private" => Some(Purpose::SearchIndexPrivate),
        "search_index_public" => Some(Purpose::SearchIndexPublic),
        other => parse_purpose_pascal_label(other),
    };
    match parsed {
        Some(Purpose::SearchIndex | Purpose::SearchIndexPrivate | Purpose::SearchIndexPublic) => {
            parsed.ok_or_else(|| FoundryRagRetrieveApiError::InvalidPurpose {
                purpose: purpose.to_string(),
            })
        }
        _ => Err(FoundryRagRetrieveApiError::InvalidPurpose {
            purpose: purpose.to_string(),
        }),
    }
}

fn validate_allowed_data_classes(
    labels: &[String],
    purpose: Purpose,
) -> Result<BTreeSet<PrivacyDataClass>, FoundryRagRetrieveApiError> {
    if labels.is_empty() {
        return Err(FoundryRagRetrieveApiError::EmptyAllowedDataClasses);
    }

    let mut classes = BTreeSet::new();
    for label in labels {
        let data_class = parse_privacy_data_class(label)?;
        if is_hard_denied_classification(purpose, data_class) {
            return Err(FoundryRagRetrieveApiError::DataClassHardDenied {
                purpose: purpose.pascal_label().to_string(),
                data_class: data_class.label().to_string(),
            });
        }
        classes.insert(data_class);
    }
    Ok(classes)
}

fn validate_consent_scope(
    receipts: &[FoundryRagConsentReceiptRequest],
    expected_purpose: Purpose,
) -> Result<ConsentScope, FoundryRagRetrieveApiError> {
    let mut scope = ConsentScope::default();
    for receipt in receipts {
        if receipt.receipt_id.trim().is_empty() {
            return Err(FoundryRagRetrieveApiError::InvalidConsentReceipt {
                receipt_id: receipt.receipt_id.clone(),
                reason: "receipt_id is required".to_string(),
            });
        }
        if receipt.subject_id.trim().is_empty() {
            return Err(FoundryRagRetrieveApiError::InvalidConsentReceipt {
                receipt_id: receipt.receipt_id.clone(),
                reason: "subject_id is required".to_string(),
            });
        }
        let receipt_purpose = parse_retrieval_purpose(&receipt.purpose).map_err(|_| {
            FoundryRagRetrieveApiError::InvalidConsentReceipt {
                receipt_id: receipt.receipt_id.clone(),
                reason: format!(
                    "purpose `{}` is not allowed for RAG retrieval",
                    receipt.purpose
                ),
            }
        })?;
        if receipt_purpose != expected_purpose {
            return Err(FoundryRagRetrieveApiError::InvalidConsentReceipt {
                receipt_id: receipt.receipt_id.clone(),
                reason: format!(
                    "purpose `{}` did not match request purpose `{}`",
                    receipt_purpose.pascal_label(),
                    expected_purpose.pascal_label()
                ),
            });
        }
        let data_class = parse_privacy_data_class(&receipt.data_class)?;
        if is_hard_denied_classification(expected_purpose, data_class) {
            return Err(FoundryRagRetrieveApiError::DataClassHardDenied {
                purpose: expected_purpose.pascal_label().to_string(),
                data_class: data_class.label().to_string(),
            });
        }
        scope = scope.allow(expected_purpose, data_class);
    }
    Ok(scope)
}

fn require_consents_for_allowed_classes(
    allowed_classes: &BTreeSet<PrivacyDataClass>,
    consent_scope: &ConsentScope,
    purpose: Purpose,
) -> Result<(), FoundryRagRetrieveApiError> {
    for data_class in allowed_classes {
        if !consent_scope.allows(purpose, *data_class) {
            return Err(FoundryRagRetrieveApiError::MissingConsentReceipt {
                purpose: purpose.pascal_label().to_string(),
                data_class: data_class.label().to_string(),
            });
        }
    }
    Ok(())
}

fn parse_privacy_data_class(label: &str) -> Result<PrivacyDataClass, FoundryRagRetrieveApiError> {
    let Some(data_class) = parse_data_class_label(label) else {
        return Err(FoundryRagRetrieveApiError::InvalidDataClassLabel {
            data_class: label.to_string(),
        });
    };
    PrivacyDataClass::try_from(data_class).map_err(|_| {
        FoundryRagRetrieveApiError::InvalidDataClassLabel {
            data_class: label.to_string(),
        }
    })
}

fn document_matches_query(
    document: &FoundryRagIndexedDocument,
    normalized_query: &str,
    terms: &[&str],
) -> bool {
    let haystack = format!("{} {}", document.title, document.excerpt).to_lowercase();
    haystack.contains(normalized_query) || terms.iter().any(|term| haystack.contains(term))
}

fn score_document(
    document: &FoundryRagIndexedDocument,
    normalized_query: &str,
    terms: &[&str],
) -> u32 {
    let haystack = format!("{} {}", document.title, document.excerpt).to_lowercase();
    let exact = u32::from(haystack.contains(normalized_query)) * 700;
    let term_score = terms
        .iter()
        .filter(|term| haystack.contains(**term))
        .count() as u32
        * 100;
    exact + term_score + (document.indexed_at_epoch_seconds % 100) as u32
}

fn stable_query_hash(query: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in query.trim().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

impl FoundryRagRetrieveFingerprint {
    fn from_request(request: &FoundryRagRetrieveApiRequest) -> Self {
        let mut receipts: Vec<String> = request
            .body
            .consent_receipts
            .iter()
            .map(|receipt| {
                format!(
                    "{}:{}:{}:{}:{}",
                    receipt.receipt_id,
                    receipt.purpose,
                    receipt.data_class,
                    receipt.subject_id,
                    receipt.issued_at_epoch_seconds
                )
            })
            .collect();
        receipts.sort();

        let mut allowed = request.body.allowed_data_classes.clone();
        allowed.sort();

        Self {
            canonical: format!(
                "path={}|tenant={}|principal={}/{}|auth={}/{}:{}:{:?}|body={}:{}:{}:{}:{}:{}:{:?}:{:?}:{}:{}:{}:{}",
                request.path_namespace,
                request.boundary.tenant_id,
                request.principal.tenant_id,
                request.principal.principal_id,
                request.authorization.tenant_id,
                request.authorization.principal_id,
                request.authorization.decision_id,
                request.authorization.allowed_surfaces,
                request.body.tenant_id,
                request.body.namespace,
                request.body.capability_id,
                request.body.query,
                request.body.top_k,
                request.body.data_use_purpose,
                allowed,
                receipts,
                request.body.search_index_id,
                request.body.index_tenant_id,
                request.body.index_epoch_seconds,
                request.body.retrieved_at_epoch_seconds,
            ),
        }
    }
}
