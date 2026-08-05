// workflow-studio env-tier default/promotion Plan/Spec/RED policy fragment
// Task: t_37183522
// Authority: specs/tenant-environment-tiers-canonical.json#workflow_studio_integration
// Dependency: t_732855bf tenancy env-tier API/Cedar contract
// Boundary: metadata-only contract; no runtime promotion, no tenant environment creation,
// no outbound delivery, no prod traffic, no hyperscaler-readiness claim.

// New definitions must resolve env_tier from tenancy authority and persist test by default.
forbid (
  principal in TenantOperator::?p,
  action == Action::"create_workflow_definition",
  resource in TenantData::?r
)
when {
  resource has requested_env_tier &&
  resource.requested_env_tier != "test"
};

// Saves fail closed when env_tier is absent or outside the accepted ladder.
forbid (
  principal in TenantOperator::?p,
  action == Action::"save_workflow_definition",
  resource in TenantData::?r
)
unless {
  resource has env_tier &&
  (resource.env_tier == "test" || resource.env_tier == "staging" || resource.env_tier == "prod") &&
  resource has tenancy_env_tier_policy_ref &&
  resource has audit_chain_tag
};

// Saves fail closed on authenticated tenant or tenancy env-tier evidence mismatch.
forbid (
  principal in TenantOperator::?p,
  action == Action::"save_workflow_definition",
  resource in TenantData::?r
)
when {
  resource has authenticated_tenant_id &&
  resource has definition_tenant_id &&
  resource.authenticated_tenant_id != resource.definition_tenant_id
};

forbid (
  principal in TenantOperator::?p,
  action == Action::"save_workflow_definition",
  resource in TenantData::?r
)
when {
  resource has tenancy_policy_tenant_id &&
  resource has definition_tenant_id &&
  resource.tenancy_policy_tenant_id != resource.definition_tenant_id
};

forbid (
  principal in TenantOperator::?p,
  action == Action::"save_workflow_definition",
  resource in TenantData::?r
)
when {
  resource has tenancy_policy_env_tier &&
  resource has env_tier &&
  resource.tenancy_policy_env_tier != resource.env_tier
};

// Promotions fail closed if the transition metadata is incomplete.
forbid (
  principal in TenantOperator::?p,
  action == Action::"promote_workflow_definition",
  resource in TenantData::?r
)
unless {
  resource has from_env_tier &&
  resource has to_env_tier &&
  resource has changeset_approval_ref &&
  resource has audit_chain_tag
};

// Promotions are forward-only: test -> staging -> prod.
forbid (
  principal in TenantOperator::?p,
  action == Action::"promote_workflow_definition",
  resource in TenantData::?r
)
when {
  resource has from_env_tier &&
  resource has to_env_tier &&
  !(
    (resource.from_env_tier == "test" && resource.to_env_tier == "staging") ||
    (resource.from_env_tier == "staging" && resource.to_env_tier == "prod")
  )
};

// Staging promotion with outbound-capable nodes requires test recipient evidence.
forbid (
  principal in TenantOperator::?p,
  action == Action::"promote_workflow_definition",
  resource in TenantData::?r
)
when {
  resource has to_env_tier &&
  resource.to_env_tier == "staging" &&
  resource has contains_outbound_capable_steps &&
  resource.contains_outbound_capable_steps == true &&
  !(resource has test_recipient_policy_ref)
};

// Prod promotion requires ChangeSet-style approval, live policy evidence, and explicit prod acks.
forbid (
  principal in TenantOperator::?p,
  action == Action::"promote_workflow_definition",
  resource in TenantData::?r
)
when {
  resource has to_env_tier &&
  resource.to_env_tier == "prod" &&
  (
    !(resource has changeset_approval_ref) ||
    !(resource has live_policy_evidence_ref) ||
    !(resource has prod_destructive_acknowledged) ||
    resource.prod_destructive_acknowledged != true ||
    !(resource has outbound_acknowledged) ||
    resource.outbound_acknowledged != true
  )
};
