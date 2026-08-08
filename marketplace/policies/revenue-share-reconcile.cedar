// Marketplace Cedar policy — revenue-share-reconcile (IP-039)
// naming_justifications: BNF v4 action marketplace.revenue-share-reconcile.execute; 13-layer-enum conformance; policy is a concern, not a layer.
permit (
  principal,
  action == Action::"marketplace.revenue-share-reconcile.execute",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.sub_scope_path has context.allowed_sub_scope &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  context.region != "" &&
  context.tenant_class == "paid" &&
  context.tenant_billing_components has "revenue_share"
};

forbid (
  principal,
  action == Action::"marketplace.revenue-share-reconcile.execute",
  resource
)
when {
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};

forbid (
  principal,
  action == Action::"marketplace.revenue-share-reconcile.execute",
  resource
)
when {
  context.tenant_class == "demo_trial"
};
