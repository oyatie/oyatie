// Marketplace Cedar policy — revenue-share-rate-override (IP-038)
// naming_justifications: BNF v4 action marketplace.revenue-share-rate-override.execute; 13-layer-enum conformance; policy is a concern, not a layer.
permit (
  principal,
  action == Action::"marketplace.revenue-share-rate-override.execute",
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
  context.private_offer_terms_ref != "" &&
  resource.rate_basis_points <= 5000
};

// permit also when manual review approved (above 5000 / 50% rate)
permit (
  principal,
  action == Action::"marketplace.revenue-share-rate-override.execute",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.tenant_class == "paid" &&
  context.tenant_billing_components has "revenue_share" &&
  context.manual_review_approval_ref != "" &&
  resource.rate_basis_points <= 10000
};

forbid (
  principal,
  action == Action::"marketplace.revenue-share-rate-override.execute",
  resource
)
when {
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};

forbid (
  principal,
  action == Action::"marketplace.revenue-share-rate-override.execute",
  resource
)
when {
  context.tenant_class == "demo_trial"
};
