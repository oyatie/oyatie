// Marketplace Cedar policy — category-workflows (ADR-0249 KS#11)
// naming_justifications: BNF v4 action marketplace.category-workflows.listing; 13-layer-enum conformance; policy is a concern, not a layer.

permit (
  principal,
  action == Action::"marketplace.category-workflows.listing.publish",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.sub_scope_path has context.allowed_sub_scope &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  context.region != "" &&
  context.workflow_definition_validated == true &&
  context.connector_permissions_reviewed == true &&
  context.sbom_present == true &&
  (
    (context.tenant_class == "paid")
    ||
    (context.tenant_class == "demo_trial" &&
     resource.review_stage in ["draft", "sandbox"] &&
     resource.pricing_model == "free")
  )
};

permit (
  principal,
  action == Action::"marketplace.category-workflows.entitlement.import",
  resource
)
when {
  context.tenant_id != "" &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  (
    resource.pricing_model == "free"
    ||
    (context.tenant_class == "paid" && resource.purchase_confirmed == true)
  )
};

forbid (
  principal,
  action,
  resource
)
when {
  action in [
    Action::"marketplace.category-workflows.listing.publish",
    Action::"marketplace.category-workflows.entitlement.import"
  ] &&
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};

// FORBID demo_trial from paid workflows
forbid (
  principal,
  action == Action::"marketplace.category-workflows.entitlement.import",
  resource
)
when {
  context.tenant_class == "demo_trial" &&
  resource.pricing_model != "free"
};
