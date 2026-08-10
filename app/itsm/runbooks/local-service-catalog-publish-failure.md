# ITSM Runbook: Service Catalog Publish Failure

Service: itsm  
Surface: local operational primitive suite  
Primary SLO: ticket-triage-latency  
Domain focus: incident, change, problem, cmdb

## Trigger
- Alert `itsm-ticket-triage-latency` burns above the 2x multi-window threshold.
- Operator report names ServiceManagementRecord state drift for tenant-scoped incident, change, problem, CMDB, knowledge base, and service-level management operations.
- Audit chain shows denied or missing event class for `incident.ticket.created`.

## Confirm
1. Query `sum(rate(oya_itsm_ticket_triage_latency_total[5m])) by (tenant_id, cell_tier)` and identify the affected tenant and cell.
2. Compare `good_total` against `total` for the same metric and confirm the burn is not a dashboard-only gap.
3. Inspect the latest policy decision for action `incident.ticket.create` and data class `incident_ticket`.
4. Verify the latest domain event on `itsm.local-ops.v1` carries `audit_event_id` and tenant scope.

## Mitigate
1. Freeze new high-volume writes for the affected tenant using the local Cedar action `change.request.approve` when burn exceeds 4x.
2. Shift traffic away from the unhealthy cell for the `ticket-api` workload.
3. Replay only idempotent events with matching `tenant_id`, `resource_id`, and `audit_event_id`.
4. Re-run the policy check endpoint before reopening operator writes.

## Recover
- Restore normal admission when ticket-triage-latency is below 1x burn for two consecutive 30 minute windows.
- Backfill missing audit evidence before resolving the incident.
- Attach dashboard snapshot `itsm-local-ticket-triage-latency` and policy file name to the ticket.

## Escalate
Escalate to the service owner when regulated data class `incident_ticket` is affected for more than 15 minutes or when breakglass was used.
