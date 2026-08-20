use global_trade_compliance_domain::{
    CustomsDeclarationInput, CustomsDeclarationState, CustomsDeclarationType,
    ExportControlAssessmentInput, ExportControlAssessmentState, ExportControlDecision,
    GlobalTradeComplianceError, LandedCostSimulationInput, LandedCostState, PartyScreeningState,
    ScreeningOutcome, TradeItemClassificationInput, TradeItemClassificationState, TradePartyRole,
    TradePartyScreeningInput, assess_export_control, classify_trade_item,
    prepare_customs_declaration, screen_trade_party, simulate_landed_cost,
};

fn party_input(score: u16) -> TradePartyScreeningInput {
    TradePartyScreeningInput {
        screening_id: "screen_export_consignee_us001".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        trade_party_id: "party_consignee_global_retailer".to_owned(),
        party_role: TradePartyRole::Consignee,
        country_code: "US".to_owned(),
        normalized_party_name: "GLOBAL-RETAILER".to_owned(),
        sanctions_list_version_ref: "list/ofac/2026-05-24".to_owned(),
        screening_score_bps: score,
        screening_source_ref: "src/global-trade/screening/global-retailer".to_owned(),
        screening_evidence_ref: "audit/global-trade/screening/screen_export_consignee_us001"
            .to_owned(),
    }
}

fn classification_input() -> TradeItemClassificationInput {
    TradeItemClassificationInput {
        classification_id: "class_laptop_export_us_eu".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        item_id: "item_laptop_13in".to_owned(),
        country_of_origin_code: "US".to_owned(),
        destination_country_code: "DE".to_owned(),
        hs_code: "847130".to_owned(),
        export_control_classification_number: "EAR99".to_owned(),
        unit_customs_value_cents: 100_000,
        duty_rate_bps: 250,
        classification_source_ref: "src/global-trade/classification/item-laptop-13in".to_owned(),
        classification_evidence_ref: "audit/global-trade/classification/class_laptop_export_us_eu"
            .to_owned(),
    }
}

fn assessment_input(
    party_screened: bool,
    item_classified: bool,
    outcome: ScreeningOutcome,
    eccn: &str,
    license_present: bool,
    embargo_country: bool,
) -> ExportControlAssessmentInput {
    ExportControlAssessmentInput {
        assessment_id: "export_laptop_de_001".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        classification_id: "class_laptop_export_us_eu".to_owned(),
        item_id: "item_laptop_13in".to_owned(),
        destination_country_code: "DE".to_owned(),
        export_control_classification_number: eccn.to_owned(),
        shipment_value_cents: 400_000,
        trade_party_screened: party_screened,
        trade_item_classified: item_classified,
        party_screening_outcome: outcome,
        license_present,
        embargo_country,
        assessment_source_ref: "src/global-trade/export-control/export_laptop_de_001".to_owned(),
        assessment_evidence_ref: "audit/global-trade/export-control/export_laptop_de_001"
            .to_owned(),
    }
}

fn declaration_input(
    export_control_assessed: bool,
    decision: ExportControlDecision,
) -> CustomsDeclarationInput {
    CustomsDeclarationInput {
        declaration_id: "decl_export_laptop_de_001".to_owned(),
        assessment_id: "export_laptop_de_001".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        item_id: "item_laptop_13in".to_owned(),
        trade_party_id: "party_consignee_global_retailer".to_owned(),
        declaration_type: CustomsDeclarationType::Export,
        export_control_assessed,
        export_control_decision: decision,
        quantity: 4,
        unit_customs_value_cents: 100_000,
        freight_cents: 20_000,
        insurance_cents: 5_000,
        incoterm_code: "DAP".to_owned(),
        customs_procedure_code: "EXP-STD".to_owned(),
        declaration_source_ref: "src/global-trade/customs/decl_export_laptop_de_001".to_owned(),
        declaration_evidence_ref: "audit/global-trade/customs/decl_export_laptop_de_001".to_owned(),
    }
}

fn landed_cost_input(declaration_prepared: bool) -> LandedCostSimulationInput {
    LandedCostSimulationInput {
        landed_cost_id: "landed_export_laptop_de_001".to_owned(),
        declaration_id: "decl_export_laptop_de_001".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        declaration_prepared,
        declared_customs_value_cents: 425_000,
        duty_rate_bps: 250,
        brokerage_fee_cents: 3_000,
        local_transport_cents: 7_000,
        cost_source_ref: "src/global-trade/landed-cost/landed_export_laptop_de_001".to_owned(),
        cost_evidence_ref: "audit/global-trade/landed-cost/landed_export_laptop_de_001".to_owned(),
    }
}

#[test]
fn party_item_export_customs_and_landed_cost_flow() {
    let screening = screen_trade_party(party_input(120)).unwrap();
    assert_eq!(screening.state.value, PartyScreeningState::Screened);
    assert_eq!(screening.outcome.value, ScreeningOutcome::Cleared);
    assert!(!screening.restricted_party_hold_required.value);
    assert!(!screening.live_sanctions_provider_attached.value);
    assert!(!screening.government_list_download_attached.value);
    assert!(!screening.business_transaction_block_mutation_attached.value);
    assert!(!screening.cloud_deployment_attached.value);

    let classification = classify_trade_item(classification_input()).unwrap();
    assert_eq!(
        classification.state.value,
        TradeItemClassificationState::Classified
    );
    assert_eq!(classification.hs_code.value.value, "847130");
    assert_eq!(classification.duty_rate_bps.value, 250);
    assert!(!classification.regulatory_content_provider_attached.value);
    assert!(!classification.legal_ruling_attached.value);
    assert!(!classification.product_master_mutation_attached.value);

    let export = assess_export_control(assessment_input(
        true,
        true,
        screening.outcome.value,
        &classification
            .export_control_classification_number
            .value
            .value,
        false,
        false,
    ))
    .unwrap();
    assert_eq!(export.state.value, ExportControlAssessmentState::Assessed);
    assert_eq!(
        export.decision.value,
        ExportControlDecision::AllowedNoLicense
    );
    assert!(!export.compliance_hold_required.value);
    assert!(!export.export_license_management_attached.value);
    assert!(!export.government_export_filing_attached.value);
    assert!(!export.order_hold_mutation_attached.value);
    assert!(!export.workflow_execution_attached.value);

    let declaration =
        prepare_customs_declaration(declaration_input(true, export.decision.value)).unwrap();
    assert_eq!(declaration.state.value, CustomsDeclarationState::Prepared);
    assert_eq!(declaration.goods_value_cents.value, 400_000);
    assert_eq!(declaration.declared_customs_value_cents.value, 425_000);
    assert!(!declaration.customs_authority_submission_attached.value);
    assert!(!declaration.broker_network_attached.value);
    assert!(!declaration.shipment_mutation_attached.value);
    assert!(!declaration.document_archive_attached.value);

    let landed_cost = simulate_landed_cost(landed_cost_input(true)).unwrap();
    assert_eq!(landed_cost.state.value, LandedCostState::Simulated);
    assert_eq!(landed_cost.duty_amount_cents.value, 10_625);
    assert_eq!(landed_cost.total_landed_cost_cents.value, 445_625);
    assert!(!landed_cost.accounting_posting_attached.value);
    assert!(!landed_cost.inventory_cost_update_attached.value);
    assert!(!landed_cost.payment_disbursement_attached.value);
    assert!(!landed_cost.cloud_deployment_attached.value);
}

#[test]
fn global_trade_refuses_unscreened_unclassified_or_held_flow() {
    assert_eq!(
        assess_export_control(assessment_input(
            false,
            true,
            ScreeningOutcome::Cleared,
            "EAR99",
            false,
            false,
        )),
        Err(GlobalTradeComplianceError::PartyScreeningRequired)
    );
    assert_eq!(
        assess_export_control(assessment_input(
            true,
            false,
            ScreeningOutcome::Cleared,
            "EAR99",
            false,
            false,
        )),
        Err(GlobalTradeComplianceError::ItemClassificationRequired)
    );
    assert_eq!(
        prepare_customs_declaration(declaration_input(
            false,
            ExportControlDecision::AllowedNoLicense,
        )),
        Err(GlobalTradeComplianceError::ExportControlAssessmentRequired)
    );
    assert_eq!(
        prepare_customs_declaration(declaration_input(true, ExportControlDecision::EmbargoHold)),
        Err(GlobalTradeComplianceError::ComplianceHoldRequired)
    );
    assert_eq!(
        simulate_landed_cost(landed_cost_input(false)),
        Err(GlobalTradeComplianceError::CustomsDeclarationRequired)
    );
}

#[test]
fn global_trade_validates_refs_codes_amounts_and_rates() {
    let mut bad_country = party_input(120);
    bad_country.country_code = "USA".to_owned();
    assert_eq!(
        screen_trade_party(bad_country),
        Err(GlobalTradeComplianceError::InvalidCountryCode)
    );

    let mut unsafe_evidence = party_input(120);
    unsafe_evidence.screening_evidence_ref = "audit/global-trade/secret-token".to_owned();
    assert_eq!(
        screen_trade_party(unsafe_evidence),
        Err(GlobalTradeComplianceError::InvalidEvidenceRef)
    );

    let mut bad_hs = classification_input();
    bad_hs.hs_code = "84 7130".to_owned();
    assert_eq!(
        classify_trade_item(bad_hs),
        Err(GlobalTradeComplianceError::InvalidHsCode)
    );

    let mut bad_rate = classification_input();
    bad_rate.duty_rate_bps = 10_001;
    assert_eq!(
        classify_trade_item(bad_rate),
        Err(GlobalTradeComplianceError::InvalidDutyRate)
    );

    let mut bad_source = classification_input();
    bad_source.classification_source_ref = "src/../classification".to_owned();
    assert_eq!(
        classify_trade_item(bad_source),
        Err(GlobalTradeComplianceError::InvalidSourceDocumentRef)
    );

    let mut zero_value = declaration_input(true, ExportControlDecision::AllowedNoLicense);
    zero_value.unit_customs_value_cents = 0;
    assert_eq!(
        prepare_customs_declaration(zero_value),
        Err(GlobalTradeComplianceError::InvalidAmount)
    );
}

#[test]
fn global_trade_records_compliance_holds_without_runtime_mutation_claim() {
    let screening = screen_trade_party(party_input(9_200)).unwrap();
    assert_eq!(
        screening.outcome.value,
        ScreeningOutcome::PotentialMatchHold
    );
    assert!(screening.restricted_party_hold_required.value);
    assert!(!screening.business_transaction_block_mutation_attached.value);

    let party_hold = assess_export_control(assessment_input(
        true,
        true,
        screening.outcome.value,
        "EAR99",
        false,
        false,
    ))
    .unwrap();
    assert_eq!(party_hold.decision.value, ExportControlDecision::PartyHold);
    assert!(party_hold.compliance_hold_required.value);
    assert!(!party_hold.order_hold_mutation_attached.value);
    assert!(!party_hold.workflow_execution_attached.value);
    assert!(!party_hold.cloud_deployment_attached.value);

    let controlled_without_license = assess_export_control(assessment_input(
        true,
        true,
        ScreeningOutcome::Cleared,
        "5A002",
        false,
        false,
    ))
    .unwrap();
    assert_eq!(
        controlled_without_license.decision.value,
        ExportControlDecision::LicenseRequiredHold
    );
    assert!(controlled_without_license.compliance_hold_required.value);

    let controlled_with_license = assess_export_control(assessment_input(
        true,
        true,
        ScreeningOutcome::Cleared,
        "5A002",
        true,
        false,
    ))
    .unwrap();
    assert_eq!(
        controlled_with_license.decision.value,
        ExportControlDecision::AllowedWithLicense
    );
    assert!(!controlled_with_license.compliance_hold_required.value);
}
