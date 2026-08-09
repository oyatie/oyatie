// Marketplace Cedar policy 1 (AMENDED 2026-05-21 per IP-037)
// naming_justifications: BNF v4 action marketplace.deal-offer-create.execute; 13-layer-enum conformance; policy is a concern, not a layer.
// Paid tenants can always create offers. Demo_trial can create only in sandbox + plugins/workflows categories + free pricing.
permit (
  principal,
  action == Action::"marketplace.deal-offer-create.execute",
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
    (
      context.tenant_class == "demo_trial" &&
      resource.review_stage in ["draft", "sandbox"] &&
      resource.category in ["plugins", "workflows"] &&
      resource.pricing_model == "free"
    )
  )
};

forbid (
  principal,
  action == Action::"marketplace.deal-offer-create.execute",
  resource
)
when {
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};

// IP-041 explicit demo_trial denial when revenue_share required
forbid (
  principal,
  action == Action::"marketplace.deal-offer-create.execute",
  resource
)
when {
  context.tenant_class == "demo_trial" &&
  resource.listing_paid_required == true
};
