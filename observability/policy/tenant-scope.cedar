// observability µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/observability/policy/schema.cedarschema (Slice D)
//
// Purpose: enforce per-tenant read scope on metrics/logs/traces/profiles served via
//          slo-engine-rest, Grafana, and SDK clients.
// Owner:   ops-security + axis-observability
// Audit:   every deny emits tenant_unauthorized_read_attempt_total
// References: tenant-isolation.md §"Invariant TI-07"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every read is refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant operator reads own tenant's data
// =============================================================================
//
// A principal that is bound to TenantOperator::"<tenant-id>" may read any
// resource that is owned by the same tenant. Read scope is gated on the
// tenant_id claim of the principal matching the resource's tenant_id label.

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_metrics",
    Action::"read_logs",
    Action::"read_traces",
    Action::"read_profiles",
    Action::"read_openslo_manifests",
    Action::"read_eligibility_verdicts",
    Action::"read_dashboards",
    Action::"read_alerts"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — tenant operator writes own OpenSLO manifests via PR (read-only here)
// =============================================================================
//
// OpenSLO authoring happens via git PR (covered by Jenkins/GitHub required checks);
// at the policy layer, tenant operators only have read access to their manifests.
// Slice D's docs/standards/observability-slo.md governs the write path.

permit (
  principal in TenantOperator::?p,
  action == Action::"read_openslo_manifests",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has data_class &&
  resource.data_class == "INTERNAL_ONLY"
};

// =============================================================================
// PERMIT 3 — tenant operator with DPA-authorised sensitive-data scope
// =============================================================================
//
// pack-us-healthcare PHI reads require explicit BAA + Cedar entitlement.
// pack-kr sensitive-data reads (PIPA Art. 23) require explicit consent
// recorded in OpenBao at onboarding.

permit (
  principal in TenantOperator::?p,
  action in [Action::"read_traces", Action::"read_logs"],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has sensitive_data_entitlements &&
  resource has data_class &&
  resource.data_class in principal.sensitive_data_entitlements
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth)
// =============================================================================
//
// Even if a permit somehow accidentally matched a cross-tenant read,
// this forbid blocks it. Cedar's deny-overrides semantics make this
// the safety net.

forbid (
  principal,
  action in [
    Action::"read_metrics",
    Action::"read_logs",
    Action::"read_traces",
    Action::"read_profiles",
    Action::"read_openslo_manifests",
    Action::"read_eligibility_verdicts",
    Action::"read_dashboards",
    Action::"read_alerts"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
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
