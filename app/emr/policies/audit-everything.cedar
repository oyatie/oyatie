// Cedar policy — audit.everything
// Owning µservice: emr
// Authored Wave 15M-B 2026-05-21
// Per ADR-0243 + ADR-0251 + ADR-0263

// Every permit decision is conditional on audit-chain availability + emission
// commitment. If the audit-chain is unreachable or the emission has not been
// armed, deny all PHI-touching access.

forbid (
    principal,
    action,
    resource
)
when {
    // Audit chain must be healthy in this cell
    context.audit_chain_health != "healthy" ||

    // Tenant audit stream must be configured
    context.audit_stream_configured == false ||

    // Emission idempotency key missing → unsafe
    context.audit_idempotency_key_present == false
}
unless {
    // Exception: read-only health-check endpoints + service-mesh probes
    action in [
        Action::"emr.health.check",
        Action::"emr.metrics.read",
        Action::"emr.ready.check"
    ]
};

// All write-actions MUST carry the audit envelope
forbid (
    principal,
    action,
    resource
)
when {
    action in [
        Action::"emr.patient.create",
        Action::"emr.patient.update",
        Action::"emr.patient.merge",
        Action::"emr.encounter.start",
        Action::"emr.encounter.discharge",
        Action::"emr.problem.add",
        Action::"emr.problem.amend",
        Action::"emr.medication.prescribe",
        Action::"emr.allergy.record",
        Action::"emr.vital.record",
        Action::"emr.note.sign",
        Action::"emr.note.amend",
        Action::"emr.order.enter",
        Action::"emr.order.cancel",
        Action::"emr.break-glass.invoke",
        Action::"emr.billing-code.capture"
    ] &&
    context.audit_envelope_complete == false
};
