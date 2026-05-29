// audit-chain µservice — region-local personal-information read policy fragment
// Cedar v4

forbid (principal, action, resource);

permit (
  principal in Auditor::?a,
  action == Action::"audit_query",
  resource in AuditEvent::?e
)
when {
  principal has scoped_packs &&
  resource has pack &&
  resource.pack in principal.scoped_packs &&
  principal has region_local_pi_read &&
  principal.region_local_pi_read == true
};
