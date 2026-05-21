// local-vector-search-access.cedar
// Wave-15A — Defect closure F-D4-D-06
// IP-038 binding
//
// Default-deny on VectorIndex::query.
// Required conditions:
// - tenant_class == "paid"
// - "vector_index_serving" billing component enabled
// - oyatie reserved tenant (foundry self-modification per ADR-0247) is
//   the only bypass
// - row-level filter passes per the semantic-model masking policy

@id("data-warehouse-vector-search-paid-only")
forbid (
  principal,
  action == Action::"VectorIndex::query",
  resource
)
when {
  resource.tenant.tenant_class != "paid"
  &&
  resource.tenant.id != "oyatie"
};

@id("data-warehouse-vector-search-billing-required")
forbid (
  principal,
  action == Action::"VectorIndex::query",
  resource
)
unless {
  "vector_index_serving" in resource.tenant.billing_components
  ||
  resource.tenant.id == "oyatie"
};

@id("data-warehouse-vector-search-filter-required-for-phi")
forbid (
  principal,
  action == Action::"VectorIndex::query",
  resource
)
when {
  resource.index.contains_phi
  &&
  resource.filter == null
};
