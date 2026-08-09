// ops-dashboard-control-center — auditor-scope policy fragment
// Cedar v4.2 LTS
//
// Purpose: external auditors (SOC 2 / ISO 27001 / HIPAA / KR-ISMS / GDPR DPA)
//          read time-boxed, tenant-scoped subsets of ops data when an engagement is active.
//          JIT tokens issued by OpenBao at engagement start.
//
// Owner:   ops-sre-reliability + council-compliance
// Binding ADR: ADR-0243 (Cedar universal gate), ADR-0244 (tenant scoping)
// References: ARCHITECTURE.md §cedar-gates, dpia.md §6
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
// PERMIT 1 — auditor reads tenant-scoped ops data within engagement window
// =============================================================================

permit (
  principal in AuditPrincipal::?a,
  action in [
    Action::"incident_read",
    Action::"deployment_history_read",
    Action::"evidence_pack_read",
    Action::"dashboard_view",
    Action::"tenant_isolation_posture_query",
    Action::"cluster_health_query",
    Action::"oncall_handoff_read",
    Action::"audit_chain_read",
    Action::"policy_audit_review"
  ],
  resource
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has tenant_id &&
  principal has scoped_tenants &&
  resource.tenant_id in principal.scoped_tenants
};

// =============================================================================
// PERMIT 2 — auditor reads policy artifacts (no per-tenant scoping needed)
// =============================================================================

permit (
  principal in AuditPrincipal::?a,
  action == Action::"read_artifact",
  resource in PolicyArtifact::?r
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has artifact_type &&
  resource.artifact_type in [
    "threat-model", "dpia", "compliance", "tenant-isolation",
    "data-residency", "policy-cedar", "runbook", "openslo-manifest",
    "adr", "prd", "architecture"
  ]
};

// =============================================================================
// FORBID — auditor cannot write anything
// =============================================================================

forbid (
  principal in AuditPrincipal::?a,
  action in [
    Action::"incident_declare",
    Action::"deployment_approve",
    Action::"rollback_execute",
    Action::"admin_console_mutation",
    Action::"pack_author_publish",
    Action::"cedar_fragment_publish",
    Action::"oncall_handoff_create"
  ],
  resource
);

// =============================================================================
// FORBID — auditor outside engagement window
// =============================================================================

forbid (
  principal in AuditPrincipal::?a,
  action,
  resource
)
unless {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to
};

// =============================================================================
// FORBID — auditor cannot pivot beyond scoped_tenants
// =============================================================================

forbid (
  principal in AuditPrincipal::?a,
  action in [
    Action::"incident_read",
    Action::"deployment_history_read",
    Action::"evidence_pack_read",
    Action::"tenant_isolation_posture_query"
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has scoped_tenants &&
  !(resource.tenant_id in principal.scoped_tenants)
};

// =============================================================================
// FORBID — auditor cannot access SECRET-class data
// =============================================================================

forbid (
  principal in AuditPrincipal::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class in ["SECRET", "AUTHENTICATION"]
};
