// cloud-billing — revenue-share settlement Cedar gates
//
// Per ADR-0330 §B.5 + §B.10.7 + §B.10.8 cloud-billing owns the rev-share
// settlement payout flow + the clawback / chargeback handling.

// ---------- Settlement compute ----------

@id("cap.cloud.billing.settlement.compute.permit")
permit (
  principal,
  action == Action::"compute_settlement",
  resource
) when {
  resource.tenant_class == "paid" &&
  "revenue_share" in resource.billing_components &&
  principal in Group::"cloud-billing-settlement-worker"
};

@id("cap.cloud.billing.settlement.compute.deny_demo_trial")
forbid (
  principal,
  action == Action::"compute_settlement",
  resource
) when {
  resource.tenant_class == "demo_trial"
};

@id("cap.cloud.billing.settlement.compute.deny_paid_without_revshare")
forbid (
  principal,
  action == Action::"compute_settlement",
  resource
) when {
  resource.tenant_class == "paid" &&
  !("revenue_share" in resource.billing_components)
};

// ---------- Payout initiation ----------

@id("cap.cloud.billing.settlement.payout.permit_worker")
permit (
  principal,
  action == Action::"initiate_payout",
  resource
) when {
  principal in Group::"cloud-billing-settlement-worker" &&
  resource.direction == "oyatie_pays" &&
  resource.state == "Computed"
};

@id("cap.cloud.billing.settlement.payout.permit_human_with_approval")
permit (
  principal,
  action == Action::"initiate_payout",
  resource
) when {
  principal.has_role("oyatie-finance-operator") &&
  context.has_reviewer_approval == true &&
  context.reviewer_principal_id != principal.principal_id
};

// ---------- Clawback handling ----------

@id("cap.cloud.billing.settlement.clawback.record")
permit (
  principal,
  action == Action::"record_revenue_event_reversal",
  resource
) when {
  resource.tenant_class == "paid" &&
  "revenue_share" in resource.billing_components &&
  (principal in Group::"payments-psp-webhook-handler" ||
   principal.has_role("oyatie-finance-operator"))
};

@id("cap.cloud.billing.settlement.clawback.dispute_period")
forbid (
  principal,
  action == Action::"record_revenue_event_reversal",
  resource
) when {
  context.original_event_age_seconds > context.dispute_period_seconds
};

// ---------- Negative revenue_share (affiliate) ----------

@id("cap.cloud.billing.settlement.affiliate_payout")
permit (
  principal,
  action == Action::"initiate_payout",
  resource
) when {
  resource.tenant_class == "paid" &&
  "revenue_share" in resource.billing_components &&
  resource.direction == "oyatie_pays" &&
  resource.contract_direction == "affiliate" &&
  principal in Group::"cloud-billing-settlement-worker"
};

// ---------- BEPS Pillar Two export ----------

@id("cap.cloud.billing.settlement.beps_export")
permit (
  principal,
  action == Action::"export_beps_pillar_two",
  resource
) when {
  resource.tenant_class == "paid" &&
  resource.beps_eligible == true &&
  principal.has_role("oyatie-finance-operator") &&
  context.has_reviewer_approval == true
};

// ---------- Sovereign invoice issuance ----------

@id("cap.cloud.billing.settlement.sovereign_invoice")
permit (
  principal,
  action == Action::"issue_sovereign_invoice",
  resource
) when {
  resource.tenant_class == "paid" &&
  resource.deployment_context in ["on-prem", "colo", "guest-on-oci"] &&
  resource.sovereign_pack_active == true &&
  principal in Group::"cloud-billing-invoice-worker"
};
