#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use oya_workplace_integration_outbound_metadata_domain::{
    WorkplaceEnvTier, WorkplaceIntegrationActionClass, WorkplaceOutboundEmissionPlan,
    WorkplaceOutboundEmissionPlanCreate, WorkplaceOutboundMetadataError, WorkplaceOutboundMode,
};

fn prod_esign_plan() -> WorkplaceOutboundEmissionPlanCreate {
    WorkplaceOutboundEmissionPlanCreate {
        tenant_id: "ten_alpha".to_string(),
        env_tier: Some(WorkplaceEnvTier::Prod),
        outbound_mode: WorkplaceOutboundMode::Live,
        action_class: WorkplaceIntegrationActionClass::ESignSessionInitiation,
        destination_binding_ref: Some(
            "workplace-integration-destination/ten_alpha/prod/live/esign-binding".to_string(),
        ),
        consent_policy_evidence_ref:
            "policy-evidence/workplace-integration/ten_alpha/prod/consent-authorization/esign-session-001"
                .to_string(),
        tenancy_env_tier_evidence_ref:
            "tenancy-evidence/workplace-integration/ten_alpha/prod/env-tier/policy-allow-001"
                .to_string(),
        runtime_delivery_authorized: false,
    }
}

#[test]
fn prod_esign_plan_records_live_mode_and_required_metadata_refs() {
    let plan = WorkplaceOutboundEmissionPlan::new(prod_esign_plan())
        .expect("complete prod e-sign metadata plan should be accepted");

    assert_eq!(plan.tenant_id, "ten_alpha");
    assert_eq!(plan.env_tier, WorkplaceEnvTier::Prod);
    assert_eq!(plan.outbound_mode, WorkplaceOutboundMode::Live);
    assert_eq!(
        plan.action_class,
        WorkplaceIntegrationActionClass::ESignSessionInitiation
    );
    assert_eq!(
        plan.destination_binding_ref.as_deref(),
        Some("workplace-integration-destination/ten_alpha/prod/live/esign-binding")
    );
    assert_eq!(
        WorkplaceEnvTier::Staging.derived_outbound_mode(),
        WorkplaceOutboundMode::TestRecipients
    );
}

#[test]
fn test_tier_preserves_intercept_log_only_without_destination_transport() {
    let mut plan = prod_esign_plan();
    plan.env_tier = Some(WorkplaceEnvTier::Test);
    plan.outbound_mode = WorkplaceOutboundMode::Intercept;
    plan.action_class = WorkplaceIntegrationActionClass::RosterBinding;
    plan.destination_binding_ref = None;
    plan.consent_policy_evidence_ref =
        "policy-evidence/workplace-integration/ten_alpha/test/consent-log-only/roster-binding-001"
            .to_string();
    plan.tenancy_env_tier_evidence_ref =
        "tenancy-evidence/workplace-integration/ten_alpha/test/env-tier/intercept-log-only-001"
            .to_string();

    let accepted = WorkplaceOutboundEmissionPlan::new(plan)
        .expect("test tier metadata plan should stay intercept/log-only");

    assert_eq!(accepted.env_tier, WorkplaceEnvTier::Test);
    assert_eq!(accepted.outbound_mode, WorkplaceOutboundMode::Intercept);
    assert!(accepted.destination_binding_ref.is_none());
}

#[test]
fn staging_test_recipients_qa_destination_is_preserved() {
    let mut plan = prod_esign_plan();
    plan.env_tier = Some(WorkplaceEnvTier::Staging);
    plan.outbound_mode = WorkplaceOutboundMode::TestRecipients;
    plan.action_class = WorkplaceIntegrationActionClass::OfferGeneration;
    plan.destination_binding_ref = Some(
        "workplace-integration-destination/ten_alpha/staging/qa/offer-recipient-binding"
            .to_string(),
    );
    plan.consent_policy_evidence_ref =
        "policy-evidence/workplace-integration/ten_alpha/staging/consent-qa/offer-generation-001"
            .to_string();
    plan.tenancy_env_tier_evidence_ref =
        "tenancy-evidence/workplace-integration/ten_alpha/staging/env-tier/qa-recipient-001"
            .to_string();

    let accepted = WorkplaceOutboundEmissionPlan::new(plan)
        .expect("staging metadata plan should allow only QA/test recipients");

    assert_eq!(accepted.env_tier, WorkplaceEnvTier::Staging);
    assert_eq!(
        accepted.outbound_mode,
        WorkplaceOutboundMode::TestRecipients
    );
    assert!(
        accepted
            .destination_binding_ref
            .as_deref()
            .is_some_and(|destination| destination.contains("/qa/"))
    );
}

#[test]
fn missing_env_tier_is_denied() {
    let mut plan = prod_esign_plan();
    plan.env_tier = None;

    let error = WorkplaceOutboundEmissionPlan::new(plan).expect_err("env_tier is required");

    assert_eq!(error, WorkplaceOutboundMetadataError::MissingEnvTier);
}

#[test]
fn outbound_mode_must_be_derived_from_env_tier() {
    let mut plan = prod_esign_plan();
    plan.env_tier = Some(WorkplaceEnvTier::Test);
    plan.outbound_mode = WorkplaceOutboundMode::Live;

    let error = WorkplaceOutboundEmissionPlan::new(plan)
        .expect_err("outbound_mode must be derived from env_tier");

    assert_eq!(
        error,
        WorkplaceOutboundMetadataError::InvalidOutboundModeForTier
    );
}

#[test]
fn test_tier_attempting_external_esign_workplace_or_payment_adjacent_delivery_is_denied() {
    for action in [
        WorkplaceIntegrationActionClass::ESignSessionInitiation,
        WorkplaceIntegrationActionClass::WorkplaceExternalNotification,
        WorkplaceIntegrationActionClass::PaymentAdjacentWebhook,
    ] {
        let mut plan = prod_esign_plan();
        plan.env_tier = Some(WorkplaceEnvTier::Test);
        plan.outbound_mode = WorkplaceOutboundMode::Intercept;
        plan.action_class = action;
        plan.destination_binding_ref = Some(
            "workplace-integration-destination/ten_alpha/test/qa/forbidden-external-delivery"
                .to_string(),
        );
        plan.consent_policy_evidence_ref =
            "policy-evidence/workplace-integration/ten_alpha/test/consent-log-only/no-delivery"
                .to_string();
        plan.tenancy_env_tier_evidence_ref =
            "tenancy-evidence/workplace-integration/ten_alpha/test/env-tier/intercept-log-only"
                .to_string();

        let error = WorkplaceOutboundEmissionPlan::new(plan).expect_err(
            "test tier cannot carry external workplace/e-sign/payment destination metadata",
        );

        assert_eq!(
            error,
            WorkplaceOutboundMetadataError::TestTierExternalSideEffectForbidden
        );
    }
}

#[test]
fn staging_without_qa_recipient_or_endpoint_is_denied() {
    let mut plan = prod_esign_plan();
    plan.env_tier = Some(WorkplaceEnvTier::Staging);
    plan.outbound_mode = WorkplaceOutboundMode::TestRecipients;
    plan.destination_binding_ref = None;
    plan.consent_policy_evidence_ref =
        "policy-evidence/workplace-integration/ten_alpha/staging/consent-qa/roster-binding"
            .to_string();
    plan.tenancy_env_tier_evidence_ref =
        "tenancy-evidence/workplace-integration/ten_alpha/staging/env-tier/qa-recipient"
            .to_string();

    let error = WorkplaceOutboundEmissionPlan::new(plan)
        .expect_err("staging requires a tenant QA recipient or endpoint binding");

    assert_eq!(error, WorkplaceOutboundMetadataError::MissingQaDestination);
}

#[test]
fn staging_live_destination_is_denied() {
    let mut plan = prod_esign_plan();
    plan.env_tier = Some(WorkplaceEnvTier::Staging);
    plan.outbound_mode = WorkplaceOutboundMode::TestRecipients;
    plan.destination_binding_ref =
        Some("workplace-integration-destination/ten_alpha/prod/live/esign-binding".to_string());
    plan.consent_policy_evidence_ref =
        "policy-evidence/workplace-integration/ten_alpha/staging/consent-qa/esign-session"
            .to_string();
    plan.tenancy_env_tier_evidence_ref =
        "tenancy-evidence/workplace-integration/ten_alpha/staging/env-tier/qa-recipient"
            .to_string();

    let error = WorkplaceOutboundEmissionPlan::new(plan)
        .expect_err("staging cannot point at a live destination binding");

    assert_eq!(
        error,
        WorkplaceOutboundMetadataError::InvalidDestinationBindingForTier
    );
}

#[test]
fn staging_live_destination_with_qa_label_is_denied() {
    let mut plan = prod_esign_plan();
    plan.env_tier = Some(WorkplaceEnvTier::Staging);
    plan.outbound_mode = WorkplaceOutboundMode::TestRecipients;
    plan.destination_binding_ref =
        Some("workplace-integration-destination/ten_alpha/prod/live/qa-shadow-binding".to_string());
    plan.consent_policy_evidence_ref =
        "policy-evidence/workplace-integration/ten_alpha/staging/consent-qa/esign-session"
            .to_string();
    plan.tenancy_env_tier_evidence_ref =
        "tenancy-evidence/workplace-integration/ten_alpha/staging/env-tier/qa-recipient"
            .to_string();

    let error = WorkplaceOutboundEmissionPlan::new(plan)
        .expect_err("staging must require the tenant staging QA/test/sandbox prefix");

    assert_eq!(
        error,
        WorkplaceOutboundMetadataError::InvalidDestinationBindingForTier
    );
}

#[test]
fn prod_destination_must_use_prod_live_prefix() {
    let mut plan = prod_esign_plan();
    plan.destination_binding_ref =
        Some("workplace-integration-destination/ten_alpha/staging/qa/esign-binding".to_string());

    let error = WorkplaceOutboundEmissionPlan::new(plan)
        .expect_err("prod metadata must bind only prod/live destination refs");

    assert_eq!(
        error,
        WorkplaceOutboundMetadataError::InvalidDestinationBindingForTier
    );
}

#[test]
fn prod_esign_offer_webhook_or_payment_adjacent_delivery_without_policy_and_consent_is_denied() {
    let mut missing_tenancy_evidence = prod_esign_plan();
    missing_tenancy_evidence.action_class = WorkplaceIntegrationActionClass::OfferGeneration;
    missing_tenancy_evidence.tenancy_env_tier_evidence_ref =
        "tenancy-evidence/workplace-integration/ten_alpha/prod/env-tier/missing-policy".to_string();

    let tenancy_error = WorkplaceOutboundEmissionPlan::new(missing_tenancy_evidence)
        .expect_err("prod offer metadata requires tenancy/env-tier policy evidence");
    assert_eq!(
        tenancy_error,
        WorkplaceOutboundMetadataError::ProdTenancyEnvTierEvidenceRequired
    );

    let mut missing_consent_evidence = prod_esign_plan();
    missing_consent_evidence.action_class = WorkplaceIntegrationActionClass::PaymentAdjacentWebhook;
    missing_consent_evidence.consent_policy_evidence_ref =
        "policy-evidence/workplace-integration/ten_alpha/prod/missing-consent".to_string();

    let consent_error = WorkplaceOutboundEmissionPlan::new(missing_consent_evidence).expect_err(
        "prod payment-adjacent webhook metadata requires consent/authorization evidence",
    );
    assert_eq!(
        consent_error,
        WorkplaceOutboundMetadataError::ProdConsentAuthorizationEvidenceRequired
    );
}

#[test]
fn evidence_refs_must_bind_same_tenant_segment_and_env_tier() {
    let mut cross_tenant_policy = prod_esign_plan();
    cross_tenant_policy.consent_policy_evidence_ref =
        "policy-evidence/workplace-integration/ten_beta/prod/consent-authorization/ten_alpha-shadow"
            .to_string();

    let error = WorkplaceOutboundEmissionPlan::new(cross_tenant_policy)
        .expect_err("policy evidence tenant segment must match the plan tenant");

    assert_eq!(error, WorkplaceOutboundMetadataError::TenantMismatch);

    let mut staging_with_prod_policy = prod_esign_plan();
    staging_with_prod_policy.env_tier = Some(WorkplaceEnvTier::Staging);
    staging_with_prod_policy.outbound_mode = WorkplaceOutboundMode::TestRecipients;
    staging_with_prod_policy.destination_binding_ref =
        Some("workplace-integration-destination/ten_alpha/staging/qa/esign-binding".to_string());
    staging_with_prod_policy.tenancy_env_tier_evidence_ref =
        "tenancy-evidence/workplace-integration/ten_alpha/staging/env-tier/qa-recipient"
            .to_string();

    let error = WorkplaceOutboundEmissionPlan::new(staging_with_prod_policy)
        .expect_err("policy evidence tier prefix must match env_tier");

    assert_eq!(
        error,
        WorkplaceOutboundMetadataError::InvalidEvidenceRefForTier
    );
}

#[test]
fn cross_tenant_destination_leakage_is_denied() {
    let mut plan = prod_esign_plan();
    plan.destination_binding_ref =
        Some("workplace-integration-destination/ten_beta/prod/live/esign-binding".to_string());

    let error = WorkplaceOutboundEmissionPlan::new(plan)
        .expect_err("destination binding tenant must match plan tenant");

    assert_eq!(error, WorkplaceOutboundMetadataError::TenantMismatch);

    let mut plan_with_matching_suffix = prod_esign_plan();
    plan_with_matching_suffix.destination_binding_ref = Some(
        "workplace-integration-destination/ten_beta/prod/live/ten_alpha-shadow-binding".to_string(),
    );

    let error = WorkplaceOutboundEmissionPlan::new(plan_with_matching_suffix)
        .expect_err("destination tenant segment, not a later suffix, must match plan tenant");

    assert_eq!(error, WorkplaceOutboundMetadataError::TenantMismatch);
}

#[test]
fn raw_esign_provider_webhook_or_payment_credential_in_fixture_data_is_denied() {
    let mut destination_credential = prod_esign_plan();
    destination_credential.destination_binding_ref = Some(
        "workplace-integration-destination/ten_alpha/prod/live/raw-credential-marker".to_string(),
    );
    let destination_error = WorkplaceOutboundEmissionPlan::new(destination_credential)
        .expect_err("destination refs must not carry raw provider credentials");
    assert_eq!(
        destination_error,
        WorkplaceOutboundMetadataError::RawSecretOrCredentialInFixture
    );

    let mut consent_token = prod_esign_plan();
    consent_token.consent_policy_evidence_ref =
        "policy-evidence/workplace-integration/ten_alpha/prod/bearer-token-marker".to_string();
    let consent_error = WorkplaceOutboundEmissionPlan::new(consent_token)
        .expect_err("consent evidence refs must not carry tokens");
    assert_eq!(
        consent_error,
        WorkplaceOutboundMetadataError::RawSecretOrCredentialInFixture
    );

    let mut tenancy_secret = prod_esign_plan();
    tenancy_secret.tenancy_env_tier_evidence_ref =
        "tenancy-evidence/workplace-integration/ten_alpha/prod/env-tier/provider-key-fragment"
            .to_string();
    let tenancy_error = WorkplaceOutboundEmissionPlan::new(tenancy_secret)
        .expect_err("tenancy evidence refs must not carry provider keys");
    assert_eq!(
        tenancy_error,
        WorkplaceOutboundMetadataError::RawSecretOrCredentialInFixture
    );
}

#[test]
fn runtime_delivery_claim_remains_forbidden_in_metadata_only_lane() {
    let mut plan = prod_esign_plan();
    plan.runtime_delivery_authorized = true;

    let error = WorkplaceOutboundEmissionPlan::new(plan)
        .expect_err("metadata-only lane cannot authorize runtime delivery");

    assert_eq!(
        error,
        WorkplaceOutboundMetadataError::RuntimeDeliveryClaimForbidden
    );
}
