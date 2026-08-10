---
runbook: dlq-overflow
microservice: connector
owner_team: axis-integration + ops-sre-reliability
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0145, ADR-0263]
doc_status: published
---

# Runbook — DLQ Overflow

## A. Trigger conditions

- `oya_connector_dlq_depth{tenant_id="<X>"} > tenant_retention_cap × 0.8`
- PG disk-pressure alert
- Tenant reports "DLQ entries being dropped"

## B. Pre-checks

1. Identify cause: vendor outage (most common) vs malformed wirings vs DLQ-replay storm
2. Check retention-cap per tenant: `kubectl get configmap connect-dlq-config -n connect -o yaml`

## C. Procedure

1. **Inventory DLQ entries** (≤3min)
   ```sql
   SELECT tenant_id, wiring_id, error_class, count(*)
   FROM connector.dlq_entries
   WHERE last_tried_at > NOW() - INTERVAL '24 hours'
   GROUP BY 1, 2, 3
   ORDER BY count DESC LIMIT 20;
   ```

2. **Bulk replay if vendor recovered** (≤10min)
   ```bash
   # Replay DLQ entries for specific wiring (idempotency-key preserved)
   curl -X POST http://connect-dlq-replay-worker.connect:8080/admin/replay-batch \
     -H "Authorization: Bearer <step-up-token>" \
     -d '{"wiring_id": "<id>", "max_age_seconds": 86400}'
   ```

3. **Per-tenant retention extension** (if needed; ≤5min)
   ```bash
   # Extend tenant's DLQ retention (max 30d per compliance.md)
   kubectl patch configmap connect-dlq-config -n connect --type merge \
     -p '{"data":{"tenant_<id>_retention_days":"30"}}'
   ```

4. **Drop oldest entries** (last resort if disk-pressure imminent)
   - Emit `DLQRetentionCap` audit event per dropped entry
   - Notify tenant via in-app + email

5. **Disk-pressure response** (if PG full)
   - Scale PG storage volume (online resize)
   - Move oldest entries to cold-tier (S3)

## D. Verification

```promql
# DLQ depth trending down
oya_connector_dlq_depth{tenant_id="<X>"}

# Replay success rate
rate(oya_connector_dlq_replay_total{outcome="success"}[5m])
  / rate(oya_connector_dlq_replay_total[5m])
```

## E. Rollback

DLQ entries dropped via retention-cap are non-recoverable. Communicate to tenant; replay from vendor-side if possible (e.g., Stripe events API).

## F. Post-incident

- If vendor outage: track vendor SLA + adjust circuit-breaker thresholds
- If tenant pattern: surface to gtm-customer-success for tier discussion
- If bug: regression test in `oya-connector-retry-and-dlq-domain`

## G. References

- ADR-0145 §invariant-1 (DLQ overflow semantics)
- ADR-0263 audit-event emission
- `gateway/connector/runbooks/connector-cascade-failure.md`
- `microservices/connector/compliance.md` (retention by pack)
