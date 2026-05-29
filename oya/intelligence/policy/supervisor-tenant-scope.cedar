// foundry-supervisor µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/intelligence/policy/schema.cedarschema (Slice D)
//
// Purpose: enforce per-tenant read + write scope on fleet state, capability
//          definitions, autonomy entitlements, kill-switch state, deployment
//          history served via foundry-supervisor-rest.
// Owner:   ops-security + axis-foundry-control-plane
// Audit:   every deny emits oya_supervisor_tenant_unauthorized_attempt_total
// References: supervisor-isolation.md §"Cedar Per-Tenant Read Authorisation"
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
// PERMIT 1 — tenant operator reads own fleet-state + deployment history
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_fleet_state",
    Action::"read_deployment_history",
    Action::"read_capability_definition",
    Action::"read_kill_switch_state",
    Action::"read_autonomy_entitlement",
    Action::"read_supervision_event",
    Action::"read_dashboards"
  ],
  resource in TenantResource::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — tenant operator engages own-scope kill-switch
// =============================================================================
//
// Tenant DPO may engage / disengage the kill-switch on resources in their own
// tenant scope (single capability or whole tenant scope; never fleet-wide).
// 2-person rule for fleet-wide is enforced separately and is NOT permitted via
// this policy.

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"engage_kill_switch",
    Action::"disengage_kill_switch"
  ],
  resource in TenantResource::?r
)
when {
  principal has tenant_id &&
  principal has role &&
  principal.role == "dpo" &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has scope &&
  resource.scope in ["tenant", "capability", "agent"]
  // resource.scope == "fleet" is NEVER permitted here; ops-security 2-person path only
};

// =============================================================================
// PERMIT 3 — tenant operator drains own fleet (operational maintenance)
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action == Action::"drain_fleet",
  resource in TenantResource::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has role &&
  principal.role in ["dpo", "ops"]
};

// =============================================================================
// PERMIT 4 — autonomy precondition (per-invocation; foundry-runtime calling)
// =============================================================================
//
// foundry-runtime requests "may capability C invoke at tier T for tenant X?"
// This is allowed when:
//   - the principal is the foundry-runtime SPIFFE identity,
//   - the tenant has an entitlement for the requested autonomy level,
//   - no kill-switch is engaged on any matching scope,
//   - the requested autonomy level ≤ the tenant's autonomy ceiling ceiling per DPA.

permit (
  principal == Service::"foundry-runtime",
  action == Action::"evaluate_autonomy_precondition",
  resource in InvocationContext::?i
)
when {
  resource has tenant_id &&
  resource has capability_id &&
  resource has requested_autonomy_level &&
  resource has tenant_autonomy_ceiling &&
  resource has kill_switch_engaged &&
  resource.requested_autonomy_level <= resource.tenant_autonomy_ceiling &&
  resource.kill_switch_engaged == false &&
  principal has tenant_class &&
  principal.tenant_class == "paid" &&
  principal has billing_components &&
  principal.billing_components contains "per_seat"
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_fleet_state",
    Action::"read_deployment_history",
    Action::"read_capability_definition",
    Action::"read_kill_switch_state",
    Action::"read_autonomy_entitlement",
    Action::"read_supervision_event",
    Action::"engage_kill_switch",
    Action::"disengage_kill_switch",
    Action::"drain_fleet"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — tenant operator cannot engage fleet-wide kill-switch
// =============================================================================
//
// Fleet-wide is reserved for ops-security with 2-person rule. Tenant operators
// are explicitly excluded here as defence-in-depth even if a misconfigured
// permit somehow matched.

forbid (
  principal in TenantOperator::?p,
  action == Action::"engage_kill_switch",
  resource
)
when {
  resource has scope &&
  resource.scope == "fleet"
};

// =============================================================================
// FORBID — never expose SECRET-class data via tenant reads
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — tenant cannot mutate own autonomy entitlement
// =============================================================================
//
// Tier escalation requires DPO + ops-security signature; tenant cannot
// self-escalate. Read is permitted (PERMIT 1); write is denied.

forbid (
  principal in TenantOperator::?p,
  action in [
    Action::"write_autonomy_entitlement",
    Action::"escalate_autonomy_level"
  ],
  resource
);
