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
fn approved_bank_account_records_cash_position_and_transfer_proposal() {
    let bank_account = approve_bank_account(bank_account_input()).unwrap();
    assert_eq!(bank_account.state.value, BankAccountState::Approved);
    assert_eq!(bank_account.bank_account_id.value.value, "ba_operating_usd");
    assert_eq!(bank_account.target_balance.value.amount_minor, 50_000);

    let position = record_cash_position(cash_position_input(true)).unwrap();
    assert_eq!(position.state.value, CashPositionState::Recorded);
    assert_eq!(position.closing_balance.value.amount_minor, 80_000);
    assert!(!position.payment_execution_attached.value);
    assert!(!position.bank_network_call_attached.value);
    assert!(!position.cloud_deployment_attached.value);
    assert!(!position.runtime_audit_chain_emission_attached.value);

    let forecast = project_liquidity_forecast(forecast_input(true)).unwrap();
    assert_eq!(forecast.state.value, LiquidityForecastState::Projected);
    assert_eq!(
        forecast.projected_closing_balance.value.amount_minor,
        25_000
    );
    assert!(forecast.liquidity_breach.value);
    assert_eq!(forecast.shortfall_amount.value.amount_minor, 25_000);

    let transfer = propose_cash_transfer(transfer_input()).unwrap();
    assert_eq!(transfer.state.value, CashTransferProposalState::Proposed);
    assert_eq!(transfer.transfer_amount.value.amount_minor, 20_000);
    assert!(!transfer.payment_execution_attached.value);
    assert!(!transfer.bank_network_call_attached.value);
    assert!(!transfer.cloud_deployment_attached.value);
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
fn treasury_validates_refs_dates_money_and_transfer_need() {
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

    let mut bad_money = forecast_input(true);
    bad_money.forecasted_inflow.currency = "usd".to_owned();
    assert_eq!(
        project_liquidity_forecast(bad_money),
        Err(TreasuryDomainError::InvalidMoney)
    );

    let mut no_need = transfer_input();
    no_need.target_closing_balance = money(55_000, "USD");
    assert_eq!(
        propose_cash_transfer(no_need),
        Err(TreasuryDomainError::NoTargetLiquidityNeed)
    );
}
