// audit-chain µservice — dual-tenant emission policy fragment
// Cedar v4
//
// Purpose: allow one canonical audit leaf to project to two tenants when both
// tenant identifiers are present and the principal is explicitly entitled to
// emit a dual-tenant event class.

forbid (principal, action, resource);

permit (
  principal in WorkloadEmitter::?w,
  action == Action::"audit_emit_dual_tenant",
  resource in AuditEvent::?e
)
when {
  principal has allowed_event_classes &&
  resource has event_class &&
  resource.event_class in principal.allowed_event_classes &&
  resource has tenant_ids &&
  resource.tenant_ids_count == 2 &&
  resource has pack &&
  principal has bound_pack &&
  resource.pack == principal.bound_pack
};
