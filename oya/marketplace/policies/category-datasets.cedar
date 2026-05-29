// Marketplace Cedar policy — category-datasets (ADR-0249 KS#11 + ADR-0308 + ADR-0251)
// naming_justifications: BNF v4 action marketplace.category-datasets.listing; 13-layer-enum conformance; policy is a concern, not a layer.

permit (
  principal,
  action == Action::"marketplace.category-datasets.listing.publish",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.sub_scope_path has context.allowed_sub_scope &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  context.region != "" &&
  context.datasheet_present == true &&
  context.provenance_attested == true &&
  context.sbom_present == true &&
  context.tenant_class == "paid" &&
  context.kyc_artifact_ref != ""
};

permit (
  principal,
  action == Action::"marketplace.category-datasets.entitlement.grant",
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
    Action::"marketplace.category-datasets.listing.publish",
    Action::"marketplace.category-datasets.entitlement.grant"
  ] &&
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};

// FORBID PII dataset access without GDPR/DPDPA/CCPA pack
forbid (
  principal,
  action == Action::"marketplace.category-datasets.entitlement.grant",
  resource
)
when {
  resource.sensitivity_class == "pii" &&
  !(
    context.active_compliance_packs has "GDPR" ||
    context.active_compliance_packs has "DPDPA" ||
    context.active_compliance_packs has "CCPA"
  )
};

// FORBID PHI dataset access without HIPAA
forbid (
  principal,
  action == Action::"marketplace.category-datasets.entitlement.grant",
  resource
)
when {
  resource.sensitivity_class == "phi" &&
  !(context.active_compliance_packs has "HIPAA")
};

// FORBID sovereign-restricted dataset access without sovereign packs
forbid (
  principal,
  action == Action::"marketplace.category-datasets.entitlement.grant",
  resource
)
when {
  resource.sensitivity_class == "sovereign-restricted" &&
  !(
    context.active_compliance_packs has "CSAP" ||
    context.active_compliance_packs has "IL5" ||
    context.active_compliance_packs has "FedRAMP-High"
  )
};

// FORBID demo_trial above public sensitivity / under-1gb size
forbid (
  principal,
  action == Action::"marketplace.category-datasets.entitlement.grant",
  resource
)
when {
  context.tenant_class == "demo_trial" &&
  (
    resource.sensitivity_class != "public" ||
    resource.size_band in ["1-100gb", "100gb-1tb", "1tb-100tb", "over-100tb"]
  )
};
