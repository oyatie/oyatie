---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M06-P04
title: Analytics Warehouse + Streaming + DP-Bounded Reports
status: stub
purpose: Ship the analytics axis substrate: event ingestion, warehouse, streaming, DP-bounded reports.
---

# M06-P04 — Analytics Warehouse + Streaming

## Purpose
Per [`../../../../../docs/SPEC.md`](../../../../../docs/SPEC.md) §9 (Axis 7) and [`../../../../../docs/DESIGN.md`](../../../../../docs/DESIGN.md) §13.7.

## Acceptance
- `analytics.event.ingest`, `analytics.warehouse.query`, `analytics.streaming.subscribe`, `analytics.report.dp-bounded` SPEC §9 rows green.
- Differential-privacy-bounded reports for cross-tenant aggregates (privacy isolation per Data Use Boundary).

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Analytics event ingestion + warehouse | stub | [`IP-001-event-warehouse.md`](IP-001-event-warehouse.md) |
| IP-002 | Streaming + DP-bounded reports | stub | [`IP-002-streaming-dp-reports.md`](IP-002-streaming-dp-reports.md) |

## Estimated parallelism
2 agents.

## Symbols-touched
`crates/oya-analytics-{event,warehouse,streaming,report,dp}-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M06-P04 complete: analytics warehouse + streaming + DP-bounded reports stable; M06 acceptance gate ready" -i critical -k "M06,P04,analytics,M06-complete"
```
