# Runbook — Tenant Onboard Failure

**Authority:** IP-002, ADR-0145, ADR-0038
**Owner:** council-analytics + council-tenancy
**Trigger:** `oya.tenancy.tenant.onboarded.v1` event received but per-tenant database not created within 30 s SLO; OR `last_exception` on the bootstrap controller cursor.
**Severity:** Sev 2 (customer onboard delayed)

## Recap

IP-002's controller consumes `oya.tenancy.tenant.onboarded.v1` and reconciles ClickHouse state. Successful onboard produces:

- `tenant_${tid}` database created.
- `tenant_${tid}_reader` / `_writer` roles granted.
- Per-tier QUOTA applied.
- `oya.analytics.tenant_database.created.v1` audit event emitted.

## Diagnosis

### Step 1: Check controller liveness

```
kubectl get pods -n analytics -l app=oya-analytics-tenant-bootstrap-app
kubectl logs -n analytics deployment/oya-analytics-tenant-bootstrap-app --tail 200
```

### Step 2: Verify event landed

```
kubectl exec -n observability pulsar-broker-0 -- pulsar-admin topics stats persistent://public/default/oya.tenancy.tenant.events | jq .subscriptions
```

Look for `analytics-tenant-bootstrap` subscription; verify `msgBacklog`.

### Step 3: Verify DB present

```sql
SHOW DATABASES LIKE 'tenant_${tid}';
```

### Step 4: Check controller cursor

```
kubectl exec -n analytics deployment/oya-analytics-tenant-bootstrap-app -- \
  curl -s http://localhost:9100/cursor | jq .last_processed_event_id
```

## Decision tree

### Symptom: Controller crashed / restart loop

**Cause:** likely transient (OpenBao timeout, Pulsar reconnect, OOM).

**Action:**

1. Restart pod:
   ```
   kubectl rollout restart deployment/oya-analytics-tenant-bootstrap-app -n analytics
   ```
2. Watch controller logs for the `tenant_id` in question.
3. If OOM → bump memory limit in Helm values.

### Symptom: Event in Pulsar, controller alive, but DB not created

**Cause:** ClickHouse DDL failing.

1. Check ClickHouse logs:
   ```
   kubectl logs -n analytics clickhouse-server-0 --tail 200 | grep -i error
   ```
2. Most common: Keeper quorum lost — see `runbooks/keeper-quorum-recovery.md`.
3. Manual override:
   ```sql
   CREATE DATABASE IF NOT EXISTS tenant_${tid} ON CLUSTER oya-cell;
   ```
4. Re-trigger controller by republishing the event:
   ```
   pulsar-client produce -m '<event-json>' persistent://public/default/oya.tenancy.tenant.events
   ```

### Symptom: DB exists but no per-tenant user / role

**Cause:** controller succeeded on DB but failed on RBAC.

```sql
CREATE USER IF NOT EXISTS tenant_${tid}_reader IDENTIFIED WITH ldap_server BY 'oyatie-ldap';
GRANT SELECT ON tenant_${tid}.* TO tenant_${tid}_reader;
CREATE USER IF NOT EXISTS tenant_${tid}_writer IDENTIFIED WITH ldap_server BY 'oyatie-ldap';
GRANT INSERT ON tenant_${tid}.* TO tenant_${tid}_writer;
```

### Symptom: DB + users exist but QUOTA missing

**Cause:** quota application failed.

```sql
CREATE QUOTA IF NOT EXISTS quota_tenant_${tid}
  KEYED BY user_name
  FOR INTERVAL 1 HOUR MAX queries = 1000, read_rows = 1000000000, written_rows = 100000000
  TO tenant_${tid}_reader, tenant_${tid}_writer;
```

### Symptom: Audit event not emitted

**Cause:** controller succeeded silently on ClickHouse but failed to publish.

1. Manually publish the audit event:
   ```
   pulsar-client produce -m '{"type":"oya.analytics.tenant_database.created.v1","data":{"tenant_id":"${tid}","cell":"${cell}"}}' persistent://public/default/oya.analytics.outbound
   ```
2. Re-emit if compliance review requires re-signing.

## Verification

1. `SHOW DATABASES LIKE 'tenant_${tid}'` returns 1 row.
2. Reader / writer users exist.
3. QUOTA exists.
4. Audit event observed downstream by audit-chain.
5. Tenancy µservice's onboard saga completes (status `analytics_provisioned = true`).

## Customer comms

- If onboard delay < 5 min after event landed: no customer comm needed.
- If > 5 min: tenant admin notified by tenancy µservice's onboard saga retry policy.

## Long-term mitigations

- Controller leader-election with 2 replicas (1 active) so one pod failure doesn't pause onboards.
- Health endpoint surfaces cursor lag.
- Reconciliation job hourly verifies every Cedar-permitted tenant has a corresponding ClickHouse DB.

## References

- IP-002, ADR-0145 inter-µservice communication, ADR-0038 DSR cascade, ADR-0003 audit chain.
