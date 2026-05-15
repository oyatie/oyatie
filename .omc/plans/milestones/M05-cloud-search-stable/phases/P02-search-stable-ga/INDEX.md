---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M05-P02
title: Search-Stable GA (Crawler + Freshness + KG + SERP + Sponsored-Slot Infra)
status: stub
purpose: Take Search axis to public GA; public web search with crawler + freshness + KG + SERP, sponsored-slot infrastructure ready (serving still off).
---

# M05-P02 — Search-Stable GA

## Purpose
Per [`../../../../../docs/ROADMAP.md`](../../../../../docs/ROADMAP.md) §2.8 W-Search-Stable.

## Acceptance
- Public web search live with crawler + freshness + KG + SERP.
- Public Search SLO met (per [`../../../../../docs/SLO-CATALOG.md`](../../../../../docs/SLO-CATALOG.md)).
- KR ranking quality bar met (council-set; per [`../../../../../docs/PRD.md`](../../../../../docs/PRD.md) §4.2 row).
- Sponsored-slot infrastructure ready; ad serving still off until M06.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Public web crawler + freshness pipeline | stub | [`IP-001-crawler-freshness.md`](IP-001-crawler-freshness.md) |
| IP-002 | Knowledge graph + SERP rendering | stub | [`IP-002-kg-serp.md`](IP-002-kg-serp.md) |
| IP-003 | Sponsored-slot infrastructure (serving off) | stub | [`IP-003-sponsored-slot-infra.md`](IP-003-sponsored-slot-infra.md) |

## Estimated parallelism
3 agents.

## Symbols-touched
`crates/oya-search-{crawler,parser,index-inverted,rank,query,serp,kg}-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M05-P02 complete: Search-Stable GA; public SLO met; KR ranking quality bar met; sponsored-slot ready" -i critical -k "M05,P02,search-stable,ga,complete"
```
