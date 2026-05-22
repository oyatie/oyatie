// cloud-billing — master Cedar permits
//
// Per ADR-0243 cedar-as-universal-gate, every state-mutation surface is
// Cedar-gated. Default-deny: any action not explicitly permitted is forbidden.
//
// Per ADR-0330 §B.11 tenant-class-conditional gates read tenant_class from
// principal claim (principal.tenant_class). Composability gates read
// billing_components (principal.billing_components Cedar set).
//
// Per ADR-0244 tenant scoping is on every resource. The principal must own
// the resource's tenant scope (principal.tenant_id == resource.tenant_id)
// for tenant-scoped actions.

// ---------- Tenant-class read ----------

@id("cap.cloud.billing.read_tenant_class")
permit (
  principal,
  action == Action::"read_tenant_class",
  resource
) when {
  principal in Group::"cloud-iam-principal-issuer" ||
  principal in Group::"tenancy-lifecycle-reader" ||
  principal in Group::"finops-portal-reader" ||
  principal.has_role("tenant-admin") ||
  principal.has_role("oyatie-finance-operator")
};

// ---------- Tenant-class convert (demo_trial → paid only) ----------

@id("cap.cloud.billing.convert_tenant")
permit (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.tenant_class == "demo_trial" &&
  context.contract_id != "" &&
  (principal.has_role("tenant-admin") ||
   principal.has_role("oyatie-finance-operator"))
};

@id("cap.cloud.billing.deny_paid_downgrade")
forbid (
  principal,
  action == Action::"convert_tenant",
  resource
) when {
  resource.tenant_class == "paid" &&
  context.target_tenant_class == "demo_trial"
};

// ---------- Billing-components mutation ----------

@id("cap.cloud.billing.mutate_billing_components")
permit (
  principal,
  action == Action::"mutate_billing_components",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.tenant_class == "paid" &&
  context.contract_amendment_id != "" &&
  (principal.has_role("tenant-admin") ||
   principal.has_role("oyatie-finance-operator"))
};

@id("cap.cloud.billing.deny_demo_trial_billing_components_mutation")
forbid (
  principal,
  action == Action::"mutate_billing_components",
  resource
) when {
  resource.tenant_class == "demo_trial"
};

// ---------- Usage event emission ----------

@id("cap.cloud.billing.emit_usage_event")
permit (
  principal,
  action == Action::"emit_usage_event",
  resource
) when {
  principal in Group::"phase-0-microservice" ||
  principal in Group::"phase-1-microservice" ||
  principal in Group::"phase-2-microservice"
};

// ---------- Invoice issuance ----------

@id("cap.cloud.billing.issue_invoice")
permit (
  principal,
  action == Action::"issue_invoice",
  resource
) when {
  principal in Group::"cloud-billing-invoice-worker" ||
  principal.has_role("oyatie-finance-operator")
};

@id("cap.cloud.billing.void_invoice")
permit (
  principal,
  action == Action::"void_invoice",
  resource
) when {
  principal.has_role("oyatie-finance-operator") &&
  context.has_reviewer_approval == true
};

@id("cap.cloud.billing.issue_credit_memo")
permit (
  principal,
  action == Action::"issue_credit_memo",
  resource
) when {
  principal.has_role("oyatie-finance-operator") &&
  context.has_reviewer_approval == true
};

// ---------- Reservation ----------

@id("cap.cloud.billing.purchase_reservation")
permit (
  principal,
  action == Action::"purchase_reservation",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.tenant_class == "paid" &&
  principal.has_role("tenant-finops-admin")
};

@id("cap.cloud.billing.convert_reservation")
permit (
  principal,
  action == Action::"convert_reservation",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.tenant_class == "paid" &&
  principal.has_role("tenant-finops-admin")
};

// ---------- Settlement (revenue_share) ----------

@id("cap.cloud.billing.compute_settlement")
permit (
  principal,
  action == Action::"compute_settlement",
  resource
) when {
  resource.tenant_class == "paid" &&
  "revenue_share" in resource.billing_components &&
  (principal in Group::"cloud-billing-settlement-worker" ||
   principal.has_role("oyatie-finance-operator"))
};

@id("cap.cloud.billing.initiate_payout")
permit (
  principal,
  action == Action::"initiate_payout",
  resource
) when {
  principal in Group::"cloud-billing-settlement-worker" ||
  (principal.has_role("oyatie-finance-operator") &&
   context.has_reviewer_approval == true)
};

// ---------- Read access (broad) ----------

@id("cap.cloud.billing.read_invoice")
permit (
  principal,
  action == Action::"read_invoice",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  (principal.has_role("tenant-finance-reader") ||
   principal.has_role("tenant-admin"))
};

@id("cap.cloud.billing.read_settlement_statement")
permit (
  principal,
  action == Action::"read_settlement_statement",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.tenant_class == "paid" &&
  "revenue_share" in resource.billing_components &&
  principal.has_role("tenant-finance-reader")
};
