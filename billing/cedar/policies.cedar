// Cedar policy fragments for cloud-billing.
// Wave 15-ZF-14 propagation for ADR-0346, ADR-0347, ADR-0348, and ADR-0349.
// ADR-0346 purpose: cargo fmt --all --check, cargo clippy --workspace --all-targets -- -D warnings, and cargo test --workspace are the local verify path and MUST locally mirror the merge-blocking presubmit matrix.
// ADR-0346 enforced_by: governance-verify-ci-mirror-coverage; governance-verify-ci-step-exit-semantics; governance-verify-skip-flag-allowlist; governance-submit-calls-verify; governance-verify-exit-code-contract.
// ADR-0347 purpose: every governance-* CI lane prefix in the Oyatie corpus RENAMES to governance-* in a single bulk-rename pull request.
// ADR-0347 enforced_by: governance-no-legacy-fitness-residue; governance-lane-prefix-vocabulary; governance-rename-inventory-presence.
// ADR-0348 purpose: cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; tenant migration honors residency + compliance pack constraints; cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243; every automation event MUST emit per ADR-0263.
// ADR-0348 enforced_by: governance-sharding-automation-coverage; governance-autosharding-manual-mode-refusal; governance-auto-rebalance-residency-honored; governance-dynamic-sharding-threshold-coverage; governance-audit-chain-emit-on-automation-events; governance-tenant-migration-reversibility.
// ADR-0349 purpose: Bridge CI orchestration remains authorized for air-gap + on-prem + colo + oyatie-as-cloud-provider contexts where hosted runners are unavailable; ArgoCD remains the GitOps CD bridge/reference adapter and replaces manual kubectl apply plus Helm CLI deploys across all contexts.
// ADR-0349 enforced_by: governance-ci-github-actions-parity; governance-argocd-application-cosign-verified; governance-argocd-tenant-namespace-isolation; governance-ci-config-as-code-only; governance-deploy-audit-chain-emit.

permit(
  principal == ServicePrincipal::"oyatie.control-plane.cell-orchestrator",
  action in [Action::"ComputeAutoshardingPlacement", Action::"ExecuteAutoRebalance", Action::"ExecuteDynamicSharding"],
  resource is ShardingAutomation
) when {
  resource.microservice == "cloud-billing" &&
  context.doctrine.adr_0348 == true &&
  context.autosharding_mode == "control_plane_driven" &&
  context.honors_residency == true &&
  context.compliance_pack_check_result == "pass" &&
  context.residency_check_result == "pass" &&
  context.audit_chain_emit == true
};

forbid(
  principal == ServicePrincipal::"oyatie.control-plane.cell-orchestrator",
  action in [Action::"ExecuteAutoRebalance", Action::"ExecuteDynamicSharding"],
  resource is ShardingAutomation
) when {
  resource.microservice == "cloud-billing" &&
  (
    context.honors_residency == false ||
    context.compliance_pack_check_result != "pass" ||
    context.residency_check_result != "pass" ||
    context.audit_chain_emit != true
  )
};

forbid(
  principal == ServicePrincipal::"oyatie.control-plane.cell-orchestrator",
  action == Action::"MigrateTenantCrossJurisdiction",
  resource is TenantMigration
) unless {
  resource.microservice == "cloud-billing" &&
  context.cedar_permit_id != "" &&
  context.cross_jurisdiction_permit_id != "" &&
  context.audit_chain_emit == true
};

permit(
  principal == ServicePrincipal::"oyatie.pipeline.ci",
  action == Action::"RunCiMirror",
  resource is ServiceCiSurface
) when {
  resource.microservice == "cloud-billing" &&
  context.doctrine.adr_0346 == true &&
  context.verify_ci_required == true &&
  context.steps.cargo_fmt_all_check == true &&
  context.steps.cargo_check_workspace_all_targets == true &&
  context.steps.cargo_clippy_workspace_all_targets_d_warnings == true &&
  context.steps.cargo_nextest_workspace_no_fail_fast == true &&
  context.steps.gate_run_all_ci_required == true &&
  context.step_exit_semantics == "block_on_exit_0_each_step" &&
  context.exit_code_contract in [0, 1, 2]
};

permit(
  principal == ServicePrincipal::"oyatie.pipeline.ci",
  action == Action::"RunSelfHostedCi",
  resource is ServiceCiSurface
) when {
  resource.microservice == "cloud-billing" &&
  context.doctrine.adr_0349 == true &&
  context.github_actions_parity_ok == true &&
  context.jcasc_declared == true &&
  context.lane_prefix in ["governance", "check"] &&
  context.audit_chain_emit == true
};

forbid(
  principal == ServicePrincipal::"oyatie.pipeline.ci",
  action in [Action::"RunCiMirror", Action::"RunSelfHostedCi"],
  resource is ServiceCiSurface
) when {
  resource.microservice == "cloud-billing" &&
  (
    context.github_actions_parity_ok == false ||
    context.jcasc_declared == false ||
    context.lane_id.matches("^governance-.*$")
  )
};

permit(
  principal == ServicePrincipal::"oyatie.argocd.gitops",
  action == Action::"SyncApplication",
  resource is GitOpsApplication
) when {
  resource.microservice == "cloud-billing" &&
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
  resource.microservice == "cloud-billing" &&
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
  resource.microservice == "cloud-billing" &&
  context.doctrine.adr_0347 == true &&
  context.lane_id.matches("^governance-.*$") &&
  context.historical_reference != true
};
