// Cedar policy fragments for workplace-integration.
// Wave 15-ZF-14 propagation for ADR-0346, ADR-0347, ADR-0348, and ADR-0349.
// D-CICD-AUTHORITY purpose: branch-protected oya-ci-required is the live cloud CI merge authority; local verifier output is transition evidence only.
// D-CLOUD-NATIVE purpose: do not revive local or dev CLI authority; re-home sound checks into cloud-native Rust gate and pipeline surfaces.
// D-GOVERNANCE-CENTRAL purpose: central governance vocabulary supersedes scattered lane naming in active authority surfaces.
// Current governance evidence: central PaC/CaC/PDP/evidence pipeline acceptance.
// ADR-0348 purpose: cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; tenant migration honors residency + compliance pack constraints; cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243; every automation event MUST emit per ADR-0263.
// ADR-0348 enforcement evidence is attached through central governance and the branch-protected oya-ci-required gate.
// D-CICD-AUTHORITY cutover: owned oya-ci is the same canonical pipeline migrated after cutover, not a parallel shadow authority.
// D-GOVERNANCE-CENTRAL purpose: central PaC/CaC/PDP/evidence pipelines own governance authority, not scattered local gates.

permit(
  principal == ServicePrincipal::"oyatie.cell-orchestrator",
  action in [Action::"ComputeAutoshardingPlacement", Action::"ExecuteAutoRebalance", Action::"ExecuteDynamicSharding"],
  resource is ShardingAutomation
) when {
  resource.microservice == "workplace-integration" &&
  context.doctrine.adr_0348 == true &&
  context.autosharding_mode == "control_plane_driven" &&
  context.honors_residency == true &&
  context.compliance_pack_check_result == "pass" &&
  context.residency_check_result == "pass" &&
  context.audit_chain_emit == true
};

forbid(
  principal == ServicePrincipal::"oyatie.cell-orchestrator",
  action in [Action::"ExecuteAutoRebalance", Action::"ExecuteDynamicSharding"],
  resource is ShardingAutomation
) when {
  resource.microservice == "workplace-integration" &&
  (
    context.honors_residency == false ||
    context.compliance_pack_check_result != "pass" ||
    context.residency_check_result != "pass" ||
    context.audit_chain_emit != true
  )
};

forbid(
  principal == ServicePrincipal::"oyatie.cell-orchestrator",
  action == Action::"MigrateTenantCrossJurisdiction",
  resource is TenantMigration
) unless {
  resource.microservice == "workplace-integration" &&
  context.cedar_permit_id != "" &&
  context.cross_jurisdiction_permit_id != "" &&
  context.audit_chain_emit == true
};

permit(
  principal == ServicePrincipal::"oyatie.github-actions.oya-ci-required",
  action == Action::"RunCloudCiRequired",
  resource is ServiceCiSurface
) when {
  resource.microservice == "workplace-integration" &&
  context.doctrine.d_cicd_authority == true &&
  context.required_context == "oya-ci-required" &&
  context.branch_protection_enforced == true &&
  context.shared_rust_gate_logic == true &&
  context.local_cli_authority == false &&
  context.step_exit_semantics == "surface_all_required_errors" &&
  context.exit_code_contract in [0, 1, 2]
};

permit(
  principal == ServicePrincipal::"oyatie.oya-ci.cutover",
  action == Action::"RunOwnedCiAfterCutover",
  resource is ServiceCiSurface
) when {
  resource.microservice == "workplace-integration" &&
  context.doctrine.d_cicd_authority == true &&
  context.required_context == "oya-ci-required" &&
  context.owned_runner_cutover == true &&
  context.not_parallel_shadow == true &&
  context.audit_chain_emit == true
};

forbid(
  (principal == ServicePrincipal::"oyatie.github-actions.oya-ci-required" || principal == ServicePrincipal::"oyatie.oya-ci.cutover"),
  action in [Action::"RunCloudCiRequired", Action::"RunOwnedCiAfterCutover"],
  resource is ServiceCiSurface
) when {
  resource.microservice == "workplace-integration" &&
  (
    context.required_context != "oya-ci-required" ||
    context.local_cli_authority == true ||
    context.branch_protection_enforced == false
  )
};

permit(
  principal == ServicePrincipal::"oyatie.argocd.gitops",
  action == Action::"SyncApplication",
  resource is GitOpsApplication
) when {
  resource.microservice == "workplace-integration" &&
  context.doctrine.adr_0349 == true &&
  context.cosign_signature_verified == true &&
  context.tenant_namespace_isolated == true &&
  context.cedar_policy_gate_granted == true &&
  context.audit_chain_emit == true
};

forbid(
  principal == ServicePrincipal::"oyatie.argocd.gitops",
  action == Action::"SyncApplication",
  resource is GitOpsApplication
) when {
  resource.microservice == "workplace-integration" &&
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
  resource.microservice == "workplace-integration" &&
  context.doctrine.adr_0347 == true &&
  context.historical_reference != true
};
