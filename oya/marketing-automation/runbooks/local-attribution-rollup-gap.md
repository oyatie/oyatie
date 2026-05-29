# Marketing Automation Runbook: Attribution Rollup Gap

Service: marketing-automation  
Surface: local operational primitive suite  
Primary SLO: send-latency  
Domain focus: campaign, journey, segment, suppression

## Trigger
- Alert `marketing-automation-send-latency` burns above the 2x multi-window threshold.
- Operator report names CampaignJourney state drift for tenant-scoped campaign orchestration, journey execution, segment membership, suppression, and deliverability operations.
- Audit chain shows denied or missing event class for `segment.membership.rebuilt`.

## Confirm
1. Query `sum(rate(oya_marketing_automation_send_latency_total[5m])) by (tenant_id, cell_tier)` and identify the affected tenant and cell.
2. Compare `good_total` against `total` for the same metric and confirm the burn is not a dashboard-only gap.
3. Inspect the latest policy decision for action `campaign.segment.read` and data class `campaign_profile`.
4. Verify the latest domain event on `marketing-automation.local-ops.v1` carries `audit_event_id` and tenant scope.

## Mitigate
1. Freeze new high-volume writes for the affected tenant using the local Cedar action `campaign.segment.write` when burn exceeds 4x.
2. Shift traffic away from the unhealthy cell for the `send-worker` workload.
3. Replay only idempotent events with matching `tenant_id`, `resource_id`, and `audit_event_id`.
4. Re-run the policy check endpoint before reopening operator writes.

## Recover
- Restore normal admission when send-latency is below 1x burn for two consecutive 30 minute windows.
- Backfill missing audit evidence before resolving the incident.
- Attach dashboard snapshot `marketing-automation-local-send-latency` and policy file name to the ticket.

## Escalate
Escalate to the service owner when regulated data class `campaign_profile` is affected for more than 15 minutes or when breakglass was used.
