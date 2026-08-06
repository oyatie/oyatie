// Marketplace Cedar policy — payout-dispatch (IP-036)
// naming_justifications: BNF v4 action marketplace.payout-dispatch.execute; 13-layer-enum conformance; policy is a concern, not a layer.
permit (
  principal,
  action == Action::"marketplace.payout-dispatch.execute",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.sub_scope_path has context.allowed_sub_scope &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  context.region != "" &&
  context.tenant_class == "paid" &&
  context.kyc_artifact_ref != "" &&
  context.payouts_enabled == true &&
  context.source_settlement_statement_id != ""
};

forbid (
  principal,
  action == Action::"marketplace.payout-dispatch.execute",
  resource
)
when {
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};

forbid (
  principal,
  action == Action::"marketplace.payout-dispatch.execute",
  resource
)
when {
  context.tenant_class == "demo_trial"
};

forbid (
  principal,
  action == Action::"marketplace.payout-dispatch.execute",
  resource
)
when {
  context.kyc_artifact_ref == ""
};
