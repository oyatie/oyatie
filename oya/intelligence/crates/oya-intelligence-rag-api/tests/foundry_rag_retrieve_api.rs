// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_intelligence_rag_api::{
    FOUNDRY_RAG_OPENAPI_CONTRACT, FOUNDRY_RAG_RETRIEVE_SURFACE, FoundryRagApiAuthorization,
    FoundryRagApiBoundaryContext, FoundryRagApiPrincipal, FoundryRagConsentReceiptRequest,
    FoundryRagIndexedDocument, FoundryRagRetrieveApiError, FoundryRagRetrieveApiErrorCode,
    FoundryRagRetrieveApiRequest, FoundryRagRetrieveApiStatus, FoundryRagRetrieveDirectory,
    FoundryRagRetrieveIdempotencyLedger, FoundryRagRetrieveRequest, retrieve_foundry_rag_from_api,
};

fn valid_request() -> FoundryRagRetrieveApiRequest {
    FoundryRagRetrieveApiRequest {
        path_namespace: "foundry.workflow".to_string(),
        boundary: FoundryRagApiBoundaryContext {
            request_id: "req-rag-001".to_string(),
            tenant_id: "tenant-acme".to_string(),
            idempotency_key: "idem-rag-001".to_string(),
        },
        principal: FoundryRagApiPrincipal {
            tenant_id: "tenant-acme".to_string(),
            principal_id: "user-foundry-builder".to_string(),
        },
        authorization: FoundryRagApiAuthorization {
            tenant_id: "tenant-acme".to_string(),
            principal_id: "user-foundry-builder".to_string(),
            decision_id: "cedar-decision-rag-001".to_string(),
            allowed_surfaces: vec![FOUNDRY_RAG_RETRIEVE_SURFACE.to_string()],
        },
        body: FoundryRagRetrieveRequest {
            tenant_id: "tenant-acme".to_string(),
            namespace: "foundry.workflow".to_string(),
            capability_id: "cap.workflow.approve-payroll".to_string(),
            query: "payroll approval evidence".to_string(),
            top_k: 2,
            data_use_purpose: "SearchIndexPrivate".to_string(),
            allowed_data_classes: vec![
                "INTERNAL_ONLY".to_string(),
                "PII_QUASI_IDENTIFIER".to_string(),
            ],
            consent_receipts: vec![
                receipt("consent-internal", "INTERNAL_ONLY"),
                receipt("consent-quasi", "PII_QUASI_IDENTIFIER"),
            ],
            search_index_id: "idx-foundry-tenant-private".to_string(),
            index_tenant_id: "tenant-acme".to_string(),
            index_epoch_seconds: 1_800_000_000,
            retrieved_at_epoch_seconds: 1_800_000_123,
        },
    }
}

fn receipt(id: &str, data_class: &str) -> FoundryRagConsentReceiptRequest {
    FoundryRagConsentReceiptRequest {
        receipt_id: id.to_string(),
        purpose: "SearchIndexPrivate".to_string(),
        data_class: data_class.to_string(),
        subject_id: "subject-payroll-001".to_string(),
        issued_at_epoch_seconds: 1_799_999_999,
    }
}

fn seeded_directory() -> FoundryRagRetrieveDirectory {
    let mut directory = FoundryRagRetrieveDirectory::default();
    for document in [
        doc(
            "doc-payroll-1",
            "chunk-1",
            "tenant-acme",
            "foundry.workflow",
            "Payroll approval evidence",
            "oyatie://tenant-acme/payroll/evidence/1",
            "The payroll approval evidence cites the manager approval and audit chain.",
            "INTERNAL_ONLY",
            Some("consent-internal"),
            1_800_000_010,
        ),
        doc(
            "doc-payroll-2",
            "chunk-1",
            "tenant-acme",
            "foundry.workflow",
            "Payroll subject reference",
            "oyatie://tenant-acme/payroll/person/1",
            "Payroll approval evidence for a quasi identifier subject reference.",
            "PII_QUASI_IDENTIFIER",
            Some("consent-quasi"),
            1_800_000_020,
        ),
        doc(
            "doc-payroll-3",
            "chunk-1",
            "tenant-acme",
            "foundry.workflow",
            "Payroll public handbook",
            "oyatie://tenant-acme/payroll/handbook",
            "Payroll approval evidence public handbook.",
            "PUBLIC",
            None,
            1_800_000_030,
        ),
        doc(
            "doc-other-tenant",
            "chunk-1",
            "tenant-other",
            "foundry.workflow",
            "Payroll approval evidence leak",
            "oyatie://tenant-other/payroll/evidence/1",
            "This cross-tenant payroll approval evidence must never be returned.",
            "INTERNAL_ONLY",
            Some("consent-internal"),
            1_800_000_040,
        ),
    ] {
        directory.register_document(document).unwrap();
    }
    directory
}

#[allow(clippy::too_many_arguments)]
fn doc(
    document_id: &str,
    chunk_id: &str,
    tenant_id: &str,
    namespace: &str,
    title: &str,
    uri: &str,
    excerpt: &str,
    data_class: &str,
    consent_receipt_id: Option<&str>,
    indexed_at_epoch_seconds: u64,
) -> FoundryRagIndexedDocument {
    FoundryRagIndexedDocument {
        document_id: document_id.to_string(),
        chunk_id: chunk_id.to_string(),
        tenant_id: tenant_id.to_string(),
        namespace: namespace.to_string(),
        title: title.to_string(),
        uri: uri.to_string(),
        excerpt: excerpt.to_string(),
        data_class: data_class.to_string(),
        consent_receipt_id: consent_receipt_id.map(str::to_string),
        indexed_at_epoch_seconds,
    }
}

#[test]
fn exports_stable_surface_contract_and_status_codes() {
    assert_eq!(FOUNDRY_RAG_RETRIEVE_SURFACE, "foundry.rag.retrieve");
    assert_eq!(
        FOUNDRY_RAG_OPENAPI_CONTRACT,
        "contracts/openapi/foundry/rag-v1.yaml"
    );
    assert_eq!(FoundryRagRetrieveApiStatus::Ok.code(), 200);
    assert_eq!(FoundryRagRetrieveApiStatus::BadRequest.code(), 400);
    assert_eq!(FoundryRagRetrieveApiStatus::Unauthorized.code(), 401);
    assert_eq!(FoundryRagRetrieveApiStatus::Forbidden.code(), 403);
    assert_eq!(FoundryRagRetrieveApiStatus::UnprocessableEntity.code(), 422);
}

#[test]
fn retrieves_tenant_scoped_citations_and_replays_idempotently() {
    let directory = seeded_directory();
    let mut ledger = FoundryRagRetrieveIdempotencyLedger::default();
    let request = valid_request();

    let first = retrieve_foundry_rag_from_api(request.clone(), &directory, &mut ledger).unwrap();
    assert_eq!(first.status_code(), 200);
    assert_eq!(first.metadata.surface, FOUNDRY_RAG_RETRIEVE_SURFACE);
    assert_eq!(first.data.tenant_id, "tenant-acme");
    assert_eq!(first.data.namespace, "foundry.workflow");
    assert_eq!(first.data.citations.len(), 2);
    assert!(
        first
            .data
            .citations
            .iter()
            .all(|citation| citation.tenant_id == "tenant-acme")
    );
    assert!(
        first
            .data
            .citations
            .iter()
            .all(|citation| citation.data_class != "PUBLIC")
    );
    assert!(first.data.query_hash.starts_with("fnv1a64:"));
    assert_eq!(ledger.len(), 1);

    let mut replay_request = request;
    replay_request.boundary.request_id = "req-rag-002".to_string();
    let replay = retrieve_foundry_rag_from_api(replay_request, &directory, &mut ledger).unwrap();
    assert_eq!(replay, first);
    assert_eq!(ledger.len(), 1);
}

#[test]
fn rejects_namespace_tenant_and_authorization_drift_before_retrieval() {
    let directory = seeded_directory();
    let mut ledger = FoundryRagRetrieveIdempotencyLedger::default();

    let mut namespace_drift = valid_request();
    namespace_drift.path_namespace = "foundry.other".to_string();
    assert!(matches!(
        retrieve_foundry_rag_from_api(namespace_drift, &directory, &mut ledger),
        Err(FoundryRagRetrieveApiError::NamespaceMismatch { .. })
    ));
    assert_eq!(ledger.len(), 0);

    let mut tenant_drift = valid_request();
    tenant_drift.body.index_tenant_id = "tenant-other".to_string();
    assert!(matches!(
        retrieve_foundry_rag_from_api(tenant_drift, &directory, &mut ledger),
        Err(FoundryRagRetrieveApiError::TenantMismatch { .. })
    ));
    assert_eq!(ledger.len(), 0);

    let mut denied = valid_request();
    denied.authorization.allowed_surfaces = vec!["foundry.capability.invoke".to_string()];
    let error = retrieve_foundry_rag_from_api(denied, &directory, &mut ledger).unwrap_err();
    assert_eq!(
        error.code(),
        FoundryRagRetrieveApiErrorCode::AuthorizationSurfaceDenied
    );
    assert_eq!(error.status_code(), 403);
    assert_eq!(ledger.len(), 0);
}

#[test]
fn requires_purpose_bound_consent_for_every_allowed_class() {
    let directory = seeded_directory();
    let mut ledger = FoundryRagRetrieveIdempotencyLedger::default();

    let mut missing_consent = valid_request();
    missing_consent.body.consent_receipts.pop();
    let error =
        retrieve_foundry_rag_from_api(missing_consent, &directory, &mut ledger).unwrap_err();
    assert_eq!(
        error.code(),
        FoundryRagRetrieveApiErrorCode::MissingConsentReceipt
    );
    assert_eq!(error.status_code(), 403);

    let mut mismatched_purpose = valid_request();
    mismatched_purpose.boundary.idempotency_key = "idem-rag-mismatch-purpose".to_string();
    mismatched_purpose.body.consent_receipts[0].purpose = "SearchIndexPublic".to_string();
    let error =
        retrieve_foundry_rag_from_api(mismatched_purpose, &directory, &mut ledger).unwrap_err();
    assert_eq!(
        error.code(),
        FoundryRagRetrieveApiErrorCode::InvalidConsentReceipt
    );
    assert_eq!(error.status_code(), 422);
}

#[test]
fn rejects_invalid_or_hard_denied_data_classes() {
    let directory = seeded_directory();
    let mut ledger = FoundryRagRetrieveIdempotencyLedger::default();

    let mut operational_label = valid_request();
    operational_label.body.allowed_data_classes = vec!["AUDIT".to_string()];
    let error =
        retrieve_foundry_rag_from_api(operational_label, &directory, &mut ledger).unwrap_err();
    assert_eq!(
        error.code(),
        FoundryRagRetrieveApiErrorCode::InvalidDataClassLabel
    );
    assert_eq!(error.status_code(), 422);

    let mut hard_denied = valid_request();
    hard_denied.boundary.idempotency_key = "idem-rag-hard-denied".to_string();
    hard_denied.body.allowed_data_classes = vec!["PHI".to_string()];
    hard_denied.body.consent_receipts = vec![receipt("consent-phi", "PHI")];
    let error = retrieve_foundry_rag_from_api(hard_denied, &directory, &mut ledger).unwrap_err();
    assert_eq!(
        error.code(),
        FoundryRagRetrieveApiErrorCode::DataClassHardDenied
    );
    assert_eq!(error.status_code(), 422);
}

#[test]
fn returns_stable_error_envelope_and_rejects_idempotency_drift() {
    let directory = seeded_directory();
    let mut ledger = FoundryRagRetrieveIdempotencyLedger::default();
    let request = valid_request();
    retrieve_foundry_rag_from_api(request.clone(), &directory, &mut ledger).unwrap();

    let mut drift = request;
    drift.body.query = "different query".to_string();
    let error = retrieve_foundry_rag_from_api(drift, &directory, &mut ledger).unwrap_err();
    assert_eq!(
        error.code(),
        FoundryRagRetrieveApiErrorCode::IdempotencyKeyReused
    );
    assert_eq!(error.status_code(), 422);

    let envelope = error.error_response("req-rag-error");
    assert_eq!(
        envelope.error.code,
        FoundryRagRetrieveApiErrorCode::IdempotencyKeyReused
            .as_str()
            .to_string()
    );
    assert_eq!(envelope.error.request_id, "req-rag-error");
    assert_eq!(envelope.error.message_localized, None);
    assert_eq!(envelope.error.retry_after_seconds, None);
    assert_eq!(envelope.error.details.len(), 1);
}
