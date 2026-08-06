// cloud-billing — tenant_class attribute binding
//
// Per ADR-0330 §B.10.1 cloud-billing is the source-of-truth for tenant_class.
// cloud-iam reads via GetTenantClass and embeds in principal claims:
//   principal.tenant_class : String enum {demo_trial, paid}
//   principal.billing_components : Set<String> subset of {revenue_share, per_seat, per_usage}
//   principal.cap_breached : Boolean (relevant for demo_trial)
//
// This file binds the attribute checks used across cloud-billing's Cedar
// permits. It is the canonical attribute schema reference.

// ---------- Principal attribute schema ----------

// principal.tenant_id : EntityUid -- ten_* or demo_* prefix
// principal.tenant_class : String -- closed enum "demo_trial" | "paid"
// principal.billing_components : Set<String> -- closed subset
// principal.cap_breached : Bool -- transient demo_trial state
// principal.roles : Set<String> -- "tenant-admin", "oyatie-finance-operator", etc.
// principal.byok_modes : Record<String, String> -- "platform_default" | "byok" | "byok_required_by_pack"
// principal.compliance_packs : Set<String>

// ---------- Resource attribute schema ----------

// resource.tenant_id : EntityUid
// resource.tenant_class : String (snapshot at audited operation per ADR-0330 §B.12.2)
// resource.billing_components : Set<String>

// ---------- Context attribute schema ----------

// context.contract_id : String (optional; required for some actions)
// context.contract_amendment_id : String (required for billing_components mutation)
// context.has_reviewer_approval : Bool (required for void / credit_memo / payout)
// context.target_tenant_class : String (for downgrade attempts; gated to forbid)

// ---------- Standard attribute denials ----------

// Demo_trial tenant cap-breach implies write-deny on this resource for 7 days.
@id("cap.cloud.billing.deny_demo_trial_writes_after_cap_breach")
forbid (
  principal,
  action in [
    Action::"emit_usage_event",
    Action::"issue_invoice",
    Action::"purchase_reservation",
    Action::"create_subscription",
    Action::"mutate_billing_components"
  ],
  resource
) when {
  principal.tenant_class == "demo_trial" &&
  principal.cap_breached == true
};

// Tenant-id mismatch between principal and resource always denies (defense in depth).
@id("cap.cloud.billing.deny_cross_tenant_access")
forbid (
  principal,
  action,
  resource
) when {
  resource has tenant_id &&
  principal.tenant_id != resource.tenant_id &&
  !(principal in Group::"oyatie-internal-operator")
};

// Tenant_class enum closed: any value other than the two canonical values is denied.
@id("cap.cloud.billing.deny_unknown_tenant_class")
forbid (
  principal,
  action == Action::"mutate_tenant_class",
  resource
) when {
  context.target_tenant_class != "demo_trial" &&
  context.target_tenant_class != "paid"
};

// Billing_components subset closed: any component other than the three canonical
// values is denied.
@id("cap.cloud.billing.deny_unknown_billing_component")
forbid (
  principal,
  action == Action::"mutate_billing_components",
  resource
) when {
  context.component != "revenue_share" &&
  context.component != "per_seat" &&
  context.component != "per_usage"
};
