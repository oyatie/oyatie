// local-time-travel-scope.cedar
// Wave-15A — Defect closure F-D4-D-01
// IP-040 binding
//
// Default-deny on Table::queryAtTimestamp.
// Allows only within the tenant's purchased time_travel_storage_days
// window. Fail-safe restoration (post-time-travel window) is gated by
// emergency-services-bypass.cedar in the policy/ substrate.

@id("data-warehouse-time-travel-window-check")
forbid (
  principal,
  action == Action::"Table::queryAtTimestamp",
  resource
)
when {
  resource.offset_days > resource.tenant.time_travel_storage_days
};

@id("data-warehouse-time-travel-demo-trial-refusal")
forbid (
  principal,
  action == Action::"Table::queryAtTimestamp",
  resource
)
when {
  resource.tenant.tenant_class == "demo_trial"
};

@id("data-warehouse-time-travel-version-check")
forbid (
  principal,
  action == Action::"Table::queryAtTimestamp",
  resource
)
when {
  resource.requested_version < resource.table.oldest_retained_version
};
