// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use intelligence_registry_api::{
    FOUNDRY_CAPABILITY_PUBLISH_SURFACE, FOUNDRY_REGISTRY_OPENAPI_CONTRACT,
    FoundryCapabilityApiAuthorization, FoundryCapabilityApiBoundaryContext,
    FoundryCapabilityApiPrincipal, FoundryCapabilityCostProfileRequest,
    FoundryCapabilityDescriptionRequest, FoundryCapabilityEvalCaseRequest,
    FoundryCapabilityProviderRequest, FoundryCapabilityPublishApiError,
    FoundryCapabilityPublishApiErrorBody, FoundryCapabilityPublishApiErrorCode,
    FoundryCapabilityPublishApiErrorDetail, FoundryCapabilityPublishApiErrorResponse,
    FoundryCapabilityPublishApiRequest, FoundryCapabilityPublishApiStatus,
    FoundryCapabilityPublishDirectory, FoundryCapabilityPublishIdempotencyLedger,
    FoundryCapabilityPublishMetadata, FoundryCapabilityPublishRecord,
    FoundryCapabilityPublishRequest, FoundryCapabilityPublishSuccessResponse,
    publish_foundry_capability_from_api,
};

fn eval_case(
    case_id: &str,
    locale: &str,
    adversarial_kind: Option<&str>,
) -> FoundryCapabilityEvalCaseRequest {
    FoundryCapabilityEvalCaseRequest {
        case_id: case_id.to_string(),
        locale: locale.to_string(),
        input_ref: format!("fixture://inputs/{case_id}"),
        expected_ref: format!("fixture://expected/{case_id}"),
        adversarial_kind: adversarial_kind.map(str::to_string),
        deterministic_seed: Some(7),
    }
}

fn publish_body(capability_id: &str) -> FoundryCapabilityPublishRequest {
    FoundryCapabilityPublishRequest {
        tenant_id: "ten_foundry".to_string(),
        capability_id: capability_id.to_string(),
        namespace: "foundry.workflow".to_string(),
        version: "0.1.0".to_string(),
        description: FoundryCapabilityDescriptionRequest {
            agent_readable: "Approve payroll changes after policy and budget checks.".to_string(),
            human_readable: "Capability for payroll approval workflow previews.".to_string(),
        },
        provider: FoundryCapabilityProviderRequest {
            preferred: "foundation-local".to_string(),
            fallback: vec!["openai-api".to_string()],
        },
        autonomy_tier_required: "T2".to_string(),
        data_classes_touched: vec!["INTERNAL_ONLY".to_string(), "PII_QUASI_IDENTIFIER".to_string()],
        evidence_emission_topic: "oya.foundry.capability.invoked".to_string(),
        cost_profile: FoundryCapabilityCostProfileRequest {
            per_invocation_limit_micros: 50_000,
            per_tenant_monthly_limit_micros: 10_000_000,
        },
        input_schema_json: r#"{"type":"object","properties":{"request_id":{"type":"string"}},"required":["request_id"]}"#.to_string(),
        output_schema_json: r#"{"type":"object","properties":{"verdict":{"type":"string"}},"required":["verdict"]}"#.to_string(),
        eval_set_version: "eval-2026-05-12".to_string(),
        eval_metric: "Composite".to_string(),
        min_pass_rate_percent: 90,
        min_p95_score_percent: 85,
        signed_eval_set: true,
        eval_cases: vec![
            eval_case(
                "cohort-alpha-prompt-injection",
                "lang-alpha1",
                Some("PromptInjection"),
            ),
            eval_case(
                "cohort-beta-data-class",
                "lang-beta1",
                Some("DataClassViolation"),
            ),
            eval_case(
                "cohort-gamma-autonomy",
                "lang-gamma1",
                Some("AutonomyBypass"),
            ),
            eval_case(
                "cohort-alpha-tool-exfiltration",
                "lang-alpha1",
                Some("ToolExfiltration"),
            ),
        ],
        eval_pass_rate_percent: 96,
        eval_p95_score_percent: 91,
        eval_adversarial_passed: true,
        eval_linguistic_passed: true,
        signed_eval_run: true,
        owner_team: "axis-foundry".to_string(),
        catalog_record_path: "registry/catalog/oya-workflow-payroll.yaml".to_string(),
        docs_path: "docs.oyatie.com/capabilities/cap.workflow.approve-payroll/".to_string(),
        published_at_epoch_seconds: 1_778_413_600,
    }
}

fn boundary_for(request_id: &str, idempotency_key: &str) -> FoundryCapabilityApiBoundaryContext {
    FoundryCapabilityApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_foundry".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> FoundryCapabilityApiPrincipal {
    FoundryCapabilityApiPrincipal {
        tenant_id: "ten_foundry".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(
    principal_id: &str,
    allowed_surfaces: Vec<&str>,
) -> FoundryCapabilityApiAuthorization {
    FoundryCapabilityApiAuthorization {
        tenant_id: "ten_foundry".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: "cedar-decision-capability-publish".to_string(),
        allowed_surfaces: allowed_surfaces.into_iter().map(str::to_string).collect(),
    }
}

fn api_request_for(
    path_capability_id: &str,
    principal_id: &str,
    body: FoundryCapabilityPublishRequest,
    request_id: &str,
    idempotency_key: &str,
) -> FoundryCapabilityPublishApiRequest {
    FoundryCapabilityPublishApiRequest {
        path_capability_id: path_capability_id.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for(principal_id),
        authorization: authorization_for(principal_id, vec![FOUNDRY_CAPABILITY_PUBLISH_SURFACE]),
        body,
    }
}

#[test]
fn foundry_capability_publish_exports_surface_contract_and_status_codes() {
    assert_eq!(
        FOUNDRY_CAPABILITY_PUBLISH_SURFACE,
        "foundry.capability.publish"
    );
    assert_eq!(
        FOUNDRY_REGISTRY_OPENAPI_CONTRACT,
        "contracts/openapi/foundry/registry-v1.yaml"
    );
    assert_eq!(FoundryCapabilityPublishApiStatus::Created.code(), 201);
    assert_eq!(FoundryCapabilityPublishApiStatus::BadRequest.code(), 400);
    assert_eq!(FoundryCapabilityPublishApiStatus::Unauthorized.code(), 401);
    assert_eq!(FoundryCapabilityPublishApiStatus::Forbidden.code(), 403);
    assert_eq!(FoundryCapabilityPublishApiStatus::Conflict.code(), 409);
    assert_eq!(
        FoundryCapabilityPublishApiStatus::UnprocessableEntity.code(),
        422
    );

    let response = FoundryCapabilityPublishSuccessResponse {
        data: FoundryCapabilityPublishRecord {
            tenant_id: "ten_foundry".to_string(),
            capability_id: "cap.workflow.approve-payroll".to_string(),
            namespace: "foundry.workflow".to_string(),
            version: "0.1.0".to_string(),
            owner_team: "axis-foundry".to_string(),
            autonomy_tier_required: "T2".to_string(),
            data_classes_touched: vec!["INTERNAL_ONLY".to_string()],
            provider_preference: vec!["foundation-local".to_string()],
            evidence_emission_topic: "oya.foundry.capability.invoked".to_string(),
            eval_set_version: "eval-2026-05-12".to_string(),
            eval_pass_rate_percent: 96,
            eval_p95_score_percent: 91,
            eval_case_count: 4,
            published_at_epoch_seconds: 1_778_413_600,
            schema_version: 1,
        },
        metadata: FoundryCapabilityPublishMetadata {
            request_id: "req-publish".to_string(),
            idempotency_key: "idem-publish".to_string(),
            surface: FOUNDRY_CAPABILITY_PUBLISH_SURFACE.to_string(),
            openapi_contract: FOUNDRY_REGISTRY_OPENAPI_CONTRACT.to_string(),
        },
    };
    assert_eq!(
        response.metadata.surface,
        FOUNDRY_CAPABILITY_PUBLISH_SURFACE
    );
}

#[test]
fn foundry_capability_publish_records_capability_and_replays_idempotently() {
    let mut directory = FoundryCapabilityPublishDirectory::default();
    let mut ledger = FoundryCapabilityPublishIdempotencyLedger::default();
    let request = api_request_for(
        "cap.workflow.approve-payroll",
        "usr_foundry_registry",
        publish_body("cap.workflow.approve-payroll"),
        "req-publish-pass",
        "idem-publish-pass",
    );

    let first =
        publish_foundry_capability_from_api(&mut directory, &mut ledger, request.clone()).unwrap();
    let replay = publish_foundry_capability_from_api(&mut directory, &mut ledger, request).unwrap();

    assert_eq!(first, replay);
    assert_eq!(directory.len(), 1);
    assert_eq!(ledger.len(), 1);
    assert_eq!(first.data.capability_id, "cap.workflow.approve-payroll");
    assert_eq!(first.data.eval_case_count, 4);
    assert_eq!(
        first.data.provider_preference,
        vec!["foundation-local", "openai-api"]
    );
    assert_eq!(
        first.metadata.openapi_contract,
        FOUNDRY_REGISTRY_OPENAPI_CONTRACT
    );
    assert!(
        directory
            .get("ten_foundry", "cap.workflow.approve-payroll")
            .is_some()
    );
}

#[test]
fn foundry_capability_publish_rejects_path_body_tenant_and_authz_drift_before_mutation() {
    let mut directory = FoundryCapabilityPublishDirectory::default();
    let mut ledger = FoundryCapabilityPublishIdempotencyLedger::default();

    let result = publish_foundry_capability_from_api(
        &mut directory,
        &mut ledger,
        api_request_for(
            "cap.workflow.approve-payroll",
            "usr_foundry_registry",
            publish_body("cap.workflow.export-ledger"),
            "req-drift",
            "idem-drift",
        ),
    );
    assert_eq!(
        result,
        Err(FoundryCapabilityPublishApiError::CapabilityIdMismatch {
            path_capability_id: "cap.workflow.approve-payroll".to_string(),
            body_capability_id: "cap.workflow.export-ledger".to_string(),
        })
    );

    let mut tenant_drift = api_request_for(
        "cap.workflow.approve-payroll",
        "usr_foundry_registry",
        publish_body("cap.workflow.approve-payroll"),
        "req-tenant-drift",
        "idem-tenant-drift",
    );
    tenant_drift.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        publish_foundry_capability_from_api(&mut directory, &mut ledger, tenant_drift),
        Err(FoundryCapabilityPublishApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: "ten_foundry".to_string(),
            authorization_tenant_id: "ten_foundry".to_string(),
            body_tenant_id: "ten_foundry".to_string(),
        })
    );

    let mut denied = api_request_for(
        "cap.workflow.approve-payroll",
        "usr_foundry_registry",
        publish_body("cap.workflow.approve-payroll"),
        "req-denied",
        "idem-denied",
    );
    denied.authorization.allowed_surfaces = vec!["foundry.capability.invoke".to_string()];
    assert_eq!(
        publish_foundry_capability_from_api(&mut directory, &mut ledger, denied),
        Err(
            FoundryCapabilityPublishApiError::AuthorizationSurfaceDenied {
                decision_id: "cedar-decision-capability-publish".to_string(),
                surface: FOUNDRY_CAPABILITY_PUBLISH_SURFACE.to_string(),
            }
        )
    );

    assert!(directory.is_empty());
    assert!(ledger.is_empty());
}

#[test]
fn foundry_capability_publish_requires_signed_passing_eval_before_registry_mutation() {
    let mut directory = FoundryCapabilityPublishDirectory::default();
    let mut ledger = FoundryCapabilityPublishIdempotencyLedger::default();

    let mut body = publish_body("cap.workflow.approve-payroll");
    body.eval_pass_rate_percent = 80;
    let error = publish_foundry_capability_from_api(
        &mut directory,
        &mut ledger,
        api_request_for(
            "cap.workflow.approve-payroll",
            "usr_foundry_registry",
            body,
            "req-below-threshold",
            "idem-below-threshold",
        ),
    )
    .unwrap_err();

    assert_eq!(
        error,
        FoundryCapabilityPublishApiError::EvalRunBelowThreshold
    );
    assert_eq!(error.status_code(), 422);
    assert!(directory.is_empty());
    assert!(ledger.is_empty());
}

#[test]
fn foundry_capability_publish_rejects_invalid_labels_duplicate_and_reused_idempotency() {
    let mut directory = FoundryCapabilityPublishDirectory::default();
    let mut ledger = FoundryCapabilityPublishIdempotencyLedger::default();

    let mut invalid_class = publish_body("cap.workflow.approve-payroll");
    invalid_class.data_classes_touched = vec!["AUDIT".to_string()];
    assert_eq!(
        publish_foundry_capability_from_api(
            &mut directory,
            &mut ledger,
            api_request_for(
                "cap.workflow.approve-payroll",
                "usr_foundry_registry",
                invalid_class,
                "req-invalid-class",
                "idem-invalid-class",
            ),
        ),
        Err(FoundryCapabilityPublishApiError::InvalidDataClass {
            data_class: "AUDIT".to_string(),
        })
    );

    let first = publish_foundry_capability_from_api(
        &mut directory,
        &mut ledger,
        api_request_for(
            "cap.workflow.approve-payroll",
            "usr_foundry_registry",
            publish_body("cap.workflow.approve-payroll"),
            "req-first",
            "idem-first",
        ),
    )
    .unwrap();
    assert_eq!(first.status_code(), 201);

    assert_eq!(
        publish_foundry_capability_from_api(
            &mut directory,
            &mut ledger,
            api_request_for(
                "cap.workflow.approve-payroll",
                "usr_foundry_registry",
                publish_body("cap.workflow.approve-payroll"),
                "req-duplicate",
                "idem-duplicate",
            ),
        ),
        Err(FoundryCapabilityPublishApiError::DuplicateCapability {
            capability_id: "cap.workflow.approve-payroll".to_string(),
        })
    );

    let mut changed_body = publish_body("cap.workflow.new-capability");
    changed_body.eval_p95_score_percent = 99;
    assert_eq!(
        publish_foundry_capability_from_api(
            &mut directory,
            &mut ledger,
            api_request_for(
                "cap.workflow.new-capability",
                "usr_foundry_registry",
                changed_body,
                "req-idem-reuse",
                "idem-first",
            ),
        ),
        Err(FoundryCapabilityPublishApiError::IdempotencyKeyReused {
            idempotency_key: "idem-first".to_string(),
        })
    );
}

#[test]
fn foundry_capability_publish_error_response_uses_stable_shape() {
    let response = FoundryCapabilityPublishApiError::MissingAdversarialCoverage
        .error_response("req-error-shape");
    assert_eq!(
        response,
        FoundryCapabilityPublishApiErrorResponse {
            error: FoundryCapabilityPublishApiErrorBody {
                code: FoundryCapabilityPublishApiErrorCode::EvalAdversarialCoverageMissing
                    .as_str()
                    .to_string(),
                message: "Capability eval set must include all mandatory adversarial cohorts".to_string(),
                message_localized: None,
                request_id: "req-error-shape".to_string(),
                details: vec![FoundryCapabilityPublishApiErrorDetail {
                    field: "body.eval_cases".to_string(),
                    issue: "missing prompt-injection, data-class-violation, autonomy-bypass, or tool-exfiltration cohort".to_string(),
                }],
                retry_after_seconds: None,
            },
        }
    );
}
