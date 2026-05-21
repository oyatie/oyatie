// ops-dashboard-control-center — tenant-scope-enforcement policy fragment
// Cedar v4.2 LTS
//
// Purpose: enforce that every resource access is scoped to the principal's
//          authorized tenant list. Prevents cross-tenant pivot by operators.
//
// Owner:   ops-sre-reliability + council-privacy
// Binding ADR: ADR-0244 (tenant scoping), ADR-0243 (Cedar universal gate)
// References: ARCHITECTURE.md §tenant-scoping, tenant-isolation.md
// Soak window: ≥60s per ADR-0294

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — internal operators (see all tenants, Cedar-scoped per action)
// =============================================================================

permit (
  principal in InternalOpsOperator::?p,
  action,
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "INTERNAL_OPS" &&
  principal has authenticated &&
  principal.authenticated == true
};

// =============================================================================
// PERMIT — partner-agency operators (sub-tenant scope only)
// =============================================================================

permit (
  principal in PartnerAgencyOperator::?p,
  action in [
    Action::"cluster_health_query",
    Action::"tenant_isolation_posture_query",
    Action::"incident_read",
    Action::"dashboard_view"
  ],
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "PARTNER_AGENCY_OPS" &&
  principal has scoped_tenants &&
  resource has tenant_id &&
  resource.tenant_id in principal.scoped_tenants
};

// =============================================================================
// PERMIT — auditors (JIT-scoped, read-only)
// =============================================================================

permit (
  principal in AuditPrincipal::?p,
  action in [
    Action::"incident_read",
    Action::"deployment_history_read",
    Action::"evidence_pack_read",
    Action::"dashboard_view",
    Action::"tenant_isolation_posture_query"
  ],
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "AUDITOR" &&
  principal has scoped_tenants &&
  resource has tenant_id &&
  resource.tenant_id in principal.scoped_tenants &&
  principal has engagement_valid_to &&
  context has now &&
  context.now <= principal.engagement_valid_to
};

// =============================================================================
// FORBID — cross-tenant pivot: partner-agency cannot read outside scope
// =============================================================================

forbid (
  principal in PartnerAgencyOperator::?p,
  action,
  resource
)
when {
  resource has tenant_id &&
  principal has scoped_tenants &&
  !(resource.tenant_id in principal.scoped_tenants)
};

// =============================================================================
// FORBID — auditor cannot pivot outside engagement scope
// =============================================================================

forbid (
  principal in AuditPrincipal::?p,
  action,
  resource
)
when {
  resource has tenant_id &&
  principal has scoped_tenants &&
  !(resource.tenant_id in principal.scoped_tenants)
};

// =============================================================================
// FORBID — no resource without tenant_id on scoped actions
// =============================================================================

forbid (
  principal,
  action in [
    Action::"tenant_isolation_posture_query",
    Action::"incident_read",
    Action::"deployment_history_read",
    Action::"evidence_pack_read",
    Action::"evidence_pack_export"
  ],
  resource
)
unless {
  resource has tenant_id
};
