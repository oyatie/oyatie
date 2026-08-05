use oya_treasury_cash_domain::{
    BankAccountApprovalInput, BankAccountState, CashPositionInput, CashPositionState,
    CashTransferProposalInput, CashTransferProposalState, LiquidityForecastInput,
    LiquidityForecastState, MoneyAmount, TreasuryDomainError, approve_bank_account,
    project_liquidity_forecast, propose_cash_transfer, record_cash_position,
};

fn money(amount_minor: i64, currency: &str) -> MoneyAmount {
    MoneyAmount {
        amount_minor,
        currency: currency.to_owned(),
    }
}

fn bank_account_input() -> BankAccountApprovalInput {
    BankAccountApprovalInput {
        bank_account_id: "ba_operating_usd".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        bank_id: "bank_global".to_owned(),
        currency: "USD".to_owned(),
        bank_account_master_ref: "src/treasury/bank-account/operating-usd".to_owned(),
        opening_balance_evidence_ref: "audit/treasury/bank-account/operating-usd/opening-balance"
            .to_owned(),
        control_evidence_ref: "audit/treasury/bank-account/operating-usd/dual-control".to_owned(),
        target_balance: money(50_000, "USD"),
        approved_at_epoch_seconds: 1_779_546_000,
    }
}

fn cash_position_input(bank_account_approved: bool) -> CashPositionInput {
    CashPositionInput {
        cash_position_id: "cpos_operating_usd_20260523".to_owned(),
        bank_account_id: "ba_operating_usd".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        bank_account_approved,
        position_date_yyyymmdd: 20260523,
        opening_balance: money(125_000, "USD"),
        actual_inflow: money(25_000, "USD"),
        actual_outflow: money(70_000, "USD"),
        bank_statement_evidence_ref: "audit/treasury/statements/operating-usd/2026-05-23"
            .to_owned(),
        exposure_flow_ref: "src/treasury/one-exposure/operating-usd/2026-05-23".to_owned(),
    }
}

fn forecast_input(cash_position_recorded: bool) -> LiquidityForecastInput {
    LiquidityForecastInput {
        forecast_id: "lfcst_operating_usd_5d".to_owned(),
        bank_account_id: "ba_operating_usd".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        cash_position_recorded,
        horizon_days: 5,
        starting_balance: money(80_000, "USD"),
        forecasted_inflow: money(10_000, "USD"),
        forecasted_outflow: money(65_000, "USD"),
        minimum_liquidity_target: money(50_000, "USD"),
        forecast_source_ref: "src/treasury/liquidity-forecast/operating-usd/5d".to_owned(),
        forecast_evidence_ref: "audit/treasury/liquidity-forecast/operating-usd/5d".to_owned(),
    }
}

fn transfer_input() -> CashTransferProposalInput {
    CashTransferProposalInput {
        transfer_proposal_id: "xfer_operating_to_payroll".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        source_bank_account_id: "ba_operating_usd".to_owned(),
        target_bank_account_id: "ba_payroll_usd".to_owned(),
        transfer_amount: money(20_000, "USD"),
        source_closing_balance: money(90_000, "USD"),
        source_minimum_balance: money(50_000, "USD"),
        target_closing_balance: money(20_000, "USD"),
        target_minimum_balance: money(50_000, "USD"),
        cash_pool_ref: "src/treasury/cash-pool/us-operating".to_owned(),
        approval_policy_evidence_ref: "audit/treasury/cash-transfer/policy".to_owned(),
        proposal_evidence_ref: "audit/treasury/cash-transfer/proposal".to_owned(),
    }
}

#[test]
fn treasury_prd_metadata_fields_are_present_and_non_claiming() {
    let bank_account = approve_bank_account(bank_account_input()).unwrap();
    assert_eq!(bank_account.state.value, BankAccountState::Approved);
    assert_eq!(bank_account.bank_account_id.value.value, "ba_operating_usd");
    assert_eq!(bank_account.tenant_id.value.value, "ten_enterprise");
    assert_eq!(bank_account.legal_entity_id.value.value, "le_us001");
    assert_eq!(bank_account.bank_id.value.value, "bank_global");
    assert_eq!(bank_account.currency.value, "USD");
    assert_eq!(
        bank_account.bank_account_master_ref.value.value,
        "src/treasury/bank-account/operating-usd"
    );
    assert_eq!(
        bank_account.opening_balance_evidence_ref.value.value,
        "audit/treasury/bank-account/operating-usd/opening-balance"
    );
    assert_eq!(
        bank_account.control_evidence_ref.value.value,
        "audit/treasury/bank-account/operating-usd/dual-control"
    );
    assert_eq!(bank_account.target_balance.value.amount_minor, 50_000);
    assert_eq!(bank_account.target_balance.value.currency, "USD");
    assert_eq!(
        bank_account.idempotency_key.value,
        "treasury:bank-account:ten_enterprise:le_us001:ba_operating_usd"
    );
    assert_eq!(bank_account.approved_at_epoch_seconds.value, 1_779_546_000);
    assert_eq!(bank_account.schema_version.value, 1);

    let position = record_cash_position(cash_position_input(true)).unwrap();
    assert_eq!(position.state.value, CashPositionState::Recorded);
    assert_eq!(
        position.cash_position_id.value.value,
        "cpos_operating_usd_20260523"
    );
    assert_eq!(position.bank_account_id.value.value, "ba_operating_usd");
    assert_eq!(position.tenant_id.value.value, "ten_enterprise");
    assert_eq!(position.legal_entity_id.value.value, "le_us001");
    assert_eq!(position.position_date_yyyymmdd.value, 20260523);
    assert_eq!(position.opening_balance.value.amount_minor, 125_000);
    assert_eq!(position.actual_inflow.value.amount_minor, 25_000);
    assert_eq!(position.actual_outflow.value.amount_minor, 70_000);
    assert_eq!(position.closing_balance.value.amount_minor, 80_000);
    assert_eq!(position.closing_balance.value.currency, "USD");
    assert_eq!(
        position.bank_statement_evidence_ref.value.value,
        "audit/treasury/statements/operating-usd/2026-05-23"
    );
    assert_eq!(
        position.exposure_flow_ref.value.value,
        "src/treasury/one-exposure/operating-usd/2026-05-23"
    );
    assert_eq!(
        position.idempotency_key.value,
        "treasury:cash-position:ten_enterprise:le_us001:ba_operating_usd:20260523"
    );
    assert!(!position.payment_execution_attached.value);
    assert!(!position.bank_network_call_attached.value);
    assert!(!position.cloud_deployment_attached.value);
    assert!(!position.durable_persistence_attached.value);
    assert!(!position.workflow_execution_attached.value);
    assert!(!position.accounting_ledger_mutation_attached.value);
    assert!(!position.statutory_filing_attached.value);
    assert!(!position.runtime_audit_chain_emission_attached.value);
    assert_eq!(position.schema_version.value, 1);

    let forecast = project_liquidity_forecast(forecast_input(true)).unwrap();
    assert_eq!(forecast.state.value, LiquidityForecastState::Projected);
    assert_eq!(forecast.forecast_id.value.value, "lfcst_operating_usd_5d");
    assert_eq!(forecast.bank_account_id.value.value, "ba_operating_usd");
    assert_eq!(forecast.tenant_id.value.value, "ten_enterprise");
    assert_eq!(forecast.legal_entity_id.value.value, "le_us001");
    assert_eq!(forecast.horizon_days.value, 5);
    assert_eq!(forecast.starting_balance.value.amount_minor, 80_000);
    assert_eq!(forecast.forecasted_inflow.value.amount_minor, 10_000);
    assert_eq!(forecast.forecasted_outflow.value.amount_minor, 65_000);
    assert_eq!(forecast.minimum_liquidity_target.value.amount_minor, 50_000);
    assert_eq!(
        forecast.projected_closing_balance.value.amount_minor,
        25_000
    );
    assert_eq!(forecast.projected_closing_balance.value.currency, "USD");
    assert!(forecast.liquidity_breach.value);
    assert_eq!(forecast.shortfall_amount.value.amount_minor, 25_000);
    assert_eq!(forecast.shortfall_amount.value.currency, "USD");
    assert_eq!(
        forecast.forecast_source_ref.value.value,
        "src/treasury/liquidity-forecast/operating-usd/5d"
    );
    assert_eq!(
        forecast.forecast_evidence_ref.value.value,
        "audit/treasury/liquidity-forecast/operating-usd/5d"
    );
    assert_eq!(
        forecast.idempotency_key.value,
        "treasury:liquidity-forecast:ten_enterprise:le_us001:ba_operating_usd:5d"
    );
    assert!(!forecast.payment_execution_attached.value);
    assert!(!forecast.bank_network_call_attached.value);
    assert!(!forecast.cloud_deployment_attached.value);
    assert!(!forecast.durable_persistence_attached.value);
    assert!(!forecast.workflow_execution_attached.value);
    assert!(!forecast.accounting_ledger_mutation_attached.value);
    assert!(!forecast.statutory_filing_attached.value);
    assert!(!forecast.runtime_audit_chain_emission_attached.value);
    assert_eq!(forecast.schema_version.value, 1);

    let transfer = propose_cash_transfer(transfer_input()).unwrap();
    assert_eq!(transfer.state.value, CashTransferProposalState::Proposed);
    assert_eq!(
        transfer.transfer_proposal_id.value.value,
        "xfer_operating_to_payroll"
    );
    assert_eq!(transfer.tenant_id.value.value, "ten_enterprise");
    assert_eq!(transfer.legal_entity_id.value.value, "le_us001");
    assert_eq!(
        transfer.source_bank_account_id.value.value,
        "ba_operating_usd"
    );
    assert_eq!(
        transfer.target_bank_account_id.value.value,
        "ba_payroll_usd"
    );
    assert_eq!(transfer.transfer_amount.value.amount_minor, 20_000);
    assert_eq!(transfer.transfer_amount.value.currency, "USD");
    assert_eq!(transfer.source_closing_balance.value.amount_minor, 90_000);
    assert_eq!(transfer.source_minimum_balance.value.amount_minor, 50_000);
    assert_eq!(transfer.target_closing_balance.value.amount_minor, 20_000);
    assert_eq!(transfer.target_minimum_balance.value.amount_minor, 50_000);
    assert_eq!(
        transfer.source_surplus_after_transfer.value.amount_minor,
        20_000
    );
    assert_eq!(
        transfer.target_shortfall_before_transfer.value.amount_minor,
        30_000
    );
    assert_eq!(
        transfer.cash_pool_ref.value.value,
        "src/treasury/cash-pool/us-operating"
    );
    assert_eq!(
        transfer.approval_policy_evidence_ref.value.value,
        "audit/treasury/cash-transfer/policy"
    );
    assert_eq!(
        transfer.proposal_evidence_ref.value.value,
        "audit/treasury/cash-transfer/proposal"
    );
    assert_eq!(
        transfer.idempotency_key.value,
        "treasury:cash-transfer:ten_enterprise:le_us001:ba_operating_usd:ba_payroll_usd"
    );
    assert!(!transfer.payment_execution_attached.value);
    assert!(!transfer.bank_network_call_attached.value);
    assert!(!transfer.cloud_deployment_attached.value);
    assert!(!transfer.durable_persistence_attached.value);
    assert!(!transfer.workflow_execution_attached.value);
    assert!(!transfer.accounting_ledger_mutation_attached.value);
    assert!(!transfer.statutory_filing_attached.value);
    assert!(!transfer.runtime_audit_chain_emission_attached.value);
    assert_eq!(transfer.schema_version.value, 1);
}

#[test]
fn treasury_refuses_unapproved_bank_account_and_missing_cash_position() {
    assert_eq!(
        record_cash_position(cash_position_input(false)),
        Err(TreasuryDomainError::BankAccountApprovalRequired)
    );
    assert_eq!(
        project_liquidity_forecast(forecast_input(false)),
        Err(TreasuryDomainError::CashPositionRequired)
    );
}

#[test]
fn treasury_refuses_currency_mismatch_and_insufficient_source_surplus() {
    let mut bad_position = cash_position_input(true);
    bad_position.actual_outflow = money(70_000, "EUR");
    assert_eq!(
        record_cash_position(bad_position),
        Err(TreasuryDomainError::CurrencyMismatch)
    );

    let mut insufficient = transfer_input();
    insufficient.source_closing_balance = money(60_000, "USD");
    assert_eq!(
        propose_cash_transfer(insufficient),
        Err(TreasuryDomainError::InsufficientSourceSurplus)
    );
}

#[test]
fn treasury_refuses_ac05_boundary_values() {
    let mut prefix_only_account = bank_account_input();
    prefix_only_account.bank_account_id = "ba_".to_owned();
    assert_eq!(
        approve_bank_account(prefix_only_account),
        Err(TreasuryDomainError::InvalidBankAccountId)
    );

    let mut whitespace_tenant = bank_account_input();
    whitespace_tenant.tenant_id = "ten_enterprise north".to_owned();
    assert_eq!(
        approve_bank_account(whitespace_tenant),
        Err(TreasuryDomainError::InvalidTenantId)
    );

    let mut control_legal_entity = bank_account_input();
    control_legal_entity.legal_entity_id = "le_us001\u{0007}".to_owned();
    assert_eq!(
        approve_bank_account(control_legal_entity),
        Err(TreasuryDomainError::InvalidLegalEntityId)
    );

    let mut prefix_only_source_ref = bank_account_input();
    prefix_only_source_ref.bank_account_master_ref = "src/".to_owned();
    assert_eq!(
        approve_bank_account(prefix_only_source_ref),
        Err(TreasuryDomainError::InvalidSourceDocumentRef)
    );

    let mut prefix_only_evidence_ref = bank_account_input();
    prefix_only_evidence_ref.opening_balance_evidence_ref = "audit/".to_owned();
    assert_eq!(
        approve_bank_account(prefix_only_evidence_ref),
        Err(TreasuryDomainError::InvalidEvidenceRef)
    );

    let mut unsafe_account = bank_account_input();
    unsafe_account.control_evidence_ref = "audit/treasury/secret-token".to_owned();
    assert_eq!(
        approve_bank_account(unsafe_account),
        Err(TreasuryDomainError::InvalidEvidenceRef)
    );

    let mut bad_date = cash_position_input(true);
    bad_date.position_date_yyyymmdd = 20261340;
    assert_eq!(
        record_cash_position(bad_date),
        Err(TreasuryDomainError::InvalidPositionDate)
    );

    let mut negative_money = cash_position_input(true);
    negative_money.actual_inflow.amount_minor = -1;
    assert_eq!(
        record_cash_position(negative_money),
        Err(TreasuryDomainError::InvalidMoney)
    );

    let mut traversal_ref = forecast_input(true);
    traversal_ref.forecast_source_ref = "src/../treasury/liquidity".to_owned();
    assert_eq!(
        project_liquidity_forecast(traversal_ref),
        Err(TreasuryDomainError::InvalidSourceDocumentRef)
    );

    let mut credential_source_ref = forecast_input(true);
    credential_source_ref.forecast_source_ref = "src/treasury/api-key".to_owned();
    assert_eq!(
        project_liquidity_forecast(credential_source_ref),
        Err(TreasuryDomainError::InvalidSourceDocumentRef)
    );

    let mut underscore_api_key_source_ref = forecast_input(true);
    underscore_api_key_source_ref.forecast_source_ref = "src/treasury/api_key".to_owned();
    assert_eq!(
        project_liquidity_forecast(underscore_api_key_source_ref),
        Err(TreasuryDomainError::InvalidSourceDocumentRef)
    );

    let mut bad_horizon = forecast_input(true);
    bad_horizon.horizon_days = 0;
    assert_eq!(
        project_liquidity_forecast(bad_horizon),
        Err(TreasuryDomainError::InvalidForecastHorizon)
    );

    let mut bad_money = forecast_input(true);
    bad_money.forecasted_inflow.currency = "usd".to_owned();
    assert_eq!(
        project_liquidity_forecast(bad_money),
        Err(TreasuryDomainError::InvalidMoney)
    );

    let mut credential_evidence_ref = transfer_input();
    credential_evidence_ref.proposal_evidence_ref = "audit/treasury/bearer-token".to_owned();
    assert_eq!(
        propose_cash_transfer(credential_evidence_ref),
        Err(TreasuryDomainError::InvalidEvidenceRef)
    );

    let mut private_key_evidence_ref = transfer_input();
    private_key_evidence_ref.proposal_evidence_ref = "audit/treasury/private_key".to_owned();
    assert_eq!(
        propose_cash_transfer(private_key_evidence_ref),
        Err(TreasuryDomainError::InvalidEvidenceRef)
    );

    let mut same_account = transfer_input();
    same_account.target_bank_account_id = same_account.source_bank_account_id.clone();
    assert_eq!(
        propose_cash_transfer(same_account),
        Err(TreasuryDomainError::SameBankAccountTransfer)
    );

    let mut no_need = transfer_input();
    no_need.target_closing_balance = money(55_000, "USD");
    assert_eq!(
        propose_cash_transfer(no_need),
        Err(TreasuryDomainError::NoTargetLiquidityNeed)
    );
}
