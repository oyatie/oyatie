// Marketplace Cedar policy 6 (AMENDED 2026-05-21 per IP-037)
// naming_justifications: BNF v4 action marketplace.mediation-open.execute; 13-layer-enum conformance; policy is a concern, not a layer.
// Mediation deliberately works for both tenant_class values — sandbox listings can still escalate.
permit (
  principal,
  action == Action::"marketplace.mediation-open.execute",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.sub_scope_path has context.allowed_sub_scope &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  context.region != "" &&
  (
    context.tenant_class == "paid"
    ||
    context.tenant_class == "demo_trial"
  )
};

forbid (
  principal,
  action == Action::"marketplace.mediation-open.execute",
  resource
)
when {
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};
