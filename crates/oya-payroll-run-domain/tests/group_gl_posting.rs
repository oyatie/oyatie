#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payroll_run_domain::{
    GroupGlPostingInput, PayrollDomainError, PayrollJournalInput, PayrollJournalLineInput,
    build_group_gl_posting,
};

mod support;
use support::digest;

// ── helpers ────────────────────────────────────────────────────────────────

fn journal_entry(
    journal_id: &str,
    run_id: &str,
    legal_entity_id: &str,
    debit: i64,
    credit: i64,
) -> PayrollJournalInput {
    PayrollJournalInput {
        journal_id: journal_id.to_owned(),
        run_id: run_id.to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: legal_entity_id.to_owned(),
        period: "2026-01".to_owned(),
        source_payroll_digest: digest(),
        approval_evidence_ref: "audit/payroll/approval/cfo".to_owned(),
        lines: vec![
            PayrollJournalLineInput {
                account_code: "EXP-WAGES".to_owned(),
                debit_minor: debit,
                credit_minor: 0,
            },
            PayrollJournalLineInput {
                account_code: "LIAB-NETPAY".to_owned(),
                debit_minor: 0,
                credit_minor: credit,
            },
        ],
    }
}

fn balanced_entry(journal_id: &str, run_id: &str, legal_entity_id: &str) -> PayrollJournalInput {
    journal_entry(journal_id, run_id, legal_entity_id, 1_000_000, 1_000_000)
}

// ── tests ──────────────────────────────────────────────────────────────────

#[test]
fn test_two_balanced_entities_produces_correct_batch() {
    let input = GroupGlPostingInput {
        rollup_id: "pgrp_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        entries: vec![
            balanced_entry("jrn_le_kr_001_2026_01", "prun_le_kr_001_2026_01", "le_kr_001"),
            balanced_entry("jrn_le_kr_002_2026_01", "prun_le_kr_002_2026_01", "le_kr_002"),
        ],
        group_idempotency_key: "pgrp_2026_01:ten_acme:gl-batch".to_owned(),
    };

    let batch = build_group_gl_posting(input).expect("batch");

    assert_eq!(batch.drafts.value.len(), 2);
    // Group totals equal sum of per-entity debits/credits.
    assert_eq!(batch.total_debit_minor.value, 2_000_000);
    assert_eq!(batch.total_credit_minor.value, 2_000_000);
    // Group debit == group credit (balanced).
    assert_eq!(batch.total_debit_minor.value, batch.total_credit_minor.value);
    // Idempotency key is preserved verbatim.
    assert_eq!(
        batch.idempotency_key.value,
        "pgrp_2026_01:ten_acme:gl-batch"
    );
    assert_eq!(batch.rollup_id.value.value, "pgrp_2026_01");
    assert_eq!(batch.tenant_id.value.value, "ten_acme");
}

#[test]
fn test_unbalanced_entity_propagates_error() {
    let mut unbalanced = balanced_entry("jrn_le_kr_001_2026_01", "prun_le_kr_001_2026_01", "le_kr_001");
    // Break balance on the second entry only.
    unbalanced.lines[1].credit_minor = 999;

    let input = GroupGlPostingInput {
        rollup_id: "pgrp_2026_02".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        entries: vec![
            balanced_entry("jrn_le_kr_002_2026_01", "prun_le_kr_002_2026_01", "le_kr_002"),
            unbalanced,
        ],
        group_idempotency_key: "pgrp_2026_02:ten_acme:gl-batch".to_owned(),
    };

    assert_eq!(
        build_group_gl_posting(input),
        Err(PayrollDomainError::UnbalancedJournal)
    );
}

#[test]
fn test_empty_entries_returns_required_error() {
    let input = GroupGlPostingInput {
        rollup_id: "pgrp_2026_03".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        entries: vec![],
        group_idempotency_key: "pgrp_2026_03:ten_acme:gl-batch".to_owned(),
    };

    assert_eq!(
        build_group_gl_posting(input),
        Err(PayrollDomainError::GroupPostingEntitiesRequired)
    );
}

#[test]
fn test_duplicate_legal_entity_returns_error() {
    let input = GroupGlPostingInput {
        rollup_id: "pgrp_2026_04".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        entries: vec![
            balanced_entry("jrn_le_kr_001_a", "prun_le_kr_001_a", "le_kr_001"),
            balanced_entry("jrn_le_kr_001_b", "prun_le_kr_001_b", "le_kr_001"),
        ],
        group_idempotency_key: "pgrp_2026_04:ten_acme:gl-batch".to_owned(),
    };

    assert_eq!(
        build_group_gl_posting(input),
        Err(PayrollDomainError::DuplicateLegalEntityInGroup)
    );
}

#[test]
fn test_invalid_identifier_returns_error() {
    // Malformed run_id (missing prefix) triggers InvalidRunId from build_payroll_journal.
    let mut bad_entry = balanced_entry("jrn_le_kr_001_2026_01", "INVALID_RUN_ID", "le_kr_001");
    bad_entry.run_id = "bad-run-no-prefix".to_owned();

    let input = GroupGlPostingInput {
        rollup_id: "pgrp_2026_05".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        entries: vec![bad_entry],
        group_idempotency_key: "pgrp_2026_05:ten_acme:gl-batch".to_owned(),
    };

    assert_eq!(
        build_group_gl_posting(input),
        Err(PayrollDomainError::InvalidRunId)
    );
}
