---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: tasks
runbook_id: RB-recurring-task-materialisation-failure
status: Accepted
date: 2026-05-17
owner_team: axis-tasks + ops-sre-reliability
severity_applicable: [Sev-1, Sev-2]
related_failure_modes: [FM-01]
related_dashboards: [throughput-and-engagement, view-and-board-perf]
doc_status: published
---

# Runbook — Recurring Task Materialisation Failure

## When this runbook fires

- `tasks_recurring_materialise_p99_ms` exceeds 1s for > 5 min, OR
- recurrence worker queue depth > 60s of cadence for > 5 min, OR
- Worker pod OOMKilled in last 15 min, OR
- `MalformedRecurrence::BoundExceeded` rate spike (legacy unbounded RRULE imports per Hyrum #6).

## Symptoms

- Tenant sees recurring-task instances not materialising.
- Recurrence worker queue building up.
- Worker pod memory pressure / OOMKilled events.
- `TaskRecurrenceMaterialised` events delayed in event-bus.

## Probable causes

1. Single tenant submitting many complex RRULEs (BYSETPOS + BYDAY + BYMONTH + EXDATE).
2. Mass update on a recurring task triggers full re-expansion.
3. Worker memory bound too low for peak.
4. Legacy import contains unbounded recurring tasks (per Hyrum #6; new µservice refuses).
5. tzdb refresh inherited from calendar's worker shifted DST → re-expansion across many tenants.

## Triage (within 15 min)

1. Acknowledge OnCall page.
2. Check Grafana dashboard `throughput-and-engagement`: tenant-cardinality of recurrence submissions.
3. Identify offending tenant via top-N submitter query:
   ```promql
   topk(5, sum by (tenant_id_hashed) (rate(tasks_recurrence_materialisation_count_total[5m])))
   ```
4. Check worker pod memory: `kubectl top pods -l app=tasks-recurrence-worker`.
5. If worker pods OOMKilled, check K8s event log.
6. If `MalformedRecurrence::BoundExceeded` spike: check legacy import context (Hyrum #6).

## Mitigation steps

### Step 1 — Rate-limit offending tenant

```bash
oya tasks rate-limit set --tenant <hashed-id> --resource rrule_expansion --limit 10/min --duration 1h --audit-reason "RB-recurring-task-materialisation-failure"
```

### Step 2 — Scale up worker pods

```bash
kubectl scale deployment -n tasks oya-tasks-recurrence-worker --replicas=30
```

### Step 3 — Verify queue drain

```promql
sum(rate(tasks_recurrence_worker_queue_depth_seconds[1m]))
```

Expected: should trend toward 0 over 5-15 min.

### Step 4 — If OOMKilled

Adjust pod memory request/limit (temporary; ADR for permanent change):

```bash
kubectl set resources deployment/oya-tasks-recurrence-worker -n tasks \
  --requests=memory=4Gi --limits=memory=8Gi
```

### Step 5 — If tenant is malicious

Engage ops-security. Apply tenant-level Cedar policy refusal:

```bash
oya tasks policy deny --tenant <hashed-id> --action recurrence_submit --duration 24h --audit-reason "suspected-abuse"
```

### Step 6 — If tzdb-shift cascade

Coordinate with calendar's tzdb refresh runbook; tasks recurrence-engine consumes calendar's tzdb-refresh event signal. Pause recurrence re-expansion until tzdb stable.

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `tasks_recurring_materialise_p99_ms` | < 1s | should return within 15 min |
| Worker queue depth | < 60s | should return within 15 min |
| Worker pod restarts | 0 in last 5 min | should be 0 |
| `MalformedRecurrence::BoundExceeded` rate | should plateau | once legacy imports drained |

## Post-incident review

- Was the RRULE bound exhaustive per ADR-TASKS-0003 + calendar ADR-CAL-0002?
- Should the per-tenant rate-limit baseline be lowered?
- Should worker memory be permanently increased?
- Update threat-model.md T-D-02 mitigation if needed.
- Coordinate with calendar's recurrence-storm.md sibling runbook.

## Drills

- Bi-annual simulated recurrence storm in staging.
- Verify rate-limit cuts in correctly + worker scales out as expected.

## References

- `failure-modes.md` FM-01.
- `threat-model.md` T-D-02.
- ADR-TASKS-0003 + calendar ADR-CAL-0002 (rrule-rs alignment).
- `dashboards/throughput-and-engagement.json`.
- `microservices/calendar/runbooks/recurrence-storm.md` — sibling reference template.
