# Data Pipeline Runbook: Quarantine Release Review

Service: data-pipeline  
Surface: local operational primitive suite  
Primary SLO: quality-null-rate  
Domain focus: ingest, transform, quality, freshness

## Trigger
- Alert `data-pipeline-quality-null-rate` burns above the 2x multi-window threshold.
- Operator report names PipelineRun state drift for tenant-scoped source ingest, transform execution, data-quality measurement, null-rate controls, dead-letter replay, and lineage operations.
- Audit chain shows denied or missing event class for `ingest.batch.accepted`.

## Confirm
1. Query `sum(rate(oya_data_pipeline_quality_null_rate_total[5m])) by (tenant_id, cell_tier)` and identify the affected tenant and cell.
2. Compare `good_total` against `total` for the same metric and confirm the burn is not a dashboard-only gap.
3. Inspect the latest policy decision for action `ingest.batch.accept` and data class `ingest_batch`.
4. Verify the latest domain event on `data-pipeline.local-ops.v1` carries `audit_event_id` and tenant scope.

## Mitigate
1. Freeze new high-volume writes for the affected tenant using the local Cedar action `transform.run.start` when burn exceeds 4x.
2. Shift traffic away from the unhealthy cell for the `ingest-worker` workload.
3. Replay only idempotent events with matching `tenant_id`, `resource_id`, and `audit_event_id`.
4. Re-run the policy check endpoint before reopening operator writes.

## Recover
- Restore normal admission when quality-null-rate is below 1x burn for two consecutive 30 minute windows.
- Backfill missing audit evidence before resolving the incident.
- Attach dashboard snapshot `data-pipeline-local-quality-null-rate` and policy file name to the ticket.

## Escalate
Escalate to the service owner when regulated data class `ingest_batch` is affected for more than 15 minutes or when breakglass was used.
