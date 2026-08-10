// Workplace Integration Cedar policy 2
// naming_justifications: BNF v4 action workplace-integration.esign-sign.execute; 13-layer-enum policy concern.
permit (
  principal,
  action == Action::"workplace-integration.esign-sign.execute",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.sub_scope_path has context.allowed_sub_scope &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  context.region != ""
};

forbid (
  principal,
  action == Action::"workplace-integration.esign-sign.execute",
  resource
)
when {
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};
