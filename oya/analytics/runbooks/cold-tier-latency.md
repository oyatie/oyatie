# Runbook — Cold-Tier Latency Triage

**Authority:** ADR-0193 §"TTL + partition rotation + cold tier", IP-006
**Owner:** council-analytics + ops-sre-reliability
**Trigger:** Cold-tier query p99 > 2 s SLO burn, OR S3 5xx rate > 5%.
**Severity:** Sev 2 (queries against data > 90 d old degraded)

## Recap

Cold tier = SeaweedFS S3-compat (per-cell). Hot-tier 90 d on local NVMe; older partitions migrate via TTL TODISK. Reads on cold tier are inherently slower (S3 latency); SLO p99 ≤ 2 s.

## Diagnosis

### Step 1: SeaweedFS health

```
kubectl get pods -n observability -l app=seaweedfs
```

All pods Ready? If any NotReady → known root cause.

### Step 2: S3 endpoint reachability

```
kubectl exec -n analytics clickhouse-server-0 -- curl -sI http://seaweedfs-s3.observability.svc.cluster.local:8333/
```

Expect 200 or 405.

### Step 3: ClickHouse cold-tier reads

```sql
SELECT
    disk_name,
    count() AS parts,
    sum(bytes_on_disk) AS bytes,
    max(modification_time) AS last_modified
FROM system.parts
WHERE active AND database LIKE 'tenant_%' AND disk_name = 's3_cold'
GROUP BY disk_name;
```

### Step 4: Slow query log

```sql
SELECT
    query,
    query_duration_ms,
    read_bytes,
    s3_read_microseconds
FROM system.query_log
WHERE event_time > now() - INTERVAL 1 HOUR
  AND query_duration_ms > 2000
  AND has(databases, 'tenant_${tid}')
ORDER BY query_duration_ms DESC
LIMIT 10;
```

## Decision tree

### SeaweedFS pods unhealthy

1. Restart unhealthy pod:
   ```
   kubectl delete pod -n observability seaweedfs-volume-0
   ```
2. Watch for `Ready`.
3. If multiple pods unhealthy → see `runbooks/incident-response.md §5.6`.

### S3 endpoint reachable but slow

1. Inspect SeaweedFS metrics for replica lag / disk pressure.
2. If transient → customer notice; wait for self-heal.
3. If sustained → check underlying storage volume health on the SeaweedFS node.

### Specific tenant's queries slow on cold tier

1. Investigate query pattern — does the tenant frequently scan >1 yr of data?
2. Paid tenant_class contract conversation: contract overlay extends hot-window from 90 d → 365 d.
3. Add dashboard-layer caching (analytics-api crate emits cache headers).

### TTL stuck (parts not migrating to cold)

1. Force TTL pass:
   ```sql
   SYSTEM TTL ON CLUSTER oya-cell;
   ```
2. Watch system.parts.disk_name for the older partitions.
3. If still stuck: verify S3 storage policy in `clickhouse-server` config:
   ```xml
   <storage_configuration>
     <disks>
       <s3_cold>
         <type>s3</type>
         <endpoint>http://seaweedfs-s3.observability.svc.cluster.local:8333/clickhouse-cold/</endpoint>
         <access_key_id from_env="S3_ACCESS_KEY"/>
         <secret_access_key from_env="S3_SECRET_KEY"/>
       </s3_cold>
     </disks>
     <policies>
       <hot_cold>
         <volumes>
           <hot><disk>default</disk></hot>
           <cold><disk>s3_cold</disk></cold>
         </volumes>
       </hot_cold>
     </policies>
   </storage_configuration>
   ```

### Cold-tier writes failing

1. Inspect ClickHouse server logs for S3 PUT 5xx.
2. Verify credentials via OpenBao:
   ```
   kubectl get externalsecret -n analytics clickhouse-s3-cold-tier
   ```
3. Rotate access key if compromised.

## Customer comms

- Status page entry: "Audit-log queries against data older than 90 days may be slow; hot data unaffected."
- Tenant notification via the configured alert channel for tenants impacted.

## Verification

1. Cold-tier query p99 returns to < 2 s within 30 min.
2. S3 5xx rate < 1%.
3. SLO burn returns < 1×.

## Long-term mitigations

- Dashboard-layer aggressive caching for cold-tier reads.
- Tiered HOT/WARM/COLD with WARM on slower NVMe (between hot and S3).
- Multi-AZ SeaweedFS replication.

## References

- ADR-0193 §"TTL + partition rotation + cold tier", IP-006.
- ClickHouse S3 storage docs: https://clickhouse.com/docs/engines/table-engines/integrations/s3
