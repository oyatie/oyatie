#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payments_charge_domain::{
    PaymentActionClass, PaymentEnvTier, PaymentOutboundEmissionPlan,
    PaymentOutboundEmissionPlanCreate, PaymentOutboundMetadataError, PaymentOutboundMode,
};

fn prod_charge_capture_plan() -> PaymentOutboundEmissionPlanCreate {
    PaymentOutboundEmissionPlanCreate {
        tenant_id: "ten_alpha".to_string(),
        env_tier: Some(PaymentEnvTier::Prod),
        outbound_mode: PaymentOutboundMode::Live,
        payment_action_class: PaymentActionClass::ChargeCapture,
        destination_binding_ref: Some(
            "payments-destination/ten_alpha/prod/psp/live/stripe-account".to_string(),
        ),
        pci_safe_evidence_ref:
            "evidence/payments/pci-safe/ten_alpha/prod/charge-capture-env-tier-001.json".to_string(),
        tenancy_cedar_policy_evidence_ref:
            "policy-evidence/ten_alpha/prod/env-tier/payment-live-allow".to_string(),
        financial_acknowledgment_ref: Some(
            "ack/financial/ten_alpha/prod/payment-live-risk-001".to_string(),
        ),
        prod_acknowledgment_ref: Some("ack/prod/ten_alpha/env-tier/payment-live-001".to_string()),
        api_key_prefix_evidence_ref: Some("api-key-prefix/ten_alpha/prod/sk_live_".to_string()),
    }
}

#[test]
fn prod_payment_plan_records_live_mode_and_required_metadata_refs() {
    let plan = PaymentOutboundEmissionPlan::new(prod_charge_capture_plan())
        .expect("complete prod charge-capture metadata plan should be accepted");

    assert_eq!(plan.tenant_id, "ten_alpha");
    assert_eq!(plan.env_tier, PaymentEnvTier::Prod);
    assert_eq!(plan.outbound_mode, PaymentOutboundMode::Live);
    assert_eq!(plan.payment_action_class, PaymentActionClass::ChargeCapture);
    assert_eq!(
        plan.destination_binding_ref.as_deref(),
        Some("payments-destination/ten_alpha/prod/psp/live/stripe-account")
    );
    assert_eq!(
        plan.pci_safe_evidence_ref,
        "evidence/payments/pci-safe/ten_alpha/prod/charge-capture-env-tier-001.json"
    );
    assert_eq!(
        PaymentEnvTier::Staging.derived_outbound_mode(),
        PaymentOutboundMode::TestRecipients
    );
}

#[test]
fn test_tier_preserves_intercept_log_only_without_destination_transport() {
    let mut plan = prod_charge_capture_plan();
    plan.env_tier = Some(PaymentEnvTier::Test);
    plan.outbound_mode = PaymentOutboundMode::Intercept;
    plan.payment_action_class = PaymentActionClass::Subscription;
    plan.destination_binding_ref = None;
    plan.pci_safe_evidence_ref =
        "evidence/payments/pci-safe/ten_alpha/test/subscription-intercept-log-only.json"
            .to_string();
    plan.tenancy_cedar_policy_evidence_ref =
        "policy-evidence/ten_alpha/test/env-tier/intercept-log-only".to_string();
    plan.financial_acknowledgment_ref = None;
    plan.prod_acknowledgment_ref = None;
    plan.api_key_prefix_evidence_ref = Some("api-key-prefix/ten_alpha/test/sk_test_".to_string());

    let accepted = PaymentOutboundEmissionPlan::new(plan)
        .expect("test tier metadata plan should stay intercept/log-only");

    assert_eq!(accepted.env_tier, PaymentEnvTier::Test);
    assert_eq!(accepted.outbound_mode, PaymentOutboundMode::Intercept);
    assert!(accepted.destination_binding_ref.is_none());
}

#[test]
fn missing_env_tier_is_denied() {
    let mut plan = prod_charge_capture_plan();
    plan.env_tier = None;

    let error = PaymentOutboundEmissionPlan::new(plan).expect_err("env_tier is required metadata");

    assert_eq!(error, PaymentOutboundMetadataError::MissingEnvTier);
}

#[test]
fn outbound_mode_must_be_derived_from_env_tier() {
    let mut plan = prod_charge_capture_plan();
    plan.env_tier = Some(PaymentEnvTier::Test);
    plan.outbound_mode = PaymentOutboundMode::Live;

    let error = PaymentOutboundEmissionPlan::new(plan)
        .expect_err("outbound_mode must be derived from env_tier");

    assert_eq!(
        error,
        PaymentOutboundMetadataError::InvalidOutboundModeForTier
    );
}

#[test]
fn test_tier_attempting_psp_capture_refund_payout_or_webhook_delivery_is_denied() {
    for action in [
        PaymentActionClass::ChargeCapture,
        PaymentActionClass::Refund,
        PaymentActionClass::Payout,
        PaymentActionClass::WebhookDelivery,
    ] {
        let mut plan = prod_charge_capture_plan();
        plan.env_tier = Some(PaymentEnvTier::Test);
        plan.outbound_mode = PaymentOutboundMode::Intercept;
        plan.payment_action_class = action;
        plan.destination_binding_ref =
            Some("payments-destination/ten_alpha/test/psp/sandbox/forbidden-delivery".to_string());
        plan.tenancy_cedar_policy_evidence_ref =
            "policy-evidence/ten_alpha/test/env-tier/intercept-log-only".to_string();
        plan.financial_acknowledgment_ref = None;
        plan.prod_acknowledgment_ref = None;
        plan.api_key_prefix_evidence_ref =
            Some("api-key-prefix/ten_alpha/test/sk_test_".to_string());

        let error = PaymentOutboundEmissionPlan::new(plan)
            .expect_err("test tier cannot carry PSP or webhook delivery metadata");

        assert_eq!(
            error,
            PaymentOutboundMetadataError::ExternalSideEffectNotAllowedForTier
        );
    }
}

#[test]
fn staging_without_tenant_qa_psp_sandbox_or_webhook_endpoint_is_denied() {
    let mut plan = prod_charge_capture_plan();
    plan.env_tier = Some(PaymentEnvTier::Staging);
    plan.outbound_mode = PaymentOutboundMode::TestRecipients;
    plan.destination_binding_ref = None;
    plan.tenancy_cedar_policy_evidence_ref =
        "policy-evidence/ten_alpha/staging/env-tier/qa-payments".to_string();
    plan.financial_acknowledgment_ref = None;
    plan.prod_acknowledgment_ref = None;
    plan.api_key_prefix_evidence_ref =
        Some("api-key-prefix/ten_alpha/staging/sk_stage_".to_string());

    let error = PaymentOutboundEmissionPlan::new(plan)
        .expect_err("staging requires a tenant QA PSP sandbox or webhook endpoint");

    assert_eq!(error, PaymentOutboundMetadataError::MissingQaDestination);
}

#[test]
fn staging_destination_must_be_qa_sandbox_or_test_recipient() {
    let mut plan = prod_charge_capture_plan();
    plan.env_tier = Some(PaymentEnvTier::Staging);
    plan.outbound_mode = PaymentOutboundMode::TestRecipients;
    plan.destination_binding_ref =
        Some("payments-destination/ten_alpha/prod/psp/live/stripe-account".to_string());
    plan.tenancy_cedar_policy_evidence_ref =
        "policy-evidence/ten_alpha/staging/env-tier/qa-payments".to_string();
    plan.financial_acknowledgment_ref = None;
    plan.prod_acknowledgment_ref = None;
    plan.api_key_prefix_evidence_ref =
        Some("api-key-prefix/ten_alpha/staging/sk_stage_".to_string());

    let error = PaymentOutboundEmissionPlan::new(plan)
        .expect_err("staging cannot point at live PSP or webhook destinations");

    assert_eq!(error, PaymentOutboundMetadataError::MissingQaDestination);
}

#[test]
fn prod_payment_invoice_or_webhook_without_policy_and_acknowledgments_is_denied() {
    let mut missing_policy = prod_charge_capture_plan();
    missing_policy.payment_action_class = PaymentActionClass::Invoice;
    missing_policy.tenancy_cedar_policy_evidence_ref =
        "policy-evidence/ten_alpha/prod/missing-env-tier".to_string();

    let policy_error = PaymentOutboundEmissionPlan::new(missing_policy)
        .expect_err("prod invoice metadata requires env-tier policy evidence");
    assert_eq!(
        policy_error,
        PaymentOutboundMetadataError::ProdPolicyEvidenceRequired
    );

    let mut missing_financial_ack = prod_charge_capture_plan();
    missing_financial_ack.payment_action_class = PaymentActionClass::WebhookDelivery;
    missing_financial_ack.financial_acknowledgment_ref = None;

    let financial_error = PaymentOutboundEmissionPlan::new(missing_financial_ack)
        .expect_err("prod webhook metadata requires financial acknowledgment evidence");
    assert_eq!(
        financial_error,
        PaymentOutboundMetadataError::ProdAcknowledgmentRequired
    );

    let mut missing_prod_ack = prod_charge_capture_plan();
    missing_prod_ack.payment_action_class = PaymentActionClass::Refund;
    missing_prod_ack.prod_acknowledgment_ref = None;

    let prod_error = PaymentOutboundEmissionPlan::new(missing_prod_ack)
        .expect_err("prod refund metadata requires prod acknowledgment evidence");
    assert_eq!(
        prod_error,
        PaymentOutboundMetadataError::ProdAcknowledgmentRequired
    );
}

#[test]
fn api_key_prefix_tier_mismatch_is_denied() {
    let mut staging_with_live_key = prod_charge_capture_plan();
    staging_with_live_key.env_tier = Some(PaymentEnvTier::Staging);
    staging_with_live_key.outbound_mode = PaymentOutboundMode::TestRecipients;
    staging_with_live_key.destination_binding_ref =
        Some("payments-destination/ten_alpha/staging/psp/sandbox/stripe-account".to_string());
    staging_with_live_key.tenancy_cedar_policy_evidence_ref =
        "policy-evidence/ten_alpha/staging/env-tier/qa-payments".to_string();
    staging_with_live_key.financial_acknowledgment_ref = None;
    staging_with_live_key.prod_acknowledgment_ref = None;
    staging_with_live_key.api_key_prefix_evidence_ref =
        Some("api-key-prefix/ten_alpha/staging/sk_live_".to_string());

    let staging_error = PaymentOutboundEmissionPlan::new(staging_with_live_key)
        .expect_err("staging cannot reference live API-key prefixes");
    assert_eq!(
        staging_error,
        PaymentOutboundMetadataError::ApiKeyPrefixTierMismatch
    );

    let mut prod_with_test_key = prod_charge_capture_plan();
    prod_with_test_key.api_key_prefix_evidence_ref =
        Some("api-key-prefix/ten_alpha/prod/sk_test_".to_string());

    let prod_error = PaymentOutboundEmissionPlan::new(prod_with_test_key)
        .expect_err("prod cannot reference test API-key prefixes");
    assert_eq!(
        prod_error,
        PaymentOutboundMetadataError::ApiKeyPrefixTierMismatch
    );
}

#[test]
fn cross_tenant_destination_leakage_is_denied() {
    let mut plan = prod_charge_capture_plan();
    plan.destination_binding_ref =
        Some("payments-destination/ten_beta/prod/psp/live/stripe-account".to_string());

    let error = PaymentOutboundEmissionPlan::new(plan)
        .expect_err("PSP destination tenant must match plan tenant");

    assert_eq!(error, PaymentOutboundMetadataError::TenantMismatch);
}

#[test]
fn raw_psp_credential_api_key_or_token_in_fixture_data_is_denied() {
    let mut destination_secret = prod_charge_capture_plan();
    destination_secret.destination_binding_ref =
        Some("payments-destination/ten_alpha/prod/psp/live/raw-api-key-fixture".to_string());
    let destination_error = PaymentOutboundEmissionPlan::new(destination_secret)
        .expect_err("destination refs must not carry raw PSP credentials");
    assert_eq!(
        destination_error,
        PaymentOutboundMetadataError::RawSecretOrCredentialInFixture
    );

    let mut evidence_token = prod_charge_capture_plan();
    evidence_token.pci_safe_evidence_ref =
        "evidence/payments/pci-safe/ten_alpha/prod/bearer-token-fixture.json".to_string();
    let evidence_error = PaymentOutboundEmissionPlan::new(evidence_token)
        .expect_err("PCI-safe evidence refs must not carry tokens");
    assert_eq!(
        evidence_error,
        PaymentOutboundMetadataError::RawSecretOrCredentialInFixture
    );

    let mut policy_credential = prod_charge_capture_plan();
    policy_credential.tenancy_cedar_policy_evidence_ref =
        "policy-evidence/ten_alpha/prod/env-tier/raw-credential-fixture".to_string();
    let policy_error = PaymentOutboundEmissionPlan::new(policy_credential)
        .expect_err("policy refs must not carry raw credentials");
    assert_eq!(
        policy_error,
        PaymentOutboundMetadataError::RawSecretOrCredentialInFixture
    );
}
