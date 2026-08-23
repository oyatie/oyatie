---
doc_class: Runbook
title: Recalc storm throttle (1k+ users editing same formula chain)
microservice: sheets
severity: "Sev-2 (cross-tenant impact) / Sev-3 (per-tenant burst)"
status: Accepted
owner_team: ops-sre-reliability + axis-sheets + ops-security
date: 2026-05-17
related_artifacts:
  - app/sheets/threat-model.md §"T-D-02" + §"T-D-04" + §"T-D-06"
  - app/sheets/capacity-model.md
  - app/sheets/PRD.md §"Horizontal Scalability"
  - /specs/app/sheets.json §goals.scalability
doc_status: published
---

# Runbook: Recalc storm throttle

## Purpose

Sheets runs a per-cell recalc-engine + WS gateway + Postgres + Valkey cluster sized for `100K active workbook sessions per region` and `1M-cell recalc p95 ≤ 10s`. A hot formula dependency chain (shared model workbook with 1k+ concurrent editors) can cause recalc-engine queue saturation. This runbook provides throttling, quarantining, and recovery steps.

## Trigger

ONE of:

1. **`sheets_recalc_queue_depth > 100` for ≥ 5 min** on a single workbook OR cluster-wide.
2. **`sheets_recalc_p99_seconds > 5` for ≥ 5 min** (budget breach).
3. **`sheets_collab_op_published_total{workbook=<id>}` rate > 100/sec** for ≥ 5 min (op storm).
4. **`sheets_postgres_connection_pool_saturation > 0.9` for ≥ 5 min**.
5. **`sheets_valkey_memory_used_bytes / sheets_valkey_memory_max_bytes > 0.85`**.
6. **`sheets_recalc_engine_pod_cpu_pct > 70` AND HPA at max replicas**.
7. **`sheets_recalc_cycle_detected_total > 0`** — formula dep-graph cycle (refused but tenant-notified).

## Severity

- Single workbook, no impact on others: Sev-3.
- Cross-tenant impact (≥ 2 tenants seeing degradation): Sev-2.
- Cluster recalc capacity exhausted: Sev-1.
- Coordinated attack pattern: Sev-1; engage ops-security.

## Impact

- Recalc latency may exceed budget; saves queue at recalc-engine.
- Active editing may slow.
- New workbook opens may return 429.
- Tenant trust impact.

## Pre-checks

1. Identify burst source: `dashboards/recalc-engine-health.json` panel "queue depth by workbook top-N" + "recalcs/sec by tenant top-N".
2. Verify HPA state: `kubectl -n sheets get hpa recalc-engine-worker collab-crdt-worker cell-grid-rest`.
3. Verify Postgres health.
4. Verify Valkey memory.
5. Identify attack vector vs legitimate burst (e.g., end-of-quarter financial-modelling spike).

## Recovery Path A — Single-workbook burst (legitimate; e.g., 1k+ users in shared model)

| Step | Action |
|---|---|
| 1 | Confirm legitimacy: contact tenant via gtm-customer-success. |
| 2 | Apply per-workbook recalc-defer: defer non-hot-range recalc (recalc-engine config); hot-range edits prioritised. |
| 3 | Scale recalc-engine HPA: `kubectl -n sheets scale deployment/recalc-engine-worker --replicas 15`. |
| 4 | Monitor: verify recalc queue drains; revert defer config. |

## Recovery Path B — Single-tenant burst (suspected abuse)

| Step | Action |
|---|---|
| 1 | Engage ops-security + gtm-customer-success. |
| 2 | Apply per-tenant recalc-rate-limit: `cargo run -p dev-cli -- sheets rate-limit --tenant <h> --burst-multiplier 0.1x --duration 1h`. |
| 3 | Tenant sees 429 banner: "your account is rate-limited; contact support". |
| 4 | gtm-customer-success contacts tenant; investigate. |
| 5 | If confirmed abuse: per tenancy ToS, suspend account. |

## Recovery Path C — Formula dep-graph cycle detected

| Step | Action |
|---|---|
| 1 | Per ADR-SHEETS-0004, recalc-engine refuses cycle and emits `#CIRCULAR` error to affected cells. |
| 2 | Tenant notified inline; no on-call action required unless cycle rate spikes (indicates engine bug). |
| 3 | If cycle rate > 1/sec sustained: file bug; recalc-engine fuzz corpus likely missed a case. |

## Recovery Path D — Cluster recalc capacity exhausted (Sev-1)

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage ops-sre-reliability + capacity-planning. |
| 2 | Add cell-cluster nodes: `cargo run -p dev-cli -- cloud-iac scale-cell --pack <pack> --ms sheets --add-nodes 5 --component recalc-engine-worker`. |
| 3 | Re-balance recalc lease assignments. |
| 4 | Verify Postgres + Valkey headroom. |
| 5 | Update `capacity-model.md` baseline. |

## Recovery Path E — Slow-formula budget breach (single formula > 30s)

Cause: tenant authored an unusually expensive formula (e.g., huge VLOOKUP across uncached range).

| Step | Action |
|---|---|
| 1 | Per ADR-SHEETS-0004 slow-formula budget: any single recalc plan > 30s is killed + tenant notified. |
| 2 | Affected cells show `#SLOW!` error; tenant prompted to refactor formula. |
| 3 | If recurring on same formula pattern: surface to council-design-system for UX guidance. |

## Recovery Path F — Connected-sheets refresh storm

Cause: many tenants triggered connected-query refreshes simultaneously (e.g., scheduled refresh thundering herd).

| Step | Action |
|---|---|
| 1 | Verify external-source health: `sheets_connected_query_external_source_p99_seconds`. |
| 2 | Apply per-(tenant, source) refresh-rate-limit. |
| 3 | Stagger scheduled-refresh windows; tenant notified. |

## Verification

After recovery:
- `sheets_recalc_queue_depth` returns to baseline (≤ 20).
- `sheets_recalc_p99_seconds` within budget (≤ 1s for 100k-cell; ≤ 10s for 1M-cell).
- HPA at < 70% target; replicas stable.
- Postgres connection pool < 70% saturation.
- Valkey memory < 80% used.

## Post-incident updates

- Postmortem within 5 business days.
- If single-tenant burst legitimate: surface to `capacity-model.md` as new traffic pattern.
- If attack: tighten rate-limit defaults.
- If cluster too small: file capacity-expansion ticket.

## References

- `app/sheets/PRD.md` §"Horizontal Scalability" + §"Performance".
- `app/sheets/threat-model.md` T-D-02, T-D-04, T-D-06, T-D-08.
- `app/sheets/capacity-model.md`.
- `/specs/app/sheets.json` §goals.scalability.
- ADR-SHEETS-0004 (recalc-engine architecture).
- Google SRE Workbook ch. 21 (cascading failures).
