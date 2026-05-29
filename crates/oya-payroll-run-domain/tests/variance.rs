#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payroll_run_domain::{
    AnomalyFlag, MoneyAmount, PayeeVarianceTotal, PayrollDomainError, PayrollVarianceInput,
    evaluate_payroll_variance,
};

// ── helpers ────────────────────────────────────────────────────────────────

fn krw(amount_minor: i64) -> MoneyAmount {
    MoneyAmount {
        amount_minor,
        currency: "KRW".to_owned(),
    }
}

fn total(payee_id: &str, amount_minor: i64) -> PayeeVarianceTotal {
    PayeeVarianceTotal {
        payee_id: payee_id.to_owned(),
        net_amount: krw(amount_minor),
    }
}

/// Base valid input — two payees, both within 500 bps (5%) tolerance.
fn base_input() -> PayrollVarianceInput {
    PayrollVarianceInput {
        run_id: "prun_kr_2026_01".to_owned(),
        current_period_totals: vec![
            total("payee_001", 1_010_000), // +1% vs prior
            total("payee_002", 2_020_000), // +1% vs prior
        ],
        prior_period_totals: vec![
            total("payee_001", 1_000_000),
            total("payee_002", 2_000_000),
        ],
        variance_tolerance_bps: 500,
        rulepack_ref: "rulepack/kr-payroll-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        evidence_refs: vec![
            "audit/payroll/variance/001".to_owned(),
            "audit/payroll/variance/002".to_owned(),
        ],
        evaluated_at: 1_700_000_000,
    }
}

// ── tests ──────────────────────────────────────────────────────────────────

#[test]
fn happy_path_within_tolerance() {
    let verdict = evaluate_payroll_variance(base_input()).expect("variance verdict");

    assert!(verdict.gate_passed.value, "gate_passed should be true");
    assert!(
        verdict.anomaly_flags.value.is_empty(),
        "no anomaly flags expected"
    );
    assert_eq!(verdict.schema_version.value, 1);

    // BPS for payee_001: (1_010_000 - 1_000_000) * 10_000 / 1_000_000 = 100 bps
    let line0 = &verdict.lines.value[0];
    assert_eq!(line0.variance_bps.value, 100);
    assert!(!line0.anomaly.value);

    // BPS for payee_002: (2_020_000 - 2_000_000) * 10_000 / 2_000_000 = 100 bps
    let line1 = &verdict.lines.value[1];
    assert_eq!(line1.variance_bps.value, 100);
    assert!(!line1.anomaly.value);

    // evidence_digest starts with "sha256:" and has 64 hex chars after prefix
    let digest = &verdict.evidence_digest.value.value;
    assert!(digest.starts_with("sha256:"), "digest prefix");
    assert_eq!(digest.len(), "sha256:".len() + 64);

    // run_id round-trips
    assert_eq!(verdict.run_id.value.value, "prun_kr_2026_01");
}

#[test]
fn over_tolerance_swing() {
    let mut input = base_input();
    // payee_001 jumps +20% = 2000 bps, exceeds 500 bps tolerance
    input.current_period_totals[0] = total("payee_001", 1_200_000);

    let verdict = evaluate_payroll_variance(input).expect("verdict");

    assert!(!verdict.gate_passed.value, "gate_passed should be false");
    let flags = &verdict.anomaly_flags.value;
    assert!(
        flags.iter().any(|f| matches!(
            f,
            AnomalyFlag::OverToleranceSwing { payee_id } if payee_id.value == "payee_001"
        )),
        "OverToleranceSwing flag for payee_001 expected"
    );
}

#[test]
fn sign_flip() {
    let mut input = base_input();
    // payee_002 flips from positive to negative
    input.current_period_totals[1] = total("payee_002", -500_000);

    let verdict = evaluate_payroll_variance(input).expect("verdict");

    assert!(!verdict.gate_passed.value, "gate_passed should be false");
    let flags = &verdict.anomaly_flags.value;
    assert!(
        flags.iter().any(|f| matches!(
            f,
            AnomalyFlag::SignFlip { payee_id } if payee_id.value == "payee_002"
        )),
        "SignFlip flag for payee_002 expected"
    );
}

#[test]
fn dropped_payee() {
    let mut input = base_input();
    // Remove payee_002 from current period entirely
    input.current_period_totals.retain(|t| t.payee_id != "payee_002");

    let verdict = evaluate_payroll_variance(input).expect("verdict");

    assert!(!verdict.gate_passed.value, "gate_passed should be false");
    let flags = &verdict.anomaly_flags.value;
    assert!(
        flags.iter().any(|f| matches!(
            f,
            AnomalyFlag::DroppedPayee { payee_id } if payee_id.value == "payee_002"
        )),
        "DroppedPayee flag for payee_002 expected"
    );
}

#[test]
fn missing_tolerance() {
    let mut input = base_input();
    input.variance_tolerance_bps = 0;

    assert_eq!(
        evaluate_payroll_variance(input),
        Err(PayrollDomainError::VarianceToleranceRequired)
    );
}

#[test]
fn invalid_evidence_ref() {
    let mut input = base_input();
    // Path traversal makes this ref invalid
    input.evidence_refs[0] = "audit/payroll/../secret".to_owned();

    assert_eq!(
        evaluate_payroll_variance(input),
        Err(PayrollDomainError::InvalidEvidenceRef)
    );
}
