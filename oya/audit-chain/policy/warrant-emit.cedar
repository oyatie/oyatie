// audit-chain µservice — warrant query emission policy fragment
// Cedar v4

forbid (principal, action, resource);

permit (
  principal in WorkloadEmitter::?w,
  action == Action::"audit_emit",
  resource in AuditEvent::?e
)
when {
  resource has event_class &&
  resource.event_class == "warrant.query.emitted" &&
  principal has warrant_emission_entitlement &&
  principal.warrant_emission_entitlement == true
};
