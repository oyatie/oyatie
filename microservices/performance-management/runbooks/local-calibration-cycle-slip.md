# Performance Management Runbook: Calibration Cycle Slip

Service: performance-management  
Surface: local operational primitive suite  
Primary SLO: calibration-cycle-completion  
Domain focus: review, calibration, rating, feedback

## Trigger
- Alert `performance-management-calibration-cycle-completion` burns above the 2x multi-window threshold.
- Operator report names PerformanceReviewCycle state drift for tenant-scoped review cycles, calibration sessions, goal alignment, feedback submission, and rating governance operations.
- Audit chain shows denied or missing event class for `review.cycle.opened`.

## Confirm
1. Query `sum(rate(oya_performance_management_calibration_cycle_completion_total[5m])) by (tenant_id, cell_tier)` and identify the affected tenant and cell.
2. Compare `good_total` against `total` for the same metric and confirm the burn is not a dashboard-only gap.
3. Inspect the latest policy decision for action `review.cycle.open` and data class `review_form`.
4. Verify the latest domain event on `performance-management.local-ops.v1` carries `audit_event_id` and tenant scope.

## Mitigate
1. Freeze new high-volume writes for the affected tenant using the local Cedar action `feedback.submit` when burn exceeds 4x.
2. Shift traffic away from the unhealthy cell for the `review-api` workload.
3. Replay only idempotent events with matching `tenant_id`, `resource_id`, and `audit_event_id`.
4. Re-run the policy check endpoint before reopening operator writes.

## Recover
- Restore normal admission when calibration-cycle-completion is below 1x burn for two consecutive 30 minute windows.
- Backfill missing audit evidence before resolving the incident.
- Attach dashboard snapshot `performance-management-local-calibration-cycle-completion` and policy file name to the ticket.

## Escalate
Escalate to the service owner when regulated data class `review_form` is affected for more than 15 minutes or when breakglass was used.
