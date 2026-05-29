---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: calendar
runbook_id: RB-recurrence-storm
status: Accepted
date: 2026-05-17
owner_team: axis-calendar + ops-sre-reliability
severity_applicable: [Sev-1, Sev-2]
related_failure_modes: [FM-01, FM-15]
related_dashboards: [event-throughput, availability-lookup-rate]
doc_status: published
---

# Runbook — Recurrence Storm

## When this runbook fires

- `recurrence_expansion_p99_ms` exceeds 1s for > 5min, OR
- recurrence worker queue depth > 60s of cadence for > 5min, OR
- Worker pod OOMKilled in last 15min.

## Symptoms

- Tenant sees event creation acknowledged but recurring occurrences not materialised.
- Recurrence worker queue building up.
- Worker pod memory pressure / OOMKilled events.
- `RecurrenceWindowExpanded` events delayed in event-bus.

## Probable causes

1. Single tenant submitting many complex RRULEs (BYSETPOS + BYDAY + BYMONTH + EXDATE).
2. Mass update on a recurring event triggers full re-expansion.
3. Worker memory bound too low; cannot handle peak.
4. Workload-mix shift: more recurring events than baseline.

## Triage (within 15 min)

1. Acknowledge OnCall page.
2. Check Grafana dashboard `event-throughput`: tenant-cardinality of recurrence submissions.
3. Identify offending tenant via top-N submitter query:
   ```promql
   topk(5, sum by (tenant_id_hashed) (rate(calendar_recurrence_expansion_count_total[5m])))
   ```
4. Check worker pod memory utilisation: `kubectl top pods -l app=calendar-recurrence-engine-worker`.
5. If worker pods OOMKilled, check K8s event log.

## Mitigation steps

### Step 1 — Rate-limit offending tenant

```bash
oya calendar rate-limit set --tenant <hashed-id> --resource rrule_expansion --limit 10/min --duration 1h --audit-reason "RB-recurrence-storm"
```

### Step 2 — Scale up worker pods

```bash
kubectl scale deployment -n calendar oya-calendar-recurrence-engine-worker --replicas=30
```

### Step 3 — Verify queue drain

```promql
sum(rate(calendar_recurrence_worker_queue_depth_seconds[1m]))
```

Expected: should trend toward 0 over 5-15 min.

### Step 4 — If OOMKilled

Adjust pod memory request/limit (temporary; ADR for permanent change):

```bash
kubectl set resources deployment/oya-calendar-recurrence-engine-worker -n calendar \
  --requests=memory=4Gi --limits=memory=8Gi
```

### Step 5 — If tenant is malicious

Engage ops-security. Apply tenant-level Cedar policy refusal:

```bash
oya calendar policy deny --tenant <hashed-id> --action rrule_submit --duration 24h --audit-reason "suspected-abuse"
```

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `recurrence_expansion_p99_ms` | < 1s | should return within 15 min |
| Worker queue depth | < 60s | should return within 15 min |
| Worker pod restarts | 0 in last 5 min | should be 0 |

## Post-incident review

- Was the RRULE bound exhaustive?
- Should the per-tenant rate-limit baseline be lowered?
- Should worker memory be permanently increased?
- Update threat-model.md T-D-01 mitigation if needed.
- Update LEAN check `oya-check-rrule-bounds` if a new attack pattern was discovered.

## Drills

- Bi-annual simulated recurrence storm in staging.
- Verify rate-limit cuts in correctly + worker scales out as expected.

## References

- `failure-modes.md` FM-01 + FM-15.
- `threat-model.md` T-D-01.
- `dashboards/event-throughput.json`.
