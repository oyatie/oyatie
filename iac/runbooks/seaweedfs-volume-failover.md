---
runbook_id: cloud-iac/seaweedfs-volume-failover
authored: 2026-05-18
oncall: axis-cloud-iac
adr_authority: ADR-0196
---

# Runbook — SeaweedFS volume server failover

## When this fires

- `SeaweedfsVolumeServerDown` (≥ 1 volume server unreachable for ≥ 5 min).
- p95 read latency on `evidence-shared-prod` > 500 ms for ≥ 5 min.

## First five minutes

1. **Identify** the affected volume server:
   ```sh
   kubectl -n cloud-iac get pods -l app.kubernetes.io/component=volume \
       | grep -E "(CrashLoopBackOff|Pending|Error)"
   ```
2. **Confirm** the master raft quorum is healthy:
   ```sh
   kubectl -n cloud-iac logs -l app.kubernetes.io/component=master --tail=200 | grep raft
   ```
3. **Check** EC shard distribution — under EC (10+4) we can lose up to
   4 volume servers without data loss:
   ```sh
   weed shell -- 'ec.balance -force'
   ```

## Recovery steps

### Single-server failure (replication absorbs)

1. Master automatically routes reads/writes to replicas.
2. Restart the failed volume server:
   ```sh
   kubectl -n cloud-iac delete pod -l app.kubernetes.io/component=volume,oya/volume-id=<id>
   ```
3. Once healthy, run rebalance:
   ```sh
   weed shell -- 'volume.balance -force'
   ```

### Multi-server failure approaching EC limit (4 down)

1. **STOP** any in-progress lifecycle / compaction tasks (they create
   additional load).
2. Open SEV-2 incident.
3. Spin up replacement volume servers via Helm upgrade increasing
   `volume.replicas`.
4. Once replacements come online, `ec.rebuild -force` recovers shards.

### Full cluster failure (lost majority)

1. Open SEV-1 incident.
2. Fall back to the most-recent Velero backup for cluster state +
   pgBackRest for the filer metadata DB.
3. Restore is destructive; coordinate with ops-finops on tenant
   communication.

## Prevention

- Maintain ≥ 14 volume servers in production (full EC 10+4 distribution).
- Volume server taints + anti-affinity ensure no two volume servers
  land on the same node.

## Evidence

- Audit-chain class `ObjectStoreIncident` sealed with the incident
  identifier.

## References

- ADR-0196 — object storage canonical.
- `docs/standards/helm-chart-convention.md` (EC shape).
