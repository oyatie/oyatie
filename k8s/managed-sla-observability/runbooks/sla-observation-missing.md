# Runbook: SLA observation missing

## Signal
`SlaObservabilityError::UnknownCluster` is returned for a tenant cluster.

## Operator action
1. Confirm the cluster exists in the cluster-lifecycle/control-plane-host inventory.
2. Confirm the SLA snapshot ingestion job is running for the tenant cell.
3. If live metrics integration is still absent, seed via the in-memory/test adapter only in local verification; do not claim production SLO evidence.

## Safety
Unknown clusters fail closed. Do not synthesize success summaries for missing observations.
