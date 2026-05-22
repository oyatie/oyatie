// cloud-billing — per-billing-component Cedar gates
//
// Per ADR-0330 §B.11 there are three composable components:
//   - revenue_share
//   - per_seat
//   - per_usage
//
// Each component has its own gate templates. These complement the master
// permits in cloud-billing.cedar.

// ---------- revenue_share component gates ----------

@id("cap.cloud.billing.rev_share.settle")
permit (
  principal,
  action == Action::"settle_rev_share",
  resource
) when {
  resource.tenant_class == "paid" &&
  "revenue_share" in resource.billing_components &&
  principal in Group::"cloud-billing-settlement-worker"
};

@id("cap.cloud.billing.rev_share.deny_without_component")
forbid (
  principal,
  action in [
    Action::"settle_rev_share",
    Action::"compute_settlement",
    Action::"emit_revenue_event",
    Action::"emit_revenue_event_reversal"
  ],
  resource
) when {
  !(resource.tenant_class == "paid" && "revenue_share" in resource.billing_components)
};

@id("cap.cloud.billing.rev_share.publish_marketplace_listing")
permit (
  principal,
  action == Action::"publish_marketplace_listing",
  resource
) when {
  principal.tenant_class == "paid" &&
  "revenue_share" in principal.billing_components &&
  principal.has_role("tenant-marketplace-seller")
};

@id("cap.cloud.billing.rev_share.deny_demo_trial_marketplace")
forbid (
  principal,
  action == Action::"publish_marketplace_listing",
  resource
) when {
  principal.tenant_class == "demo_trial"
};

// ---------- per_seat component gates ----------

@id("cap.cloud.billing.per_seat.read_seat_count")
permit (
  principal,
  action == Action::"read_seat_count",
  resource
) when {
  resource.tenant_class == "paid" &&
  "per_seat" in resource.billing_components &&
  (principal in Group::"cloud-billing-seat-counter" ||
   principal.has_role("tenant-admin"))
};

@id("cap.cloud.billing.per_seat.deny_without_component")
forbid (
  principal,
  action in [
    Action::"read_seat_count",
    Action::"add_seat",
    Action::"remove_seat",
    Action::"set_seat_ceiling"
  ],
  resource
) when {
  !(resource.tenant_class == "paid" && "per_seat" in resource.billing_components)
};

@id("cap.cloud.billing.per_seat.add_seat")
permit (
  principal,
  action == Action::"add_seat",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.tenant_class == "paid" &&
  "per_seat" in resource.billing_components &&
  principal.has_role("tenant-admin")
};

@id("cap.cloud.billing.per_seat.deny_over_ceiling")
forbid (
  principal,
  action == Action::"authenticate_principal",
  resource
) when {
  resource has seat_count_active &&
  resource has seat_count_ceiling &&
  resource.seat_count_active > resource.seat_count_ceiling &&
  resource.seat_grace_window_expired == true
};

// ---------- per_usage component gates ----------

@id("cap.cloud.billing.per_usage.set_soft_cap")
permit (
  principal,
  action == Action::"set_usage_soft_cap",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.tenant_class == "paid" &&
  "per_usage" in resource.billing_components &&
  principal.has_role("tenant-admin")
};

@id("cap.cloud.billing.per_usage.set_hard_cap")
permit (
  principal,
  action == Action::"set_usage_hard_cap",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.tenant_class == "paid" &&
  "per_usage" in resource.billing_components &&
  principal.has_role("tenant-admin")
};

@id("cap.cloud.billing.per_usage.deny_above_hard_cap")
forbid (
  principal,
  action == Action::"emit_usage_event",
  resource
) when {
  resource has hard_cap_active &&
  resource.hard_cap_active == true &&
  resource has hard_cap_exceeded &&
  resource.hard_cap_exceeded == true
};

@id("cap.cloud.billing.per_usage.read_meter_aggregate")
permit (
  principal,
  action == Action::"read_meter_aggregate",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  principal.has_role("tenant-finance-reader")
};
