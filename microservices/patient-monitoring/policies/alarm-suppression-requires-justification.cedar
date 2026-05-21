// alarm-suppression-requires-justification.cedar
// Per ADR-MS-002: alarm suppression requires non-empty justification + duration ≤ 4h.
// Per ADR-0332: Cedar fails CLOSED for suppress operations.

permit (
    principal in oya::patient_monitoring::Role::"physician_attending",
    action == oya::patient_monitoring::Action::"suppress_alarm",
    resource
)
when {
    principal.tenant_id == resource.tenant_id
    && resource.bed_id in principal.authority_bed_group
    && context has justification_text
    && context.justification_text.length >= 16
    && context.justification_text.length <= 1024
    && context has duration_minutes
    && context.duration_minutes >= 1
    && context.duration_minutes <= 240
    && principal.authority_paused == false
    // Forbid suppression of life-threatening alarms entirely
    && resource.severity != "life-threatening"
};

// Forbid suppression of life-threatening (defense in depth)
forbid (
    principal,
    action == oya::patient_monitoring::Action::"suppress_alarm",
    resource
) when {
    resource.severity == "life-threatening"
};

// Forbid suppression by non-physician principals
forbid (
    principal,
    action == oya::patient_monitoring::Action::"suppress_alarm",
    resource
) unless {
    principal in oya::patient_monitoring::Role::"physician_attending"
};
