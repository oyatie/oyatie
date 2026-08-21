#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use payroll_run_domain::{
    MoneyAmount, PayeeVarianceTotal, PayrollDomainError, RetroAdjustmentInput, RetroPayeeClass,
    evaluate_retro_adjustment,
};

// ── helpers ────────────────────────────────────────────────────────────────

fn krw(amount_minor: i64) -> MoneyAmount {
    MoneyAmount {
        amount_minor,
        currency: "KRW".to_owned(),
    }
}

fn usd(amount_minor: i64) -> MoneyAmount {
    MoneyAmount {
        amount_minor,
        currency: "USD".to_owned(),
    }
}

fn total(payee_id: &str, amount: MoneyAmount) -> PayeeVarianceTotal {
    PayeeVarianceTotal {
        payee_id: payee_id.to_owned(),
        net_amount: amount,
    }
}

fn base_input() -> RetroAdjustmentInput {
    RetroAdjustmentInput {
        run_id: "prun_kr_2025_12".to_owned(),
        run_ref: "audit/payroll/retro/prun-kr-2025-12".to_owned(),
        original_period_totals: vec![
            total("payee_001", krw(1_000_000)),
            total("payee_002", krw(2_000_000)),
        ],
        corrected_period_totals: vec![
            total("payee_001", krw(1_100_000)),
            total("payee_002", krw(2_000_000)),
        ],
        evidence_refs: vec!["audit/payroll/retro/ev-001".to_owned()],
    }
}

// ── (a) Pure delta for matched payee (Changed) ─────────────────────────────

#[test]
fn matched_payee_delta_is_corrected_minus_original() {
    let verdict = evaluate_retro_adjustment(base_input()).expect("retro verdict");

    // payee_001: 1_100_000 - 1_000_000 = +100_000
    let line = verdict
        .lines
        .value
        .iter()
        .find(|l| l.payee_id.value.value == "payee_001")
        .expect("payee_001 line must be present");

    assert_eq!(
        line.delta_amount.value.amount_minor, 100_000,
        "delta must be corrected minus original"
    );
    assert_eq!(line.delta_amount.value.currency, "KRW");
    assert_eq!(
        line.payee_class.value,
        RetroPayeeClass::Changed,
        "payee_001 must be classified as Changed"
    );
}

// ── (b) Currency mismatch → typed error ───────────────────────────────────

#[test]
fn currency_mismatch_returns_error() {
    let mut input = base_input();
    // payee_001 original is KRW, corrected will be USD — mismatch
    input.corrected_period_totals[0] = total("payee_001", usd(1_100_000));

    assert_eq!(
        evaluate_retro_adjustment(input),
        Err(PayrollDomainError::CurrencyMismatch),
        "mismatched currencies on same payee must return CurrencyMismatch"
    );
}

// ── (c) Newly added payee delta = full corrected amount ───────────────────

#[test]
fn added_payee_delta_equals_full_corrected_amount() {
    let mut input = base_input();
    // payee_003 exists only in corrected totals
    input
        .corrected_period_totals
        .push(total("payee_003", krw(500_000)));

    let verdict = evaluate_retro_adjustment(input).expect("retro verdict");

    let line = verdict
        .lines
        .value
        .iter()
        .find(|l| l.payee_id.value.value == "payee_003")
        .expect("payee_003 line must be present");

    assert_eq!(
        line.delta_amount.value.amount_minor, 500_000,
        "added payee delta must equal the corrected amount"
    );
    assert_eq!(
        line.original_amount.value.amount_minor, 0,
        "original synthetic amount for added payee must be zero"
    );
    assert_eq!(
        line.payee_class.value,
        RetroPayeeClass::Added,
        "payee_003 must be classified as Added"
    );
}

// ── (d) Removed payee delta = negative original ───────────────────────────

#[test]
fn removed_payee_delta_is_negative_original() {
    let mut input = base_input();
    // Remove payee_002 from corrected totals entirely
    input
        .corrected_period_totals
        .retain(|t| t.payee_id != "payee_002");

    let verdict = evaluate_retro_adjustment(input).expect("retro verdict");

    let line = verdict
        .lines
        .value
        .iter()
        .find(|l| l.payee_id.value.value == "payee_002")
        .expect("payee_002 line must be present");

    assert_eq!(
        line.delta_amount.value.amount_minor, -2_000_000,
        "removed payee delta must be negative original amount"
    );
    assert_eq!(
        line.corrected_amount.value.amount_minor, 0,
        "corrected synthetic amount for removed payee must be zero"
    );
    assert_eq!(
        line.payee_class.value,
        RetroPayeeClass::Removed,
        "payee_002 must be classified as Removed"
    );
}

// ── (e) Zero-delta payee retained with delta 0 ────────────────────────────

#[test]
fn zero_delta_payee_retained_as_unchanged() {
    // payee_002 has identical original and corrected amounts in base_input
    let verdict = evaluate_retro_adjustment(base_input()).expect("retro verdict");

    let line = verdict
        .lines
        .value
        .iter()
        .find(|l| l.payee_id.value.value == "payee_002")
        .expect("payee_002 line must be present even with zero delta");

    assert_eq!(
        line.delta_amount.value.amount_minor, 0,
        "zero-delta payee must have delta 0"
    );
    assert_eq!(
        line.payee_class.value,
        RetroPayeeClass::Unchanged,
        "payee_002 must be classified as Unchanged"
    );
}

// ── (f) run_net_delta = sum of line deltas ────────────────────────────────

#[test]
fn run_net_delta_equals_sum_of_line_deltas() {
    // base: payee_001 +100_000, payee_002 +0 → net = +100_000
    let verdict = evaluate_retro_adjustment(base_input()).expect("retro verdict");

    let sum_of_lines: i64 = verdict
        .lines
        .value
        .iter()
        .map(|l| l.delta_amount.value.amount_minor)
        .sum();

    assert_eq!(
        verdict.run_net_delta.value.amount_minor, sum_of_lines,
        "run_net_delta must equal sum of all line deltas"
    );
    assert_eq!(verdict.run_net_delta.value.amount_minor, 100_000);
    assert!(verdict.balanced.value, "verdict must be balanced");
}

// ── (g) Invalid run_id / evidence_ref rejected ────────────────────────────

#[test]
fn invalid_run_id_prefix_is_rejected() {
    let mut input = base_input();
    input.run_id = "run_kr_2025_12".to_owned(); // missing 'p'

    assert_eq!(
        evaluate_retro_adjustment(input),
        Err(PayrollDomainError::InvalidRunId),
        "run_id without prun_ prefix must return InvalidRunId"
    );
}

#[test]
fn empty_evidence_refs_is_rejected() {
    let mut input = base_input();
    input.evidence_refs.clear();

    assert_eq!(
        evaluate_retro_adjustment(input),
        Err(PayrollDomainError::RetroEvidenceRequired),
        "empty evidence_refs must return RetroEvidenceRequired"
    );
}

#[test]
fn invalid_evidence_ref_path_traversal_is_rejected() {
    let mut input = base_input();
    input.evidence_refs[0] = "audit/payroll/../secret".to_owned();

    assert_eq!(
        evaluate_retro_adjustment(input),
        Err(PayrollDomainError::InvalidEvidenceRef),
        "path traversal in evidence_ref must return InvalidEvidenceRef"
    );
}
