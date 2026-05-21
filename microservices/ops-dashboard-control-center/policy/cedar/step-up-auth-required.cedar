// ops-dashboard-control-center — step-up-auth-required policy fragment
// Cedar v4.2 LTS
//
// Purpose: enforce step-up authentication classes for all mutation actions.
//          This fragment is evaluated BEFORE the action-authorization fragment
//          to gate the step-up challenge at the auth middleware layer.
//
// Owner:   ops-sre-reliability + council-security
// Binding ADR: ADR-0243, ADR-0253 (TLS), docs/standards/step-up-auth-classes.md
// References: ARCHITECTURE.md §cedar-gates, compliance.md §insider-threat-controls
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
// PERMIT — read actions require no step-up
// =============================================================================

permit (
  principal in OpsOperator::?p,
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
  principal.authenticated == true &&
  principal has tenant_scope &&
  resource has tenant_id &&
  resource.tenant_id in principal.tenant_scope
};

// =============================================================================
// PERMIT — T2 mutations require TOTP/passkey step-up
// =============================================================================

permit (
  principal in OpsOperator::?p,
  action in [
    Action::"incident_declare",
    Action::"incident_severity_change",
    Action::"incident_remediation_approve",
    Action::"deployment_freeze",
    Action::"evidence_pack_export",
    Action::"oncall_handoff_create",
    Action::"oncall_handoff_acknowledge"
  ],
  resource
)
when {
  principal has step_up_class &&
  principal.step_up_class in ["STEP_UP_TOTP_OR_PASSKEY", "STEP_UP_HARDWARE_KEY", "STEP_UP_HARDWARE_KEY_QUORUM_2"] &&
  principal has step_up_completed_at &&
  context has now &&
  // 4h window for TOTP/passkey step-up
  (context.now - principal.step_up_completed_at) < 14400
};

// =============================================================================
// PERMIT — T3 mutations require hardware key step-up (1h window)
// =============================================================================

permit (
  principal in OpsAdmin::?p,
  action in [
    Action::"deployment_approve",
    Action::"rollback_execute",
    Action::"tenant_isolation_policy_change",
    Action::"admin_console_mutation",
    Action::"pack_author_publish",
    Action::"adr_promotion_triage_recommend"
  ],
  resource
)
when {
  principal has step_up_class &&
  principal.step_up_class in ["STEP_UP_HARDWARE_KEY", "STEP_UP_HARDWARE_KEY_QUORUM_2"] &&
  principal has step_up_completed_at &&
  context has now &&
  // 1h window for hardware key step-up
  (context.now - principal.step_up_completed_at) < 3600
};

// =============================================================================
// PERMIT — Cedar fragment publish requires quorum-2 hardware key (30min window)
// =============================================================================

permit (
  principal in CedarAdmin::?p,
  action == Action::"cedar_fragment_publish",
  resource
)
when {
  principal has step_up_class &&
  principal.step_up_class == "STEP_UP_HARDWARE_KEY_QUORUM_2" &&
  principal has step_up_quorum_count &&
  principal.step_up_quorum_count >= 2 &&
  principal has step_up_completed_at &&
  context has now &&
  // 30min window for quorum step-up
  (context.now - principal.step_up_completed_at) < 1800 &&
  resource has soak_elapsed_seconds &&
  resource.soak_elapsed_seconds >= 60
};

// =============================================================================
// FORBID — step-up expired or missing on any mutation
// =============================================================================

forbid (
  principal,
  action in [
    Action::"incident_declare",
    Action::"incident_remediation_approve",
    Action::"deployment_approve",
    Action::"rollback_execute",
    Action::"deployment_freeze",
    Action::"tenant_isolation_policy_change",
    Action::"admin_console_mutation",
    Action::"pack_author_publish",
    Action::"cedar_fragment_publish",
    Action::"oncall_handoff_create",
    Action::"evidence_pack_export"
  ],
  resource
)
unless {
  principal has step_up_class &&
  principal has step_up_completed_at
};
