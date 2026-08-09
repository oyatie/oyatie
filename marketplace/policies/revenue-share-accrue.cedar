// Marketplace Cedar policy — revenue-share-accrue (AMENDED 2026-05-21 per IP-037 + ADR-0329..0331)
// naming_justifications: BNF v4 action marketplace.revenue-share-accrue.execute; 13-layer-enum conformance; policy is a concern, not a layer.
permit (
  principal,
  action == Action::"marketplace.revenue-share-accrue.execute",
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
  context.revenue_share_cohort_id != ""
};

forbid (
  principal,
  action == Action::"marketplace.revenue-share-accrue.execute",
  resource
)
when {
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};

// Explicit demo_trial denial (IP-041 deny-reason MKT_DENY_DEMO_TRIAL_NO_REVENUE_SHARE)
forbid (
  principal,
  action == Action::"marketplace.revenue-share-accrue.execute",
  resource
)
when {
  context.tenant_class == "demo_trial"
};

// Forbid accrual when billing_components missing revenue_share (paid but no rev-share contract)
forbid (
  principal,
  action == Action::"marketplace.revenue-share-accrue.execute",
  resource
)
when {
  context.tenant_class == "paid" &&
  !(context.tenant_billing_components has "revenue_share")
};
