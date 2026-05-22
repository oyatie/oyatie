// cloud-billing — demo_trial → paid conversion Cedar gates
//
// Per ADR-0330 §B.10.4 + §D.9 the conversion is a single atomic transaction
// that updates tenant_class, sets the initial billing_components, records the
// contract terms, and emits a tenant-class-transition event to audit-chain.

// ---------- Conversion permit ----------

@id("cap.cloud.billing.conversion.permit")
permit (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  // Tenant scope match
  principal.tenant_id == resource.tenant_id &&

  // Source state demo_trial
  resource.tenant_class == "demo_trial" &&

  // Target state paid (one-way conversion)
  context.target_tenant_class == "paid" &&

  // Contract record required
  context.contract_id != "" &&

  // Initial billing_components is a valid subset
  context.initial_billing_components_subset_of(
    ["revenue_share", "per_seat", "per_usage"]
  ) &&

  // Principal has tenant-admin role (or oyatie-finance-operator as fallback)
  (principal.has_role("tenant-admin") ||
   principal.has_role("oyatie-finance-operator"))
};

// ---------- Conversion deny rules ----------

@id("cap.cloud.billing.conversion.deny_downgrade")
forbid (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  resource.tenant_class == "paid" &&
  context.target_tenant_class == "demo_trial"
};

@id("cap.cloud.billing.conversion.deny_demo_to_demo")
forbid (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  resource.tenant_class == "demo_trial" &&
  context.target_tenant_class == "demo_trial"
};

@id("cap.cloud.billing.conversion.deny_paid_to_paid_via_convert")
forbid (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  resource.tenant_class == "paid" &&
  context.target_tenant_class == "paid"
};

@id("cap.cloud.billing.conversion.deny_unknown_target_class")
forbid (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  context.target_tenant_class != "demo_trial" &&
  context.target_tenant_class != "paid"
};

@id("cap.cloud.billing.conversion.deny_missing_contract")
forbid (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  resource.tenant_class == "demo_trial" &&
  context.target_tenant_class == "paid" &&
  context.contract_id == ""
};

// ---------- Conversion during cap-breach grace remains permitted ----------

@id("cap.cloud.billing.conversion.permit_during_grace")
permit (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  resource.tenant_class == "demo_trial" &&
  context.target_tenant_class == "paid" &&
  context.contract_id != "" &&
  principal.cap_breached == true
};

// ---------- Conversion during suspension also permitted (within retention) ----------

@id("cap.cloud.billing.conversion.permit_during_suspension")
permit (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  resource.tenant_class == "demo_trial" &&
  resource.tenant_state == "suspended" &&
  context.within_retention_window == true &&
  context.target_tenant_class == "paid" &&
  context.contract_id != ""
};

// ---------- Conversion atomicity invariant ----------

@id("cap.cloud.billing.conversion.deny_partial_state")
forbid (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  context.transaction_state == "partial" ||
  context.contract_signed == false ||
  context.pre_conversion_settlement_state == "pending"
};

// ---------- Conversion requires audit-chain event sealing ----------

@id("cap.cloud.billing.conversion.require_audit_chain_seal")
permit (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  context.audit_chain_seal_required == true &&
  context.audit_chain_endpoint_available == true
};
