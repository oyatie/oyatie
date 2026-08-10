// Cedar policy fragments for itsm.
// Wave 15-ZF-14 propagation for ADR-0346, ADR-0347, ADR-0348, and ADR-0349.
// ADR-0346 purpose: ./bin/oya verify --ci-required is the canonical local pre-push verifier and MUST locally mirror the full CI matrix.
// ADR-0346 enforced_by: oya-governance-oya-verify-ci-mirror-coverage; oya-governance-oya-verify-ci-step-exit-semantics; oya-governance-oya-verify-skip-flag-allowlist; oya-governance-oya-submit-calls-verify; oya-governance-oya-verify-exit-code-contract.
// ADR-0347 purpose: every oya-governance-* CI lane prefix in the Oyatie corpus RENAMES to oya-governance-* in a single bulk-rename pull request.
// ADR-0347 enforced_by: oya-governance-no-foundry-fitness-residue; oya-governance-lane-prefix-vocabulary; oya-governance-rename-inventory-presence.
// ADR-0348 purpose: cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; tenant migration honors residency + compliance pack constraints; cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243; every automation event MUST emit per ADR-0263.
// ADR-0348 enforced_by: oya-governance-sharding-automation-coverage; oya-governance-autosharding-manual-mode-refusal; oya-governance-auto-rebalance-residency-honored; oya-governance-dynamic-sharding-threshold-coverage; oya-governance-audit-chain-emit-on-automation-events; oya-governance-tenant-migration-reversibility.
// ADR-0349 purpose: Jenkins is the canonical CI orchestrator for air-gap + on-prem + colo + oyatie-as-cloud-provider contexts where GitHub Actions runners are not available; ArgoCD is the canonical GitOps CD orchestrator; ArgoCD REPLACES manual kubectl apply and Helm CLI deploys across all contexts.
// ADR-0349 enforced_by: oya-governance-jenkins-github-actions-parity; oya-governance-argocd-application-cosign-verified; oya-governance-argocd-tenant-namespace-isolation; oya-governance-jenkins-jcasc-only; oya-governance-deploy-audit-chain-emit.

permit(
  principal == ServicePrincipal::"oyatie.foundry.cell-orchestrator",
  action in [Action::"ComputeAutoshardingPlacement", Action::"ExecuteAutoRebalance", Action::"ExecuteDynamicSharding"],
  resource is ShardingAutomation
) when {
  resource.microservice == "itsm" &&
  context.doctrine.adr_0348 == true &&
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
  resource.microservice == "itsm" &&
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
  resource.microservice == "itsm" &&
  context.cedar_permit_id != "" &&
  context.cross_jurisdiction_permit_id != "" &&
  context.audit_chain_emit == true
};

permit(
  principal == ServicePrincipal::"oyatie.jenkins.ci",
  action == Action::"RunCiMirror",
  resource is ServiceCiSurface
) when {
  resource.microservice == "itsm" &&
  context.doctrine.adr_0346 == true &&
  context.oya_verify_ci_required == true &&
  context.steps.cargo_fmt_all_check == true &&
  context.steps.cargo_check_workspace_all_targets == true &&
  context.steps.cargo_clippy_workspace_all_targets_d_warnings == true &&
  context.steps.cargo_nextest_workspace_no_fail_fast == true &&
  context.steps.oya_gate_run_all_ci_required == true &&
  context.step_exit_semantics == "block_on_exit_0_each_step" &&
  context.exit_code_contract in [0, 1, 2]
};

permit(
  principal == ServicePrincipal::"oyatie.jenkins.ci",
  action == Action::"RunSelfHostedCi",
  resource is ServiceCiSurface
) when {
  resource.microservice == "itsm" &&
  context.doctrine.adr_0349 == true &&
  context.github_actions_parity_ok == true &&
  context.jcasc_declared == true &&
  context.lane_prefix in ["oya-governance", "oya-check"] &&
  context.audit_chain_emit == true
};

forbid(
  principal == ServicePrincipal::"oyatie.jenkins.ci",
  action in [Action::"RunCiMirror", Action::"RunSelfHostedCi"],
  resource is ServiceCiSurface
) when {
  resource.microservice == "itsm" &&
  (
    context.github_actions_parity_ok == false ||
    context.jcasc_declared == false ||
    context.lane_id.matches("^oya-governance-.*$")
  )
};

permit(
  principal == ServicePrincipal::"oyatie.argocd.gitops",
  action == Action::"SyncApplication",
  resource is GitOpsApplication
) when {
  resource.microservice == "itsm" &&
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
  resource.microservice == "itsm" &&
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
  resource.microservice == "itsm" &&
  context.doctrine.adr_0347 == true &&
  context.lane_id.matches("^oya-governance-.*$") &&
  context.historical_reference != true
};
