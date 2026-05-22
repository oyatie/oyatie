// ops-dashboard-control-center — audit-emission-required policy fragment
// Cedar v4.2 LTS
//
// Purpose: enforce that every admin action produces a confirmed audit event
//          per ADR-0263 before the action is considered complete. This is the
//          "audit-or-deny" baseline — if audit emission fails, the action MUST
//          be denied (not silently completed without a trail).
//
// Owner:   ops-sre-reliability + council-security
// Binding ADR: ADR-0263 (observability emission contract), ADR-0028 (audit chain)
// References: ARCHITECTURE.md §observability, compliance.md §insider-threat-controls
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
// PERMIT — read-only actions (audit emitted asynchronously, not blocking)
// =============================================================================

permit (
  principal,
  action in [
    Action::"cluster_health_query",
    Action::"tenant_isolation_posture_query",
    Action::"incident_read",
    Action::"deployment_history_read",
    Action::"evidence_pack_read",
    Action::"dashboard_view",
    Action::"oncall_handoff_read",
    Action::"adr_queue_read",
    Action::"finops_panel_read",
    Action::"observability_pivot_read"
  ],
  resource
)
when {
  principal has authenticated &&
  principal.authenticated == true
};

// =============================================================================
// PERMIT — mutating actions require synchronous audit emission confirmation
// =============================================================================
//
// The `audit_emission_confirmed` context key is set by the audit middleware
// AFTER the audit event has been durably written to the Kafka outbox.
// If the outbox write fails, `audit_emission_confirmed` is false → FORBID fires.

permit (
  principal,
  action in [
    Action::"incident_declare",
    Action::"incident_severity_change",
    Action::"incident_remediation_approve",
    Action::"deployment_approve",
    Action::"deployment_freeze",
    Action::"rollback_execute",
    Action::"evidence_pack_export",
    Action::"oncall_handoff_create",
    Action::"oncall_handoff_acknowledge",
    Action::"admin_console_mutation",
    Action::"break_glass_access",
    Action::"pack_author_publish",
    Action::"cedar_fragment_publish",
    Action::"tenant_isolation_policy_change",
    Action::"adr_promotion_triage_recommend"
  ],
  resource
)
when {
  context has audit_emission_confirmed &&
  context.audit_emission_confirmed == true &&
  context has audit_event_class &&
  context.audit_event_class != "" &&
  context has audit_seal_ref &&
  context.audit_seal_ref != ""
};

// =============================================================================
// FORBID — mutation without confirmed audit emission (defence-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"incident_declare",
    Action::"incident_severity_change",
    Action::"incident_remediation_approve",
    Action::"deployment_approve",
    Action::"deployment_freeze",
    Action::"rollback_execute",
    Action::"evidence_pack_export",
    Action::"oncall_handoff_create",
    Action::"admin_console_mutation",
    Action::"break_glass_access",
    Action::"cedar_fragment_publish"
  ],
  resource
)
when {
  !(context has audit_emission_confirmed) ||
  context.audit_emission_confirmed == false
};

// =============================================================================
// FORBID — audit emit cannot be bypassed by any principal class
// =============================================================================
//
// Anti-pattern to prevent: operator claims EMERGENCY_SERVICES to skip audit.
// Emergency bypass never skips audit — it only skips challenge (per §3.2.3).

forbid (
  principal,
  action == Action::"bypass_audit_emission",
  resource
);
