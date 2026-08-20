use payroll_run_domain::{
    MoneyAmount, PayeeClass, PayeeInput, PayrollTrialCloseInput, WageLedgerEntryInput, WageLineKind,
};

pub fn digest() -> String {
    format!("sha256:{}", "a".repeat(64))
}

pub fn trial_close_input() -> PayrollTrialCloseInput {
    PayrollTrialCloseInput {
        run_id: "prun_kr_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        rulepack_ref: "rulepack/kr-payroll-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        evidence_digest: digest(),
        approval_evidence_ref: "audit/payroll/trial-close/approval".to_owned(),
        payees: vec![PayeeInput {
            payee_id: "payee_001".to_owned(),
            legal_entity_id: "le_kr_001".to_owned(),
            payee_class: PayeeClass::Employee,
            person_or_vendor_ref: "person/acme/001".to_owned(),
            tax_profile_ref: "tax/kr/employee/001".to_owned(),
            wage_ledger: vec![
                WageLedgerEntryInput {
                    entry_id: "wage_001_gross".to_owned(),
                    payee_id: "payee_001".to_owned(),
                    line_kind: WageLineKind::GrossEarnings,
                    amount: MoneyAmount {
                        amount_minor: 1_000_000,
                        currency: "KRW".to_owned(),
                    },
                    source_ref: "audit/hr/time/001".to_owned(),
                },
                WageLedgerEntryInput {
                    entry_id: "wage_001_net".to_owned(),
                    payee_id: "payee_001".to_owned(),
                    line_kind: WageLineKind::NetPay,
                    amount: MoneyAmount {
                        amount_minor: -800_000,
                        currency: "KRW".to_owned(),
                    },
                    source_ref: "audit/payroll/net/001".to_owned(),
                },
            ],
        }],
    }
}
