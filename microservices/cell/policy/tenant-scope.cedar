// cell µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/cell/policy/schema.cedarschema
//
// Purpose: enforce per-tenant read scope on cell-assignment + cell-metadata served via
//          cell-registry-rest, cell-sdk, and tenant-assignment-rest.
// Owner:   ops-security + axis-cell-substrate
// Audit:   every deny emits oya_cell_unauthorized_read_attempt_total
// References: cell-boundary.md §"Invariant CB-07"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every read is refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant operator reads own cell-assignment + cell metadata
// =============================================================================
//
// A principal bound to TenantOperator::"<tenant-id>" may read its own
// cell-assignment record + the cell metadata of the cell it is bound to.
// The principal MAY NOT read other tenants' assignments or other cells.

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_cell_assignment",
    Action::"read_own_cell_metadata"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — tenant operator reads own migration history
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action == Action::"read_migration_history",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 3 — workload µservice reads own cell-id (hot path)
// =============================================================================
//
// Workload µservices that have been issued a cell-bound SPIFFE SVID may
// read the cell metadata for their bound cell only. Used for self-discovery
// (e.g., postgres connection string resolution) on the hot path.

permit (
  principal in Service::?s,
  action == Action::"read_own_cell_metadata",
  resource in Cell::?c
)
when {
  principal has bound_cell_id &&
  resource has cell_id &&
  principal.bound_cell_id == resource.cell_id
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_cell_assignment",
    Action::"read_migration_history",
    Action::"read_own_cell_metadata"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — explicit cross-cell deny for workload µservices
// =============================================================================

forbid (
  principal in Service::?s,
  action,
  resource in Cell::?c
)
when {
  principal has bound_cell_id &&
  resource has cell_id &&
  principal.bound_cell_id != resource.cell_id
};

// =============================================================================
// FORBID — never expose SECRET-class data via tenant reads
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — never expose cell-adjacency (which other tenants share my cell)
// =============================================================================
//
// Cell adjacency is a re-identification vector for small tenants; even own-cell
// reads must not surface co-resident tenant_ids.

forbid (
  principal in TenantOperator::?p,
  action == Action::"read_cell_adjacency",
  resource
);
