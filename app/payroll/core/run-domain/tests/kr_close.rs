#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use payroll_run_domain::{
    MoneyAmount, PayeeClass, PayrollDomainError, PayrollRunState, WageLineKind, trial_close,
};

mod support;
use support::trial_close_input;

#[test]
fn test_trial_close_requires_rulepack_and_evidence() {
    let run = trial_close(trial_close_input()).expect("trial close");

    assert_eq!(run.state.value, PayrollRunState::TrialClosed);
    assert_eq!(run.legal_entity_id.value.value, "le_kr_001");
    assert_eq!(run.rulepack_effective_date.value.value, "2026-01-01");
    assert_eq!(run.payees.value[0].payee_class.value, PayeeClass::Employee);
    assert_eq!(run.payees.value[0].wage_ledger.value.len(), 2);
    assert_eq!(run.evidence_digest.value.value.len(), "sha256:".len() + 64);

    let mut missing_rulepack = trial_close_input();
    missing_rulepack.rulepack_ref = "rulepack/".to_owned();
    assert_eq!(
        trial_close(missing_rulepack),
        Err(PayrollDomainError::InvalidRulepackRef)
    );

    let mut missing_wage_ledger = trial_close_input();
    missing_wage_ledger.payees[0].wage_ledger.clear();
    assert_eq!(
        trial_close(missing_wage_ledger),
        Err(PayrollDomainError::PayeeMissingWageLedger)
    );

    let mut bad_source = trial_close_input();
    bad_source.payees[0].wage_ledger[0].source_ref = "audit/hr/../time".to_owned();
    assert_eq!(
        trial_close(bad_source),
        Err(PayrollDomainError::InvalidEvidenceRef)
    );

    let mut zero_line = trial_close_input();
    zero_line.payees[0].wage_ledger[0].amount = MoneyAmount {
        amount_minor: 0,
        currency: "KRW".to_owned(),
    };
    zero_line.payees[0].wage_ledger[0].line_kind = WageLineKind::GrossEarnings;
    assert_eq!(
        trial_close(zero_line),
        Err(PayrollDomainError::InvalidMoney)
    );
}
