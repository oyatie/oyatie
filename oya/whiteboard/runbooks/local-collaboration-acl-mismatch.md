# Whiteboard Runbook: Collaboration Acl Mismatch

Service: whiteboard  
Surface: local operational primitive suite  
Primary SLO: cursor-latency  
Domain focus: whiteboard, cursor, stroke, crdt

## Trigger
- Alert `whiteboard-cursor-latency` burns above the 2x multi-window threshold.
- Operator report names WhiteboardSession state drift for tenant-scoped real-time board collaboration, cursor presence, stroke persistence, CRDT merge, export rendering, and board loading operations.
- Audit chain shows denied or missing event class for `board.opened`.

## Confirm
1. Query `sum(rate(oya_whiteboard_cursor_latency_total[5m])) by (tenant_id, cell_tier)` and identify the affected tenant and cell.
2. Compare `good_total` against `total` for the same metric and confirm the burn is not a dashboard-only gap.
3. Inspect the latest policy decision for action `board.open` and data class `board_state`.
4. Verify the latest domain event on `whiteboard.local-ops.v1` carries `audit_event_id` and tenant scope.

## Mitigate
1. Freeze new high-volume writes for the affected tenant using the local Cedar action `cursor.broadcast` when burn exceeds 4x.
2. Shift traffic away from the unhealthy cell for the `cursor-relay` workload.
3. Replay only idempotent events with matching `tenant_id`, `resource_id`, and `audit_event_id`.
4. Re-run the policy check endpoint before reopening operator writes.

## Recover
- Restore normal admission when cursor-latency is below 1x burn for two consecutive 30 minute windows.
- Backfill missing audit evidence before resolving the incident.
- Attach dashboard snapshot `whiteboard-local-cursor-latency` and policy file name to the ticket.

## Escalate
Escalate to the service owner when regulated data class `board_state` is affected for more than 15 minutes or when breakglass was used.
