// foundry-providers µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/intelligence/policy/schema.cedarschema
//
// Purpose: enforce per-tenant scope on provider-router invocations.
// Owner:   ops-security + axis-foundry
// Audit:   every deny emits oya_foundry_providers_tenant_unauthorized_invoke_total
// References: threat-model.md T-10 (cross-tenant authz bypass); credential-isolation.md CI-INV-05
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every router invocation is refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — workload µservice principal invokes router for own tenant
// =============================================================================
//
// A workload µservice principal bound to SPIFFE identity tenant-X may invoke
// the router only with `resource.tenant_id == X`. Cross-tenant invocation is
// forbidden by the default-deny + an explicit defence-in-depth forbid below.

permit (
  principal in WorkloadPrincipal::?p,
  action in [
    Action::"router_invoke",
    Action::"router_decide",
    Action::"router_get_decision",
    Action::"provider_list_capabilities"
  ],
  resource in ProviderInvocation::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has spiffe_id &&
  principal.spiffe_id like "spiffe://oyatie.dev/*/tenant/*"
};

// =============================================================================
// PERMIT 2 — tenant operator reads own invocation history
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"router_get_decision",
    Action::"router_read_history",
    Action::"provider_read_health"
  ],
  resource in ProviderInvocation::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 3 — tenant operator updates own provider configuration
// =============================================================================
//
// Tenant operators may update which vendors their workload uses (subject to
// the per-pack permitted-vendor matrix in policy/data-residency.md).

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"provider_config_read",
    Action::"provider_config_update",
    Action::"capability_profile_update"
  ],
  resource in ProviderConfig::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has pack &&
  principal has pack &&
  principal.pack == resource.pack
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"router_invoke",
    Action::"router_decide",
    Action::"router_get_decision",
    Action::"router_read_history",
    Action::"provider_config_read",
    Action::"provider_config_update",
    Action::"capability_profile_update",
    Action::"provider_read_health"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — cross-pack invocation requires DPO-recorded SCC entitlement
// =============================================================================
//
// Per data-residency.md: pack-X tenants cannot invoke a vendor edge in pack-Y
// unless they hold an explicit SCC entitlement recorded at onboarding.

forbid (
  principal,
  action == Action::"router_invoke",
  resource in ProviderInvocation::?r
)
when {
  resource has source_pack &&
  resource has target_vendor_pack &&
  resource.source_pack != resource.target_vendor_pack &&
  !(principal has scc_entitlements &&
    resource.target_vendor_pack in principal.scc_entitlements)
};

// =============================================================================
// FORBID — never invoke a forbidden (pack, vendor, transport) tuple
// =============================================================================

forbid (
  principal,
  action == Action::"router_invoke",
  resource in ProviderInvocation::?r
)
when {
  resource has residency_compliant &&
  resource.residency_compliant == false
};

// =============================================================================
// FORBID — subscription transports for PHI tenants
// =============================================================================
//
// Per data-residency.md pack-us-healthcare: subscription channels do not
// support BAA; only API transports may be used for PHI workloads.

forbid (
  principal,
  action == Action::"router_invoke",
  resource in ProviderInvocation::?r
)
when {
  resource has data_class &&
  resource.data_class == "PHI" &&
  resource has transport &&
  resource.transport == "subscription"
};
