use oya_cloud_billing_domain::{
    BillingEnvTier, BillingEventClass, BillingOutboundEmissionPlan,
    BillingOutboundEmissionPlanCreate, BillingOutboundMode, CloudBillingError,
};

fn live_invoice_plan() -> BillingOutboundEmissionPlanCreate {
    BillingOutboundEmissionPlanCreate {
        tenant_id: "ten_alpha".to_string(),
        region: "region-alpha".to_string(),
        env_tier: Some(BillingEnvTier::Prod),
        outbound_mode: BillingOutboundMode::Live,
        billing_event_class: BillingEventClass::Invoice,
        destination_binding_ref: Some(
            "billing-destination/ten_alpha/region-alpha/live/invoice-email".to_string(),
        ),
        invoice_or_metering_evidence_ref:
            "evidence/billing/invoice/ten_alpha/region-alpha/inv_env_tier_001.json".to_string(),
        policy_evidence_ref: "policy-evidence/ten_alpha/prod/env-tier/live-invoice-allow"
            .to_string(),
    }
}

#[test]
fn prod_invoice_plan_records_derived_live_mode_and_metadata_refs() {
    let plan = BillingOutboundEmissionPlan::new(live_invoice_plan())
        .expect("complete prod invoice metadata plan should be accepted");

    assert_eq!(plan.env_tier.value, BillingEnvTier::Prod);
    assert_eq!(plan.outbound_mode.value, BillingOutboundMode::Live);
    assert_eq!(plan.billing_event_class.value, BillingEventClass::Invoice);
    assert_eq!(
        plan.destination_binding_ref.value.as_deref(),
        Some("billing-destination/ten_alpha/region-alpha/live/invoice-email")
    );
    assert_eq!(
        plan.invoice_or_metering_evidence_ref.value,
        "evidence/billing/invoice/ten_alpha/region-alpha/inv_env_tier_001.json"
    );
    assert_eq!(
        BillingEnvTier::Staging.derived_outbound_mode(),
        BillingOutboundMode::TestRecipients
    );
}

#[test]
fn test_tier_preserves_intercept_log_only_without_destination_transport() {
    let mut plan = live_invoice_plan();
    plan.env_tier = Some(BillingEnvTier::Test);
    plan.outbound_mode = BillingOutboundMode::Intercept;
    plan.billing_event_class = BillingEventClass::Metering;
    plan.destination_binding_ref = None;
    plan.invoice_or_metering_evidence_ref =
        "evidence/billing/metering/ten_alpha/region-alpha/test-usage-capture.json".to_string();
    plan.policy_evidence_ref =
        "policy-evidence/ten_alpha/test/env-tier/intercept-log-only".to_string();

    let accepted = BillingOutboundEmissionPlan::new(plan)
        .expect("test tier metadata plan should stay intercept/log-only");

    assert_eq!(accepted.env_tier.value, BillingEnvTier::Test);
    assert_eq!(accepted.outbound_mode.value, BillingOutboundMode::Intercept);
    assert!(accepted.destination_binding_ref.value.is_none());
}

#[test]
fn missing_env_tier_is_denied() {
    let mut plan = live_invoice_plan();
    plan.env_tier = None;

    let error = BillingOutboundEmissionPlan::new(plan).expect_err("env tier is required metadata");

    assert_eq!(error, CloudBillingError::MissingEnvTier);
}

#[test]
fn outbound_mode_mismatch_is_denied() {
    let mut plan = live_invoice_plan();
    plan.env_tier = Some(BillingEnvTier::Test);
    plan.outbound_mode = BillingOutboundMode::Live;

    let error = BillingOutboundEmissionPlan::new(plan)
        .expect_err("outbound mode must be derived from the tier");

    assert_eq!(error, CloudBillingError::InvalidOutboundModeForTier);
}

#[test]
fn test_tier_attempting_invoice_transport_is_denied() {
    let mut plan = live_invoice_plan();
    plan.env_tier = Some(BillingEnvTier::Test);
    plan.outbound_mode = BillingOutboundMode::Intercept;
    plan.destination_binding_ref =
        Some("billing-destination/ten_alpha/region-alpha/live/invoice-email".to_string());
    plan.policy_evidence_ref =
        "policy-evidence/ten_alpha/test/env-tier/intercept-log-only".to_string();

    let error = BillingOutboundEmissionPlan::new(plan)
        .expect_err("test tier cannot carry invoice transport metadata");

    assert_eq!(error, CloudBillingError::ExternalDeliveryNotAllowedForTier);
}

#[test]
fn staging_without_qa_invoice_recipient_or_export_endpoint_is_denied() {
    let mut plan = live_invoice_plan();
    plan.env_tier = Some(BillingEnvTier::Staging);
    plan.outbound_mode = BillingOutboundMode::TestRecipients;
    plan.destination_binding_ref = None;
    plan.policy_evidence_ref = "policy-evidence/ten_alpha/staging/env-tier/qa-invoice".to_string();

    let error = BillingOutboundEmissionPlan::new(plan)
        .expect_err("staging requires a tenant QA invoice recipient or export endpoint");

    assert_eq!(error, CloudBillingError::MissingQaDestination);
}

#[test]
fn staging_requires_qa_or_test_destination_binding() {
    let mut plan = live_invoice_plan();
    plan.env_tier = Some(BillingEnvTier::Staging);
    plan.outbound_mode = BillingOutboundMode::TestRecipients;
    plan.destination_binding_ref =
        Some("billing-destination/ten_alpha/region-alpha/live/invoice-email".to_string());
    plan.policy_evidence_ref = "policy-evidence/ten_alpha/staging/env-tier/qa-invoice".to_string();

    let error = BillingOutboundEmissionPlan::new(plan)
        .expect_err("staging cannot point at live invoice destinations");

    assert_eq!(error, CloudBillingError::MissingQaDestination);
}

#[test]
fn prod_without_tenancy_env_tier_policy_evidence_is_denied() {
    let mut plan = live_invoice_plan();
    plan.policy_evidence_ref = "policy-evidence/ten_alpha/prod/missing-env-tier".to_string();

    let error = BillingOutboundEmissionPlan::new(plan)
        .expect_err("prod live mode requires tenancy/env-tier policy evidence");

    assert_eq!(error, CloudBillingError::ProdPolicyEvidenceRequired);
}

#[test]
fn cross_tenant_invoice_destination_is_denied() {
    let mut plan = live_invoice_plan();
    plan.destination_binding_ref =
        Some("billing-destination/ten_beta/region-alpha/live/invoice-email".to_string());

    let error = BillingOutboundEmissionPlan::new(plan)
        .expect_err("invoice destination tenant must match plan tenant");

    assert_eq!(error, CloudBillingError::TenantMismatch);
}

#[test]
fn secret_like_destination_or_evidence_refs_are_denied() {
    let mut destination_secret = live_invoice_plan();
    destination_secret.destination_binding_ref =
        Some("billing-destination/ten_alpha/region-alpha/live/api_key".to_string());
    let destination_error = BillingOutboundEmissionPlan::new(destination_secret)
        .expect_err("destination binding refs must not carry secret-like markers");
    assert_eq!(
        destination_error,
        CloudBillingError::InvalidOutboundMetadataRef
    );

    let mut evidence_secret = live_invoice_plan();
    evidence_secret.invoice_or_metering_evidence_ref =
        "evidence/billing/invoice/ten_alpha/region-alpha/bearer-token.json".to_string();
    let evidence_error = BillingOutboundEmissionPlan::new(evidence_secret)
        .expect_err("evidence refs must not carry secret-like markers");
    assert_eq!(evidence_error, CloudBillingError::InvalidBillingEvidenceRef);
}
