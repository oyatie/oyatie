# Data Warehouse Runbook: Freshness Burn

Service: data-warehouse  
Surface: local operational primitive suite  
Primary SLO: freshness  
Domain focus: pipeline, sla-tier, freshness, completeness

## Trigger
- Alert `data-warehouse-freshness` burns above the 2x multi-window threshold.
- Operator report names WarehouseDataset state drift for tenant-scoped warehouse pipeline scheduling, SLA-tier enforcement, dimensional completeness, query serving, and lineage operations.
- Audit chain shows denied or missing event class for `pipeline.partition.refreshed`.

## Confirm
1. Query `sum(rate(oya_data_warehouse_freshness_total[5m])) by (tenant_id, cell_tier)` and identify the affected tenant and cell.
2. Compare `good_total` against `total` for the same metric and confirm the burn is not a dashboard-only gap.
3. Inspect the latest policy decision for action `pipeline.schedule` and data class `warehouse_table`.
4. Verify the latest domain event on `data-warehouse.local-ops.v1` carries `audit_event_id` and tenant scope.

## Mitigate
1. Freeze new high-volume writes for the affected tenant using the local Cedar action `sla.tier.assign` when burn exceeds 4x.
2. Shift traffic away from the unhealthy cell for the `partition-refresher` workload.
3. Replay only idempotent events with matching `tenant_id`, `resource_id`, and `audit_event_id`.
4. Re-run the policy check endpoint before reopening operator writes.

## Recover
- Restore normal admission when freshness is below 1x burn for two consecutive 30 minute windows.
- Backfill missing audit evidence before resolving the incident.
- Attach dashboard snapshot `data-warehouse-local-freshness` and policy file name to the ticket.

## Escalate
Escalate to the service owner when regulated data class `warehouse_table` is affected for more than 15 minutes or when breakglass was used.
