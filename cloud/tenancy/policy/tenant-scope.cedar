// tenancy µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/tenancy/policy/schema.cedarschema (Slice D)
//
// Purpose: enforce per-tenant CRUD scope on tenant lifecycle + DSR + cell-assignment +
//          RLS-policy reads served via tenant-lifecycle-rest, dsr-cascade-rest, SDK clients.
// Owner:   ops-security + axis-tenancy
// Audit:   every deny emits oya_tenancy_cedar_deny_total
// References: rls-isolation.md §"Cedar Policy Enforcement"; threat-model.md T-S-02, T-E-03
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every action is refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant operator reads own tenant metadata
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_tenant",
    Action::"read_tenant_status",
    Action::"read_tenant_lifecycle_history",
    Action::"read_cell_assignment",
    Action::"read_jurisdiction_code",
    Action::"read_plan_tier"
  ],
  resource in Tenant::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — tenant operator suspends / resumes OWN tenant
// =============================================================================
//
// Suspend is a non-destructive operational state change; tenant operator can
// initiate but resume requires same operator (or platform-operator override).
// Delete is explicitly NOT permitted here — see platform-scope.cedar fragment.

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"suspend_tenant",
    Action::"resume_tenant"
  ],
  resource in Tenant::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has plan_tier_role &&
  principal.plan_tier_role in ["admin", "owner"]
};

// =============================================================================
// PERMIT 3 — tenant operator submits own-tenant DSR request
// =============================================================================
//
// DSR submission by tenant operator on behalf of tenant's end-user (Art. 26
// joint controllership). Tenant operator can read own DSR cascade history but
// not other tenants' DSRs.

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"submit_dsr_request",
    Action::"read_dsr_request",
    Action::"read_proof_of_erasure"
  ],
  resource in DsrRequest::?d
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 4 — tenant operator reads own environment-tier contract view
// =============================================================================
//
// ADR-0163 readiness fragment: tenant DPO/operator self-service can inspect the
// tenant's test/staging/prod configuration. This is read-only; it does not
// create environments, route traffic, copy data, or bypass the policy-engine
// substrate.

permit (
  principal in TenantOperator::?p,
  action == Action::"read_tenant_environments",
  resource in TenantEnvironment::?e
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has env_tier &&
  resource.env_tier in ["test", "staging", "prod"]
};

// =============================================================================
// PERMIT 5 — ADR-0163 API-key issuance role ladder by environment tier
// =============================================================================
//
// Prefix/tier gates: sk_test_/pk_test_ require tenant developer or higher;
// sk_stage_/pk_stage_ require tenant maintainer or higher; sk_live_/pk_live_
// require tenant admin only. Runtime issuance must still flow through the
// shared policy engine; this fragment only declares the Cedar contract shape.

permit (
  principal in TenantOperator::?p,
  action == Action::"issue_api_key",
  resource in TenantEnvironment::?e
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has plan_tier_role &&
  resource has env_tier &&
  resource has api_key_prefix &&
  (
    (
      resource.env_tier == "test" &&
      resource.api_key_prefix in ["sk_test_", "pk_test_"] &&
      principal.plan_tier_role in ["developer", "maintainer", "admin", "owner"]
    ) ||
    (
      resource.env_tier == "staging" &&
      resource.api_key_prefix in ["sk_stage_", "pk_stage_"] &&
      principal.plan_tier_role in ["maintainer", "admin", "owner"]
    ) ||
    (
      resource.env_tier == "prod" &&
      resource.api_key_prefix in ["sk_live_", "pk_live_"] &&
      principal.plan_tier_role == "admin"
    )
  )
};

// =============================================================================
// PERMIT 6 — non-prod outbound configuration update
// =============================================================================
//
// test remains intercept/log-only; staging can deliver to QA/test recipients;
// prod live outbound delivery is not mutable through this API surface.

permit (
  principal in TenantOperator::?p,
  action == Action::"update_outbound_config",
  resource in TenantEnvironment::?e
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has plan_tier_role &&
  principal.plan_tier_role in ["admin", "owner"] &&
  resource has env_tier &&
  resource.env_tier in ["test", "staging"] &&
  resource has outbound_mode &&
  resource.outbound_mode in ["intercept", "test_recipients"]
};

forbid (
  principal in TenantOperator::?p,
  action == Action::"update_outbound_config",
  resource in TenantEnvironment::?e
)
when {
  resource has env_tier &&
  resource.env_tier == "prod"
};

// =============================================================================
// PERMIT 7 — prod destructive operation acknowledgment
// =============================================================================
//
// API boundaries map request header `x-oya-prod-destructive-ack: true` into the
// Cedar context flag `prod_destructive_acknowledged: true` before prod-tier DSR
// delete, tenant offboarding, bulk delete, cell migration, or residency-class
// change operations can proceed.

permit (
  principal in TenantOperator::?p,
  action == Action::"perform_prod_destructive_operation",
  resource in TenantDestructiveOperation::?d
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has plan_tier_role &&
  principal.plan_tier_role in ["admin", "owner"] &&
  resource has env_tier &&
  resource.env_tier == "prod" &&
  context has prod_destructive_acknowledged &&
  context.prod_destructive_acknowledged == true
};

forbid (
  principal,
  action == Action::"perform_prod_destructive_operation",
  resource in TenantDestructiveOperation::?d
)
when {
  resource has env_tier &&
  resource.env_tier == "prod"
}
unless {
  context has prod_destructive_acknowledged &&
  context.prod_destructive_acknowledged == true
};

// =============================================================================
// PERMIT 8 — tenant operator reads OWN RLS policy registry (read-only)
// =============================================================================
//
// Tenant operators may want to verify that their tables are RLS-protected.
// Read-only on the RLS YAML manifests for tables in own-tenant scope.
// RLS-policy mutation is platform-operator + git-PR only.

permit (
  principal in TenantOperator::?p,
  action == Action::"read_rls_policy",
  resource in RlsPolicy::?r
)
when {
  principal has tenant_id &&
  resource has scope &&
  resource.scope == "per-tenant" &&
  resource has tenant_id &&
  resource.tenant_id == principal.tenant_id
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth)
// =============================================================================
//
// Even if a permit somehow accidentally matched a cross-tenant action,
// this forbid blocks it. Cedar's deny-overrides semantics make this
// the safety net. NOTE: this complements but does NOT replace the Postgres
// RLS load-bearing primitive — see policy/rls-isolation.md.

forbid (
  principal in TenantOperator::?p,
  action in [
    Action::"read_tenant",
    Action::"read_tenant_status",
    Action::"read_tenant_lifecycle_history",
    Action::"read_cell_assignment",
    Action::"read_jurisdiction_code",
    Action::"read_plan_tier",
    Action::"suspend_tenant",
    Action::"resume_tenant",
    Action::"submit_dsr_request",
    Action::"read_dsr_request",
    Action::"read_proof_of_erasure",
    Action::"read_tenant_environments",
    Action::"issue_api_key",
    Action::"update_outbound_config",
    Action::"perform_prod_destructive_operation",
    Action::"read_rls_policy"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — tenant operator cannot create / delete tenants
// =============================================================================
//
// Create + delete are reserved to platform-operator + 2-person rule + JIT
// elevation via OpenBao. Tenant operators escalate via support, never via API.

forbid (
  principal in TenantOperator::?p,
  action in [
    Action::"create_tenant",
    Action::"delete_tenant",
    Action::"hard_delete_tenant",
    Action::"change_jurisdiction",
    Action::"change_pack"
  ],
  resource
);

// =============================================================================
// FORBID — tenant operator cannot read SECRET-class data
// =============================================================================
//
// JWT signing keys, Postgres replication password, etc. are SECRET-class
// and never exposed via tenant operator paths.

forbid (
  principal in TenantOperator::?p,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — tenant operator cannot mutate RLS policies
// =============================================================================
//
// RLS policy mutation is via git PR (CODEOWNERS: axis-tenancy + ops-security)
// + LEAN check + audit-chain seal. No API mutation path exists; this is the
// belt-and-suspenders.

forbid (
  principal in TenantOperator::?p,
  action in [
    Action::"create_rls_policy",
    Action::"alter_rls_policy",
    Action::"drop_rls_policy",
    Action::"disable_rls"
  ],
  resource
);

// =============================================================================
// FORBID — tenant operator cannot impersonate platform / DBA / CI principals
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action in [
    Action::"assume_role",
    Action::"elevate_privilege",
    Action::"impersonate"
  ],
  resource
);
