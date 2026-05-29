# Financial Planning Regional Failover

## Purpose

Recover budget, forecast, consolidation-close, FX-rate, and board-report flows when the active region cannot satisfy the Financial Planning SLOs.

## Trigger

- `availability.openslo.yaml` burns a regional error budget faster than the Wave-15 promotion gate permits.
- `replay-freshness.openslo.yaml` or `local-close-cycle-latency.openslo.yaml` remains in breach after source-ingest backoff and worker restart.
- Regional control-plane, Postgres, Valkey, OpenBao, or object-storage dependencies are unavailable for the current close cycle.

## Failover Steps

1. Freeze new scenario-version publication through the Cedar emergency gate and record the audit-chain event ID.
2. Promote the standby region's Postgres WAL-G restore target and validate schema migration parity.
3. Warm Valkey projections from the restored close-cycle and forecast-read models.
4. Rebind OpenBao secret leases for planner import, FX-rate, and board-report adapters.
5. Replay pending `financial-planning.*` AsyncAPI events from the last sealed audit-chain offset.
6. Run read, write, policy-decision, and close-cycle SLO smoke checks before reopening tenant traffic.

## Rollback

Return traffic to the primary region only after WAL catch-up, audit-chain offset convergence, and tenant-facing version-pinning checks are green.
