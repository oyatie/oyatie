# Financial Planning Runbook: Fx Rate Backfill Gap

Service: financial-planning  
Surface: local operational primitive suite  
Primary SLO: fx-rate-freshness  
Domain focus: budget, forecast, variance, close

## Trigger
- Alert `financial-planning-fx-rate-freshness` burns above the 2x multi-window threshold.
- Operator report names PlanningCycle state drift for tenant-scoped budget locking, forecast recalculation, variance explanation, close-cycle workflow, FX rate, and board-report operations.
- Audit chain shows denied or missing event class for `budget.locked`.

## Confirm
1. Query `sum(rate(oya_financial_planning_fx_rate_freshness_total[5m])) by (tenant_id, cell_tier)` and identify the affected tenant and cell.
2. Compare `good_total` against `total` for the same metric and confirm the burn is not a dashboard-only gap.
3. Inspect the latest policy decision for action `budget.lock` and data class `budget_line`.
4. Verify the latest domain event on `financial-planning.local-ops.v1` carries `audit_event_id` and tenant scope.

## Mitigate
1. Freeze new high-volume writes for the affected tenant using the local Cedar action `forecast.recalculate` when burn exceeds 4x.
2. Shift traffic away from the unhealthy cell for the `forecast-worker` workload.
3. Replay only idempotent events with matching `tenant_id`, `resource_id`, and `audit_event_id`.
4. Re-run the policy check endpoint before reopening operator writes.

## Recover
- Restore normal admission when fx-rate-freshness is below 1x burn for two consecutive 30 minute windows.
- Backfill missing audit evidence before resolving the incident.
- Attach dashboard snapshot `financial-planning-local-fx-rate-freshness` and policy file name to the ticket.

## Escalate
Escalate to the service owner when regulated data class `budget_line` is affected for more than 15 minutes or when breakglass was used.
