---
doc_class: CostBudget
template_id: TPL-COST-BUDGET
microservice: calendar
status: Accepted
date: 2026-05-17
owner_team: axis-calendar + finops
related_adrs: [ADR-0117, ADR-0131]
doc_status: published
---

# Cost Budget — calendar µservice

## Purpose

Define per-tenant and per-cell unit economics for calendar. FinOps gate: per-tenant monthly cost per active calendar / event / cross-tenant lookup must stay within budget; cross-budget breach holds promotion + auto-throttles.

## Unit economics (per tenant per month, baseline pricing OCI ap-seoul-1, 2026-05)

| Unit | Baseline cost | Notes |
|---|---|---|
| Active calendar (1 user) | $0.18 / mo | Postgres row + RLS overhead + index footprint |
| Event row (year-of-active-data) | $0.0004 / event-mo | Postgres + tenant-DEK envelope encryption overhead |
| Cross-tenant availability lookup | $0.00002 / lookup | Valkey cache + cross-pack mesh hop |
| Recurrence expansion (1y horizon) | $0.0008 / expansion | worker CPU + memory |
| Room booking | $0.0001 / booking | Postgres FOR UPDATE + audit emit |
| Invitation send (1 attendee) | $0.0003 / invitation | event-bus + mail handoff |
| .ics import (1k events) | $0.05 / job | streaming parse + write fanout |
| .ics export (1k events) | $0.02 / job | streaming read + emit |
| CalDAV PROPFIND | $0.00001 / req | Postgres index read |

## Per-cell cost envelope (steady-state, p50 load)

| Component | Baseline / cell / mo | Notes |
|---|---|---|
| Postgres (event-store; 3-replica HA + per-tenant RLS) | $4,200 | OCI VM.Standard3.Flex 8 OCPU × 3 + 2TB persistent block + S3 backup |
| Valkey (availability cache; cluster mode 3-shard) | $900 | OCI Caching Service standard tier |
| Kubernetes nodes (rest + worker pods) | $2,100 | OCI VM.Standard.E5.Flex × 6 baseline |
| IANA tzdata refresh job | $5 | CronJob trivial |
| Egress (cross-tenant availability + invitation mail handoff) | $400 | OCI egress to mail-µservice intra-region |
| Cross-pack mesh egress (when SCC-gated cross-pack queries) | $50 | per-pack pair |
| Observability fan-out (metrics + logs + traces) | $300 | per the observability µservice envelope |
| Backup storage (Postgres snapshots + .ics export retention) | $250 | S3-compatible cold-tier |
| **Total per cell baseline** | **$8,205 / mo** | sized for 100k active calendars |

## Per-tenant breakeven

| Tenant tier | Monthly bill | Notes |
|---|---|---|
| Free | $0; 10 active calendars, 100 events, 100 lookups/mo | hard quota, cost-controlled |
| Starter | $5; 50 active calendars, 5k events, 10k lookups | covers infra + 30% margin |
| Pro | $25; 500 active calendars, 100k events, 200k lookups | covers infra + 40% margin |
| Enterprise | custom + per-seat | covers infra + 50% margin + SLA premium |

Breakeven: per-tenant gross margin ≥ 30% at Starter; ≥ 40% at Pro; ≥ 50% at Enterprise.

## Cost governance gates

| Gate | Threshold | Action |
|---|---|---|
| Per-tenant monthly cost > 80% of tier bill | warn FinOps | review for plan upgrade |
| Per-tenant monthly cost > 100% of tier bill | warn FinOps + tenant | over-quota notification + offer upgrade |
| Per-tenant monthly cost > 150% of tier bill | throttle (rate-limit lowered to baseline-tier limits) | auto-throttle |
| Per-cell total cost > 130% of baseline | hold promotion + finops review | review |
| Cross-pack egress > 5% of cell cost | finops review | identify cause |
| Recurrence-expansion worker cost > 20% of cell cost | engineering review | optimisation needed |

## Cost-meter implementation

Cost-meter emitted as Mimir metric per ADR-0123 + finops standards:

| Metric | Cardinality | Labels |
|---|---|---|
| `calendar_tenant_cost_dollars_per_month` | per-tenant | `tenant_id_hashed`, `tier`, `pack_tag` |
| `calendar_unit_cost_dollars` | per-unit | `unit_type` (calendar/event/lookup/expansion/booking/invitation/ics_import/ics_export/caldav_req), `pack_tag` |
| `calendar_cell_cost_dollars_per_month` | per-cell | `pack_tag`, `cell_id` |
| `calendar_egress_dollars` | per-cross-pack-pair | `from_pack`, `to_pack` |

## Optimisations identified at design time

| Optimisation | Estimated savings | Status |
|---|---|---|
| Recurrence expansion cache (Valkey; window-hashed) | 60% on recurrence cost | Implemented in PHASE-01 CS-04 |
| Cross-tenant availability cache TTL ≤ 60s + single-flight | 70% on cross-tenant lookup cost | Implemented in PHASE-01 CS-05 |
| Postgres per-tenant partition pruning | 40% on read latency + 25% on storage | Implemented in PHASE-01 CS-02 |
| .ics streaming parse (not full materialisation) | 90% on memory; required for 100k events | Implemented in PHASE-01 CS-08 |
| Pre-warm pod pool (5 standby) | sub-1s cold-start vs. 5s on-demand | Implemented in PHASE-01 CS-03 |

## FinOps reporting cadence

- Per-tenant cost report: monthly.
- Per-cell cost report: monthly.
- Cross-pack egress report: weekly.
- Cost-anomaly alert: daily (≥ 30% deviation from rolling 7d baseline).

## References

- ADR-0117: cloud-native infrastructure.
- ADR-0131: per-microservice layout.
- `capacity-model.md`, `multi-region.md`.
- OCI ap-seoul-1 pricing (2026-05).
