// Cedar policy fragments for learning-management.
// Active doctrine fields are capability-named; decision-record IDs stay in docs/decisions.
// Native oya-ci/Prow and release-conveyor seams own durable CI/CD.

permit(
  principal == ServicePrincipal::"oyatie.foundry.cell-orchestrator",
  action in [Action::"ComputeAutoshardingPlacement", Action::"ExecuteAutoRebalance", Action::"ExecuteDynamicSharding"],
  resource is ShardingAutomation
) when {
  resource.microservice == "learning-management" &&
  context.doctrine.autosharding_residency_reversible_audit_emission == true &&
  context.autosharding_mode == "control_plane_driven" &&
  context.honors_residency == true &&
  context.compliance_pack_check_result == "pass" &&
  context.residency_check_result == "pass" &&
  context.audit_chain_emit == true
};

forbid(
  principal == ServicePrincipal::"oyatie.foundry.cell-orchestrator",
  action in [Action::"ExecuteAutoRebalance", Action::"ExecuteDynamicSharding"],
  resource is ShardingAutomation
) when {
  resource.microservice == "learning-management" &&
  (
    context.honors_residency == false ||
    context.compliance_pack_check_result != "pass" ||
    context.residency_check_result != "pass" ||
    context.audit_chain_emit != true
  )
};

forbid(
  principal == ServicePrincipal::"oyatie.foundry.cell-orchestrator",
  action == Action::"MigrateTenantCrossJurisdiction",
  resource is TenantMigration
) unless {
  resource.microservice == "learning-management" &&
  context.cedar_permit_id != "" &&
  context.cross_jurisdiction_permit_id != "" &&
  context.audit_chain_emit == true
};

permit(
  principal == ServicePrincipal::"oyatie.prow.oya-ci",
  action == Action::"RunBuck2Verification",
  resource is ServiceCiSurface
) when {
  resource.microservice == "learning-management" &&
  context.doctrine.buck2_prow_ci_authority == true &&
  context.buck2_evidence_collected == true &&
  context.steps.rustfmt_check == true &&
  context.steps.buck2_build_check == true &&
  context.steps.rust_lint_check == true &&
  context.steps.buck2_test_check == true &&
  context.steps.buck2_policy_check == true &&
  context.step_exit_semantics == "block_on_exit_0_each_step" &&
  context.exit_code_contract in [0, 1, 2]
};

permit(
  principal == ServicePrincipal::"oyatie.prow.oya-ci",
  action == Action::"RunProwJob",
  resource is ServiceCiSurface
) when {
  resource.microservice == "learning-management" &&
  context.doctrine.buck2_prow_ci_authority == true &&
  context.github_shadow_evidence_ok == true &&
  context.prow_job_evidence_ok == true &&
  context.lane_prefix in ["oya-governance", "oya-check"] &&
  context.audit_chain_emit == true
};

forbid(
  principal == ServicePrincipal::"oyatie.prow.oya-ci",
  action in [Action::"RunBuck2Verification", Action::"RunProwJob"],
  resource is ServiceCiSurface
) when {
  resource.microservice == "learning-management" &&
  (
    context.github_shadow_evidence_ok == false ||
    context.prow_job_evidence_ok == false ||
    context.lane_id.matches("^oya-governance-.*$")
  )
};

permit(
  principal == ServicePrincipal::"oyatie.release-conveyor.controller",
  action == Action::"SyncRelease",
  resource is ReleaseConveyorApplication
) when {
  resource.microservice == "learning-management" &&
  context.doctrine.release_conveyor_cd_authority == true &&
  context.cosign_signature_verified == true &&
  context.tenant_namespace_isolated == true &&
  context.cedar_policy_gate_granted == true &&
  context.audit_chain_emit == true
};

forbid(
  principal == ServicePrincipal::"oyatie.release-conveyor.controller",
  action == Action::"SyncRelease",
  resource is ReleaseConveyorApplication
) when {
  resource.microservice == "learning-management" &&
  (
    context.cosign_signature_verified == false ||
    context.tenant_namespace_isolated == false ||
    context.audit_chain_emit != true
  )
};

forbid(
  principal,
  action == Action::"AuthorGovernanceLane",
  resource is ServiceCiSurface
) when {
  resource.microservice == "learning-management" &&
  context.doctrine.retired_governance_lane_prefix_block == true &&
  context.lane_id.matches("^oya-governance-.*$") &&
  context.historical_reference != true
};
