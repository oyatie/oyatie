// audit-chain µservice — j90-sealed-evidence-chain policy fragment
// Cedar v4
//
// Purpose: journey-specific audit-chain policy overlay referenced by the
// corresponding IP artifact. It narrows the action to audit-chain emission,
// query, verification, or export for the named journey without creating a
// sibling-µservice dependency.
// Owner: axis-audit-chain + owning journey lane

forbid (principal, action, resource);

permit (
  principal in WorkloadEmitter::?w,
  action in [Action::"audit_emit", Action::"audit_read_proof", Action::"audit_verify"],
  resource in AuditEvent::?e
)
when {
  principal has allowed_journey_policies &&
  "j90-sealed-evidence-chain" in principal.allowed_journey_policies &&
  resource has policy_overlay &&
  resource.policy_overlay == "j90-sealed-evidence-chain" &&
  principal has bound_pack &&
  resource has pack &&
  principal.bound_pack == resource.pack
};
