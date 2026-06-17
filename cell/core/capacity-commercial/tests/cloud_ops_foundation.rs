use cell_capacity_commercial::{
    CapacityUnits, CellCapacityEnvelope, CloudCapacityError, CloudOpsFoundationGuardrail,
    CloudOpsFoundationGuardrailCreate, MAX_REBALANCE_MOVE_BPS, REQUIRED_STABLE_HEADROOM_BPS,
};

fn units(vcpu: u32, memory_gb: u32) -> CapacityUnits {
    CapacityUnits {
        vcpu,
        memory_gb,
        gpu_count: 0,
        local_ssd_gb: 0,
    }
}

fn envelope() -> CellCapacityEnvelope {
    CellCapacityEnvelope {
        total: units(1_000, 4_000),
        allocated: units(300, 1_200),
        reserved: units(100, 400),
        spot_assigned: units(50, 200),
    }
}

fn create() -> CloudOpsFoundationGuardrailCreate {
    CloudOpsFoundationGuardrailCreate {
        tenant_id: "ten_alpha".to_string(),
        region: "region-alpha".to_string(),
        cell_id: "cell-region-alpha-a-001".to_string(),
        stable_capacity_envelope: envelope(),
        stable_reservation_units: units(200, 800),
        rebalance_source_total: units(1_000, 4_000),
        rebalance_move_units: units(50, 200),
        capacity_evidence_ref:
            "evidence/cloud-ops/capacity/ten_alpha/region-alpha/cell-region-alpha-a-001/capacity.json"
                .to_string(),
        cell_lifecycle_evidence_ref:
            "evidence/cloud-ops/cell/ten_alpha/region-alpha/cell-region-alpha-a-001/lifecycle.json"
                .to_string(),
        dcops_evidence_ref:
            "evidence/cloud-ops/dcops/ten_alpha/region-alpha/cell-region-alpha-a-001/site.json"
                .to_string(),
        finops_evidence_ref:
            "evidence/cloud-ops/finops/ten_alpha/region-alpha/cell-region-alpha-a-001/allocation.json"
                .to_string(),
        marketplace_evidence_ref:
            "evidence/cloud-ops/marketplace/ten_alpha/region-alpha/cell-region-alpha-a-001/entitlement.json"
                .to_string(),
        filesystem_handoff_evidence_ref:
            "evidence/cloud-ops/fsh/ten_alpha/region-alpha/cell-region-alpha-a-001/handoff.json"
                .to_string(),
        audit_chain_ref:
            "audit-chain/cloud-ops/ten_alpha/region-alpha/cell-region-alpha-a-001/cloud-ops.jsonl"
                .to_string(),
    }
}

#[test]
fn accepts_guardrail_when_all_ops_lanes_are_cell_scoped_and_bounded() {
    let guardrail = CloudOpsFoundationGuardrail::new(create()).expect("guardrail");

    assert_eq!(guardrail.tenant_id.value, "ten_alpha");
    assert_eq!(guardrail.region.value.value, "region-alpha");
    assert_eq!(guardrail.cell_id.value.value, "cell-region-alpha-a-001");
    assert!(guardrail.stable_headroom_bps.value >= REQUIRED_STABLE_HEADROOM_BPS);
    assert_eq!(guardrail.rebalance_move_bps.value, 500);
    assert!(guardrail.rebalance_move_bps.value <= MAX_REBALANCE_MOVE_BPS);
    assert_eq!(guardrail.schema_version.value, 1);
}

#[test]
fn rejects_capacity_slice_that_consumes_required_stable_headroom() {
    let err = CloudOpsFoundationGuardrail::new(CloudOpsFoundationGuardrailCreate {
        stable_reservation_units: units(500, 2_000),
        ..create()
    })
    .expect_err("stable reservations must preserve headroom");

    assert_eq!(err, CloudCapacityError::InvalidHeadroom);
}

#[test]
fn rejects_rebalance_moves_or_cells_that_exceed_local_foundation_bounds() {
    let move_err = CloudOpsFoundationGuardrail::new(CloudOpsFoundationGuardrailCreate {
        rebalance_move_units: units(200, 800),
        ..create()
    })
    .expect_err("rebalance move is capped");
    assert_eq!(move_err, CloudCapacityError::InvalidRebalanceMove);

    let cell_err = CloudOpsFoundationGuardrail::new(CloudOpsFoundationGuardrailCreate {
        cell_id: "cell-region-beta-a-001".to_string(),
        capacity_evidence_ref:
            "evidence/cloud-ops/capacity/ten_alpha/region-alpha/cell-region-beta-a-001/capacity.json"
                .to_string(),
        cell_lifecycle_evidence_ref:
            "evidence/cloud-ops/cell/ten_alpha/region-alpha/cell-region-beta-a-001/lifecycle.json"
                .to_string(),
        dcops_evidence_ref:
            "evidence/cloud-ops/dcops/ten_alpha/region-alpha/cell-region-beta-a-001/site.json"
                .to_string(),
        finops_evidence_ref:
            "evidence/cloud-ops/finops/ten_alpha/region-alpha/cell-region-beta-a-001/allocation.json"
                .to_string(),
        marketplace_evidence_ref:
            "evidence/cloud-ops/marketplace/ten_alpha/region-alpha/cell-region-beta-a-001/entitlement.json"
                .to_string(),
        filesystem_handoff_evidence_ref:
            "evidence/cloud-ops/fsh/ten_alpha/region-alpha/cell-region-beta-a-001/handoff.json"
                .to_string(),
        audit_chain_ref:
            "audit-chain/cloud-ops/ten_alpha/region-alpha/cell-region-beta-a-001/cloud-ops.jsonl"
                .to_string(),
        ..create()
    })
    .expect_err("cell id must match region");
    assert_eq!(cell_err, CloudCapacityError::InvalidCellId);
}

#[test]
fn rejects_secret_like_or_wrong_lane_evidence_references() {
    let secret_err = CloudOpsFoundationGuardrail::new(CloudOpsFoundationGuardrailCreate {
        finops_evidence_ref:
            "evidence/cloud-ops/finops/ten_alpha/region-alpha/cell-region-alpha-a-001/openbao-token.json"
                .to_string(),
        ..create()
    })
    .expect_err("evidence references must not embed secret material");
    assert_eq!(secret_err, CloudCapacityError::InvalidOpsEvidenceRef);

    let prefix_err = CloudOpsFoundationGuardrail::new(CloudOpsFoundationGuardrailCreate {
        marketplace_evidence_ref:
            "evidence/cloud-ops/finops/ten_alpha/region-alpha/cell-region-alpha-a-001/entitlement.json"
                .to_string(),
        ..create()
    })
    .expect_err("marketplace evidence must stay in the marketplace lane");
    assert_eq!(prefix_err, CloudCapacityError::InvalidOpsEvidenceRef);

    let audit_err = CloudOpsFoundationGuardrail::new(CloudOpsFoundationGuardrailCreate {
        audit_chain_ref:
            "evidence/cloud-ops/capacity/ten_alpha/region-alpha/cell-region-alpha-a-001/cloud-ops.jsonl"
                .to_string(),
        ..create()
    })
    .expect_err("audit chain refs must stay in the audit-chain lane");
    assert_eq!(audit_err, CloudCapacityError::InvalidAuditChainRef);
}

#[test]
fn rejects_tenant_region_or_cell_drift_inside_evidence_references() {
    let tenant_err = CloudOpsFoundationGuardrail::new(CloudOpsFoundationGuardrailCreate {
        capacity_evidence_ref:
            "evidence/cloud-ops/capacity/ten_beta/region-alpha/cell-region-alpha-a-001/capacity.json"
                .to_string(),
        ..create()
    })
    .expect_err("evidence tenant must match guardrail tenant");
    assert_eq!(tenant_err, CloudCapacityError::InvalidOpsEvidenceRef);

    let region_err = CloudOpsFoundationGuardrail::new(CloudOpsFoundationGuardrailCreate {
        dcops_evidence_ref:
            "evidence/cloud-ops/dcops/ten_alpha/region-beta/cell-region-alpha-a-001/site.json"
                .to_string(),
        ..create()
    })
    .expect_err("evidence region must match guardrail region");
    assert_eq!(region_err, CloudCapacityError::InvalidOpsEvidenceRef);

    let cell_err = CloudOpsFoundationGuardrail::new(CloudOpsFoundationGuardrailCreate {
        filesystem_handoff_evidence_ref:
            "evidence/cloud-ops/fsh/ten_alpha/region-alpha/cell-region-alpha-b-001/handoff.json"
                .to_string(),
        ..create()
    })
    .expect_err("evidence cell must match guardrail cell");
    assert_eq!(cell_err, CloudCapacityError::InvalidOpsEvidenceRef);
}
