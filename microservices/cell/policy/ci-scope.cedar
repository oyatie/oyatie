// cell µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope the CI lanes' access to cell substrate data; cell-boundary lane
//          and cell-no-cross-cell-query lane read declarative state only.
// Owner:   axis-foundry + axis-cell-substrate + ops-security
// References: cell-boundary.md §"Reserved cell IDs", §"Invariant CB-06"

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — CI lane reads cell-registry declarative state
// =============================================================================
//
// The oya-cell-boundary lane (running as Service::"oya-ci") reads the
// declarative cell-registry state (cells in oya-control-plane) + workload
// µservice manifests to validate cell-boundary invariants. It MAY NOT read
// customer-tenant per-cell data.

permit (
  principal == Service::"oya-ci",
  action == Action::"read_cell_metadata",
  resource in Cell::?c
)
when {
  resource has cell_id &&
  resource.cell_id in ["cell:oya-control-plane", "cell:oya-warm-pool", "cell:oya-decommissioning-quarantine"]
};

// =============================================================================
// PERMIT 2 — CI lane reads aggregate cell-utilisation
// =============================================================================
//
// Aggregate metrics from tenant:oya-aggregate are DP-noise protected and
// safe for CI consumption (e.g., capacity-budget gate).

permit (
  principal == Service::"oya-ci",
  action == Action::"read_metrics",
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  resource.tenant_id in ["tenant:oya-aggregate", "tenant:oya-self"] &&
  resource has metric_name &&
  (
    resource.metric_name like "oya_cell_*" ||
    resource.metric_name == "oya:cell_utilization_band:by_pack"
  )
};

// =============================================================================
// PERMIT 3 — lifecycle-manager writes cell-lifecycle state
// =============================================================================
//
// The lifecycle-manager worker (SPIFFE: spiffe://oyatie/cell/lifecycle-manager)
// is the sole authorised writer of cell-state transitions.

permit (
  principal == Service::"lifecycle-manager",
  action in [
    Action::"write_cell_metadata",
    Action::"transition_cell_state"
  ],
  resource in Cell::?c
);

// =============================================================================
// PERMIT 4 — scheduler writes cell-assignment
// =============================================================================
//
// The scheduler (SPIFFE: spiffe://oyatie/cell/scheduler) is the sole authorised
// writer of (tenant_id, cell_id) assignments.

permit (
  principal == Service::"scheduler",
  action == Action::"write_cell_assignment",
  resource in CellAssignment::?a
)
when {
  resource has pack &&
  principal has bound_pack &&
  resource.pack == principal.bound_pack
};

// =============================================================================
// PERMIT 5 — host-pool writes host inventory
// =============================================================================

permit (
  principal == Service::"host-pool",
  action in [
    Action::"write_host_inventory",
    Action::"drain_host",
    Action::"provision_host"
  ],
  resource in HostNode::?h
);

// =============================================================================
// FORBID — CI principal cannot read customer-tenant per-cell data
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"read_cell_assignment",
    Action::"read_migration_history"
  ],
  resource in TenantData::?r
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:oya-aggregate", "tenant:oya-self"])
};

// =============================================================================
// FORBID — CI principal cannot write anything destructive
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"write_cell_metadata",
    Action::"write_cell_assignment",
    Action::"transition_cell_state",
    Action::"drain_host",
    Action::"decommission_cell"
  ],
  resource
);

// =============================================================================
// FORBID — scheduler may not write cross-pack assignment
// =============================================================================
//
// Even with scheduler authority, cross-pack writes are refused. This is the
// load-bearing residency check at the Cedar layer (Postgres RLS re-checks at commit).

forbid (
  principal == Service::"scheduler",
  action == Action::"write_cell_assignment",
  resource in CellAssignment::?a
)
unless {
  resource has pack &&
  principal has bound_pack &&
  resource.pack == principal.bound_pack
};

// =============================================================================
// FORBID — lifecycle-manager may not decommission without 2-person quorum
// =============================================================================
//
// Decommission requires elevated principal carrying quorum_acks claim.

forbid (
  principal,
  action == Action::"decommission_cell",
  resource
)
unless {
  principal has quorum_acks &&
  principal.quorum_acks >= 2
};
