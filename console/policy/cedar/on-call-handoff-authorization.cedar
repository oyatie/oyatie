// ops-dashboard-control-center — on-call-handoff-authorization policy fragment
// Cedar v4.2 LTS
//
// Purpose: gate on-call handoff creation, acknowledgement, and escalation.
//          On-call handoff is T2 (TOTP/passkey step-up). Escalation to incident
//          command requires T3 (hardware key).
//          EMERGENCY_SERVICES page: bypass all friction; handoff created automatically.
//
// Owner:   ops-sre-reliability
// Binding ADR: ADR-0243, ADR-0263 (audit emission), ADR-0298 (emergency bypass)
// References: ARCHITECTURE.md §cedar-gates, runbooks/oncall-handoff-failure.md
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
// PERMIT — on-call handoff read (no step-up required)
// =============================================================================

permit (
  principal in OpsOperator::?p,
  action in [
    Action::"oncall_handoff_read",
    Action::"oncall_rotation_read",
    Action::"oncall_schedule_read"
  ],
  resource
)
when {
  principal has authenticated &&
  principal.authenticated == true
};

// =============================================================================
// PERMIT — on-call handoff create + ack (T2 step-up)
// =============================================================================

permit (
  principal in OpsOperator::?p,
  action in [
    Action::"oncall_handoff_create",
    Action::"oncall_handoff_acknowledge",
    Action::"oncall_handoff_update"
  ],
  resource
)
when {
  principal has step_up_class &&
  principal.step_up_class in ["STEP_UP_TOTP_OR_PASSKEY", "STEP_UP_HARDWARE_KEY", "STEP_UP_HARDWARE_KEY_QUORUM_2"] &&
  principal has step_up_completed_at &&
  context has now &&
  (context.now - principal.step_up_completed_at) < 14400 &&
  principal has idempotency_key &&
  context has audit_emission_confirmed &&
  context.audit_emission_confirmed == true
};

// =============================================================================
// PERMIT — EMERGENCY_SERVICES on-call page (no step-up, no friction)
// =============================================================================
//
// On-call paged for emergency: handoff created automatically on EMERGENCY_SERVICES signal.
// Audit emitted as AbuseDefenceEmergencyServiceBypass + OnCallHandoffCreated.

permit (
  principal,
  action in [
    Action::"oncall_handoff_create",
    Action::"oncall_emergency_page"
  ],
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES" &&
  context has audit_emission_confirmed &&
  context.audit_emission_confirmed == true
};

// =============================================================================
// PERMIT — escalate handoff to incident (T3 hardware key)
// =============================================================================

permit (
  principal in OpsAdmin::?p,
  action == Action::"oncall_handoff_escalate_to_incident",
  resource
)
when {
  principal has step_up_class &&
  principal.step_up_class in ["STEP_UP_HARDWARE_KEY", "STEP_UP_HARDWARE_KEY_QUORUM_2"] &&
  principal has step_up_completed_at &&
  context has now &&
  (context.now - principal.step_up_completed_at) < 3600 &&
  context has audit_emission_confirmed &&
  context.audit_emission_confirmed == true
};

// =============================================================================
// FORBID — handoff without idempotency key
// =============================================================================

forbid (
  principal,
  action in [
    Action::"oncall_handoff_create",
    Action::"oncall_handoff_acknowledge"
  ],
  resource
)
unless {
  principal has idempotency_key ||
  (principal has audience_type && principal.audience_type == "EMERGENCY_SERVICES")
};
