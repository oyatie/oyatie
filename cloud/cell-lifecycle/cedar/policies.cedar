// Cedar policy fragments for cell-lifecycle per ADR-0276 D-3.5.
// Principal namespace retained by ADR-0203 after legacy service retirement.

permit(
  principal == Principal::"oyatie.control-plane.cell-lifecycle",
  action in [Action::"RegisterCell", Action::"ActivateCell", Action::"ReadLifecycle"],
  resource is Cell
) when {
  context.oyatie_version.matches("^\d{4}-\d{2}-\d{2}$") &&
  context.request_id != "" &&
  context.audit_chain_available == true
};

permit(
  principal,
  action == Action::"DrainCell",
  resource is Cell
) when {
  principal has role &&
  principal.role in ["ops-cellular", "cellular-sre", "cell-orchestrator"] &&
  context.evidence_pack_id != "" &&
  context.evidence_pack_sha256.matches("^[a-f0-9]{64}$") &&
  context.cedar_permit_id != "" &&
  context.drain_reason in ["hardware_failure", "load_skew", "planned_decommission", "compliance_withdrawal", "emergency_containment", "promotion_rollback"] &&
  resource.state in ["Activated", "Promoted-T4", "Promoted-T3", "Promoted-T2", "Promoted-T1", "Promoted-T0"]
};

forbid(
  principal,
  action == Action::"DecommissionCell",
  resource is Cell
) unless {
  resource.state == "Draining" &&
  context.tenancy_resident_count == 0 &&
  context.resident_count_snapshot_id != "" &&
  context.audit_chain_available == true
};

permit(
  principal,
  action == Action::"PromoteToT4",
  resource is Cell
) when {
  principal has role &&
  principal.role in ["ops-cellular", "cellular-sre", "cell-orchestrator"] &&
  context.target_tier == "T4" &&
  context.evidence_pack_id != "" &&
  context.gate_snapshot_sha256.matches("^[a-f0-9]{64}$") &&
  context.gates.error_budget_remaining_ok == true &&
  context.gates.warm_soak_ok == true &&
  context.gates.canary_slo_ok == true &&
  context.gates.cross_cell_mesh_ok == true &&
  context.gates.tenant_class_coverage_ok == true &&
  context.gates.compliance_pack_coverage_ok == true &&
  context.blast_radius_check_ok == true &&
  context.cedar_permit_id != ""
};

permit(
  principal,
  action == Action::"PromoteToT3",
  resource is Cell
) when {
  principal has role &&
  principal.role in ["ops-cellular", "cellular-sre", "cell-orchestrator"] &&
  context.target_tier == "T3" &&
  context.evidence_pack_id != "" &&
  context.gate_snapshot_sha256.matches("^[a-f0-9]{64}$") &&
  context.gates.error_budget_remaining_ok == true &&
  context.gates.warm_soak_ok == true &&
  context.gates.canary_slo_ok == true &&
  context.gates.cross_cell_mesh_ok == true &&
  context.gates.tenant_class_coverage_ok == true &&
  context.gates.compliance_pack_coverage_ok == true &&
  context.blast_radius_check_ok == true &&
  context.cedar_permit_id != ""
};

permit(
  principal,
  action == Action::"PromoteToT2",
  resource is Cell
) when {
  principal has role &&
  principal.role in ["ops-cellular", "cellular-sre", "cell-orchestrator"] &&
  context.target_tier == "T2" &&
  context.evidence_pack_id != "" &&
  context.gate_snapshot_sha256.matches("^[a-f0-9]{64}$") &&
  context.gates.error_budget_remaining_ok == true &&
  context.gates.warm_soak_ok == true &&
  context.gates.canary_slo_ok == true &&
  context.gates.cross_cell_mesh_ok == true &&
  context.gates.tenant_class_coverage_ok == true &&
  context.gates.compliance_pack_coverage_ok == true &&
  context.blast_radius_check_ok == true &&
  context.cedar_permit_id != ""
};

permit(
  principal,
  action == Action::"PromoteToT1",
  resource is Cell
) when {
  principal has role &&
  principal.role in ["ops-cellular", "cellular-sre", "cell-orchestrator"] &&
  context.target_tier == "T1" &&
  context.evidence_pack_id != "" &&
  context.gate_snapshot_sha256.matches("^[a-f0-9]{64}$") &&
  context.gates.error_budget_remaining_ok == true &&
  context.gates.warm_soak_ok == true &&
  context.gates.canary_slo_ok == true &&
  context.gates.cross_cell_mesh_ok == true &&
  context.gates.tenant_class_coverage_ok == true &&
  context.gates.compliance_pack_coverage_ok == true &&
  context.blast_radius_check_ok == true &&
  context.cedar_permit_id != ""
};

permit(
  principal,
  action == Action::"PromoteToT0",
  resource is Cell
) when {
  principal has role &&
  principal.role in ["ops-cellular", "cellular-sre", "cell-orchestrator"] &&
  context.target_tier == "T0" &&
  context.evidence_pack_id != "" &&
  context.gate_snapshot_sha256.matches("^[a-f0-9]{64}$") &&
  context.gates.error_budget_remaining_ok == true &&
  context.gates.warm_soak_ok == true &&
  context.gates.canary_slo_ok == true &&
  context.gates.cross_cell_mesh_ok == true &&
  context.gates.tenant_class_coverage_ok == true &&
  context.gates.compliance_pack_coverage_ok == true &&
  context.blast_radius_check_ok == true &&
  context.cedar_permit_id != ""
};

permit(
  principal,
  action == Action::"EmergencyPromotionOverride",
  resource is Cell
) when {
  principal.role == "cellular-sre" &&
  context.incident_id != "" &&
  context.signatures.incident_commander == true &&
  context.signatures.on_call_sre == true &&
  context.signatures.council_security == true &&
  context.audit_chain_available == true
};

forbid(
  principal,
  action in [Action::"PromoteToT0", Action::"PromoteToT1", Action::"PromoteToT2", Action::"PromoteToT3", Action::"PromoteToT4"],
  resource is Cell
) when {
  context.gates.compliance_pack_coverage_ok == false ||
  context.gates.tenant_class_coverage_ok == false ||
  context.blast_radius_check_ok == false
};
