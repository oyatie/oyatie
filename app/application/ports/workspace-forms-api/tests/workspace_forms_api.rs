// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use workspace_forms_api::{
    WORKSPACE_FORMS_OPENAPI_CONTRACT, WORKSPACE_FORMS_SUBMISSION_INGEST_SURFACE,
    WorkspaceFormsAnswerRequest, WorkspaceFormsApiAuthorization, WorkspaceFormsApiError,
    WorkspaceFormsApiPrincipal, WorkspaceFormsFieldSeed, WorkspaceFormsFormSeed,
    WorkspaceFormsIngestBoundaryContext, WorkspaceFormsSubmissionDirectory,
    WorkspaceFormsSubmissionIngestApiRequest, WorkspaceFormsSubmissionIngestApiStatus,
    WorkspaceFormsSubmissionIngestIdempotencyLedger, WorkspaceFormsSubmissionIngestRequest,
    WorkspaceFormsSubmissionMetadata, WorkspaceFormsSubmissionRecord,
    ingest_workspace_forms_submission_from_api,
};

const FORM_ID: &str = "form_001";
const SUBMISSION_ID: &str = "submission_001";
const TENANT_ID: &str = "ten_workspace_alpha";
const SUBMITTER: &str = "user:submitter@example.com";

fn boundary(request_id: &str, idempotency_key: &str) -> WorkspaceFormsIngestBoundaryContext {
    WorkspaceFormsIngestBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal(principal_id: &str) -> WorkspaceFormsApiPrincipal {
    WorkspaceFormsApiPrincipal {
        tenant_id: TENANT_ID.to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization(principal_id: &str, surfaces: &[&str]) -> WorkspaceFormsApiAuthorization {
    WorkspaceFormsApiAuthorization {
        tenant_id: TENANT_ID.to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn seeded_directory() -> WorkspaceFormsSubmissionDirectory {
    let mut directory = WorkspaceFormsSubmissionDirectory::default();
    directory
        .insert_form_seed(WorkspaceFormsFormSeed {
            form_id: FORM_ID.to_string(),
            tenant_id: TENANT_ID.to_string(),
            region: "region-home".to_string(),
            cell_id: "cell-workspace-kr-001".to_string(),
            object_graph_route_id: "og_route_forms_intake".to_string(),
            title: "Customer intake".to_string(),
            data_class: "PII_IDENTIFYING".to_string(),
            fields: vec![
                WorkspaceFormsFieldSeed {
                    field_id: "name".to_string(),
                    label: "Name".to_string(),
                    kind: "short_text".to_string(),
                    required: true,
                    choice_options: Vec::new(),
                },
                WorkspaceFormsFieldSeed {
                    field_id: "score".to_string(),
                    label: "Score".to_string(),
                    kind: "number".to_string(),
                    required: false,
                    choice_options: Vec::new(),
                },
            ],
            created_at_epoch_seconds: 1_700_000_000,
        })
        .expect("seed form is valid");
    directory
}

fn answers() -> Vec<WorkspaceFormsAnswerRequest> {
    vec![WorkspaceFormsAnswerRequest {
        field_id: "name".to_string(),
        value_kind: "short_text".to_string(),
        value: "Ada Lovelace".to_string(),
    }]
}

fn ingest_body(submission_id: &str) -> WorkspaceFormsSubmissionIngestRequest {
    WorkspaceFormsSubmissionIngestRequest {
        submission_id: submission_id.to_string(),
        form_id: FORM_ID.to_string(),
        tenant_id: TENANT_ID.to_string(),
        submitter_ref: SUBMITTER.to_string(),
        answers: answers(),
        data_class: "PII_IDENTIFYING".to_string(),
        submitted_at_epoch_seconds: 1_700_000_010,
    }
}

fn ingest_request(
    request_id: &str,
    idempotency_key: &str,
) -> WorkspaceFormsSubmissionIngestApiRequest {
    WorkspaceFormsSubmissionIngestApiRequest {
        path_form_id: FORM_ID.to_string(),
        path_submission_id: SUBMISSION_ID.to_string(),
        boundary: boundary(request_id, idempotency_key),
        principal: principal(SUBMITTER),
        authorization: authorization(SUBMITTER, &[WORKSPACE_FORMS_SUBMISSION_INGEST_SURFACE]),
        body: ingest_body(SUBMISSION_ID),
    }
}

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(
        WORKSPACE_FORMS_SUBMISSION_INGEST_SURFACE,
        "workspace.forms.submission.ingest"
    );
    assert_eq!(
        WORKSPACE_FORMS_OPENAPI_CONTRACT,
        "contracts/openapi/workspace/workspace-forms-v1.yaml"
    );
    assert_eq!(
        WorkspaceFormsSubmissionIngestApiStatus::Accepted.code(),
        202
    );
    assert_eq!(
        WorkspaceFormsSubmissionIngestApiStatus::BadRequest.code(),
        400
    );
    assert_eq!(
        WorkspaceFormsSubmissionIngestApiStatus::Forbidden.code(),
        403
    );
    assert_eq!(
        WorkspaceFormsSubmissionIngestApiStatus::NotFound.code(),
        404
    );
    assert_eq!(
        WorkspaceFormsSubmissionIngestApiStatus::Conflict.code(),
        409
    );
    assert_eq!(
        WorkspaceFormsSubmissionIngestApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn ingest_forms_submission_accepts_once_and_replays_same_idempotent_result() {
    let mut directory = seeded_directory();
    let mut ledger = WorkspaceFormsSubmissionIngestIdempotencyLedger::default();
    let request = ingest_request("req-workspace-forms-ingest", "idem-workspace-forms-ingest");

    let first =
        ingest_workspace_forms_submission_from_api(&mut directory, &mut ledger, request.clone())
            .expect("authorized Forms submission ingest succeeds");
    let second = ingest_workspace_forms_submission_from_api(&mut directory, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(directory.submission_len(), 1);
    assert_eq!(first.metadata.request_id, "req-workspace-forms-ingest");
    assert_eq!(
        first.metadata.surface,
        WORKSPACE_FORMS_SUBMISSION_INGEST_SURFACE
    );
    assert_eq!(first.data.submission_id, SUBMISSION_ID);
    assert_eq!(first.data.form_id, FORM_ID);
    assert_eq!(first.data.submitter_ref, SUBMITTER);
    assert_eq!(first.data.object_graph_route_id, "og_route_forms_intake");
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn ingest_forms_submission_rejects_path_body_tenant_and_submitter_drift_before_mutation() {
    let mut directory = seeded_directory();
    let mut ledger = WorkspaceFormsSubmissionIngestIdempotencyLedger::default();
    let mut request = ingest_request("req-workspace-forms-drift", "idem-workspace-forms-drift");
    request.body.submission_id = "submission_other".to_string();

    let error = ingest_workspace_forms_submission_from_api(&mut directory, &mut ledger, request)
        .expect_err("path/body submission id drift is rejected");
    assert_eq!(
        error,
        WorkspaceFormsApiError::SubmissionIdMismatch {
            path_submission_id: SUBMISSION_ID.to_string(),
            body_submission_id: "submission_other".to_string(),
        }
    );
    assert_eq!(error.submission_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(directory.submission_len(), 0);

    let mut tenant_drift =
        ingest_request("req-workspace-forms-tenant", "idem-workspace-forms-tenant");
    tenant_drift.boundary.tenant_id = "ten_other".to_string();
    let error =
        ingest_workspace_forms_submission_from_api(&mut directory, &mut ledger, tenant_drift)
            .expect_err("tenant drift is rejected");
    assert_eq!(error.submission_status_code(), 403);
    assert!(matches!(
        error,
        WorkspaceFormsApiError::TenantMismatch { .. }
    ));
    assert!(ledger.is_empty());

    let mut submitter_drift = ingest_request(
        "req-workspace-forms-submitter",
        "idem-workspace-forms-submitter",
    );
    submitter_drift.body.submitter_ref = "user:other@example.com".to_string();
    let error =
        ingest_workspace_forms_submission_from_api(&mut directory, &mut ledger, submitter_drift)
            .expect_err("principal cannot impersonate another submitter");
    assert_eq!(error.submission_status_code(), 403);
    assert!(matches!(
        error,
        WorkspaceFormsApiError::SubmitterPermissionDenied { .. }
    ));
    assert!(ledger.is_empty());
}

#[test]
fn ingest_forms_submission_rejects_authorization_missing_form_and_reused_idempotency_key() {
    let mut directory = seeded_directory();
    let mut ledger = WorkspaceFormsSubmissionIngestIdempotencyLedger::default();
    let mut request = ingest_request("req-workspace-forms-authz", "idem-workspace-forms-authz");
    request.authorization.allowed_surfaces = vec!["workspace.drive.get".to_string()];

    let error = ingest_workspace_forms_submission_from_api(&mut directory, &mut ledger, request)
        .expect_err("authorization decision does not allow Forms submission ingest");
    assert_eq!(
        error,
        WorkspaceFormsApiError::AuthorizationDenied {
            surface: WORKSPACE_FORMS_SUBMISSION_INGEST_SURFACE.to_string(),
        }
    );
    assert_eq!(error.submission_status_code(), 403);
    assert!(ledger.is_empty());

    let mut missing = ingest_request(
        "req-workspace-forms-missing",
        "idem-workspace-forms-missing",
    );
    missing.path_form_id = "form_missing".to_string();
    missing.body.form_id = "form_missing".to_string();
    let error = ingest_workspace_forms_submission_from_api(&mut directory, &mut ledger, missing)
        .expect_err("missing form is not found");
    assert_eq!(error.submission_status_code(), 404);
    assert!(matches!(error, WorkspaceFormsApiError::FormNotFound { .. }));
    assert!(ledger.is_empty());

    let request = ingest_request("req-workspace-forms-idem", "idem-workspace-forms-idem");
    ingest_workspace_forms_submission_from_api(&mut directory, &mut ledger, request.clone())
        .expect("initial Forms submission ingest succeeds");
    let mut drifted = request;
    drifted.body.answers[0].value = "Grace Hopper".to_string();
    assert_eq!(
        ingest_workspace_forms_submission_from_api(&mut directory, &mut ledger, drifted),
        Err(WorkspaceFormsApiError::IdempotencyKeyReused {
            idempotency_key: "idem-workspace-forms-idem".to_string(),
        })
    );
    assert_eq!(directory.submission_len(), 1);
}

#[test]
fn ingest_forms_submission_maps_duplicate_invalid_answer_kind_and_data_class() {
    let mut directory = seeded_directory();
    let mut ledger = WorkspaceFormsSubmissionIngestIdempotencyLedger::default();
    ingest_workspace_forms_submission_from_api(
        &mut directory,
        &mut ledger,
        ingest_request("req-workspace-forms-dup-1", "idem-workspace-forms-dup-1"),
    )
    .expect("first Forms submission ingest succeeds");

    let duplicate = ingest_workspace_forms_submission_from_api(
        &mut directory,
        &mut ledger,
        ingest_request("req-workspace-forms-dup-2", "idem-workspace-forms-dup-2"),
    )
    .expect_err("same submission through new idempotency key is conflict");
    assert_eq!(duplicate.submission_status_code(), 409);
    assert!(matches!(
        duplicate,
        WorkspaceFormsApiError::SubmissionAlreadyExists { .. }
    ));

    let mut invalid_kind = ingest_request("req-workspace-forms-kind", "idem-workspace-forms-kind");
    invalid_kind.body.answers[0].value_kind = "currency".to_string();
    let error =
        ingest_workspace_forms_submission_from_api(&mut directory, &mut ledger, invalid_kind)
            .expect_err("unknown answer kind is rejected before kernel");
    assert_eq!(error.submission_status_code(), 400);
    assert!(matches!(
        error,
        WorkspaceFormsApiError::InvalidFieldKind { .. }
    ));

    let mut invalid_class =
        ingest_request("req-workspace-forms-class", "idem-workspace-forms-class");
    invalid_class.body.data_class = "AUDIT".to_string();
    let error =
        ingest_workspace_forms_submission_from_api(&mut directory, &mut ledger, invalid_class)
            .expect_err("operational data class is rejected for public privacy field");
    assert_eq!(error.submission_status_code(), 400);
    assert!(matches!(
        error,
        WorkspaceFormsApiError::InvalidDataClassLabel { .. }
    ));

    let mut missing_answer = ingest_request(
        "req-workspace-forms-required",
        "idem-workspace-forms-required",
    );
    missing_answer.body.answers.clear();
    let error =
        ingest_workspace_forms_submission_from_api(&mut directory, &mut ledger, missing_answer)
            .expect_err("missing required form answer is rejected before mutation");
    assert_eq!(error.submission_status_code(), 400);
    assert!(matches!(error, WorkspaceFormsApiError::Form(_)));
}

#[test]
fn stable_error_response_shape_uses_request_id_and_field_details() {
    let error = WorkspaceFormsApiError::InvalidDataClassLabel {
        data_class: "AUDIT".to_string(),
    };

    let response = error.error_response("req-workspace-forms-error");

    assert_eq!(response.error.code, "WORKSPACE_FORMS_DATA_CLASS_INVALID");
    assert_eq!(response.error.request_id, "req-workspace-forms-error");
    assert_eq!(response.error.details[0].field, "body.data_class");
    assert_eq!(response.error.retry_after_seconds, None);
}

#[test]
fn public_response_structs_keep_contract_names_stable() {
    let _metadata = WorkspaceFormsSubmissionMetadata {
        request_id: "req-workspace-forms-structs".to_string(),
        surface: WORKSPACE_FORMS_SUBMISSION_INGEST_SURFACE.to_string(),
        openapi_contract: WORKSPACE_FORMS_OPENAPI_CONTRACT.to_string(),
    };
    let _record = WorkspaceFormsSubmissionRecord {
        submission_id: SUBMISSION_ID.to_string(),
        form_id: FORM_ID.to_string(),
        tenant_id: TENANT_ID.to_string(),
        submitter_ref: SUBMITTER.to_string(),
        answers: Vec::new(),
        data_class: "PII_IDENTIFYING".to_string(),
        submitted_at_epoch_seconds: 1_700_000_010,
        object_graph_route_id: "og_route_forms_intake".to_string(),
        schema_version: 1,
    };
}
