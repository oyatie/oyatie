use oya_treasury_cash_domain::{
    TreasuryActionClass, TreasuryDomainError, TreasuryEnvTier, TreasuryOutboundMode,
    TreasuryOutboundPlanInput, plan_treasury_outbound_metadata,
};

fn valid_plan(env_tier: TreasuryEnvTier) -> TreasuryOutboundPlanInput {
    let destination_binding_ref = match env_tier {
        TreasuryEnvTier::Test => "dest/intercept/treasury/bank-notification-log",
        TreasuryEnvTier::Staging => "dest/qa/ten_enterprise/treasury/bank-payment-sandbox",
        TreasuryEnvTier::Prod => "dest/live/ten_enterprise/treasury/settlement-bank-binding",
    };

    TreasuryOutboundPlanInput {
        tenant_id: "ten_enterprise".to_owned(),
        env_tier: Some(env_tier),
        caller_supplied_outbound_mode: None,
        treasury_action_class: TreasuryActionClass::SettlementNotification,
        destination_tenant_id: "ten_enterprise".to_owned(),
        destination_binding_ref: destination_binding_ref.to_owned(),
        policy_evidence_ref: "audit/tenancy/env-tier/cedar/treasury-policy".to_owned(),
        destination_ownership_evidence_ref:
            "audit/treasury/destination/ownership/ten-enterprise-bank".to_owned(),
        intercept_evidence_ref: Some("audit/treasury/intercept/log-only-plan".to_owned()),
        financial_acknowledgment_ref: Some(
            "audit/treasury/prod-financial-ack/settlement".to_owned(),
        ),
        bank_network_call_attached: false,
        swift_transport_attached: false,
        host_to_host_transport_attached: false,
        payment_execution_attached: false,
        ledger_mutation_attached: false,
        durable_persistence_attached: false,
        runtime_audit_chain_emission_attached: false,
        production_delivery_claimed: false,
        hyperscaler_maturity_claimed: false,
    }
}

#[test]
fn treasury_env_tier_outbound_plan_derives_modes_and_preserves_non_claims() {
    let test_plan = plan_treasury_outbound_metadata(valid_plan(TreasuryEnvTier::Test)).unwrap();
    assert_eq!(test_plan.env_tier.value, TreasuryEnvTier::Test);
    assert_eq!(
        test_plan.outbound_mode.value,
        TreasuryOutboundMode::Intercept
    );
    assert_eq!(
        test_plan
            .intercept_evidence_ref
            .as_ref()
            .unwrap()
            .value
            .value,
        "audit/treasury/intercept/log-only-plan"
    );
    assert!(!test_plan.bank_network_call_attached.value);
    assert!(!test_plan.swift_transport_attached.value);
    assert!(!test_plan.host_to_host_transport_attached.value);
    assert!(!test_plan.payment_execution_attached.value);
    assert!(!test_plan.ledger_mutation_attached.value);
    assert!(!test_plan.durable_persistence_attached.value);
    assert!(!test_plan.runtime_audit_chain_emission_attached.value);
    assert!(!test_plan.production_delivery_claimed.value);
    assert!(!test_plan.hyperscaler_maturity_claimed.value);

    let staging_plan =
        plan_treasury_outbound_metadata(valid_plan(TreasuryEnvTier::Staging)).unwrap();
    assert_eq!(staging_plan.env_tier.value, TreasuryEnvTier::Staging);
    assert_eq!(
        staging_plan.outbound_mode.value,
        TreasuryOutboundMode::TestRecipients
    );
    assert_eq!(
        staging_plan.destination_binding_ref.value.value,
        "dest/qa/ten_enterprise/treasury/bank-payment-sandbox"
    );

    let prod_plan = plan_treasury_outbound_metadata(valid_plan(TreasuryEnvTier::Prod)).unwrap();
    assert_eq!(prod_plan.env_tier.value, TreasuryEnvTier::Prod);
    assert_eq!(prod_plan.outbound_mode.value, TreasuryOutboundMode::Live);
    assert_eq!(
        prod_plan
            .financial_acknowledgment_ref
            .as_ref()
            .unwrap()
            .value
            .value,
        "audit/treasury/prod-financial-ack/settlement"
    );
}

#[test]
fn treasury_env_tier_outbound_plan_fails_closed_for_red_fixtures() {
    let mut missing_env_tier = valid_plan(TreasuryEnvTier::Test);
    missing_env_tier.env_tier = None;
    assert_eq!(
        plan_treasury_outbound_metadata(missing_env_tier),
        Err(TreasuryDomainError::MissingEnvTier)
    );

    let mut caller_mode_mismatch = valid_plan(TreasuryEnvTier::Staging);
    caller_mode_mismatch.caller_supplied_outbound_mode = Some(TreasuryOutboundMode::Live);
    assert_eq!(
        plan_treasury_outbound_metadata(caller_mode_mismatch),
        Err(TreasuryDomainError::OutboundModeMustBeDerived)
    );

    let mut test_attempts_execution = valid_plan(TreasuryEnvTier::Test);
    test_attempts_execution.bank_network_call_attached = true;
    test_attempts_execution.swift_transport_attached = true;
    test_attempts_execution.payment_execution_attached = true;
    assert_eq!(
        plan_treasury_outbound_metadata(test_attempts_execution),
        Err(TreasuryDomainError::RuntimeBankOrPaymentClaimDenied)
    );

    let mut staging_without_qa_bank_payment_endpoint = valid_plan(TreasuryEnvTier::Staging);
    staging_without_qa_bank_payment_endpoint.destination_binding_ref =
        "dest/live/ten_enterprise/treasury/production-bank".to_owned();
    assert_eq!(
        plan_treasury_outbound_metadata(staging_without_qa_bank_payment_endpoint),
        Err(TreasuryDomainError::StagingQaDestinationRequired)
    );

    let mut prod_without_env_tier_policy = valid_plan(TreasuryEnvTier::Prod);
    prod_without_env_tier_policy.policy_evidence_ref = "audit/treasury/general-policy".to_owned();
    assert_eq!(
        plan_treasury_outbound_metadata(prod_without_env_tier_policy),
        Err(TreasuryDomainError::ProdPolicyEvidenceRequired)
    );

    let mut prod_without_financial_acknowledgment = valid_plan(TreasuryEnvTier::Prod);
    prod_without_financial_acknowledgment.financial_acknowledgment_ref = None;
    assert_eq!(
        plan_treasury_outbound_metadata(prod_without_financial_acknowledgment),
        Err(TreasuryDomainError::ProdFinancialAcknowledgmentRequired)
    );

    let mut cross_tenant_bank_destination = valid_plan(TreasuryEnvTier::Prod);
    cross_tenant_bank_destination.destination_tenant_id = "ten_other".to_owned();
    assert_eq!(
        plan_treasury_outbound_metadata(cross_tenant_bank_destination),
        Err(TreasuryDomainError::CrossTenantBankDestination)
    );

    let mut raw_bank_credential = valid_plan(TreasuryEnvTier::Staging);
    raw_bank_credential.destination_binding_ref =
        "dest/qa/ten_enterprise/treasury/raw-bank-credential".to_owned();
    assert_eq!(
        plan_treasury_outbound_metadata(raw_bank_credential),
        Err(TreasuryDomainError::RawCredentialOrSwiftSecret)
    );

    let mut swift_secret = valid_plan(TreasuryEnvTier::Prod);
    swift_secret.destination_binding_ref =
        "dest/live/ten_enterprise/treasury/swift-secret".to_owned();
    assert_eq!(
        plan_treasury_outbound_metadata(swift_secret),
        Err(TreasuryDomainError::RawCredentialOrSwiftSecret)
    );
}
