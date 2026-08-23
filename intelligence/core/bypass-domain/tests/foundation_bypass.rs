// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use data_boundary_kernel::DataClass;
use intelligence_bypass_domain::{
    AutonomyBreakGlassInput, AutonomyTier, BreakGlassQuorum, BypassError, BypassGate, BypassLedger,
    FoundationBypassInput,
};

#[test]
fn foundation_bypass_records_validate_window_and_public_classification() {
    let record = valid_bypass("byp_0001", 10)
        .build()
        .expect("bypass record is valid");

    assert_eq!(record.id.value, "byp_0001");
    assert_eq!(record.gate_bypassed.value, BypassGate::Architecture);
    assert_eq!(record.regression_window_days.value, 10);
    assert_eq!(record.data_class.value, DataClass::Public);
    assert_eq!(record.schema_version.value, 1);
}

#[test]
fn foundation_bypass_rejects_zero_window_and_invalid_shapes() {
    let zero_window = FoundationBypassInput {
        regression_window_days: 0,
        ..valid_bypass("byp_0002", 10)
    };
    assert_eq!(
        zero_window.build(),
        Err(BypassError::InvalidRegressionWindow)
    );

    let invalid_gate = FoundationBypassInput {
        gate_bypassed: "unknown-gate".into(),
        ..valid_bypass("byp_0003", 10)
    };
    assert_eq!(invalid_gate.build(), Err(BypassError::InvalidGate));

    let empty_rationale = FoundationBypassInput {
        rationale: " ".into(),
        ..valid_bypass("byp_0004", 10)
    };
    assert_eq!(empty_rationale.build(), Err(BypassError::EmptyRationale));
}

#[test]
fn bypass_ledger_rejects_duplicates_and_expired_open_records() {
    let open = valid_bypass("byp_0005", 10).build().unwrap();
    let duplicate = open.clone();
    assert_eq!(
        BypassLedger::from_records(vec![open.clone(), duplicate]),
        Err(BypassError::DuplicateBypass)
    );

    let ledger = BypassLedger::from_records(vec![open]).expect("ledger is valid");
    assert_eq!(ledger.open_count(), 1);
    assert_eq!(ledger.validate_windows(19), Ok(()));
    assert_eq!(ledger.validate_windows(21), Err(BypassError::ExpiredBypass));
}

#[test]
fn bypass_ledger_rejects_records_remediated_after_expiry() {
    let late = FoundationBypassInput {
        remediated_at_epoch_days: Some(25),
        ..valid_bypass("byp_0006", 10)
    }
    .build()
    .unwrap();
    let ledger = BypassLedger::from_records(vec![late]).expect("ledger is valid");

    assert_eq!(ledger.open_count(), 0);
    assert_eq!(ledger.validate_windows(30), Err(BypassError::ExpiredBypass));
}

#[test]
fn autonomy_break_glass_records_validate_quorum_expiry_and_public_classification() {
    let record = valid_break_glass("abg_0001", &["usr_security", "usr_privacy"], "two-of-three")
        .build()
        .expect("standard break-glass record is valid");

    assert_eq!(record.id.value, "abg_0001");
    assert_eq!(record.requested_tier.value, AutonomyTier::T4AutoExecute);
    assert_eq!(record.permitted_tier.value, AutonomyTier::T4AutoExecute);
    assert_eq!(record.approval_quorum.value, BreakGlassQuorum::TwoOfThree);
    assert_eq!(record.data_class.value, DataClass::Public);
    assert_eq!(record.schema_version.value, 1);

    let ledger =
        BypassLedger::from_ledger_records(vec![record.into()]).expect("ledger accepts record");
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger.open_count(), 1);
    assert_eq!(ledger.validate_windows(12), Ok(()));
    assert_eq!(ledger.validate_windows(13), Err(BypassError::ExpiredBypass));
}

#[test]
fn autonomy_break_glass_rejects_weak_quorum_self_approval_and_late_revoke() {
    let insufficient = valid_break_glass("abg_0002", &["usr_security"], "two-of-three");
    assert_eq!(
        insufficient.build(),
        Err(BypassError::InsufficientBreakGlassApprovals)
    );

    let duplicate = valid_break_glass(
        "abg_0003",
        &["usr_security", "usr_security"],
        "two-of-three",
    );
    assert_eq!(
        duplicate.build(),
        Err(BypassError::DuplicateBreakGlassApprover)
    );

    let self_approval = valid_break_glass(
        "abg_0004",
        &["usr_operator", "usr_security"],
        "two-of-three",
    );
    assert_eq!(
        self_approval.build(),
        Err(BypassError::BreakGlassSelfApproval)
    );

    let catastrophic = valid_break_glass(
        "abg_0005",
        &["usr_security", "usr_privacy"],
        "three-of-five",
    );
    assert_eq!(
        catastrophic.build(),
        Err(BypassError::InsufficientBreakGlassApprovals)
    );

    let tier_escalation = AutonomyBreakGlassInput {
        requested_tier: AutonomyTier::T2Advisory,
        permitted_tier: AutonomyTier::T4AutoExecute,
        ..valid_break_glass("abg_0006", &["usr_security", "usr_privacy"], "two-of-three")
    };
    assert_eq!(
        tier_escalation.build(),
        Err(BypassError::InvalidBreakGlassTier)
    );

    let late_revoke = AutonomyBreakGlassInput {
        revoked_at_epoch_days: Some(13),
        ..valid_break_glass("abg_0007", &["usr_security", "usr_privacy"], "two-of-three")
    }
    .build()
    .expect("late revoke shape builds so ledger can enforce expiry SLA");
    let ledger = BypassLedger::from_ledger_records(vec![late_revoke.into()])
        .expect("ledger accepts late-revoked record");
    assert_eq!(ledger.validate_windows(14), Err(BypassError::ExpiredBypass));
}

fn valid_bypass(id: &str, regression_window_days: u32) -> FoundationBypassInput {
    FoundationBypassInput {
        id: id.into(),
        pr_ref: "gh:oyatie/oyatie#123".into(),
        crate_ref: "intelligence-capability-kernel".into(),
        gate_bypassed: "architecture".into(),
        bypassing_actor: "usr_architect".into(),
        rationale: "temporary foundation sequencing gap".into(),
        regression_window_days,
        created_at_epoch_days: 10,
        remediated_at_epoch_days: None,
    }
}

fn valid_break_glass(
    id: &str,
    approving_actors: &[&str],
    approval_quorum: &str,
) -> AutonomyBreakGlassInput {
    AutonomyBreakGlassInput {
        id: id.into(),
        tenant_id: "ten_healthcare".into(),
        capability_id: "cap.clinical.assist".into(),
        requested_tier: AutonomyTier::T4AutoExecute,
        permitted_tier: AutonomyTier::T4AutoExecute,
        requesting_actor: "usr_operator".into(),
        approving_actors: approving_actors
            .iter()
            .map(|actor| (*actor).into())
            .collect(),
        approval_quorum: approval_quorum.into(),
        rationale: "patient safety emergency with explicit expiry".into(),
        created_at_epoch_days: 10,
        expires_at_epoch_days: 12,
        revoked_at_epoch_days: None,
    }
}
