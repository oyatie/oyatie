use billing_accounting_journal::{JournalLineInput, JournalPostInput, PeriodState};

pub fn digest() -> String {
    format!("sha256:{}", "b".repeat(64))
}

pub fn lines() -> Vec<JournalLineInput> {
    vec![
        JournalLineInput {
            account_code: "EXP-WAGES".to_owned(),
            debit_minor: 1_000_000,
            credit_minor: 0,
        },
        JournalLineInput {
            account_code: "LIAB-NETPAY".to_owned(),
            debit_minor: 0,
            credit_minor: 1_000_000,
        },
    ]
}

pub fn journal_input() -> JournalPostInput {
    JournalPostInput {
        journal_id: "jrn_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        period_state: PeriodState::Open,
        source_documents: vec!["src/payroll/run/prun_kr_2026_01".to_owned()],
        approval_evidence_ref: "audit/accounting/journal/approval".to_owned(),
        lines: lines(),
    }
}
