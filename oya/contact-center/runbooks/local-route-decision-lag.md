# Contact Center Runbook: Route Decision Lag

Service: contact-center  
Surface: local operational primitive suite  
Primary SLO: route-decision-latency  
Domain focus: omnichannel, routing, call, transfer

## Trigger
- Alert `contact-center-route-decision-latency` burns above the 2x multi-window threshold.
- Operator report names OmnichannelInteraction state drift for tenant-scoped omnichannel routing, voice call handling, transfer success, callback scheduling, and recording consent operations.
- Audit chain shows denied or missing event class for `interaction.route.assigned`.

## Confirm
1. Query `sum(rate(oya_contact_center_route_decision_latency_total[5m])) by (tenant_id, cell_tier)` and identify the affected tenant and cell.
2. Compare `good_total` against `total` for the same metric and confirm the burn is not a dashboard-only gap.
3. Inspect the latest policy decision for action `omnichannel.route.assign` and data class `conversation_record`.
4. Verify the latest domain event on `contact-center.local-ops.v1` carries `audit_event_id` and tenant scope.

## Mitigate
1. Freeze new high-volume writes for the affected tenant using the local Cedar action `voice.call.transfer` when burn exceeds 4x.
2. Shift traffic away from the unhealthy cell for the `voice-router` workload.
3. Replay only idempotent events with matching `tenant_id`, `resource_id`, and `audit_event_id`.
4. Re-run the policy check endpoint before reopening operator writes.

## Recover
- Restore normal admission when route-decision-latency is below 1x burn for two consecutive 30 minute windows.
- Backfill missing audit evidence before resolving the incident.
- Attach dashboard snapshot `contact-center-local-route-decision-latency` and policy file name to the ticket.

## Escalate
Escalate to the service owner when regulated data class `conversation_record` is affected for more than 15 minutes or when breakglass was used.
