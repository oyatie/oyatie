// local-federated-query-target.cedar
// Wave-15A — Defect closure F-D4-D-08
// IP-039 binding
//
// Default-deny on ExternalTable::scan.
// Required conditions:
// - tenant_class == "paid"
// - "federated_query_bytes" billing component enabled
// - URL is on the tenant's allow-list
// - residency pack permits cross-region read (if cross-region)

@id("data-warehouse-federated-query-paid-only")
forbid (
  principal,
  action == Action::"ExternalTable::scan",
  resource
)
when {
  resource.tenant.tenant_class != "paid"
};

@id("data-warehouse-federated-query-billing-required")
forbid (
  principal,
  action == Action::"ExternalTable::scan",
  resource
)
unless {
  "federated_query_bytes" in resource.tenant.billing_components
};

@id("data-warehouse-federated-query-url-allowlist")
forbid (
  principal,
  action == Action::"ExternalTable::scan",
  resource
)
unless {
  resource.target_url_host in resource.tenant.federated_query_allowlist
};

@id("data-warehouse-federated-query-residency-pack")
forbid (
  principal,
  action == Action::"ExternalTable::scan",
  resource
)
when {
  resource.target_region != resource.tenant.home_region
  &&
  !resource.tenant.residency_pack.allows_cross_region_read
};
