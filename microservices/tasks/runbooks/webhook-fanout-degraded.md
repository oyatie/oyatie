---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: tasks
runbook_id: RB-webhook-fanout-degraded
status: Accepted
date: 2026-05-17
owner_team: axis-tasks + ops-sre-reliability
severity_applicable: [Sev-2]
related_failure_modes: [FM-06]
related_dashboards: [throughput-and-engagement]
doc_status: published
---

# Runbook — Webhook Fanout Degraded

## When this runbook fires

- Per-subscription `tasks_webhook_circuit_breaker_open_count > 0` for > 5 min, OR
- Per-subscription error-rate > 50% over 1 min, OR
- `tasks_webhook_fanout_p95_ms > 200ms` for > 5 min (PRD AC budget breach).

## Symptoms

- Webhook subscriber's destination is unhealthy (500s / DNS failure / SSL expired).
- Per-subscriber circuit-breaker opens; events queued for retry.
- Other subscribers' fanout unaffected (per-subscription isolation).

## Probable causes

1. Subscriber's webhook destination down / unhealthy.
2. SSL certificate expired on subscriber's endpoint.
3. DNS resolution failure for subscriber's hostname.
4. Subscriber's endpoint returns slow responses (timing out at 30s).
5. Per-tenant webhook subscription cap exceeded (per `threat-model.md` T-D-05).

## Triage (within 30 min)

1. Acknowledge OnCall page.
2. Identify affected subscription(s):
   ```promql
   topk(5, sum by (subscription_id) (rate(tasks_webhook_failed_total[5m])))
   ```
3. Check subscription destination health:
   ```bash
   oya tasks webhook test --subscription <id>
   ```
4. Check circuit-breaker state:
   ```bash
   oya tasks webhook circuit-state --subscription <id>
   ```

## Mitigation steps

### Step 1 — Auto circuit-breaker (already active)

Confirm exponential backoff + circuit-breaker engaged:
```promql
tasks_webhook_circuit_breaker_state{subscription_id="<id>"} == "open"
```

### Step 2 — Tenant operator notification

After 1h without success, surface to tenant via OnCall:
```bash
oya tasks webhook notify-tenant --subscription <id> --reason "circuit-breaker-open" --audit-reason "RB-webhook-fanout-degraded"
```

Tenant operator can:
- Update webhook destination URL.
- Renew SSL certificate.
- Mark subscription `held` (pause delivery).

### Step 3 — Mark subscription `held` if abandoned

```bash
oya tasks webhook hold --subscription <id> --audit-reason "RB-webhook-fanout-degraded"
```

Queued events accumulate but are NOT dropped; tenant can resume.

### Step 4 — Per-tenant subscription cap check

If tenant has 50+ subscriptions (cap per T-D-05):
```bash
oya tasks webhook list --tenant <hashed-id> --count
```

Surface to tenant: "you're at cap; remove unused subscriptions to add more."

### Step 5 — If sustained subscriber outage

After 24h without recovery, escalate:
- Tenant operator + customer-success engagement.
- Optional: tenant operator may export queued events via `oya tasks webhook drain --subscription <id> --to-file /tmp/queued.jsonl`.

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| Per-subscription circuit-breaker state | closed | once subscriber healthy |
| `tasks_webhook_fanout_p95_ms` | < 200ms | within 5 min |
| Per-subscription error-rate | < 5% | within 5 min |

## Post-incident review

- Should default circuit-breaker timeout be lowered from 30s?
- Should subscription cap be raised for paid tenant_class workloads?
- Should we add tenant-facing "subscription health" dashboard?

## Drills

- Quarterly: simulated subscriber outage (HTTP 500 + slow response) in synthetic tenant.

## References

- `failure-modes.md` FM-06.
- PRD §"Performance" (webhook-fire p95 ≤ 200ms).
- `dashboards/throughput-and-engagement.json`.
- Hyrum #4 + #5 in `migration-from-connect.md` (webhook ordering + notification timing observable).
