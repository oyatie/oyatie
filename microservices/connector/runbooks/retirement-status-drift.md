# Retirement Status Drift Runbook

## Trigger

Retirement status differs between `RETIREMENT-PLAN.md`, the manifest, and emitted retirement evidence.

## Checks

1. Compare all eight sub-service readiness states against their own folders.
2. Reconcile manifest `status` and retirement evidence with `RETIREMENT-PLAN.md`.
3. Reject any new runtime scope attempting to land under `microservices/connector`.
4. Emit `oya.connector.retirement.status_changed` with the corrected evidence reference.
