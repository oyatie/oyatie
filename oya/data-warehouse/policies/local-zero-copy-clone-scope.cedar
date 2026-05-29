// local-zero-copy-clone-scope.cedar
// Wave-15A — Defect closure F-D4-D-02
// IP-041 binding
//
// Default-deny on Database::clone. Allows only:
// - same tenant_id source and destination
// - destination region compatible with source residency pack
// Cross-tenant clones (even within the same oyatie account / agency) are
// refused at this layer; Cedar is the second gate after catalog-layer
// boundary check.

@id("data-warehouse-clone-default-deny")
forbid (
  principal,
  action == Action::"Database::clone",
  resource
)
unless {
  resource.source.tenant_id == resource.destination.tenant_id
  &&
  resource.destination.region in resource.source.residency_pack.allowed_regions
};

@id("data-warehouse-clone-demo-trial-refusal")
forbid (
  principal,
  action == Action::"Database::clone",
  resource
)
when {
  resource.source.tenant_class == "demo_trial"
};

@id("data-warehouse-clone-time-travel-window-check")
forbid (
  principal,
  action == Action::"Database::clone",
  resource
)
when {
  resource.at_timestamp_offset_days > resource.source.tenant.time_travel_storage_days
};
