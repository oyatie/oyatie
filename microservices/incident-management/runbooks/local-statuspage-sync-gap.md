# Incident Management Runbook: Statuspage Sync Gap

Service: incident-management  
Surface: local operational primitive suite  
Primary SLO: statuspage-sync-freshness  
Domain focus: war-room, escalation, page, acknowledge

## Trigger
- Alert `incident-management-statuspage-sync-freshness` burns above the 2x multi-window threshold.
- Operator report names IncidentCommand state drift for tenant-scoped paging, escalation, war-room creation, stakeholder update, status page, and postmortem operations.
- Audit chain shows denied or missing event class for `page.dispatched`.

## Confirm
1. Query `sum(rate(oya_incident_management_statuspage_sync_freshness_total[5m])) by (tenant_id, cell_tier)` and identify the affected tenant and cell.
2. Compare `good_total` against `total` for the same metric and confirm the burn is not a dashboard-only gap.
3. Inspect the latest policy decision for action `page.dispatch` and data class `page_event`.
4. Verify the latest domain event on `incident-management.local-ops.v1` carries `audit_event_id` and tenant scope.

## Mitigate
1. Freeze new high-volume writes for the affected tenant using the local Cedar action `page.acknowledge` when burn exceeds 4x.
2. Shift traffic away from the unhealthy cell for the `pager-dispatcher` workload.
3. Replay only idempotent events with matching `tenant_id`, `resource_id`, and `audit_event_id`.
4. Re-run the policy check endpoint before reopening operator writes.

## Recover
- Restore normal admission when statuspage-sync-freshness is below 1x burn for two consecutive 30 minute windows.
- Backfill missing audit evidence before resolving the incident.
- Attach dashboard snapshot `incident-management-local-statuspage-sync-freshness` and policy file name to the ticket.

## Escalate
Escalate to the service owner when regulated data class `page_event` is affected for more than 15 minutes or when breakglass was used.
