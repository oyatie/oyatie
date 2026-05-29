---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: docs
runbook_id: RB-editor-session-storm-throttle
status: Accepted
date: 2026-05-17
owner_team: axis-docs + ops-sre-reliability
severity_applicable: [Sev-2, Sev-1]
related_failure_modes: [FM-07, FM-13]
doc_status: published
---

# Runbook — Editor-session storm throttle

## When this runbook fires

- `oya_docs_ws_lease_count` > 80% of cell max.
- `oya_docs_ws_lease_acquisition_p99_seconds > 1`.
- `pg_connection_pool_utilisation > 85%`.
- Tenant reports inability to open editor; sessions queued or refused.

## Severity

- Single-tenant storm without other-tenant impact: Sev-3.
- Multi-tenant impact (DB headroom degraded): Sev-2.
- Coordinated DoS suspected: Sev-1 (page ops-security).

## Symptoms

- WS upgrade requests stalling or 429ing.
- Postgres connection pool exhaustion (cascading 5xx).
- Cell-wide saturation; HPA struggles to scale fast enough.

## Probable causes

1. Malicious tenant or misbehaving SDK retry loop.
2. Legitimate high-volume authoring (e.g., 1000-person workshop on shared docs).
3. CRDT op-log replay storm post-restart.
4. Distributed-attacker DoS.

## Triage (within 15 min)

1. Acknowledge page.
2. Identify top tenants by WS lease count:
   ```promql
   topk(5, sum by (tenant_id) (oya_docs_ws_lease_count))
   ```
3. Identify if storm is one-tenant or distributed.
4. Check DB connection pool: `pg_stat_activity` count vs `max_connections`.

## Section A — Single-tenant rate-limit

```bash
oya docs rate-limit set --tenant <hashed-id> --resource ws_session_open --limit 100/min --duration 1h --audit-reason "RB-editor-session-storm-throttle"
oya docs rate-limit set --tenant <hashed-id> --resource crdt_op_publish --limit 1000/min --duration 1h --audit-reason "RB-editor-session-storm-throttle"
```

## Section B — Scale up WS gateway + DB

```bash
kubectl scale deployment -n docs oya-docs-collab-crdt-worker --replicas=200
kubectl scale deployment -n docs oya-docs-document-store-rest --replicas=100
```

Verify within 10 min:
```promql
sum(oya_docs_ws_lease_acquisition_p99_seconds)
```

Expected: trend toward < 1s.

## Section C — Drain stuck leases

If leases are accumulating without churn:
```bash
oya docs lease drain --pack <pack> --stale-older-than 1h
```

This evicts leases whose `last_activity_at` is older than the threshold.

## Section D — DB headroom recovery

If pg_connection_pool > 85%:
```bash
kubectl set env deployment/oya-docs-document-store-rest -n docs PG_POOL_SIZE_PER_POD=20
kubectl rollout restart deployment/oya-docs-document-store-rest -n docs
```

Effectively reduces per-pod pool fan-out; relies on more pods (Section B HPA scale).

## Section E — Cedar policy refusal for malicious tenant

If ops-security confirms abuse:
```bash
oya docs policy deny --tenant <hashed-id> --action ws_session_open --duration 24h --audit-reason "suspected-abuse"
```

## Section F — Distributed DoS

| Step | Action |
|---|---|
| 1 | Engage ops-security. |
| 2 | WAF rule update for the attack signature. |
| 3 | Per-IP rate-limit at ingress. |
| 4 | If recurring: enable CAPTCHA on WS upgrade for the affected pack. |

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `oya_docs_ws_lease_count` per-tenant | < 1k for starter, < 10k for pro/enterprise | within 15 min |
| `oya_docs_ws_lease_acquisition_p99_seconds` | < 1s | within 15 min |
| `pg_connection_pool_utilisation` | < 70% | within 30 min |
| Tenant smoke-test (open editor) | succeeds | yes |

## Post-incident review

- Was the per-tenant rate-limit baseline appropriate?
- Should WS gateway HPA scale faster (lower stabilization window)?
- Update threat-model.md T-D-01 mitigation if needed.
- If recurring legitimate use: re-tune tier-quotas.

## Drills

- Bi-annual simulated editor-session storm in staging.
- Verify rate-limit cuts in correctly + WS gateway scales out as expected.

## References

- `failure-modes.md` FM-07, FM-13.
- `threat-model.md` T-D-01.
- `dashboards/editor-experience.json`, `collab-health.json`.
- `policy/tenant-scope.cedar`.
- Google SRE Workbook ch. 21 (handling overload).
