// audit-chain µservice — data residency enforcement policy fragment
// Cedar v4
//
// Purpose: refuse emit/query/export requests whose resource pack does not match
// the principal's pack binding. Complements policy/data-residency.md and
// policy/tenant-scope.cedar.
// Owner: ops-security + council-privacy + axis-audit-chain

forbid (
  principal,
  action in [Action::"audit_emit", Action::"audit_query", Action::"audit_export_request"],
  resource
)
unless {
  principal has bound_pack &&
  resource has pack &&
  principal.bound_pack == resource.pack
};

permit (
  principal in WorkloadEmitter::?w,
  action == Action::"audit_emit",
  resource in AuditEvent::?e
)
when {
  principal has bound_pack &&
  resource has pack &&
  principal.bound_pack == resource.pack &&
  principal has bound_tenant_id &&
  resource has tenant_id &&
  principal.bound_tenant_id == resource.tenant_id
};
