#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payroll_run_domain::{
    EntityCloseSnapshot, PayrollDomainError, PayrollRunState, close_group_rollup,
};

mod support;
use support::digest;

#[test]
fn test_entity_close_before_group_rollup() {
    let closes = vec![entity_close("le_kr_001"), entity_close("le_kr_002")];
    let rollup = close_group_rollup("pgrp_2026_01", "ten_acme", closes).expect("rollup");

    assert_eq!(rollup.state.value, PayrollRunState::GroupRolledUp);
    assert_eq!(rollup.entity_closes.value.len(), 2);

    let mut incomplete = entity_close("le_kr_003");
    incomplete.state = PayrollRunState::TrialClosed;
    assert_eq!(
        close_group_rollup("pgrp_2026_02", "ten_acme", vec![incomplete]),
        Err(PayrollDomainError::EntityCloseIncomplete)
    );

    let mut unredacted = entity_close("le_kr_004");
    unredacted.detachment_history_redacted = false;
    assert_eq!(
        close_group_rollup("pgrp_2026_03", "ten_acme", vec![unredacted]),
        Err(PayrollDomainError::DetachmentHistoryNotRedacted)
    );
}

fn entity_close(legal_entity_id: &str) -> EntityCloseSnapshot {
    EntityCloseSnapshot {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: legal_entity_id.to_owned(),
        run_id: format!("prun_{legal_entity_id}_2026_01"),
        state: PayrollRunState::EntityClosed,
        evidence_digest: digest(),
        detachment_history_redacted: true,
    }
}
