// Marketplace Cedar policy — category-models (ADR-0249 KS#11 + ADR-0308 EU AI Act)
// naming_justifications: BNF v4 action marketplace.category-models.listing; 13-layer-enum conformance; policy is a concern, not a layer.

permit (
  principal,
  action == Action::"marketplace.category-models.listing.publish",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.sub_scope_path has context.allowed_sub_scope &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  context.region != "" &&
  context.artifact_signature_verified == true &&
  context.model_card_present == true &&
  context.evals_present == true &&
  context.training_data_provenance_attested == true &&
  context.sbom_present == true &&
  context.tenant_class == "paid" &&
  context.kyc_artifact_ref != "" &&
  resource.eu_ai_act_risk_class != "unacceptable"
};

permit (
  principal,
  action == Action::"marketplace.category-models.entitlement.fetch",
  resource
)
when {
  context.tenant_id != "" &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  resource.eu_ai_act_risk_class != "unacceptable" &&
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
    Action::"marketplace.category-models.listing.publish",
    Action::"marketplace.category-models.entitlement.fetch"
  ] &&
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};

// FORBID unacceptable EU AI Act risk class (ADR-0308)
forbid (
  principal,
  action,
  resource
)
when {
  action in [
    Action::"marketplace.category-models.listing.publish",
    Action::"marketplace.category-models.entitlement.fetch"
  ] &&
  resource.eu_ai_act_risk_class == "unacceptable"
};

// FORBID GPAI systemic-risk model fetch without active compliance pack
forbid (
  principal,
  action == Action::"marketplace.category-models.entitlement.fetch",
  resource
)
when {
  resource.eu_ai_act_risk_class == "gpai-systemic-risk" &&
  !(context.active_compliance_packs has "EU-AI-Act-gpai-systemic")
};

// FORBID demo_trial above small model size band
forbid (
  principal,
  action == Action::"marketplace.category-models.entitlement.fetch",
  resource
)
when {
  context.tenant_class == "demo_trial" &&
  resource.model_size_band in ["mid-7b-30b", "large-30b-200b", "frontier-200b+"]
};
