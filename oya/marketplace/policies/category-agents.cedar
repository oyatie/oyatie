// Marketplace Cedar policy — category-agents (ADR-0249 KS#11 + ADR-0308 EU AI Act)
// naming_justifications: BNF v4 action marketplace.category-agents.listing; 13-layer-enum conformance; policy is a concern, not a layer.

permit (
  principal,
  action == Action::"marketplace.category-agents.listing.publish",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.sub_scope_path has context.allowed_sub_scope &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  context.region != "" &&
  context.tool_manifest_reviewed == true &&
  context.red_team_passed == true &&
  context.safety_card_present == true &&
  context.tenant_class == "paid" &&
  context.kyc_artifact_ref != "" &&
  resource.eu_ai_act_risk_class != "unacceptable"
};

permit (
  principal,
  action == Action::"marketplace.category-agents.entitlement.register",
  resource
)
when {
  context.tenant_id != "" &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  resource.eu_ai_act_risk_class != "unacceptable" &&
  (
    (resource.pricing_model == "free" && resource.autonomy_level == "supervised")
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
    Action::"marketplace.category-agents.listing.publish",
    Action::"marketplace.category-agents.entitlement.register"
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
    Action::"marketplace.category-agents.listing.publish",
    Action::"marketplace.category-agents.entitlement.register"
  ] &&
  resource.eu_ai_act_risk_class == "unacceptable"
};

// FORBID high-risk agents without human oversight attestation
forbid (
  principal,
  action == Action::"marketplace.category-agents.listing.publish",
  resource
)
when {
  resource.eu_ai_act_risk_class == "high" &&
  context.human_oversight_attestation_ref == ""
};

// FORBID demo_trial from fully-autonomous agents
forbid (
  principal,
  action == Action::"marketplace.category-agents.entitlement.register",
  resource
)
when {
  context.tenant_class == "demo_trial" &&
  resource.autonomy_level == "fully-autonomous"
};
