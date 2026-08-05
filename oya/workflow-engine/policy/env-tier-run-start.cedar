// workflow-engine env-tier spec-store/run-start Plan/Spec/RED policy fragment
// Task: t_37183522
// Authority: specs/tenant-environment-tiers-canonical.json#workflow_studio_integration
// Dependency: t_732855bf tenancy env-tier API/Cedar contract
// Boundary: metadata-only contract; no runtime endpoint implementation, no live promotion,
// no outbound delivery, no prod traffic, no hyperscaler-readiness claim.

// Spec-store submissions fail closed without canonical env_tier and tenancy evidence.
forbid (
  principal in TenantOperator::?p,
  action == Action::"submit_workflow_spec",
  resource in TenantData::?r
)
unless {
  resource has env_tier &&
  (resource.env_tier == "test" || resource.env_tier == "staging" || resource.env_tier == "prod") &&
  resource has tenancy_env_tier_policy_ref &&
  resource has audit_chain_tag
};

// Run starts fail closed without canonical env_tier and tenancy evidence.
forbid (
  principal in TenantOperator::?p,
  action == Action::"start_workflow_run",
  resource in TenantData::?r
)
unless {
  resource has env_tier &&
  (resource.env_tier == "test" || resource.env_tier == "staging" || resource.env_tier == "prod") &&
  resource has spec_env_tier &&
  resource has tenancy_env_tier_policy_ref &&
  resource has audit_chain_tag
};

// Run start must match authenticated tenant, stored spec tenant, and tenancy env-tier evidence.
forbid (
  principal in TenantOperator::?p,
  action == Action::"start_workflow_run",
  resource in TenantData::?r
)
when {
  resource has tenant_id &&
  resource has spec_tenant_id &&
  resource.tenant_id != resource.spec_tenant_id
};

forbid (
  principal in TenantOperator::?p,
  action == Action::"start_workflow_run",
  resource in TenantData::?r
)
when {
  resource has spec_env_tier &&
  resource has env_tier &&
  resource.spec_env_tier != resource.env_tier
};

forbid (
  principal in TenantOperator::?p,
  action == Action::"start_workflow_run",
  resource in TenantData::?r
)
when {
  resource has tenancy_policy_tenant_id &&
  resource has tenant_id &&
  resource.tenancy_policy_tenant_id != resource.tenant_id
};

forbid (
  principal in TenantOperator::?p,
  action == Action::"start_workflow_run",
  resource in TenantData::?r
)
when {
  resource has tenancy_policy_env_tier &&
  resource has env_tier &&
  resource.tenancy_policy_env_tier != resource.env_tier
};

// Staging/prod starts are promotion-sensitive and require ChangeSet-style evidence.
forbid (
  principal in TenantOperator::?p,
  action == Action::"start_workflow_run",
  resource in TenantData::?r
)
when {
  resource has env_tier &&
  (resource.env_tier == "staging" || resource.env_tier == "prod") &&
  !(resource has changeset_approval_ref)
};

// Prod starts require ChangeSet-style approval/evidence and explicit destructive/outbound acknowledgments.
forbid (
  principal in TenantOperator::?p,
  action == Action::"start_workflow_run",
  resource in TenantData::?r
)
when {
  resource has env_tier &&
  resource.env_tier == "prod" &&
  (
    !(resource has changeset_approval_ref) ||
    !(resource has live_policy_evidence_ref) ||
    !(resource has prod_destructive_acknowledged) ||
    resource.prod_destructive_acknowledged != true ||
    !(resource has outbound_acknowledged) ||
    resource.outbound_acknowledged != true
  )
};

// Test and staging outbound semantics are metadata-only until outbound-effect lanes implement delivery controls.
forbid (
  principal in TenantOperator::?p,
  action == Action::"start_workflow_run",
  resource in TenantData::?r
)
when {
  resource has env_tier &&
  resource.env_tier == "staging" &&
  resource has contains_outbound_capable_steps &&
  resource.contains_outbound_capable_steps == true &&
  !(resource has test_recipient_policy_ref)
};
