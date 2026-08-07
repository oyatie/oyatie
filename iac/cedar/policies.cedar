// Cedar policy fragments for cloud-iac.
// Wave 15-ZF-14 propagation for ADR-0346, ADR-0347, ADR-0348, and ADR-0349.
// ADR-0346 supersession: retired local oya verifier/gate/check wrappers are not authority mechanisms for active work; cloud-ci/oya-ci produces the single required oya-ci-required context.
// ADR-0346 enforced_by: oya-ci-required; cloud-ci shared Rust gate logic; local-authority-enforcer.
// ADR-0347 purpose: every oya-governance-* CI lane prefix in the Oyatie corpus RENAMES to oya-governance-* in a single bulk-rename pull request.
// ADR-0347 enforced_by: retired-fitness-residue detection; oya-governance-lane-prefix-vocabulary; oya-governance-rename-inventory-presence.
// ADR-0348 purpose: cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; tenant migration honors residency + compliance pack constraints; cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243; every automation event MUST emit per ADR-0263.
// ADR-0348 enforced_by: oya-governance-sharding-automation-coverage; oya-governance-autosharding-manual-mode-refusal; oya-governance-auto-rebalance-residency-honored; oya-governance-dynamic-sharding-threshold-coverage; oya-governance-audit-chain-emit-on-automation-events; oya-governance-tenant-migration-reversibility.
// ADR-0349 supersession: GitHub Actions produces the live oya-ci-required context until owned oya-ci runner cutover; ArgoCD remains the GitOps CD bridge/reference adapter and replaces manual kubectl apply / Helm CLI deploys.
// ADR-0349 enforced_by: oya-ci-required; oya-governance-argocd-application-cosign-verified; oya-governance-argocd-tenant-namespace-isolation; oya-governance-deploy-audit-chain-emit.

permit(
  principal == ServicePrincipal::"oyatie.cloud.cell-orchestrator",
  action in [Action::"ComputeAutoshardingPlacement", Action::"ExecuteAutoRebalance", Action::"ExecuteDynamicSharding"],
  resource is ShardingAutomation
) when {
  resource.microservice == "cloud-iac" &&
  context.doctrine.adr_0348 == true &&
  context.autosharding_mode == "control_plane_driven" &&
  context.honors_residency == true &&
  context.compliance_pack_check_result == "pass" &&
  context.residency_check_result == "pass" &&
  context.audit_chain_emit == true
};

forbid(
  principal == ServicePrincipal::"oyatie.cloud.cell-orchestrator",
  action in [Action::"ExecuteAutoRebalance", Action::"ExecuteDynamicSharding"],
  resource is ShardingAutomation
) when {
  resource.microservice == "cloud-iac" &&
  (
    context.honors_residency == false ||
    context.compliance_pack_check_result != "pass" ||
    context.residency_check_result != "pass" ||
    context.audit_chain_emit != true
  )
};

forbid(
  principal == ServicePrincipal::"oyatie.cloud.cell-orchestrator",
  action == Action::"MigrateTenantCrossJurisdiction",
  resource is TenantMigration
) unless {
  resource.microservice == "cloud-iac" &&
  context.cedar_permit_id != "" &&
  context.cross_jurisdiction_permit_id != "" &&
  context.audit_chain_emit == true
};

permit(
  principal == ServicePrincipal::"oyatie.github-actions.oya-ci",
  action == Action::"RunRequiredCi",
  resource is ServiceCiSurface
) when {
  resource.microservice == "cloud-iac" &&
  context.required_context == "oya-ci-required" &&
  context.shared_rust_gate_logic == true &&
  context.surface_all_aggregator == true &&
  context.local_cli_authority == false &&
  context.audit_chain_emit == true
};

permit(
  principal == ServicePrincipal::"oyatie.owned-oya-ci.runner",
  action == Action::"RunRequiredCi",
  resource is ServiceCiSurface
) when {
  resource.microservice == "cloud-iac" &&
  context.required_context == "oya-ci-required" &&
  context.cutover_proven == true &&
  context.shared_rust_gate_logic == true &&
  context.surface_all_aggregator == true &&
  context.local_cli_authority == false &&
  context.audit_chain_emit == true
};

forbid(
  principal == ServicePrincipal::"oyatie.github-actions.oya-ci",
  action == Action::"RunRequiredCi",
  resource is ServiceCiSurface
) when {
  resource.microservice == "cloud-iac" &&
  (
    context.required_context != "oya-ci-required" ||
    context.local_cli_authority == true ||
    context.lane_id.matches("^oya-governance-.*$")
  )
};

forbid(
  principal == ServicePrincipal::"oyatie.owned-oya-ci.runner",
  action == Action::"RunRequiredCi",
  resource is ServiceCiSurface
) when {
  resource.microservice == "cloud-iac" &&
  (
    context.required_context != "oya-ci-required" ||
    context.local_cli_authority == true ||
    context.lane_id.matches("^oya-governance-.*$")
  )
};

permit(
  principal == ServicePrincipal::"oyatie.argocd.gitops",
  action == Action::"SyncApplication",
  resource is GitOpsApplication
) when {
  resource.microservice == "cloud-iac" &&
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
  resource.microservice == "cloud-iac" &&
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
  resource.microservice == "cloud-iac" &&
  context.doctrine.adr_0347 == true &&
  context.lane_id.matches("^oya-governance-.*$") &&
  context.historical_reference != true
};
