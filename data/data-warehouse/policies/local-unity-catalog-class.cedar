// local-unity-catalog-class.cedar
// Wave-15A — Defect closure F-D4-L-02
// IP-034 binding
//
// Default-deny on Catalog/Schema/DwObject CRUD.
// Required conditions:
// - tenant_id on resource matches principal.tenant_id (or principal is
//   oyatie foundry tenant per ADR-0247)
// - mutations require operator-class principal; reads require any
//   authenticated principal of the owning tenant

@id("data-warehouse-catalog-tenant-scope-required")
forbid (
  principal,
  action,
  resource
)
when {
  (
    resource is Catalog || resource is Schema || resource is DwObject
  )
  &&
  principal.tenant_id != resource.tenant_id
  &&
  principal.tenant_id != "oyatie"
};

@id("data-warehouse-catalog-mutate-operator-only")
forbid (
  principal,
  action,
  resource
)
when {
  (
    action == Action::"Catalog::create" ||
    action == Action::"Catalog::delete" ||
    action == Action::"Schema::create" ||
    action == Action::"Schema::delete" ||
    action == Action::"DwObject::create" ||
    action == Action::"DwObject::delete"
  )
  &&
  principal.role != "DATA_PLATFORM_OPERATOR"
};
