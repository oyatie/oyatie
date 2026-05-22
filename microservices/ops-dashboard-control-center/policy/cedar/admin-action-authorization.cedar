// ops-dashboard-control-center — admin-action-authorization policy fragment
// Cedar v4.2 LTS
//
// Purpose: gate every mutating admin action behind step-up auth + explicit permit.
//          This is the PRIMARY authorization fragment for all operator mutations.
//
// Owner:   ops-sre-reliability + council-security
// Binding ADR: ADR-0243 (Cedar universal gate), ADR-0247 (step-up auth), ADR-0263 (audit emission)
// References: ARCHITECTURE.md §cedar-gates, threat-model.md, compliance.md §insider-threat-controls
// Soak window: per ADR-0294 — ≥60s soak before activation of any new PERMIT.
//
// Principal slugs (per ADR-0242 + ARCHITECTURE.md §principals):
//   oyatie.ops.dashboard-operator
//   oyatie.platform-ops.admin-console
//   oyatie.ops.release-manager
//   oyatie.ops.compliance-operator
//   oyatie.ops.cedar-admin

// =============================================================================
// Default deny — MUST be first
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — T2 admin actions (TOTP/passkey step-up)
// =============================================================================
//
// Covers: incident-declare, on-call-handoff-create, deployment-freeze,
//         evidence-pack-export, policy-audit-review.
// Step-up class: STEP_UP_TOTP_OR_PASSKEY (4h session window).

permit (
  principal in OpsOperator::?p,
  action in [
    Action::"incident_declare",
    Action::"incident_severity_change",
    Action::"incident_remediation_approve",
    Action::"deployment_freeze",
    Action::"evidence_pack_export",
    Action::"policy_audit_review",
    Action::"oncall_handoff_create",
    Action::"oncall_handoff_acknowledge"
  ],
  resource
)
when {
  principal has step_up_auth_class &&
  principal.step_up_auth_class in ["STEP_UP_TOTP_OR_PASSKEY", "STEP_UP_HARDWARE_KEY", "STEP_UP_HARDWARE_KEY_QUORUM_2"] &&
  principal has step_up_auth_expires_at &&
  context has now &&
  context.now < principal.step_up_auth_expires_at &&
  principal has idempotency_key &&
  context has audit_emission_confirmed &&
  context.audit_emission_confirmed == true
};

// =============================================================================
// PERMIT 2 — T3 admin actions (hardware key step-up)
// =============================================================================
//
// Covers: deployment-approve, rollback-execute, tenant-isolation-policy-change,
//         admin-console mutations, ADR-promotion-triage-recommend.
// Step-up class: STEP_UP_HARDWARE_KEY (1h session window).

permit (
  principal in OpsAdmin::?p,
  action in [
    Action::"deployment_approve",
    Action::"rollback_execute",
    Action::"tenant_isolation_policy_change",
    Action::"admin_console_mutation",
    Action::"adr_promotion_triage_recommend"
  ],
  resource
)
when {
  principal has step_up_auth_class &&
  principal.step_up_auth_class in ["STEP_UP_HARDWARE_KEY", "STEP_UP_HARDWARE_KEY_QUORUM_2"] &&
  principal has step_up_auth_expires_at &&
  context has now &&
  context.now < principal.step_up_auth_expires_at &&
  principal has idempotency_key &&
  principal has rationale &&
  principal.rationale != "" &&
  context has audit_emission_confirmed &&
  context.audit_emission_confirmed == true
};

// =============================================================================
// PERMIT 3 — Break-glass (post-hoc audit-and-justify)
// =============================================================================
//
// Break-glass is ONLY permitted when an open incident ticket exists.
// Post-hoc justify: operator must submit rationale within 1h of break-glass use.
// Per ADR-0247 break-glass pattern: post-hoc audit, not pre-action friction.

permit (
  principal in OpsAdmin::?p,
  action == Action::"break_glass_access",
  resource
)
when {
  context has open_incident_ticket &&
  context.open_incident_ticket == true &&
  principal has step_up_auth_class &&
  principal.step_up_auth_class in ["STEP_UP_HARDWARE_KEY", "STEP_UP_HARDWARE_KEY_QUORUM_2"] &&
  context has audit_emission_confirmed &&
  context.audit_emission_confirmed == true
};

// =============================================================================
// FORBID — defence-in-depth: no action without idempotency key on mutations
// =============================================================================

forbid (
  principal,
  action in [
    Action::"deployment_approve",
    Action::"rollback_execute",
    Action::"incident_declare",
    Action::"incident_remediation_approve",
    Action::"oncall_handoff_create",
    Action::"break_glass_access",
    Action::"admin_console_mutation"
  ],
  resource
)
unless {
  principal has idempotency_key
};

// =============================================================================
// FORBID — no T3 action on expired step-up session
// =============================================================================

forbid (
  principal,
  action in [
    Action::"deployment_approve",
    Action::"rollback_execute",
    Action::"tenant_isolation_policy_change",
    Action::"admin_console_mutation"
  ],
  resource
)
when {
  !(principal has step_up_auth_class) ||
  !(principal has step_up_auth_expires_at) ||
  (context has now && context.now >= principal.step_up_auth_expires_at)
};

// =============================================================================
// FORBID — audit emission must be confirmed before action completes
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  action in [
    Action::"deployment_approve",
    Action::"rollback_execute",
    Action::"incident_declare",
    Action::"incident_remediation_approve",
    Action::"admin_console_mutation",
    Action::"break_glass_access"
  ] &&
  (!(context has audit_emission_confirmed) || context.audit_emission_confirmed == false)
};
