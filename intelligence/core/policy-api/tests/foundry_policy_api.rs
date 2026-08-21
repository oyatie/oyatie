// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use intelligence_policy_api::{
    FOUNDRY_POLICY_AUTONOMY_CEILING_PUBLISH_SURFACE, FOUNDRY_POLICY_OPENAPI_CONTRACT,
    FoundryPolicyApiAuthorization, FoundryPolicyApiError, FoundryPolicyApiPrincipal,
    FoundryPolicyAutonomyBoundaryContext, FoundryPolicyAutonomyCeilingDirectory,
    FoundryPolicyAutonomyCeilingPublishApiRequest, FoundryPolicyAutonomyCeilingPublishApiStatus,
    FoundryPolicyAutonomyCeilingPublishIdempotencyLedger,
    FoundryPolicyAutonomyCeilingPublishRequest, FoundryPolicyAutonomyCeilingRecord,
    FoundryPolicyAutonomyMetadata, publish_foundry_policy_autonomy_ceiling_from_api,
};

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(
        FOUNDRY_POLICY_AUTONOMY_CEILING_PUBLISH_SURFACE,
        "foundry.policy.autonomy-ceiling.publish"
    );
    assert_eq!(
        FOUNDRY_POLICY_OPENAPI_CONTRACT,
        "contracts/openapi/foundry/policy-v1.yaml"
    );
    assert_eq!(
        FoundryPolicyAutonomyCeilingPublishApiStatus::Created.code(),
        201
    );
    assert_eq!(
        FoundryPolicyAutonomyCeilingPublishApiStatus::BadRequest.code(),
        400
    );
    assert_eq!(
        FoundryPolicyAutonomyCeilingPublishApiStatus::Forbidden.code(),
        403
    );
    assert_eq!(
        FoundryPolicyAutonomyCeilingPublishApiStatus::Conflict.code(),
        409
    );
    assert_eq!(
        FoundryPolicyAutonomyCeilingPublishApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn autonomy_ceiling_publish_records_effective_policy_and_replays_idempotently() {
    let mut directory = FoundryPolicyAutonomyCeilingDirectory::default();
    let mut ledger = FoundryPolicyAutonomyCeilingPublishIdempotencyLedger::default();
    let request = api_request("idem-1");

    let response = publish_foundry_policy_autonomy_ceiling_from_api(
        &mut directory,
        &mut ledger,
        request.clone(),
    )
    .expect("policy should publish");

    assert_eq!(response.metadata.request_id, "req-1");
    assert_eq!(
        response.metadata.surface,
        FOUNDRY_POLICY_AUTONOMY_CEILING_PUBLISH_SURFACE
    );
    assert_eq!(response.data.policy_id, "policy-ten-alpha-cap-demo");
    assert_eq!(response.data.tenant_id, "ten_alpha");
    assert_eq!(response.data.capability_id, "cap.demo.draft");
    assert_eq!(response.data.configured_ceiling, "t3_execute_with_approval");
    assert_eq!(response.data.principal_ceiling, "t3_execute_with_approval");
    assert_eq!(response.data.capability_required_tier, "t2_advisory");
    assert_eq!(response.data.effective_ceiling, "t2_advisory");
    assert_eq!(response.data.denial_threshold, "t3_execute_with_approval");
    assert_eq!(response.data.verdict, "allow");
    assert_eq!(response.data.blocking_cap_source, None);
    assert_eq!(response.data.lowering_cap_source, "capability_required");
    assert_eq!(
        response.data.cedar_policy_refs,
        vec!["cedar://ten_alpha/foundry/autonomy/v1"]
    );
    assert_eq!(response.data.schema_version, 1);
    assert_eq!(directory.len(), 1);
    assert_eq!(ledger.len(), 1);

    let replay =
        publish_foundry_policy_autonomy_ceiling_from_api(&mut directory, &mut ledger, request)
            .expect("same idempotency key should replay");
    assert_eq!(replay, response);
    assert_eq!(directory.len(), 1);
    assert_eq!(ledger.len(), 1);
}

#[test]
fn autonomy_ceiling_publish_captures_agentic_ads_and_subject_class_caps() {
    let mut directory = FoundryPolicyAutonomyCeilingDirectory::default();
    let mut ledger = FoundryPolicyAutonomyCeilingPublishIdempotencyLedger::default();
    let mut request = api_request("idem-minor-ads");
    request.body.policy_id = "policy-ten-alpha-ads-bid".into();
    request.path_policy_id = request.body.policy_id.clone();
    request.body.capability_id = "cap.ads.campaign.bid".into();
    request.body.capability_action = "ads_bid".into();
    request.body.capability_required_tier = "t3_execute_with_approval".into();
    request.body.tenant_configured_ceiling = "t4_auto_execute".into();
    request.body.principal_ceiling = "t4_auto_execute".into();
    request.body.data_classes = vec!["BEHAVIORAL_ADS".into()];
    request.body.subject_class = "minor_under14".into();

    let response =
        publish_foundry_policy_autonomy_ceiling_from_api(&mut directory, &mut ledger, request)
            .expect("denying policy decisions are still publishable evidence");

    assert_eq!(response.data.agentic_ads_cap, "t1_view_only");
    assert_eq!(response.data.subject_class_cap, "t1_view_only");
    assert_eq!(response.data.effective_ceiling, "t1_view_only");
    assert_eq!(response.data.verdict, "deny");
    assert_eq!(
        response.data.blocking_cap_source,
        Some("agentic_ads".into())
    );
    assert_eq!(
        response.data.blocking_cap_reason,
        Some("agentic_ads_default".into())
    );
}

#[test]
fn autonomy_ceiling_publish_rejects_path_body_tenant_and_principal_drift_before_mutation() {
    let mut directory = FoundryPolicyAutonomyCeilingDirectory::default();
    let mut ledger = FoundryPolicyAutonomyCeilingPublishIdempotencyLedger::default();

    let mut path_drift = api_request("idem-path-drift");
    path_drift.path_policy_id = "different-policy".into();
    let err =
        publish_foundry_policy_autonomy_ceiling_from_api(&mut directory, &mut ledger, path_drift)
            .expect_err("path/body policy drift must fail");
    assert_eq!(err.status_code(), 400);

    let mut tenant_drift = api_request("idem-tenant-drift");
    tenant_drift.body.tenant_id = "ten_beta".into();
    let err =
        publish_foundry_policy_autonomy_ceiling_from_api(&mut directory, &mut ledger, tenant_drift)
            .expect_err("tenant drift must fail before mutation");
    assert_eq!(err.status_code(), 403);

    let mut principal_drift = api_request("idem-principal-drift");
    principal_drift.authorization.principal_id = "user-other".into();
    let err = publish_foundry_policy_autonomy_ceiling_from_api(
        &mut directory,
        &mut ledger,
        principal_drift,
    )
    .expect_err("authorization principal drift must fail");
    assert_eq!(err.status_code(), 403);
    assert_eq!(directory.len(), 0);
    assert_eq!(ledger.len(), 0);
}

#[test]
fn autonomy_ceiling_publish_rejects_authorization_duplicate_and_reused_idempotency_key() {
    let mut directory = FoundryPolicyAutonomyCeilingDirectory::default();
    let mut ledger = FoundryPolicyAutonomyCeilingPublishIdempotencyLedger::default();

    let mut denied = api_request("idem-denied");
    denied.authorization.allowed_surfaces = vec!["foundry.eval.run".into()];
    let err = publish_foundry_policy_autonomy_ceiling_from_api(&mut directory, &mut ledger, denied)
        .expect_err("missing surface authorization must fail");
    assert_eq!(err.status_code(), 403);

    let first = api_request("idem-first");
    publish_foundry_policy_autonomy_ceiling_from_api(&mut directory, &mut ledger, first)
        .expect("first publish succeeds");

    let mut duplicate = api_request("idem-second");
    duplicate.boundary.request_id = "req-2".into();
    let err =
        publish_foundry_policy_autonomy_ceiling_from_api(&mut directory, &mut ledger, duplicate)
            .expect_err("same tenant/policy with new key is duplicate");
    assert_eq!(err.status_code(), 409);

    let mut drift = api_request("idem-first");
    drift.body.capability_required_tier = "t3_execute_with_approval".into();
    let err = publish_foundry_policy_autonomy_ceiling_from_api(&mut directory, &mut ledger, drift)
        .expect_err("same idempotency key with drift must fail");
    assert_eq!(err.status_code(), 422);
}

#[test]
fn autonomy_ceiling_publish_maps_invalid_labels_and_missing_cedar_refs() {
    let mut directory = FoundryPolicyAutonomyCeilingDirectory::default();
    let mut ledger = FoundryPolicyAutonomyCeilingPublishIdempotencyLedger::default();

    let mut invalid_tier = api_request("idem-invalid-tier");
    invalid_tier.body.tenant_configured_ceiling = "t5_unbounded".into();
    let err =
        publish_foundry_policy_autonomy_ceiling_from_api(&mut directory, &mut ledger, invalid_tier)
            .expect_err("unknown tier must fail");
    assert_eq!(err.status_code(), 400);

    let mut invalid_data_class = api_request("idem-invalid-class");
    invalid_data_class.body.data_classes = vec!["AUDIT".into()];
    let err = publish_foundry_policy_autonomy_ceiling_from_api(
        &mut directory,
        &mut ledger,
        invalid_data_class,
    )
    .expect_err("operational classes cannot be smuggled into capability privacy classes");
    assert_eq!(err.status_code(), 400);

    let mut invalid_subject = api_request("idem-invalid-subject");
    invalid_subject.body.subject_class = "child".into();
    let err = publish_foundry_policy_autonomy_ceiling_from_api(
        &mut directory,
        &mut ledger,
        invalid_subject,
    )
    .expect_err("subject class labels are closed");
    assert_eq!(err.status_code(), 400);

    let mut missing_cedar = api_request("idem-missing-cedar");
    missing_cedar.body.cedar_policy_refs.clear();
    let err = publish_foundry_policy_autonomy_ceiling_from_api(
        &mut directory,
        &mut ledger,
        missing_cedar,
    )
    .expect_err("Cedar-backed publish must cite at least one policy ref");
    assert_eq!(err.status_code(), 400);
    assert_eq!(directory.len(), 0);
    assert_eq!(ledger.len(), 0);
}

#[test]
fn stable_error_response_shape_uses_request_id_and_field_details() {
    let err = FoundryPolicyApiError::InvalidAutonomyTierLabel {
        field: "body.tenant_configured_ceiling".into(),
        autonomy_tier: "t5_unbounded".into(),
    };
    let response = err.error_response("req-error");
    assert_eq!(response.error.code, "FOUNDRY_POLICY_AUTONOMY_TIER_INVALID");
    assert_eq!(response.error.request_id, "req-error");
    assert_eq!(response.error.retry_after_seconds, None);
    assert_eq!(
        response.error.details[0].field,
        "body.tenant_configured_ceiling"
    );
}

#[test]
fn public_response_structs_keep_contract_names_stable() {
    assert!(
        std::any::type_name::<FoundryPolicyAutonomyCeilingRecord>()
            .contains("FoundryPolicyAutonomyCeilingRecord")
    );
    assert!(
        std::any::type_name::<FoundryPolicyAutonomyMetadata>()
            .contains("FoundryPolicyAutonomyMetadata")
    );
}

fn api_request(idempotency_key: &str) -> FoundryPolicyAutonomyCeilingPublishApiRequest {
    FoundryPolicyAutonomyCeilingPublishApiRequest {
        path_policy_id: "policy-ten-alpha-cap-demo".into(),
        boundary: FoundryPolicyAutonomyBoundaryContext {
            request_id: "req-1".into(),
            tenant_id: "ten_alpha".into(),
            idempotency_key: idempotency_key.into(),
        },
        principal: FoundryPolicyApiPrincipal {
            tenant_id: "ten_alpha".into(),
            principal_id: "user-alpha".into(),
        },
        authorization: FoundryPolicyApiAuthorization {
            tenant_id: "ten_alpha".into(),
            principal_id: "user-alpha".into(),
            decision_id: "authz-1".into(),
            allowed_surfaces: vec![FOUNDRY_POLICY_AUTONOMY_CEILING_PUBLISH_SURFACE.into()],
        },
        body: FoundryPolicyAutonomyCeilingPublishRequest {
            policy_id: "policy-ten-alpha-cap-demo".into(),
            tenant_id: "ten_alpha".into(),
            capability_id: "cap.demo.draft".into(),
            policy_version: "1.0.0".into(),
            tenant_configured_ceiling: "t3_execute_with_approval".into(),
            principal_ceiling: "t3_execute_with_approval".into(),
            capability_required_tier: "t2_advisory".into(),
            capability_action: "other".into(),
            data_classes: vec!["PUBLIC".into()],
            regulatory_packs: vec!["global-baseline".into()],
            subject_class: "adult".into(),
            cedar_policy_refs: vec!["cedar://ten_alpha/foundry/autonomy/v1".into()],
            evidence_event_hash: "fnv1a64:policy".into(),
            published_at_epoch_seconds: 1_778_544_000,
        },
    }
}
