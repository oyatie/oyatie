---
purpose: "Publish public SLA (Cloud 99.99%) and SLO (Search per catalog) commitments backed by measured evidence."
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M05-P04
title: Cloud SLA + Search SLO Public Commitment
status: stub
purpose: Publish public SLA (Cloud 99.99%) and SLO (Search per catalog) commitments backed by measured evidence.
---

# M05-P04 — SLA + SLO Public Commitment

## Purpose
Per [`../../../../../docs/SLO-CATALOG.md`](../../../../../docs/SLO-CATALOG.md). Public commitment requires measured evidence and uphold-or-credit policy.

## Acceptance
- Cloud public SLA at 99.99% with uphold-or-credit policy published.
- Search public SLO per [`../../../../../docs/SLO-CATALOG.md`](../../../../../docs/SLO-CATALOG.md) committed.
- Monthly SLA report auto-generated and published.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Cloud SLA public commitment + uphold-or-credit policy | stub | [`IP-001-cloud-sla.md`](IP-001-cloud-sla.md) |
| IP-002 | Search SLO public commitment | stub | [`IP-002-search-slo.md`](IP-002-search-slo.md) |

## Estimated parallelism
2 agents.

## Symbols-touched
`docs/SLO-CATALOG.md`, `crates/oya-ops-sre-sla-report-app`, `docs/legal/sla.md`.

## Agent-handoff
```
icm store -t context-oyatie -c "M05-P04 complete: Cloud 99.99% SLA + Search SLO publicly committed; M05 acceptance gate ready" -i critical -k "M05,P04,sla,slo,M05-complete"
```
