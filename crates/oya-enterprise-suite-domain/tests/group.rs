#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_enterprise_suite_domain::{
    CloseBoundaryState, EnterpriseSuiteDomainError, GroupRollupInput, Jurisdiction,
    LegalEntityCloseSnapshot, roll_up_group_close_status,
};

#[test]
fn test_group_rollup_preserves_entity_boundaries() {
    let rollup = roll_up_group_close_status(GroupRollupInput {
        tenant_id: "ten_acme".to_owned(),
        group_id: "grp_acme_kr".to_owned(),
        jurisdiction: Jurisdiction::Korea,
        dashboard_projection_ref: "projection/enterprise-suite/group/kr".to_owned(),
        legal_entities: vec![
            closed_snapshot(
                "le_kr_001",
                "audit/payroll/le1",
                "audit/accounting/le1",
                7,
                11,
            ),
            closed_snapshot(
                "le_kr_002",
                "audit/payroll/le2",
                "audit/accounting/le2",
                3,
                5,
            ),
        ],
    })
    .expect("group rollup");

    assert_eq!(rollup.tenant_id.value.value, "ten_acme");
    assert_eq!(rollup.group_id.value.value, "grp_acme_kr");
    assert!(rollup.all_entities_closed.value);
    assert_eq!(rollup.legal_entity_count.value, 2);
    assert_eq!(rollup.legal_entity_projections.value.len(), 2);
    assert_eq!(
        rollup.legal_entity_projections.value[0]
            .legal_entity_id
            .value
            .value,
        "le_kr_001"
    );
    assert_eq!(
        rollup.legal_entity_projections.value[1]
            .legal_entity_id
            .value
            .value,
        "le_kr_002"
    );
    assert_eq!(
        rollup.legal_entity_projections.value[0]
            .payroll_evidence_ref
            .value
            .value,
        "audit/payroll/le1"
    );
    assert_eq!(
        rollup.legal_entity_projections.value[1]
            .accounting_evidence_ref
            .value
            .value,
        "audit/accounting/le2"
    );
    assert_eq!(
        rollup.dashboard_projection_ref.value.value,
        "projection/enterprise-suite/group/kr"
    );
}

#[test]
fn test_group_rollup_rejects_cross_tenant_entity() {
    let error = roll_up_group_close_status(GroupRollupInput {
        tenant_id: "ten_acme".to_owned(),
        group_id: "grp_acme_kr".to_owned(),
        jurisdiction: Jurisdiction::Korea,
        dashboard_projection_ref: "projection/enterprise-suite/group/kr".to_owned(),
        legal_entities: vec![LegalEntityCloseSnapshot {
            tenant_id: "ten_other".to_owned(),
            ..closed_snapshot(
                "le_kr_001",
                "audit/payroll/le1",
                "audit/accounting/le1",
                1,
                1,
            )
        }],
    })
    .expect_err("cross-tenant rollup must be refused");

    assert_eq!(error, EnterpriseSuiteDomainError::CrossTenantLegalEntity);
}

fn closed_snapshot(
    legal_entity_id: &str,
    payroll_evidence_ref: &str,
    accounting_evidence_ref: &str,
    payroll_close_version: u64,
    accounting_close_version: u64,
) -> LegalEntityCloseSnapshot {
    LegalEntityCloseSnapshot {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: legal_entity_id.to_owned(),
        payroll_close_state: CloseBoundaryState::ProductionClosed,
        accounting_close_state: CloseBoundaryState::ProductionClosed,
        payroll_evidence_ref: payroll_evidence_ref.to_owned(),
        accounting_evidence_ref: accounting_evidence_ref.to_owned(),
        payroll_close_version,
        accounting_close_version,
    }
}
