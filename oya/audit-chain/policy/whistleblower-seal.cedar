// audit-chain µservice — whistleblower evidence seal policy fragment
// Cedar v4

forbid (principal, action, resource);

permit (
  principal in WorkloadEmitter::?w,
  action == Action::"audit_emit",
  resource in AuditEvent::?e
)
when {
  resource has event_class &&
  resource.event_class == "whistleblower.evidence.sealed" &&
  resource has subject_hash &&
  principal has whistleblower_seal_entitlement &&
  principal.whistleblower_seal_entitlement == true
};
