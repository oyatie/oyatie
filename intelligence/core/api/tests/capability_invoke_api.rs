// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use intelligence_api::{
    ApiBoundaryContext, CAPABILITY_INVOKE_SURFACE, CapabilityInvocationPrincipal,
    CapabilityInvocationReceipt, CapabilityInvocationRequest, CapabilityInvokeApiError,
    CapabilityInvokeApiErrorBody, CapabilityInvokeApiErrorCode, CapabilityInvokeApiErrorDetail,
    CapabilityInvokeApiErrorResponse, CapabilityInvokeApiRequest,
    CapabilityInvokeApiResponseMetadata, CapabilityInvokeApiStatus,
    CapabilityInvokeApiSuccessResponse, CapabilityInvokeIdempotencyLedger, Foundation,
    invoke_capability_from_api,
};
use application_app::{
    AdversarialKind, AutonomyTier, CapabilityAction, CapabilityRegistration,
    CostBudgetRegistration, DataClass, EvalCaseInput, EvalMetric, EvalRunInput, EvalSetInput,
    FoundationError, IdentityRegistration, PolicyEffect, PolicyRuleInput, PolicyScope,
    PolicyVersion, Purpose, SubjectClass, TenantCapabilityGrant, TenantRegistration,
};

fn request_for(capability_id: &str) -> CapabilityInvocationRequest {
    CapabilityInvocationRequest {
        tenant_id: "ten_api".to_string(),
        user_id: "usr_api".to_string(),
        capability_id: capability_id.to_string(),
        purpose: Purpose::CapabilityInvocation,
        subject_class: SubjectClass::Adult,
        budget_window_id: "2026-05".to_string(),
        projected_cost_micros: 10,
        started_at_epoch_seconds: 1_778_413_600,
    }
}

fn principal_for(user_id: &str) -> CapabilityInvocationPrincipal {
    CapabilityInvocationPrincipal {
        tenant_id: "ten_api".to_string(),
        user_id: user_id.to_string(),
        autonomy_ceiling: AutonomyTier::T2Advisory,
    }
}

fn boundary_for(request_id: &str, idempotency_key: &str) -> ApiBoundaryContext {
    ApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_api".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn api_request_for(
    path_capability_id: &str,
    principal_user_id: &str,
    body: CapabilityInvocationRequest,
    request_id: &str,
    idempotency_key: &str,
) -> CapabilityInvokeApiRequest {
    CapabilityInvokeApiRequest {
        path_capability_id: path_capability_id.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for(principal_user_id),
        body,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundationSideEffectCounts {
    audit_events: usize,
    runs: usize,
    steps: usize,
    evidence_records: usize,
    outbox_records: usize,
}

fn side_effect_counts(foundation: &Foundation) -> FoundationSideEffectCounts {
    FoundationSideEffectCounts {
        audit_events: foundation.audit_chain().events().len(),
        runs: foundation.foundry_runs().len(),
        steps: foundation.foundry_steps().len(),
        evidence_records: foundation.foundry_evidence_chain().records().len(),
        outbox_records: foundation.outbox_records().len(),
    }
}

#[test]
fn capability_invoke_api_rejects_path_body_capability_drift_before_foundation() {
    let mut foundation = Foundation::default();
    let mut ledger = CapabilityInvokeIdempotencyLedger::default();
    let result = invoke_capability_from_api(
        &mut foundation,
        &mut ledger,
        api_request_for(
            "cap.workflow.approve-payroll",
            "usr_api",
            request_for("cap.workflow.export-ledger"),
            "req-drift",
            "idem-drift",
        ),
    );

    assert_eq!(
        result,
        Err(CapabilityInvokeApiError::CapabilityIdMismatch {
            path_capability_id: "cap.workflow.approve-payroll".to_string(),
            body_capability_id: "cap.workflow.export-ledger".to_string(),
        })
    );
    assert!(foundation.audit_chain().events().is_empty());
    assert!(ledger.is_empty());
}

#[test]
fn capability_invoke_api_rejects_empty_path_capability_before_foundation() {
    let mut foundation = Foundation::default();
    let mut ledger = CapabilityInvokeIdempotencyLedger::default();
    let result = invoke_capability_from_api(
        &mut foundation,
        &mut ledger,
        api_request_for(
            "  ",
            "usr_api",
            request_for("cap.workflow.approve-payroll"),
            "req-empty-path",
            "idem-empty-path",
        ),
    );

    assert_eq!(result, Err(CapabilityInvokeApiError::EmptyPathCapabilityId));
    assert!(foundation.audit_chain().events().is_empty());
    assert!(ledger.is_empty());
}

#[test]
fn capability_invoke_api_rejects_principal_body_mismatch_before_foundation() {
    let mut foundation = Foundation::default();
    let mut ledger = CapabilityInvokeIdempotencyLedger::default();
    let result = invoke_capability_from_api(
        &mut foundation,
        &mut ledger,
        api_request_for(
            "cap.workflow.approve-payroll",
            "usr_impersonator",
            request_for("cap.workflow.approve-payroll"),
            "req-principal-mismatch",
            "idem-principal-mismatch",
        ),
    );

    assert_eq!(
        result,
        Err(CapabilityInvokeApiError::PrincipalMismatch {
            principal_tenant_id: "ten_api".to_string(),
            principal_user_id: "usr_impersonator".to_string(),
            body_tenant_id: "ten_api".to_string(),
            body_user_id: "usr_api".to_string(),
        })
    );
    assert!(foundation.audit_chain().events().is_empty());
    assert!(ledger.is_empty());
}

#[test]
fn capability_invoke_api_rejects_empty_required_headers_before_foundation() {
    let mut foundation = Foundation::default();
    let mut ledger = CapabilityInvokeIdempotencyLedger::default();

    let mut request = api_request_for(
        "cap.workflow.approve-payroll",
        "usr_api",
        request_for("cap.workflow.approve-payroll"),
        " ",
        "idem-empty-request-id",
    );
    assert_eq!(
        invoke_capability_from_api(&mut foundation, &mut ledger, request.clone()),
        Err(CapabilityInvokeApiError::EmptyRequestId)
    );

    request.boundary.request_id = "req-empty-tenant".to_string();
    request.boundary.tenant_id = " ".to_string();
    assert_eq!(
        invoke_capability_from_api(&mut foundation, &mut ledger, request.clone()),
        Err(CapabilityInvokeApiError::EmptyTenantHeader)
    );

    request.boundary.tenant_id = "ten_api".to_string();
    request.boundary.idempotency_key = " ".to_string();
    assert_eq!(
        invoke_capability_from_api(&mut foundation, &mut ledger, request),
        Err(CapabilityInvokeApiError::EmptyIdempotencyKey)
    );

    assert_eq!(
        side_effect_counts(&foundation),
        FoundationSideEffectCounts {
            audit_events: 0,
            runs: 0,
            steps: 0,
            evidence_records: 0,
            outbox_records: 0,
        }
    );
    assert!(ledger.is_empty());
}

#[test]
fn capability_invoke_api_rejects_tenant_header_mismatch_before_foundation() {
    let mut foundation = Foundation::default();
    let mut ledger = CapabilityInvokeIdempotencyLedger::default();
    let mut request = api_request_for(
        "cap.workflow.approve-payroll",
        "usr_api",
        request_for("cap.workflow.approve-payroll"),
        "req-tenant-mismatch",
        "idem-tenant-mismatch",
    );
    request.boundary.tenant_id = "ten_other".to_string();

    let result = invoke_capability_from_api(&mut foundation, &mut ledger, request);

    assert_eq!(
        result,
        Err(CapabilityInvokeApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: "ten_api".to_string(),
            body_tenant_id: "ten_api".to_string(),
        })
    );
    assert!(foundation.audit_chain().events().is_empty());
    assert!(ledger.is_empty());
}

#[test]
fn capability_invoke_api_passes_matching_request_to_foundation() {
    let mut foundation = Foundation::default();
    let mut ledger = CapabilityInvokeIdempotencyLedger::default();
    let result = invoke_capability_from_api(
        &mut foundation,
        &mut ledger,
        api_request_for(
            "cap.workflow.approve-payroll",
            "usr_api",
            request_for("cap.workflow.approve-payroll"),
            "req-foundation-error",
            "idem-foundation-error",
        ),
    );

    assert_eq!(
        result,
        Err(CapabilityInvokeApiError::Foundation(
            FoundationError::TenantNotFound
        ))
    );
    assert_eq!(CAPABILITY_INVOKE_SURFACE, "foundry.capability.invoke");
    assert_eq!(ledger.len(), 1);
}

#[test]
fn capability_invoke_api_success_path_preserves_request_id_metadata() {
    let mut foundation = Foundation::default();
    let mut ledger = CapabilityInvokeIdempotencyLedger::default();
    seed_passing_eval(&mut foundation, "cap.workflow.approve-payroll");
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_api".to_string(),
            legal_name: "API Tenant".to_string(),
            home_region: "failover-region".to_string(),
            residency_class: "global".into(),
            regulatory_packs: vec!["pack-gamma".to_string()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .expect("tenant can be onboarded");
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: "ten_api".to_string(),
            user_id: "usr_api".to_string(),
            primary_identifier: "api@example.test".to_string(),
            display_name: "API User".to_string(),
            roles: vec!["tenant-admin".to_string()],
        })
        .expect("identity can be upserted");
    publish_invoke_policy(&mut foundation, "ten_api", "tenant-admin");
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.workflow.approve-payroll".to_string(),
            namespace: "workflow".to_string(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T2Advisory,
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
                DataClass::InternalOnly,
            ])
            .unwrap(),
            evidence_topic: "oyatie.foundry.capability.invoked".to_string(),
        })
        .expect("capability can be registered after eval readiness");
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_api".to_string(),
            capability_id: capability.id.clone(),
            mcp_visible: true,
        })
        .expect("capability can be granted to tenant");
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: "ten_api".to_string(),
            capability_id: None,
            window_id: "2026-05".to_string(),
            monthly_limit_micros: 1_000,
            per_invocation_limit_micros: 100,
            warning_threshold_percent: 80,
        })
        .expect("tenant budget can be configured");

    let response = invoke_capability_from_api(
        &mut foundation,
        &mut ledger,
        api_request_for(
            &capability.id,
            "usr_api",
            request_for(&capability.id),
            "req-202-full-path",
            "idem-202-full-path",
        ),
    )
    .expect("configured API invocation succeeds");

    assert_eq!(response.metadata.request_id, "req-202-full-path");
    assert_eq!(response.data.tenant_id, "ten_api");
    assert_eq!(response.data.user_id, "usr_api");
    assert_eq!(response.data.capability_id, capability.id);
    assert!(response.data.run_id.is_some());
    assert!(response.data.foundry_step_id.is_some());
    assert!(response.data.foundry_evidence_id.is_some());
    assert_eq!(ledger.len(), 1);
}

#[test]
fn capability_invoke_api_replays_same_idempotency_key_without_foundation_side_effects() {
    let (mut foundation, capability_id) =
        configured_foundation_for_api_capability("cap.workflow.approve-payroll");
    let mut ledger = CapabilityInvokeIdempotencyLedger::default();
    let first_request = api_request_for(
        &capability_id,
        "usr_api",
        request_for(&capability_id),
        "req-replay-original",
        "idem-replay-success",
    );
    let first_response = invoke_capability_from_api(&mut foundation, &mut ledger, first_request)
        .expect("first request succeeds");
    let counts_after_first = side_effect_counts(&foundation);

    let replay_request = api_request_for(
        &capability_id,
        "usr_api",
        request_for(&capability_id),
        "req-replay-ignored",
        "idem-replay-success",
    );
    let replay_response = invoke_capability_from_api(&mut foundation, &mut ledger, replay_request)
        .expect("same idempotency key and fingerprint replays");

    assert_eq!(replay_response, first_response);
    assert_eq!(side_effect_counts(&foundation), counts_after_first);
    assert_eq!(ledger.len(), 1);
    assert!(foundation.audit_chain().verify());
}

#[test]
fn capability_invoke_api_rejects_same_idempotency_key_different_fingerprint_before_foundation() {
    let (mut foundation, capability_id) =
        configured_foundation_for_api_capability("cap.workflow.approve-payroll");
    let mut ledger = CapabilityInvokeIdempotencyLedger::default();
    let first_request = api_request_for(
        &capability_id,
        "usr_api",
        request_for(&capability_id),
        "req-conflict-original",
        "idem-conflict",
    );
    invoke_capability_from_api(&mut foundation, &mut ledger, first_request)
        .expect("first request succeeds");
    let counts_after_first = side_effect_counts(&foundation);

    let mut changed_body = request_for(&capability_id);
    changed_body.projected_cost_micros += 1;
    let conflict_request = api_request_for(
        &capability_id,
        "usr_api",
        changed_body,
        "req-conflict-changed",
        "idem-conflict",
    );
    let result = invoke_capability_from_api(&mut foundation, &mut ledger, conflict_request);

    assert_eq!(
        result,
        Err(CapabilityInvokeApiError::IdempotencyKeyReused {
            idempotency_key: "idem-conflict".to_string(),
        })
    );
    assert_eq!(
        CapabilityInvokeApiError::IdempotencyKeyReused {
            idempotency_key: "idem-conflict".to_string(),
        }
        .status_code(),
        400
    );
    assert_eq!(side_effect_counts(&foundation), counts_after_first);
    assert_eq!(ledger.len(), 1);
}

#[test]
fn capability_invoke_api_replays_foundation_mapped_400_without_foundation_side_effects() {
    let (mut foundation, capability_id) =
        configured_foundation_for_api_capability("cap.workflow.approve-payroll");
    let mut ledger = CapabilityInvokeIdempotencyLedger::default();
    let mut invalid_body = request_for(&capability_id);
    invalid_body.projected_cost_micros = 0;

    let first_result = invoke_capability_from_api(
        &mut foundation,
        &mut ledger,
        api_request_for(
            &capability_id,
            "usr_api",
            invalid_body.clone(),
            "req-foundation-400-original",
            "idem-foundation-400",
        ),
    );
    assert_eq!(
        first_result,
        Err(CapabilityInvokeApiError::Foundation(
            FoundationError::InvalidInput
        ))
    );
    assert_eq!(ledger.len(), 1);
    let counts_after_first = side_effect_counts(&foundation);

    let replay_result = invoke_capability_from_api(
        &mut foundation,
        &mut ledger,
        api_request_for(
            &capability_id,
            "usr_api",
            invalid_body,
            "req-foundation-400-replay",
            "idem-foundation-400",
        ),
    );

    assert_eq!(replay_result, first_result);
    assert_eq!(side_effect_counts(&foundation), counts_after_first);
    assert_eq!(ledger.len(), 1);
}

#[test]
fn capability_invoke_api_does_not_cache_pre_foundation_validation_errors() {
    let (mut foundation, capability_id) =
        configured_foundation_for_api_capability("cap.workflow.approve-payroll");
    let mut ledger = CapabilityInvokeIdempotencyLedger::default();
    let invalid_request = api_request_for(
        "cap.workflow.export-ledger",
        "usr_api",
        request_for(&capability_id),
        "req-validation-error",
        "idem-validation-error",
    );
    let invalid_result = invoke_capability_from_api(&mut foundation, &mut ledger, invalid_request);
    assert!(matches!(
        invalid_result,
        Err(CapabilityInvokeApiError::CapabilityIdMismatch { .. })
    ));
    assert!(ledger.is_empty());

    let valid_request = api_request_for(
        &capability_id,
        "usr_api",
        request_for(&capability_id),
        "req-validation-retry",
        "idem-validation-error",
    );
    let response = invoke_capability_from_api(&mut foundation, &mut ledger, valid_request)
        .expect("valid retry with same key is not poisoned by pre-foundation reject");

    assert_eq!(response.metadata.request_id, "req-validation-retry");
    assert_eq!(ledger.len(), 1);
    assert_eq!(foundation.foundry_runs().len(), 1);
}

#[test]
fn capability_invoke_api_surfaces_foundation_data_use_denial_for_underdeclared_ads_action() {
    let mut foundation = Foundation::default();
    let mut ledger = CapabilityInvokeIdempotencyLedger::default();
    seed_passing_eval(&mut foundation, "cap.ads.underdeclared-api");
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_api".to_string(),
            legal_name: "API Ads Tenant".to_string(),
            home_region: "failover-region".to_string(),
            residency_class: "global".into(),
            regulatory_packs: vec!["pack-gamma".to_string()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .expect("tenant can be onboarded");
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: "ten_api".to_string(),
            user_id: "usr_api".to_string(),
            primary_identifier: "api-ads@example.test".to_string(),
            display_name: "API Ads User".to_string(),
            roles: vec!["tenant-admin".to_string()],
        })
        .expect("identity can be upserted");
    publish_invoke_policy(&mut foundation, "ten_api", "tenant-admin");
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.ads.underdeclared-api".to_string(),
            namespace: "ads".to_string(),
            action: CapabilityAction::AdsBid,
            required_tier: AutonomyTier::T1ViewOnly,
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
                DataClass::InternalOnly,
            ])
            .unwrap(),
            evidence_topic: "oyatie.foundry.capability.invoked".to_string(),
        })
        .expect("capability can be registered after eval readiness");
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_api".to_string(),
            capability_id: capability.id.clone(),
            mcp_visible: true,
        })
        .expect("capability can be granted to tenant");
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: "ten_api".to_string(),
            capability_id: None,
            window_id: "2026-05".to_string(),
            monthly_limit_micros: 1_000,
            per_invocation_limit_micros: 100,
            warning_threshold_percent: 80,
        })
        .expect("tenant budget can be configured");

    let result = invoke_capability_from_api(
        &mut foundation,
        &mut ledger,
        api_request_for(
            &capability.id,
            "usr_api",
            request_for(&capability.id),
            "req-underdeclared-ads",
            "idem-underdeclared-ads",
        ),
    );

    assert_eq!(
        result,
        Err(CapabilityInvokeApiError::Foundation(
            FoundationError::DataUseNotAllowed
        ))
    );
    assert_eq!(ledger.len(), 1);
    assert!(foundation.audit_chain().events().iter().any(|event| {
        event.surface == "privacy.data-use.evaluate"
            && event.purpose == Purpose::AdsTargeting
            && event.decision == "DENY"
    }));
    assert!(
        foundation
            .foundry_evidence_chain()
            .records()
            .last()
            .expect("data-use denial records evidence")
            .fields
            .value
            .get("data_use_denial_reason")
            .is_some_and(|reason| reason == "underdeclared_ads_purpose")
    );
    let counts_after_denial = side_effect_counts(&foundation);

    let replay_result = invoke_capability_from_api(
        &mut foundation,
        &mut ledger,
        api_request_for(
            &capability.id,
            "usr_api",
            request_for(&capability.id),
            "req-underdeclared-ads-replay",
            "idem-underdeclared-ads",
        ),
    );
    assert_eq!(
        replay_result,
        Err(CapabilityInvokeApiError::Foundation(
            FoundationError::DataUseNotAllowed
        ))
    );
    assert_eq!(side_effect_counts(&foundation), counts_after_denial);
}

#[test]
fn capability_invoke_api_status_mapping_matches_openapi_responses() {
    assert_eq!(CapabilityInvokeApiStatus::Accepted.code(), 202);
    assert_eq!(CapabilityInvokeApiStatus::BadRequest.code(), 400);
    assert_eq!(CapabilityInvokeApiStatus::Forbidden.code(), 403);

    assert_eq!(
        CapabilityInvokeApiError::EmptyPathCapabilityId.status_code(),
        400
    );
    assert_eq!(
        CapabilityInvokeApiError::CapabilityIdMismatch {
            path_capability_id: "cap.workflow.approve-payroll".to_string(),
            body_capability_id: "cap.workflow.export-ledger".to_string(),
        }
        .status_code(),
        400
    );
    assert_eq!(CapabilityInvokeApiError::EmptyRequestId.status_code(), 400);
    assert_eq!(
        CapabilityInvokeApiError::EmptyTenantHeader.status_code(),
        400
    );
    assert_eq!(
        CapabilityInvokeApiError::EmptyIdempotencyKey.status_code(),
        400
    );
    assert_eq!(
        CapabilityInvokeApiError::IdempotencyKeyReused {
            idempotency_key: "idem-reused".to_string(),
        }
        .status_code(),
        400
    );
    assert_eq!(
        CapabilityInvokeApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: "ten_api".to_string(),
            body_tenant_id: "ten_api".to_string(),
        }
        .status_code(),
        403
    );
    assert_eq!(
        CapabilityInvokeApiError::PrincipalMismatch {
            principal_tenant_id: "ten_api".to_string(),
            principal_user_id: "usr_impersonator".to_string(),
            body_tenant_id: "ten_api".to_string(),
            body_user_id: "usr_api".to_string(),
        }
        .status_code(),
        403
    );
    assert_eq!(
        CapabilityInvokeApiError::Foundation(FoundationError::InvalidInput).status_code(),
        400
    );
    assert_eq!(
        CapabilityInvokeApiError::Foundation(FoundationError::TenantNotFound).status_code(),
        403
    );
    assert_eq!(
        CapabilityInvokeApiError::Foundation(FoundationError::CapabilityNotLicensed).status(),
        CapabilityInvokeApiStatus::Forbidden
    );
}

#[test]
fn capability_invoke_api_success_response_matches_openapi_envelope_shape() {
    let receipt = CapabilityInvocationReceipt {
        tenant_id: "tenant.acme".to_string(),
        user_id: "user.alice".to_string(),
        capability_id: "cap.workflow.approve-payroll".to_string(),
        evidence_event_hash: "fnv1a64:accepted".to_string(),
        cost_reservation_id: Some("reservation-1".to_string()),
        cost_budget_warning: None,
        run_id: Some("run_000000000001".to_string()),
        foundry_step_id: Some("step_000000000001_000001".to_string()),
        foundry_evidence_id: Some("ev_000000000001".to_string()),
    };

    let response = CapabilityInvokeApiSuccessResponse::accepted(receipt.clone(), "req-202");

    assert_eq!(
        response,
        CapabilityInvokeApiSuccessResponse {
            data: receipt,
            metadata: CapabilityInvokeApiResponseMetadata {
                request_id: "req-202".to_string(),
            },
        }
    );
}

#[test]
fn capability_invoke_api_error_response_matches_openapi_error_shape() {
    let bad_request = CapabilityInvokeApiError::CapabilityIdMismatch {
        path_capability_id: "cap.workflow.approve-payroll".to_string(),
        body_capability_id: "cap.workflow.export-ledger".to_string(),
    }
    .error_response("req-400");

    assert_eq!(
        bad_request,
        CapabilityInvokeApiErrorResponse {
            error: CapabilityInvokeApiErrorBody {
                code: CapabilityInvokeApiErrorCode::CapabilityIdMismatch
                    .as_str()
                    .to_string(),
                message: "Path and body capability ids must match".to_string(),
                message_localized: None,
                request_id: "req-400".to_string(),
                details: vec![CapabilityInvokeApiErrorDetail {
                    field: "capability_id".to_string(),
                    issue: "path and body capability_id must match".to_string(),
                }],
                retry_after_seconds: None,
            },
        }
    );

    let forbidden = CapabilityInvokeApiError::Foundation(FoundationError::TenantNotFound)
        .error_response("req-403");

    assert_eq!(
        forbidden,
        CapabilityInvokeApiErrorResponse {
            error: CapabilityInvokeApiErrorBody {
                code: CapabilityInvokeApiErrorCode::CapabilityInvocationForbidden
                    .as_str()
                    .to_string(),
                message: "Foundation policy rejected the capability invocation".to_string(),
                message_localized: None,
                request_id: "req-403".to_string(),
                details: vec![CapabilityInvokeApiErrorDetail {
                    field: "foundation".to_string(),
                    issue: "foundation policy rejected invocation".to_string(),
                }],
                retry_after_seconds: None,
            },
        }
    );
}

fn configured_foundation_for_api_capability(capability_id: &str) -> (Foundation, String) {
    let mut foundation = Foundation::default();
    seed_passing_eval(&mut foundation, capability_id);
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_api".to_string(),
            legal_name: "API Tenant".to_string(),
            home_region: "failover-region".to_string(),
            residency_class: "global".into(),
            regulatory_packs: vec!["pack-gamma".to_string()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .expect("tenant can be onboarded");
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: "ten_api".to_string(),
            user_id: "usr_api".to_string(),
            primary_identifier: "api@example.test".to_string(),
            display_name: "API User".to_string(),
            roles: vec!["tenant-admin".to_string()],
        })
        .expect("identity can be upserted");
    publish_invoke_policy(&mut foundation, "ten_api", "tenant-admin");
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: capability_id.to_string(),
            namespace: "workflow".to_string(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T2Advisory,
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
                DataClass::InternalOnly,
            ])
            .unwrap(),
            evidence_topic: "oyatie.foundry.capability.invoked".to_string(),
        })
        .expect("capability can be registered after eval readiness");
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_api".to_string(),
            capability_id: capability.id.clone(),
            mcp_visible: true,
        })
        .expect("capability can be granted to tenant");
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: "ten_api".to_string(),
            capability_id: None,
            window_id: "2026-05".to_string(),
            monthly_limit_micros: 1_000,
            per_invocation_limit_micros: 100,
            warning_threshold_percent: 80,
        })
        .expect("tenant budget can be configured");
    (foundation, capability.id)
}

fn seed_passing_eval(foundation: &mut Foundation, capability_id: &str) {
    foundation
        .register_capability_eval_set(passing_eval_set(capability_id))
        .expect("eval set gates capability publish");
    foundation
        .record_capability_eval_run(passing_eval_run(capability_id))
        .expect("passing eval run gates capability publish");
}

fn publish_invoke_policy(foundation: &mut Foundation, tenant_id: &str, role: &str) {
    foundation
        .publish_policy(PolicyVersion {
            policy_id: format!("pol_{tenant_id}_{role}_invoke").replace('-', "_"),
            version: "1.0.0".to_string(),
            scope: PolicyScope::Tenant(tenant_id.to_string()),
            supersedes: None,
            rules: vec![PolicyRuleInput {
                effect: PolicyEffect::Allow,
                principal_role: role.to_string(),
                action: "foundry.capability.invoke".to_string(),
                resource_prefix: "capability:cap.".to_string(),
                required_attribute: None,
                annotations: vec![],
            }],
        })
        .expect("invoke policy is valid");
}

/// A publishable eval set: full adversarial coverage across the required linguistic cohorts.
///
/// The locales are load-bearing. `validate_linguistic_coverage` requires every locale in
/// `REQUIRED_LINGUISTIC_COHORT_LOCALES` (`lang-alpha1`, `lang-beta1`, `lang-gamma1`) to appear
/// among the cases. This fixture used `generic` / `pack-primary` / `pack-secondary`, so the set
/// failed that check and every publish surfaced as `CapabilityEvalGateNotReady` — the six tests
/// below never reached the invoke behaviour they were written to cover.
fn passing_eval_set(capability_id: &str) -> EvalSetInput {
    let mut cases = vec![
        eval_case("case-cohort-alpha", "lang-alpha1", None),
        eval_case("case-cohort-beta", "lang-beta1", None),
        eval_case("case-cohort-gamma", "lang-gamma1", None),
    ];
    for (case_id, locale, kind) in [
        (
            "adv-prompt",
            "lang-alpha1",
            AdversarialKind::PromptInjection,
        ),
        (
            "adv-class",
            "lang-beta1",
            AdversarialKind::DataClassViolation,
        ),
        (
            "adv-autonomy",
            "lang-gamma1",
            AdversarialKind::AutonomyBypass,
        ),
        ("adv-tool", "lang-alpha1", AdversarialKind::ToolExfiltration),
    ] {
        cases.push(eval_case(case_id, locale, Some(kind)));
    }
    EvalSetInput {
        capability_id: capability_id.to_string(),
        version: "eval-v1".to_string(),
        metric: EvalMetric::ExactMatch,
        min_pass_rate_percent: 80,
        min_p95_score_percent: 80,
        signed: true,
        cases,
    }
}

fn passing_eval_run(capability_id: &str) -> EvalRunInput {
    EvalRunInput {
        capability_id: capability_id.to_string(),
        eval_set_version: "eval-v1".to_string(),
        pass_rate_percent: 95,
        p95_score_percent: 90,
        adversarial_passed: true,
        linguistic_passed: true,
        signed: true,
    }
}

fn eval_case(
    case_id: &str,
    locale: &str,
    adversarial_kind: Option<AdversarialKind>,
) -> EvalCaseInput {
    EvalCaseInput {
        case_id: case_id.to_string(),
        locale: locale.to_string(),
        input_ref: format!("inputs/{case_id}.json"),
        expected_ref: format!("expected/{case_id}.json"),
        adversarial_kind,
        deterministic_seed: Some(42),
    }
}
