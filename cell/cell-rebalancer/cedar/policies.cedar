
// cell-rebalancer Cedar policy fragments per ADR-0276 D-2.5.
// Principal/action/entity names are specification-stage placeholders for downstream schema binding.

permit(
  principal in Role::"ops-platform",
  action == Action::"cellRebalancerCreateJob",
  resource is RebalanceJob
)
when {
  resource.trigger in ["auto_rebalance", "manual_ops", "compliance_pack_rotation", "residency_change", "cell_drain"] &&
  resource.eligible_tenants_count <= 100 &&
  context.cedar_pre_evaluation == "permit" &&
  context.audit_chain_intent_present == true
};

permit(
  principal == ServicePrincipal::"oyatie.control-plane.cell-rebalancer",
  action == Action::"cellRebalancerCreateJob",
  resource is RebalanceJob
)
when {
  resource.trigger == "auto_rebalance" &&
  resource.eligible_tenants_count <= 100 &&
  principal.is_automated_with_baseline_signed_workflow &&
  principal.is_attested_by_meta_trust_root &&
  context.load_skew_threshold_percent >= 30 &&
  context.audit_chain_intent_present == true
};

permit(
  principal in Role::"ops-sre-reliability",
  action == Action::"cellRebalancerAbortJob",
  resource is RebalanceJob
)
when {
  resource.state in ["Queued", "Validated", "Migrating"] &&
  resource.eligible_tenants_count <= 100 &&
  context.abort_reason != "" &&
  context.audit_chain_intent_present == true
};

forbid(
  principal,
  action in [Action::"cellRebalancerCreateJob", Action::"cellRebalancerMigrateTenant"],
  resource is RebalanceJob
)
when {
  resource.eligible_tenants_count > 100
};

permit(
  principal,
  action == Action::"cellRebalancerMigrateTenantCrossJurisdiction",
  resource is TenantMigration
)
when {
  context.cross_jurisdiction_permit_id != "" &&
  context.residency_check_result == "pass" &&
  context.compliance_pack_check_result == "pass" &&
  context.target_cell_certified == true &&
  context.audit_chain_intent_present == true
};

permit(
  principal in Role::"ops-sre-reliability",
  action == Action::"cellRebalancerEmergencyAbort",
  resource is RebalanceJob
)
when {
  context.incident_severity in ["SEV-1", "SEV-2"] &&
  resource.eligible_tenants_count <= context.emergency_blast_radius_cap &&
  context.emergency_blast_radius_cap <= 100 &&
  context.audit_chain_intent_present == true
};

forbid(
  principal,
  action == Action::"cellRebalancerMigrateTenant",
  resource is TenantMigration
)
when {
  context.residency_check_result != "pass" ||
  context.compliance_pack_check_result != "pass" ||
  context.target_cell_headroom_percent < 30
};
