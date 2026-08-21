// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use check_eval_domain::REQUIRED_LINGUISTIC_COHORT_LOCALES;
use oya_governance_eval_usecase::{
    FOUNDRY_EVAL_RUN_OPENAPI_CONTRACT, FOUNDRY_EVAL_RUN_SURFACE, FoundryEvalApiAuthorization,
    FoundryEvalApiBoundaryContext, FoundryEvalApiPrincipal, FoundryEvalCaseRequest,
    FoundryEvalRunApiError, FoundryEvalRunApiErrorBody, FoundryEvalRunApiErrorCode,
    FoundryEvalRunApiErrorDetail, FoundryEvalRunApiErrorResponse, FoundryEvalRunApiRequest,
    FoundryEvalRunApiStatus, FoundryEvalRunDirectory, FoundryEvalRunIdempotencyLedger,
    FoundryEvalRunMetadata, FoundryEvalRunRecord, FoundryEvalRunRequest,
    FoundryEvalRunSuccessResponse, run_foundry_eval_from_api,
};

fn eval_case(
    case_id: &str,
    locale: &str,
    adversarial_kind: Option<&str>,
) -> FoundryEvalCaseRequest {
    FoundryEvalCaseRequest {
        case_id: case_id.to_string(),
        locale: locale.to_string(),
        input_ref: format!("fixture://inputs/{case_id}"),
        expected_ref: format!("fixture://expected/{case_id}"),
        adversarial_kind: adversarial_kind.map(str::to_string),
        deterministic_seed: Some(42),
    }
}

fn body_for(capability_id: &str) -> FoundryEvalRunRequest {
    FoundryEvalRunRequest {
        tenant_id: "ten_foundry".to_string(),
        capability_id: capability_id.to_string(),
        eval_set_version: "2026.05.12".to_string(),
        metric: "Composite".to_string(),
        min_pass_rate_percent: 90,
        min_p95_score_percent: 85,
        signed_eval_set: true,
        cases: vec![
            eval_case(
                "lang-alpha-prompt-injection",
                REQUIRED_LINGUISTIC_COHORT_LOCALES[0],
                Some("PromptInjection"),
            ),
            eval_case(
                "lang-beta-data-class",
                REQUIRED_LINGUISTIC_COHORT_LOCALES[1],
                Some("DataClassViolation"),
            ),
            eval_case(
                "lang-gamma-autonomy",
                REQUIRED_LINGUISTIC_COHORT_LOCALES[2],
                Some("AutonomyBypass"),
            ),
            eval_case(
                "lang-alpha-tool-exfiltration",
                REQUIRED_LINGUISTIC_COHORT_LOCALES[0],
                Some("ToolExfiltration"),
            ),
        ],
        pass_rate_percent: 96,
        p95_score_percent: 91,
        adversarial_passed: true,
        linguistic_passed: true,
        signed_run: true,
        run_started_at_epoch_seconds: 1_778_413_600,
    }
}

fn boundary_for(request_id: &str, idempotency_key: &str) -> FoundryEvalApiBoundaryContext {
    FoundryEvalApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_foundry".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> FoundryEvalApiPrincipal {
    FoundryEvalApiPrincipal {
        tenant_id: "ten_foundry".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(
    principal_id: &str,
    allowed_surfaces: Vec<&str>,
) -> FoundryEvalApiAuthorization {
    FoundryEvalApiAuthorization {
        tenant_id: "ten_foundry".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: "cedar-decision-eval-run".to_string(),
        allowed_surfaces: allowed_surfaces.into_iter().map(str::to_string).collect(),
    }
}

fn api_request_for(
    path_capability_id: &str,
    principal_id: &str,
    body: FoundryEvalRunRequest,
    request_id: &str,
    idempotency_key: &str,
) -> FoundryEvalRunApiRequest {
    FoundryEvalRunApiRequest {
        path_capability_id: path_capability_id.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for(principal_id),
        authorization: authorization_for(principal_id, vec![FOUNDRY_EVAL_RUN_SURFACE]),
        body,
    }
}

#[test]
fn foundry_eval_run_api_exports_surface_contract_and_status_codes() {
    assert_eq!(FOUNDRY_EVAL_RUN_SURFACE, "foundry.eval.run");
    assert_eq!(
        FOUNDRY_EVAL_RUN_OPENAPI_CONTRACT,
        "contracts/openapi/foundry/eval-v1.yaml"
    );
    assert_eq!(FoundryEvalRunApiStatus::Created.code(), 201);
    assert_eq!(FoundryEvalRunApiStatus::BadRequest.code(), 400);
    assert_eq!(FoundryEvalRunApiStatus::Unauthorized.code(), 401);
    assert_eq!(FoundryEvalRunApiStatus::Forbidden.code(), 403);
    assert_eq!(FoundryEvalRunApiStatus::UnprocessableEntity.code(), 422);

    let response = FoundryEvalRunSuccessResponse {
        data: FoundryEvalRunRecord {
            tenant_id: "ten_foundry".to_string(),
            capability_id: "cap.workflow.approve-payroll".to_string(),
            eval_set_version: "2026.05.12".to_string(),
            metric: "Composite".to_string(),
            pass_rate_percent: 96,
            p95_score_percent: 91,
            adversarial_passed: true,
            linguistic_passed: true,
            passed: true,
            signed_eval_set: true,
            signed_run: true,
            case_count: 4,
            run_started_at_epoch_seconds: 1_778_413_600,
            schema_version: 1,
        },
        metadata: FoundryEvalRunMetadata {
            request_id: "req-eval".to_string(),
            idempotency_key: "idem-eval".to_string(),
            surface: FOUNDRY_EVAL_RUN_SURFACE.to_string(),
            openapi_contract: FOUNDRY_EVAL_RUN_OPENAPI_CONTRACT.to_string(),
        },
    };
    assert_eq!(response.metadata.surface, FOUNDRY_EVAL_RUN_SURFACE);
}

#[test]
fn foundry_eval_run_api_records_passing_eval_and_replays_same_idempotency_key() {
    let mut directory = FoundryEvalRunDirectory::default();
    let mut ledger = FoundryEvalRunIdempotencyLedger::default();
    let request = api_request_for(
        "cap.workflow.approve-payroll",
        "usr_foundry_eval",
        body_for("cap.workflow.approve-payroll"),
        "req-eval-pass",
        "idem-eval-pass",
    );

    let first = run_foundry_eval_from_api(&mut directory, &mut ledger, request.clone()).unwrap();
    let replay = run_foundry_eval_from_api(&mut directory, &mut ledger, request).unwrap();

    assert_eq!(first, replay);
    assert_eq!(directory.len(), 1);
    assert_eq!(ledger.len(), 1);
    assert_eq!(first.data.tenant_id, "ten_foundry");
    assert_eq!(first.data.capability_id, "cap.workflow.approve-payroll");
    assert_eq!(first.data.eval_set_version, "2026.05.12");
    assert_eq!(first.data.case_count, 4);
    assert!(first.data.passed);
    assert_eq!(first.metadata.request_id, "req-eval-pass");
}

#[test]
fn foundry_eval_run_api_rejects_path_body_and_tenant_drift_before_mutation() {
    let mut directory = FoundryEvalRunDirectory::default();
    let mut ledger = FoundryEvalRunIdempotencyLedger::default();

    let result = run_foundry_eval_from_api(
        &mut directory,
        &mut ledger,
        api_request_for(
            "cap.workflow.approve-payroll",
            "usr_foundry_eval",
            body_for("cap.workflow.export-ledger"),
            "req-drift",
            "idem-drift",
        ),
    );
    assert_eq!(
        result,
        Err(FoundryEvalRunApiError::CapabilityIdMismatch {
            path_capability_id: "cap.workflow.approve-payroll".to_string(),
            body_capability_id: "cap.workflow.export-ledger".to_string(),
        })
    );

    let mut tenant_drift = api_request_for(
        "cap.workflow.approve-payroll",
        "usr_foundry_eval",
        body_for("cap.workflow.approve-payroll"),
        "req-tenant-drift",
        "idem-tenant-drift",
    );
    tenant_drift.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        run_foundry_eval_from_api(&mut directory, &mut ledger, tenant_drift),
        Err(FoundryEvalRunApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: "ten_foundry".to_string(),
            authorization_tenant_id: "ten_foundry".to_string(),
            body_tenant_id: "ten_foundry".to_string(),
        })
    );

    assert!(directory.is_empty());
    assert!(ledger.is_empty());
}

#[test]
fn foundry_eval_run_api_separates_authentication_and_authorization_failures() {
    let mut directory = FoundryEvalRunDirectory::default();
    let mut ledger = FoundryEvalRunIdempotencyLedger::default();

    let mut unauthenticated = api_request_for(
        "cap.workflow.approve-payroll",
        "usr_foundry_eval",
        body_for("cap.workflow.approve-payroll"),
        "req-empty-principal",
        "idem-empty-principal",
    );
    unauthenticated.principal.principal_id = " ".to_string();
    assert_eq!(
        run_foundry_eval_from_api(&mut directory, &mut ledger, unauthenticated),
        Err(FoundryEvalRunApiError::EmptyPrincipalId)
    );

    let mut denied = api_request_for(
        "cap.workflow.approve-payroll",
        "usr_foundry_eval",
        body_for("cap.workflow.approve-payroll"),
        "req-denied",
        "idem-denied",
    );
    denied.authorization.allowed_surfaces = vec!["foundry.capability.invoke".to_string()];
    assert_eq!(
        run_foundry_eval_from_api(&mut directory, &mut ledger, denied),
        Err(FoundryEvalRunApiError::AuthorizationSurfaceDenied {
            decision_id: "cedar-decision-eval-run".to_string(),
            surface: FOUNDRY_EVAL_RUN_SURFACE.to_string(),
        })
    );

    assert_eq!(FoundryEvalRunApiError::EmptyPrincipalId.status_code(), 401);
    assert_eq!(
        FoundryEvalRunApiError::AuthorizationSurfaceDenied {
            decision_id: "cedar-decision-eval-run".to_string(),
            surface: FOUNDRY_EVAL_RUN_SURFACE.to_string(),
        }
        .status_code(),
        403
    );
    assert!(directory.is_empty());
    assert!(ledger.is_empty());
}

#[test]
fn foundry_eval_run_api_maps_eval_kernel_and_idempotency_errors() {
    let mut directory = FoundryEvalRunDirectory::default();
    let mut ledger = FoundryEvalRunIdempotencyLedger::default();

    let mut below_threshold = body_for("cap.workflow.approve-payroll");
    below_threshold.pass_rate_percent = 89;
    let error = run_foundry_eval_from_api(
        &mut directory,
        &mut ledger,
        api_request_for(
            "cap.workflow.approve-payroll",
            "usr_foundry_eval",
            below_threshold,
            "req-below-threshold",
            "idem-below-threshold",
        ),
    )
    .unwrap_err();
    assert_eq!(error, FoundryEvalRunApiError::EvalRunBelowThreshold);
    assert_eq!(error.status_code(), 422);
    assert!(directory.is_empty());
    assert!(ledger.is_empty());

    let success = run_foundry_eval_from_api(
        &mut directory,
        &mut ledger,
        api_request_for(
            "cap.workflow.approve-payroll",
            "usr_foundry_eval",
            body_for("cap.workflow.approve-payroll"),
            "req-idem-original",
            "idem-reuse",
        ),
    )
    .unwrap();
    assert!(success.data.passed);

    let mut changed_body = body_for("cap.workflow.approve-payroll");
    changed_body.p95_score_percent = 99;
    assert_eq!(
        run_foundry_eval_from_api(
            &mut directory,
            &mut ledger,
            api_request_for(
                "cap.workflow.approve-payroll",
                "usr_foundry_eval",
                changed_body,
                "req-idem-reuse",
                "idem-reuse",
            ),
        ),
        Err(FoundryEvalRunApiError::IdempotencyKeyReused {
            idempotency_key: "idem-reuse".to_string(),
        })
    );
}

#[test]
fn foundry_eval_run_api_error_response_uses_stable_shape() {
    let response =
        FoundryEvalRunApiError::MissingAdversarialCoverage.error_response("req-error-shape");
    assert_eq!(
        response,
        FoundryEvalRunApiErrorResponse {
            error: FoundryEvalRunApiErrorBody {
                code: FoundryEvalRunApiErrorCode::EvalAdversarialCoverageMissing
                    .as_str()
                    .to_string(),
                message: "Eval set must include all mandatory adversarial cohorts".to_string(),
                message_localized: None,
                request_id: "req-error-shape".to_string(),
                details: vec![FoundryEvalRunApiErrorDetail {
                    field: "body.cases".to_string(),
                    issue: "missing prompt-injection, data-class-violation, autonomy-bypass, or tool-exfiltration cohort".to_string(),
                }],
                retry_after_seconds: None,
            },
        }
    );
    assert_eq!(
        FoundryEvalRunApiErrorCode::EvalAdversarialCoverageMissing.as_str(),
        "EVAL_ADVERSARIAL_COVERAGE_MISSING"
    );
}
