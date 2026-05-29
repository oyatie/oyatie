// Marketplace Cedar policy — revenue-share-clawback (IP-034)
// naming_justifications: BNF v4 action marketplace.revenue-share-clawback.execute; 13-layer-enum conformance; policy is a concern, not a layer.
permit (
  principal,
  action == Action::"marketplace.revenue-share-clawback.execute",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.sub_scope_path has context.allowed_sub_scope &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  context.region != "" &&
  context.tenant_class == "paid" &&
  context.tenant_billing_components has "revenue_share" &&
  context.chargeback_evidence_ref != "" &&
  context.original_accrual_id != ""
};

forbid (
  principal,
  action == Action::"marketplace.revenue-share-clawback.execute",
  resource
)
when {
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};

forbid (
  principal,
  action == Action::"marketplace.revenue-share-clawback.execute",
  resource
)
when {
  context.tenant_class == "demo_trial"
};
