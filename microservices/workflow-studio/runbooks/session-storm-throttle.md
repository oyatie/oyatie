---
doc_class: Runbook
title: Editor session storm throttle (per-tenant + cluster-wide DoS containment)
microservice: workflow-studio
severity: "Sev-2 (cross-tenant impact) / Sev-3 (per-tenant burst)"
status: Accepted
owner_team: ops-sre-reliability + axis-workflow + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/workflow-studio/threat-model.md §"T-D-01" + §"T-D-02" + §"T-D-05"
  - microservices/workflow-studio/capacity-model.md
  - microservices/workflow-studio/PRD.md §"Horizontal Scalability"
  - /specs/microservices/workflow-studio.json §goals.scalability
doc_status: published
---

# Runbook: Session storm throttle

## Purpose

Studio runs a per-cell WS gateway + Postgres + Valkey cluster sized for `100K active editor sessions per region`. A misbehaving tenant (or attack) can spam editor-open / save / collab-op requests, threatening cluster availability for legitimate tenants. This runbook provides throttling, quarantining, and recovery steps.

## Trigger

ONE of:

1. **`oya_workflow_studio_editor_session_open_total{tenant=<h>}` rate > 100/min** (10x normal) for ≥ 5 min.
2. **`oya_workflow_studio_collab_op_published_total{tenant=<h>}` rate > 1000/sec** (10x normal) for ≥ 5 min.
3. **`oya_workflow_studio_ws_gateway_pod_cpu_pct > 70` AND HPA at max replicas.**
4. **`oya_workflow_studio_postgres_connection_pool_saturation > 0.9` for ≥ 5 min.**
5. **`oya_workflow_studio_redis_memory_used_bytes / oya_workflow_studio_redis_memory_max_bytes > 0.85`.**
6. **`oya_workflow_studio_editor_session_429_total` rate > 100/min cluster-wide** (many tenants hitting cap; suggests cluster too small OR coordinated).
7. **`oya_workflow_studio_save_round_trip_seconds{quantile="0.99"} > 1.0` for ≥ 5 min** — likely save backpressure.

## Severity

- Single tenant burst, no impact on others: Sev-3.
- Cross-tenant impact (≥ 2 tenants seeing degradation): Sev-2.
- Cluster capacity exhausted, new tenants locked out: Sev-1.
- Coordinated attack pattern (multiple tenants with synchronized burst): Sev-1; engage ops-security.

## Impact

- New editor opens may return 429 (correct fair-share behavior per `threat-model.md` T-D-01).
- Active editing may slow (CRDT op merge p99 budget breached).
- Saves may queue at engine spec-store (cross-µservice backpressure).
- Tenant trust impact for legitimate tenants caught in collateral.

## Pre-checks

1. Identify burst source: `dashboards/editor-experience.json` panel "active sessions by tenant top-N" + "ops/sec by tenant top-N".
2. Verify HPA state: `kubectl -n workflow-studio get hpa collab-crdt-worker visual-canvas-rest`.
3. Verify Postgres health: `kubectl -n workflow-studio exec postgres-primary -- psql -c "SELECT count(*) FROM pg_stat_activity"`.
4. Verify Valkey memory: `kubectl exec redis-primary -- redis-cli INFO memory`.
5. Identify attack vector: legitimate burst (e.g., tenant migrating 1000 workflows simultaneously) OR malicious (single tenant ID pattern, synthetic-looking traffic)?

## Recovery Path A — Single-tenant burst (legitimate)

Cause: tenant is doing a bulk operation (migration, data import).

| Step | Action |
|---|---|
| 1 | Confirm legitimacy: contact tenant via gtm-customer-success. |
| 2 | Apply per-tenant elevated rate-limit (temporary): `cargo run -p oya-dev-cli -- workflow-studio rate-limit --tenant <h> --burst-multiplier 5x --duration 2h`. |
| 3 | Scale WS gateway HPA (if not already auto-scaling) to handle burst: `kubectl -n workflow-studio scale deployment/collab-crdt-worker --replicas 30`. |
| 4 | Monitor: verify tenant burst completes; revert rate-limit elevation. |

## Recovery Path B — Single-tenant burst (suspected abuse)

Cause: tenant traffic pattern looks abusive (synthetic load OR known-bad signature).

| Step | Action | Time |
|---|---|---|
| 1 | Engage ops-security + gtm-customer-success. | ≤ 10 min |
| 2 | Apply per-tenant throttle: `cargo run -p oya-dev-cli -- workflow-studio rate-limit --tenant <h> --burst-multiplier 0.1x --duration 1h`. | ≤ 5 min |
| 3 | Tenant sees 429 banner: "your account is rate-limited; contact support". | – |
| 4 | gtm-customer-success contacts tenant; investigate use-case + abuse-policy violation. | per priority |
| 5 | If confirmed abuse: per tenancy ToS + DPA, suspend account; engage legal. | per priority |

## Recovery Path C — Coordinated multi-tenant attack (Sev-1)

Cause: multiple tenants synchronously burst — likely external account-compromise pattern OR botnet.

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage ops-security + ops-sre-reliability + axis-workflow. |
| 2 | Activate WAF rule set: `cargo run -p oya-dev-cli -- waf activate-ruleset --ms workflow-studio --ruleset emergency-ddos-v1`. |
| 3 | Apply per-IP rate-limit at WAF (10 req/s per IP). |
| 4 | Per-tenant analysis: which tenant accounts are affected? Force OIDC re-auth on suspicious accounts. |
| 5 | If account compromise: reset tenant operator credentials; force MFA re-enrollment; audit. |
| 6 | Tenant notification per pack regulatory if accounts compromised. |

## Recovery Path D — Cluster capacity exhausted (Sev-1)

Cause: legitimate tenant growth has outpaced cluster sizing; new editor opens fail.

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage ops-sre-reliability + capacity-planning. |
| 2 | Add cell-cluster nodes: `cargo run -p oya-dev-cli -- cloud-iac scale-cell --pack <pack> --ms workflow-studio --add-nodes 5`. |
| 3 | Re-balance WS gateway lease assignments (consistent-hash auto-spreads). |
| 4 | Verify Postgres has connection-pool headroom; tune `max_connections` if necessary. |
| 5 | Verify Valkey capacity (if needed: add Valkey cluster nodes). |
| 6 | Update `capacity-model.md` with the new baseline; this informs next cell-cluster provisioning. |

## Recovery Path E — Save backpressure from engine

Cause: Studio's save round-trip slowed because workflow-engine spec-store is itself backpressured.

| Step | Action |
|---|---|
| 1 | Verify engine spec-store: `microservices/workflow-engine/dashboards/spec-store-health.json`. |
| 2 | If engine slow: engage workflow-engine on-call per `microservices/workflow-engine/runbooks/spec-store-perf.md`. |
| 3 | Studio side: increase save-buffer + retry policy (already exponential backoff); editor UX shows "save queued, retrying"; no data loss. |
| 4 | Tenants see slight delay; not Studio's fault but Studio's signal will reflect upstream pressure. |

## Recovery Path F — WS gateway pod evicted under memory pressure

Cause: a WS gateway pod is evicted by Kubernetes; active sessions on that pod see WS close; clients auto-reconnect (per `threat-model.md` T-D-08).

| Step | Action |
|---|---|
| 1 | Verify auto-reconnect: clients reconnect within ≤ 5s; CRDT state preserved (Valkey ephemeral + Postgres seal-deltas). |
| 2 | Tune pod memory limits if eviction recurring: `kubectl -n workflow-studio edit deployment collab-crdt-worker` → bump memory limit. |
| 3 | If recurring during low-load: memory-leak suspect; engage workflow-studio team. |

## Verification

After recovery:
- `oya_workflow_studio_editor_session_open_total` rate returns to baseline (≤ 10/min per tenant).
- `oya_workflow_studio_collab_op_published_total` rate within budget.
- WS gateway HPA at < 70% target; replicas stable.
- Postgres connection pool < 70% saturation.
- Valkey memory < 80% used.
- `editor_session_429_total` rate < 1/min cluster-wide.
- Save round-trip p99 returns to budget.

## Post-incident updates

- Postmortem within 5 business days (immediate for Sev-1).
- If single-tenant burst legitimate: surface to capacity-model.md as new traffic pattern.
- If attack: WAF rule permanently active; tighten rate-limit defaults.
- If cluster too small: file capacity-expansion ticket; update `capacity-model.md` baselines.
- Per-pack tenant comms if collateral damage to legitimate tenants.

## References

- `microservices/workflow-studio/PRD.md` §"Horizontal Scalability" + §"Performance".
- `microservices/workflow-studio/threat-model.md` T-D-01, T-D-02, T-D-05, T-D-08.
- `microservices/workflow-studio/capacity-model.md`.
- `/specs/microservices/workflow-studio.json` §goals.scalability.
- Google SRE Workbook ch. 21 (Addressing cascading failures).
- AWS Well-Architected Framework — Reliability Pillar (graceful degradation).
- Cloudflare DDoS protection patterns — `developers.cloudflare.com/ddos-protection/`.
