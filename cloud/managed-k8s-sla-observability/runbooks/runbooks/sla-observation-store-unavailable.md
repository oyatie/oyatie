# Runbook: SLA observation store unavailable

## Symptom
Managed-K8s SLA summary reads or internal observation ingestion return
`SlaObservabilityError::Store`, or a known cluster has no readable latest
observation after the ingestion job runs.

## Impact
The affected tenant/cluster has no trustworthy SLA summary or evidence handle.
Rollout, rollback, public-SLA, and customer-facing evidence workflows must treat
the cluster as no-data/unavailable until a reviewed observation is ingested.

## Steps
1. Confirm the cluster exists in the control-plane-host inventory and that the
   requested `(tenant_id, cluster_name)` matches the caller scope.
2. Confirm the SLA snapshot ingestion job or local test harness is running for
   the tenant cell and is producing normalized `ControlPlaneSlaSnapshot` records.
3. Review application logs for `SlaObservabilityError::Store`, poisoned
   in-memory adapter state, or stale observation timestamps.
4. Restart only the affected local/dev adapter or collector process after the
   underlying store issue is corrected. Do not backfill a healthy summary without
   a reviewed source observation.
5. Re-ingest a known observation and verify the summary read returns tenant- and
   cluster-scoped availability, provisioning-latency, and error-budget fields.

## Prevention
Wire follow-on live collector and durable evidence storage lanes behind the
`SlaObservabilityPort` with freshness deadlines, tenant/cell metadata, and
reviewed rollback/observability evidence before citing production SLO proof.
