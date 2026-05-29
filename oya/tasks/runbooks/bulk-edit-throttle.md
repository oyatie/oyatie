---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: tasks
runbook_id: RB-bulk-edit-throttle
status: Accepted
date: 2026-05-17
owner_team: axis-tasks
severity_applicable: [Sev-2]
related_failure_modes: [FM-05]
related_dashboards: [throughput-and-engagement]
doc_status: published
---

# Runbook — Bulk-Edit Throttle Exhaustion

## When this runbook fires

- `tasks_bulk_edit_in_flight_count > 1` per tenant, OR
- Per-tenant rate-limit-exhausted counter > 0, OR
- Tenant operator reports "my bulk edit of 50k tasks is hanging".

## Symptoms

- Postgres write storm; tenant's other operations queued.
- Other tenants unaffected (per-tenant rate-limit + RLS isolation).
- `BulkEdit::ThrottleExhausted` errors surfaced to tenant.

## Probable causes

1. Tenant operator initiated bulk-edit without second-confirmation pre-check.
2. Per-tenant rate-limit baseline too low for the tenant_class and paid billing-component capacity policy.
3. Worker pool insufficiently scaled.

## Triage (within 30 min)

1. Acknowledge OnCall page.
2. Identify affected tenant + bulk-op-id:
   ```promql
   tasks_bulk_edit_in_flight_count{tenant_id_hashed!=""}
   ```
3. Check bulk-op size:
   ```bash
   oya tasks bulk-edit status --op-id <id>
   ```
4. Check if second-confirmation was signed (per Cedar `tenant-scope.cedar`):
   ```bash
   oya tasks bulk-edit audit-trail --op-id <id>
   ```

## Mitigation steps

### Step 1 — Per-tenant rate-limit

If bulk-op is legitimate but tenant overwhelming their own limit:
```bash
oya tasks rate-limit set --tenant <hashed-id> --resource bulk_edit --limit 100/sec --duration 1h --audit-reason "RB-bulk-edit-throttle"
```

### Step 2 — Surface "operation queued" to tenant

Tenant should see UI status: "queued, ~5 min remaining".

### Step 3 — Process bulk in 1000-task batches

The per-batch atomicity prevents full-tenant lockout; verify:
```promql
sum(rate(tasks_bulk_edit_batch_completed_total{op_id="<id>"}[1m]))
```

Expected: batches drain over 5-15 min.

### Step 4 — If bulk-op > 10k without second-confirmation

Cedar policy refuses; surface to tenant operator:
```bash
oya tasks bulk-edit refuse --op-id <id> --reason "second-confirmation required for >10k tasks" --audit-reason "RB-bulk-edit-throttle"
```

Operator must re-submit with confirmation flag.

### Step 5 — If sustained pattern

Escalate to ops-sre-reliability; consider scaling worker pool baseline.

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `tasks_bulk_edit_in_flight_count` | ≤ 1 | within 15 min |
| `tasks_bulk_edit_p95_ms` (per 100 tasks) | < 300ms | unchanged |
| Postgres connection pool utilisation | < 70% | within 5 min |

## Post-incident review

- Should default second-confirmation threshold be lowered from 10k to 5k?
- Should per-tenant rate-limit baseline scale with tenant_class usage caps and paid billing components?
- Was Hyrum surface for atomicity (1000-task batch) clear to tenant operator?

## Drills

- Quarterly: simulated 50k-task bulk-edit in synthetic tenant.

## References

- `failure-modes.md` FM-05.
- PRD AC-05 (bulk-update p95 ≤ 300ms).
- Cedar `tenant-scope.cedar` §"BULK-EDIT".
- `dashboards/throughput-and-engagement.json`.
