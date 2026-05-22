// Cedar v4.2 LTS — Ontology write-path quota gates
// Authority: ADR-0297 anti-bot quota gate
// Authority: ADR-0257 amendment write-path service

forbid (
  principal,
  action in [Action::"OntologyEntityWrite", Action::"OntologyRelationshipWrite"],
  resource
) when {
  principal has tenant
  && principal.tenant has write_quota_remaining_per_minute
  && principal.tenant.write_quota_remaining_per_minute <= 0
};

forbid (
  principal,
  action == Action::"OntologyBulkImport",
  resource
) when {
  principal has tenant
  && !(principal.tenant has bulk_import_entitlement
        && principal.tenant.bulk_import_entitlement == true)
};

forbid (
  principal,
  action in [Action::"OntologyEntityWrite"],
  resource
) when {
  // Reject writes that would exceed the per-entity-type cardinality cap
  resource has entity_count
  && resource has cardinality_cap
  && resource.entity_count >= resource.cardinality_cap
};
