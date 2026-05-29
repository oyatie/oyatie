// Marketplace Cedar policy — category-plugins (ADR-0249 KS#11)
// naming_justifications: BNF v4 action marketplace.category-plugins.listing; 13-layer-enum conformance; policy is a concern, not a layer.
// Per ADR-0329 + ADR-0330 + ADR-0331: tenant_class principal claim required.

// PERMIT publishing a plugin listing when fully paid OR when sandboxed-demo with allowed sub-category
permit (
  principal,
  action == Action::"marketplace.category-plugins.listing.publish",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.sub_scope_path has context.allowed_sub_scope &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  context.region != "" &&
  context.artifact_signature_verified == true &&
  context.sbom_present == true &&
  (
    (context.tenant_class == "paid")
    ||
    (context.tenant_class == "demo_trial" &&
     resource.review_stage in ["draft", "sandbox"] &&
     resource.pricing_model == "free")
  )
};

// PERMIT consuming (installing / using) a plugin
permit (
  principal,
  action == Action::"marketplace.category-plugins.entitlement.fetch",
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

// FORBID cross-tenant access without explicit grant
forbid (
  principal,
  action,
  resource
)
when {
  action in [
    Action::"marketplace.category-plugins.listing.publish",
    Action::"marketplace.category-plugins.entitlement.fetch"
  ] &&
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};

// FORBID demo_trial from paid listings
forbid (
  principal,
  action == Action::"marketplace.category-plugins.entitlement.fetch",
  resource
)
when {
  context.tenant_class == "demo_trial" &&
  resource.pricing_model != "free"
};
