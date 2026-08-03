use oya_crm_customer_engagement_domain::{
    CrmCustomerEngagementError, CustomerAccountInput, CustomerAccountState, LoyaltyActivityInput,
    LoyaltyActivityState, MarketingCampaignInput, MarketingCampaignState, MarketingChannelMix,
    OpportunityQualificationInput, OpportunitySalesStage, OpportunityState, QuotePreparationInput,
    QuoteState, RelationshipTier, ServiceCaseInput, ServiceCasePriority, ServiceCaseSeverity,
    ServiceCaseState, open_service_case, plan_marketing_campaign, prepare_quote,
    qualify_opportunity, record_loyalty_activity, register_customer_account,
};

fn account_input() -> CustomerAccountInput {
    CustomerAccountInput {
        account_id: "acct_global_retailer".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        market_id: "market_us".to_owned(),
        customer_profile_id: "cust_global_retailer_360".to_owned(),
        primary_contact_id: "contact_buyer_001".to_owned(),
        relationship_tier: RelationshipTier::StrategicAccount,
        lifecycle_score: 850,
        consent_basis_ref: "policy/crm/consent-b2b-contract".to_owned(),
        account_source_ref: "src/crm/account/global-retailer".to_owned(),
        registration_evidence_ref: "audit/crm/account/acct_global_retailer/register".to_owned(),
    }
}

fn opportunity_input(account_registered: bool) -> OpportunityQualificationInput {
    OpportunityQualificationInput {
        opportunity_id: "opp_global_retailer_renewal".to_owned(),
        lead_id: "lead_global_retailer_q3".to_owned(),
        account_id: "acct_global_retailer".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        market_id: "market_us".to_owned(),
        account_registered,
        sales_stage: OpportunitySalesStage::Propose,
        estimated_revenue_cents: 1_000_000,
        win_probability_bps: 6_500,
        expected_close_yyyymmdd: 20260831,
        opportunity_source_ref: "src/crm/opportunity/global-retailer-renewal".to_owned(),
        qualification_evidence_ref: "audit/crm/opportunity/opp_global_retailer_renewal/qualify"
            .to_owned(),
    }
}

fn quote_input(opportunity_qualified: bool) -> QuotePreparationInput {
    QuotePreparationInput {
        quote_id: "quote_global_retailer_renewal".to_owned(),
        opportunity_id: "opp_global_retailer_renewal".to_owned(),
        account_id: "acct_global_retailer".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        market_id: "market_us".to_owned(),
        opportunity_qualified,
        quote_amount_cents: 1_000_000,
        discount_bps: 1_000,
        margin_guardrail_bps: 2_000,
        valid_until_yyyymmdd: 20260930,
        quote_source_ref: "src/crm/quote/global-retailer-renewal".to_owned(),
        approval_evidence_ref: "audit/crm/quote/quote_global_retailer_renewal/approval".to_owned(),
    }
}

fn service_case_input(account_registered: bool) -> ServiceCaseInput {
    ServiceCaseInput {
        case_id: "case_global_retailer_delivery".to_owned(),
        account_id: "acct_global_retailer".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        market_id: "market_us".to_owned(),
        product_ref: "item_support_plan".to_owned(),
        account_registered,
        priority: ServiceCasePriority::Critical,
        severity: ServiceCaseSeverity::Major,
        opened_on_yyyymmdd: 20260701,
        sla_due_yyyymmdd: 20260702,
        case_source_ref: "src/crm/service-case/global-retailer-delivery".to_owned(),
        case_evidence_ref: "audit/crm/service-case/case_global_retailer_delivery/open".to_owned(),
    }
}

fn campaign_input() -> MarketingCampaignInput {
    MarketingCampaignInput {
        campaign_id: "camp_renewal_q3".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        market_id: "market_us".to_owned(),
        segment_id: "seg_strategic_renewals".to_owned(),
        channel_mix: MarketingChannelMix::Omnichannel,
        consent_basis_ref: "policy/crm/consent-b2b-contract".to_owned(),
        planned_start_yyyymmdd: 20260701,
        planned_end_yyyymmdd: 20260731,
        campaign_budget_cents: 250_000,
        campaign_source_ref: "src/crm/campaign/renewal-q3".to_owned(),
        campaign_evidence_ref: "audit/crm/campaign/camp_renewal_q3/plan".to_owned(),
    }
}

fn loyalty_activity_input(account_registered: bool) -> LoyaltyActivityInput {
    LoyaltyActivityInput {
        loyalty_activity_id: "loyalty_global_retailer_purchase".to_owned(),
        account_id: "acct_global_retailer".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        market_id: "market_us".to_owned(),
        account_registered,
        points_delta: 125,
        monetary_value_cents: 50_000,
        activity_yyyymmdd: 20260715,
        activity_source_ref: "src/crm/loyalty/global-retailer-purchase".to_owned(),
        activity_evidence_ref: "audit/crm/loyalty/loyalty_global_retailer_purchase/record"
            .to_owned(),
    }
}

#[test]
fn account_drives_opportunity_quote_service_case_and_loyalty_activity() {
    let account = register_customer_account(account_input()).unwrap();
    assert_eq!(account.state.value, CustomerAccountState::Registered);
    assert_eq!(account.lifecycle_score.value, 850);
    assert!(!account.durable_customer_master_attached.value);
    assert!(!account.cdp_unification_attached.value);
    assert!(!account.cloud_deployment_attached.value);

    let opportunity = qualify_opportunity(opportunity_input(true)).unwrap();
    assert_eq!(opportunity.state.value, OpportunityState::Qualified);
    assert_eq!(opportunity.weighted_revenue_cents.value, 650_000);
    assert!(!opportunity.cpq_runtime_attached.value);
    assert!(!opportunity.forecast_write_attached.value);

    let quote = prepare_quote(quote_input(true)).unwrap();
    assert_eq!(quote.state.value, QuoteState::Prepared);
    assert_eq!(quote.discount_amount_cents.value, 100_000);
    assert_eq!(quote.net_quote_amount_cents.value, 900_000);
    assert!(!quote.cpq_price_engine_attached.value);
    assert!(!quote.order_management_attached.value);
    assert!(!quote.erp_price_sync_attached.value);

    let service_case = open_service_case(service_case_input(true)).unwrap();
    assert_eq!(service_case.state.value, ServiceCaseState::Opened);
    assert!(service_case.escalation_required.value);
    assert!(!service_case.service_routing_attached.value);
    assert!(!service_case.knowledge_base_attached.value);
    assert!(!service_case.workflow_execution_attached.value);

    let campaign = plan_marketing_campaign(campaign_input()).unwrap();
    assert_eq!(campaign.state.value, MarketingCampaignState::Planned);
    assert_eq!(campaign.campaign_budget_cents.value, 250_000);
    assert!(!campaign.journey_runtime_attached.value);
    assert!(!campaign.message_delivery_attached.value);
    assert!(!campaign.cdp_segment_runtime_attached.value);

    let loyalty = record_loyalty_activity(loyalty_activity_input(true)).unwrap();
    assert_eq!(loyalty.state.value, LoyaltyActivityState::Recorded);
    assert_eq!(loyalty.points_earned.value, 125);
    assert_eq!(loyalty.points_redeemed.value, 0);
    assert!(!loyalty.loyalty_wallet_runtime_attached.value);
    assert!(!loyalty.reward_settlement_attached.value);
    assert!(!loyalty.marketing_journey_attached.value);
}

#[test]
fn crm_prd_metadata_fields_are_present_and_non_claiming() {
    let account = register_customer_account(account_input()).unwrap();
    assert_eq!(account.account_id.value.value, "acct_global_retailer");
    assert_eq!(account.tenant_id.value.value, "ten_enterprise");
    assert_eq!(account.legal_entity_id.value.value, "le_us001");
    assert_eq!(account.market_id.value.value, "market_us");
    assert_eq!(
        account.customer_profile_id.value.value,
        "cust_global_retailer_360"
    );
    assert_eq!(account.primary_contact_id.value.value, "contact_buyer_001");
    assert_eq!(
        account.relationship_tier.value,
        RelationshipTier::StrategicAccount
    );
    assert_eq!(account.lifecycle_score.value, 850);
    assert_eq!(
        account.consent_basis_ref.value.value,
        "policy/crm/consent-b2b-contract"
    );
    assert_eq!(
        account.account_source_ref.value.value,
        "src/crm/account/global-retailer"
    );
    assert_eq!(
        account.registration_evidence_ref.value.value,
        "audit/crm/account/acct_global_retailer/register"
    );
    assert_eq!(account.state.value, CustomerAccountState::Registered);
    assert_eq!(
        account.idempotency_key.value,
        "crm:account:ten_enterprise:le_us001:market_us:acct_global_retailer"
    );
    assert!(!account.durable_customer_master_attached.value);
    assert!(!account.cdp_unification_attached.value);
    assert!(!account.cloud_deployment_attached.value);
    assert_eq!(account.schema_version.value, 1);

    let opportunity = qualify_opportunity(opportunity_input(true)).unwrap();
    assert_eq!(
        opportunity.opportunity_id.value.value,
        "opp_global_retailer_renewal"
    );
    assert_eq!(opportunity.lead_id.value.value, "lead_global_retailer_q3");
    assert_eq!(opportunity.account_id.value.value, "acct_global_retailer");
    assert_eq!(opportunity.tenant_id.value.value, "ten_enterprise");
    assert_eq!(opportunity.legal_entity_id.value.value, "le_us001");
    assert_eq!(opportunity.market_id.value.value, "market_us");
    assert_eq!(
        opportunity.sales_stage.value,
        OpportunitySalesStage::Propose
    );
    assert_eq!(opportunity.estimated_revenue_cents.value, 1_000_000);
    assert_eq!(opportunity.win_probability_bps.value, 6_500);
    assert_eq!(opportunity.weighted_revenue_cents.value, 650_000);
    assert_eq!(opportunity.expected_close_yyyymmdd.value, 20260831);
    assert_eq!(
        opportunity.opportunity_source_ref.value.value,
        "src/crm/opportunity/global-retailer-renewal"
    );
    assert_eq!(
        opportunity.qualification_evidence_ref.value.value,
        "audit/crm/opportunity/opp_global_retailer_renewal/qualify"
    );
    assert_eq!(opportunity.state.value, OpportunityState::Qualified);
    assert_eq!(
        opportunity.idempotency_key.value,
        "crm:opportunity:ten_enterprise:le_us001:market_us:opp_global_retailer_renewal"
    );
    assert!(!opportunity.cpq_runtime_attached.value);
    assert!(!opportunity.forecast_write_attached.value);
    assert!(!opportunity.workflow_execution_attached.value);
    assert!(!opportunity.cloud_deployment_attached.value);
    assert_eq!(opportunity.schema_version.value, 1);

    let quote = prepare_quote(quote_input(true)).unwrap();
    assert_eq!(quote.quote_id.value.value, "quote_global_retailer_renewal");
    assert_eq!(
        quote.opportunity_id.value.value,
        "opp_global_retailer_renewal"
    );
    assert_eq!(quote.account_id.value.value, "acct_global_retailer");
    assert_eq!(quote.tenant_id.value.value, "ten_enterprise");
    assert_eq!(quote.legal_entity_id.value.value, "le_us001");
    assert_eq!(quote.market_id.value.value, "market_us");
    assert_eq!(quote.quote_amount_cents.value, 1_000_000);
    assert_eq!(quote.discount_bps.value, 1_000);
    assert_eq!(quote.discount_amount_cents.value, 100_000);
    assert_eq!(quote.net_quote_amount_cents.value, 900_000);
    assert_eq!(quote.margin_guardrail_bps.value, 2_000);
    assert_eq!(quote.valid_until_yyyymmdd.value, 20260930);
    assert_eq!(
        quote.quote_source_ref.value.value,
        "src/crm/quote/global-retailer-renewal"
    );
    assert_eq!(
        quote.approval_evidence_ref.value.value,
        "audit/crm/quote/quote_global_retailer_renewal/approval"
    );
    assert_eq!(quote.state.value, QuoteState::Prepared);
    assert_eq!(
        quote.idempotency_key.value,
        "crm:quote:ten_enterprise:le_us001:market_us:quote_global_retailer_renewal"
    );
    assert!(!quote.cpq_price_engine_attached.value);
    assert!(!quote.order_management_attached.value);
    assert!(!quote.erp_price_sync_attached.value);
    assert!(!quote.cloud_deployment_attached.value);
    assert_eq!(quote.schema_version.value, 1);

    let service_case = open_service_case(service_case_input(true)).unwrap();
    assert_eq!(
        service_case.case_id.value.value,
        "case_global_retailer_delivery"
    );
    assert_eq!(service_case.account_id.value.value, "acct_global_retailer");
    assert_eq!(service_case.tenant_id.value.value, "ten_enterprise");
    assert_eq!(service_case.legal_entity_id.value.value, "le_us001");
    assert_eq!(service_case.market_id.value.value, "market_us");
    assert_eq!(service_case.product_ref.value.value, "item_support_plan");
    assert_eq!(service_case.priority.value, ServiceCasePriority::Critical);
    assert_eq!(service_case.severity.value, ServiceCaseSeverity::Major);
    assert_eq!(service_case.opened_on_yyyymmdd.value, 20260701);
    assert_eq!(service_case.sla_due_yyyymmdd.value, 20260702);
    assert!(service_case.escalation_required.value);
    assert_eq!(
        service_case.case_source_ref.value.value,
        "src/crm/service-case/global-retailer-delivery"
    );
    assert_eq!(
        service_case.case_evidence_ref.value.value,
        "audit/crm/service-case/case_global_retailer_delivery/open"
    );
    assert_eq!(service_case.state.value, ServiceCaseState::Opened);
    assert_eq!(
        service_case.idempotency_key.value,
        "crm:case:ten_enterprise:le_us001:market_us:case_global_retailer_delivery"
    );
    assert!(!service_case.service_routing_attached.value);
    assert!(!service_case.knowledge_base_attached.value);
    assert!(!service_case.workflow_execution_attached.value);
    assert!(!service_case.cloud_deployment_attached.value);
    assert_eq!(service_case.schema_version.value, 1);

    let campaign = plan_marketing_campaign(campaign_input()).unwrap();
    assert_eq!(campaign.campaign_id.value.value, "camp_renewal_q3");
    assert_eq!(campaign.tenant_id.value.value, "ten_enterprise");
    assert_eq!(campaign.legal_entity_id.value.value, "le_us001");
    assert_eq!(campaign.market_id.value.value, "market_us");
    assert_eq!(campaign.segment_id.value.value, "seg_strategic_renewals");
    assert_eq!(campaign.channel_mix.value, MarketingChannelMix::Omnichannel);
    assert_eq!(
        campaign.consent_basis_ref.value.value,
        "policy/crm/consent-b2b-contract"
    );
    assert_eq!(campaign.planned_start_yyyymmdd.value, 20260701);
    assert_eq!(campaign.planned_end_yyyymmdd.value, 20260731);
    assert_eq!(campaign.campaign_budget_cents.value, 250_000);
    assert_eq!(
        campaign.campaign_source_ref.value.value,
        "src/crm/campaign/renewal-q3"
    );
    assert_eq!(
        campaign.campaign_evidence_ref.value.value,
        "audit/crm/campaign/camp_renewal_q3/plan"
    );
    assert_eq!(campaign.state.value, MarketingCampaignState::Planned);
    assert_eq!(
        campaign.idempotency_key.value,
        "crm:campaign:ten_enterprise:le_us001:market_us:camp_renewal_q3"
    );
    assert!(!campaign.journey_runtime_attached.value);
    assert!(!campaign.message_delivery_attached.value);
    assert!(!campaign.cdp_segment_runtime_attached.value);
    assert!(!campaign.cloud_deployment_attached.value);
    assert_eq!(campaign.schema_version.value, 1);

    let loyalty = record_loyalty_activity(loyalty_activity_input(true)).unwrap();
    assert_eq!(
        loyalty.loyalty_activity_id.value.value,
        "loyalty_global_retailer_purchase"
    );
    assert_eq!(loyalty.account_id.value.value, "acct_global_retailer");
    assert_eq!(loyalty.tenant_id.value.value, "ten_enterprise");
    assert_eq!(loyalty.legal_entity_id.value.value, "le_us001");
    assert_eq!(loyalty.market_id.value.value, "market_us");
    assert_eq!(loyalty.points_delta.value, 125);
    assert_eq!(loyalty.points_earned.value, 125);
    assert_eq!(loyalty.points_redeemed.value, 0);
    assert_eq!(loyalty.monetary_value_cents.value, 50_000);
    assert_eq!(loyalty.activity_yyyymmdd.value, 20260715);
    assert_eq!(
        loyalty.activity_source_ref.value.value,
        "src/crm/loyalty/global-retailer-purchase"
    );
    assert_eq!(
        loyalty.activity_evidence_ref.value.value,
        "audit/crm/loyalty/loyalty_global_retailer_purchase/record"
    );
    assert_eq!(loyalty.state.value, LoyaltyActivityState::Recorded);
    assert_eq!(
        loyalty.idempotency_key.value,
        "crm:loyalty:ten_enterprise:le_us001:market_us:loyalty_global_retailer_purchase"
    );
    assert!(!loyalty.loyalty_wallet_runtime_attached.value);
    assert!(!loyalty.reward_settlement_attached.value);
    assert!(!loyalty.marketing_journey_attached.value);
    assert!(!loyalty.cloud_deployment_attached.value);
    assert_eq!(loyalty.schema_version.value, 1);
}

#[test]
fn crm_refuses_unregistered_or_unqualified_flow() {
    assert_eq!(
        qualify_opportunity(opportunity_input(false)),
        Err(CrmCustomerEngagementError::AccountRegistrationRequired)
    );
    assert_eq!(
        prepare_quote(quote_input(false)),
        Err(CrmCustomerEngagementError::OpportunityQualificationRequired)
    );
    assert_eq!(
        open_service_case(service_case_input(false)),
        Err(CrmCustomerEngagementError::AccountRegistrationRequired)
    );
    assert_eq!(
        record_loyalty_activity(loyalty_activity_input(false)),
        Err(CrmCustomerEngagementError::AccountRegistrationRequired)
    );
}

#[test]
fn crm_validates_refs_dates_scores_amounts_and_discounts() {
    let mut unsafe_account = account_input();
    unsafe_account.registration_evidence_ref = "audit/crm/secret-token".to_owned();
    assert_eq!(
        register_customer_account(unsafe_account),
        Err(CrmCustomerEngagementError::InvalidEvidenceRef)
    );

    let mut bad_score = account_input();
    bad_score.lifecycle_score = 1_001;
    assert_eq!(
        register_customer_account(bad_score),
        Err(CrmCustomerEngagementError::InvalidScore)
    );

    let mut bad_probability = opportunity_input(true);
    bad_probability.win_probability_bps = 0;
    assert_eq!(
        qualify_opportunity(bad_probability),
        Err(CrmCustomerEngagementError::InvalidProbability)
    );

    let mut bad_discount = quote_input(true);
    bad_discount.discount_bps = 2_500;
    bad_discount.margin_guardrail_bps = 2_000;
    assert_eq!(
        prepare_quote(bad_discount),
        Err(CrmCustomerEngagementError::InvalidDiscount)
    );

    let mut bad_case_date = service_case_input(true);
    bad_case_date.sla_due_yyyymmdd = 20260630;
    assert_eq!(
        open_service_case(bad_case_date),
        Err(CrmCustomerEngagementError::InvalidDate)
    );

    let mut bad_campaign_ref = campaign_input();
    bad_campaign_ref.campaign_source_ref = "src/../campaign".to_owned();
    assert_eq!(
        plan_marketing_campaign(bad_campaign_ref),
        Err(CrmCustomerEngagementError::InvalidSourceDocumentRef)
    );
}

#[test]
fn crm_records_loyalty_redemption_without_wallet_or_settlement_claim() {
    let mut redemption = loyalty_activity_input(true);
    redemption.loyalty_activity_id = "loyalty_global_retailer_redemption".to_owned();
    redemption.points_delta = -40;
    let loyalty = record_loyalty_activity(redemption).unwrap();
    assert_eq!(loyalty.points_earned.value, 0);
    assert_eq!(loyalty.points_redeemed.value, 40);
    assert!(!loyalty.loyalty_wallet_runtime_attached.value);
    assert!(!loyalty.reward_settlement_attached.value);
    assert!(!loyalty.marketing_journey_attached.value);
    assert!(!loyalty.cloud_deployment_attached.value);

    let mut zero_points = loyalty_activity_input(true);
    zero_points.points_delta = 0;
    assert_eq!(
        record_loyalty_activity(zero_points),
        Err(CrmCustomerEngagementError::InvalidLoyaltyPoints)
    );
}

#[test]
fn crm_rejects_prefix_only_whitespace_zero_amounts_dates_and_credential_refs() {
    let mut prefix_only_account = account_input();
    prefix_only_account.account_id = "acct_".to_owned();
    assert_eq!(
        register_customer_account(prefix_only_account),
        Err(CrmCustomerEngagementError::InvalidAccountId)
    );

    let mut whitespace_tenant = account_input();
    whitespace_tenant.tenant_id = "ten enterprise".to_owned();
    assert_eq!(
        register_customer_account(whitespace_tenant),
        Err(CrmCustomerEngagementError::InvalidTenantId)
    );

    let mut prefix_only_policy_ref = account_input();
    prefix_only_policy_ref.consent_basis_ref = "policy/".to_owned();
    assert_eq!(
        register_customer_account(prefix_only_policy_ref),
        Err(CrmCustomerEngagementError::InvalidPolicyRef)
    );

    let mut credential_source_ref = account_input();
    credential_source_ref.account_source_ref = "src/crm/password".to_owned();
    assert_eq!(
        register_customer_account(credential_source_ref),
        Err(CrmCustomerEngagementError::InvalidSourceDocumentRef)
    );

    let mut zero_revenue = opportunity_input(true);
    zero_revenue.estimated_revenue_cents = 0;
    assert_eq!(
        qualify_opportunity(zero_revenue),
        Err(CrmCustomerEngagementError::InvalidAmount)
    );

    let mut invalid_close_date = opportunity_input(true);
    invalid_close_date.expected_close_yyyymmdd = 20261301;
    assert_eq!(
        qualify_opportunity(invalid_close_date),
        Err(CrmCustomerEngagementError::InvalidDate)
    );

    let mut zero_quote_amount = quote_input(true);
    zero_quote_amount.quote_amount_cents = 0;
    assert_eq!(
        prepare_quote(zero_quote_amount),
        Err(CrmCustomerEngagementError::InvalidAmount)
    );

    let mut invalid_quote_date = quote_input(true);
    invalid_quote_date.valid_until_yyyymmdd = 20260230;
    assert_eq!(
        prepare_quote(invalid_quote_date),
        Err(CrmCustomerEngagementError::InvalidDate)
    );

    let mut inverted_campaign_dates = campaign_input();
    inverted_campaign_dates.planned_end_yyyymmdd = 20260630;
    assert_eq!(
        plan_marketing_campaign(inverted_campaign_dates),
        Err(CrmCustomerEngagementError::InvalidDate)
    );

    let mut zero_campaign_budget = campaign_input();
    zero_campaign_budget.campaign_budget_cents = 0;
    assert_eq!(
        plan_marketing_campaign(zero_campaign_budget),
        Err(CrmCustomerEngagementError::InvalidAmount)
    );

    let mut zero_loyalty_value = loyalty_activity_input(true);
    zero_loyalty_value.monetary_value_cents = 0;
    assert_eq!(
        record_loyalty_activity(zero_loyalty_value),
        Err(CrmCustomerEngagementError::InvalidAmount)
    );
}
