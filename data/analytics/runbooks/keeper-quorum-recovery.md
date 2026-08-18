# Runbook — ClickHouse Keeper Quorum Recovery

**Authority:** ADR-0193, IP-001
**Owner:** ops-sre-reliability + council-analytics
**Trigger:** `ClickHouseKeeperNoLeader` alert OR `clickhouse_keeper_is_leader{cluster="analytics-clickhouse-1"} == 0` for >5 min.
**Severity:** Sev 1 (DDL operations halted; tenant onboard blocked; reads continue)

## What broke

ClickHouse Keeper provides Raft-quorum consensus for replication and DDL. With 3 Keepers, the cluster tolerates 1 failure (2 of 3 = quorum). When 2 fail, quorum is lost: cluster DDL (`CREATE`, `ALTER`, `DROP`) fails immediately; `INSERT` still works against the leader replica per shard.

## Reads continue?

YES. Replication queue is in Keeper, but a query against a replica's local storage proceeds. Stale-data risk: queries may see data up to `replication_lag_seconds` (~1-2s) behind another replica.

## DDL halted?

YES. Every `CREATE` / `ALTER` / `DROP` queues against Keeper and fails after the configured timeout (default 30s).

## Immediate action

### Step 1: Triage which Keepers are down

```
kubectl get pods -n analytics -l app.kubernetes.io/component=clickhouse-keeper
```

Expected: 3 pods. If 0/3 or 1/3 ready, quorum is lost.

### Step 2: Investigate pod-level cause

```
kubectl describe pod -n analytics clickhouse-keeper-0
kubectl logs -n analytics clickhouse-keeper-0 --tail 200
```

Common causes:
- OOM kill (memory limit too low under burst).
- Disk full (Keeper log can grow large under high write rate).
- Node drain in progress (planned maintenance).

### Step 3: Restart healthy node

If the pod is failing on restart loop:

```
kubectl delete pod -n analytics clickhouse-keeper-1
```

Kubernetes recreates it. Watch for `Ready=True`:

```
kubectl wait --for=condition=Ready pod/clickhouse-keeper-1 -n analytics --timeout=300s
```

### Step 4: If disk-full

```
kubectl exec -n analytics clickhouse-keeper-1 -- du -h /var/lib/clickhouse-keeper/logs
```

Trim oldest Keeper log files:

```
kubectl exec -n analytics clickhouse-keeper-1 -- clickhouse-keeper-client -p 9181 -q "rmr /requests/old-uuid"
```

Increase PVC if size remains insufficient:

```yaml
# values.yaml
keeper:
  pvc:
    size: 100Gi  # up from 20Gi
```

### Step 5: If 2+ Keepers are gone (true quorum loss)

Quorum is lost; manual intervention required.

1. Identify the most up-to-date Keeper (highest last_log_index):

   ```
   kubectl exec -n analytics clickhouse-keeper-2 -- clickhouse-keeper-client -p 9181 -q "stat" | grep last_log_index
   ```

2. On the surviving Keeper, force-restart in standalone mode by editing its config to set `force_recovery=true`:

   ```yaml
   # Helm overlay
   keeper:
     forceRecovery: true
     replicas: 1
   ```

   `helm upgrade analytics-clickhouse ... --set keeper.forceRecovery=true --set keeper.replicas=1`

3. Verify single-Keeper quorum is operational:

   ```
   kubectl exec -n analytics clickhouse-keeper-2 -- clickhouse-keeper-client -p 9181 -q "stat" | grep "Mode: leader"
   ```

4. Re-add the other two Keepers one at a time (set `replicas: 3`; remove `forceRecovery`); they will sync from the surviving leader.

5. Verify final quorum: 3 of 3 Ready, exactly one in `Mode: leader`.

### Step 6: Restore Keeper snapshot (last resort)

If no Keeper has a recoverable state:

1. Stop all Keeper pods.
2. Restore the most recent Keeper snapshot from S3:
   ```
   aws s3 cp s3://oyatie-analytics-backups/keeper-snapshots/<latest>.bin /var/lib/clickhouse-keeper/snapshots/
   ```
3. Start Keeper pods.
4. Verify and validate state.

This causes data loss for replication queue events between the snapshot and now (~10 min). Acceptable for restoring service; affected partitions may need re-replication.

## Post-recovery verification

1. `kubectl get pods -n analytics -l app.kubernetes.io/component=clickhouse-keeper` — 3/3 Ready.
2. Exactly one Keeper in `Mode: leader`.
3. Test DDL: `clickhouse-client --query "CREATE DATABASE keeper_recovery_test_$(date +%s)"` succeeds.
4. Test replication: `system.replication_queue` drains.
5. All cluster alerts cleared.

## Communication

- Status page: "Tenant onboard temporarily delayed; reads unaffected" if quorum lost >5 min.
- `#analytics-incidents`: declare Sev 1; track via Scribe.
- DPO loop-in if quorum loss > 1 h (potential availability obligation impact).

## Long-term mitigations (deferred to phase 2)

- 5-replica Keeper quorum for production cells (already in pack-eu / pack-kr overlay).
- Cross-cell Keeper snapshot replication.
- Keeper-disk size pre-emptive expansion at 70% full.

## References

- ADR-0193, IP-001 (cluster IaC), `data/analytics/runbooks/clickhouse.md`.
- ClickHouse Keeper recovery docs: https://clickhouse.com/docs/guides/sre/keeper/clickhouse-keeper
