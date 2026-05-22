// Marketplace Cedar policy — category-apps (ADR-0249 KS#11)
// naming_justifications: BNF v4 action marketplace.category-apps.listing; 13-layer-enum conformance; policy is a concern, not a layer.

permit (
  principal,
  action == Action::"marketplace.category-apps.listing.publish",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.sub_scope_path has context.allowed_sub_scope &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  context.region != "" &&
  context.container_signature_verified == true &&
  context.sbom_present == true &&
  context.security_review_passed == true &&
  context.tenant_class == "paid" &&            // apps require paid tenant
  context.kyc_artifact_ref != ""
};

permit (
  principal,
  action == Action::"marketplace.category-apps.entitlement.provision",
  resource
)
when {
  context.tenant_id != "" &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  (
    (resource.pricing_model == "free" && context.kyc_artifact_ref != "")
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
    Action::"marketplace.category-apps.listing.publish",
    Action::"marketplace.category-apps.entitlement.provision"
  ] &&
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};

// FORBID demo_trial from publishing apps (kyc/security review required)
forbid (
  principal,
  action == Action::"marketplace.category-apps.listing.publish",
  resource
)
when {
  context.tenant_class == "demo_trial"
};

// FORBID provisioning apps that target regulated domains without the matching compliance pack
forbid (
  principal,
  action == Action::"marketplace.category-apps.entitlement.provision",
  resource
)
when {
  resource.target_domain == "healthcare" &&
  !(context.active_compliance_packs has "HIPAA")
};
