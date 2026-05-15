---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M04-P04
title: Evidence Collection + Retention Measurement + Audit Pack
status: stub
purpose: Continuous evidence collection during pilot operation; retention measurement over 8 weeks; regulator audit pack on first request.
---

# M04-P04 — Evidence + Retention + Audit Pack

## Purpose
Per [`../../../../../docs/PRD.md`](../../../../../docs/PRD.md) §4.1 metrics. Without measured evidence, M04 acceptance gate cannot pass.

## Acceptance
- ≥ 50K Foundry agent runs/week at ≥ 99.5% success during pilot window.
- 100% audit-chain evidence completeness on regulated capability invocations.
- Pilot retention ≥ 80% over 8 weeks measured.
- Zero tenant-data egress without consent receipt (hard zero).
- Regulator-facing evidence-pack regeneration ≤ 4 hours from request.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Continuous evidence collection pipeline | stub | [`IP-001-evidence-pipeline.md`](IP-001-evidence-pipeline.md) |
| IP-002 | Retention measurement + KPI dashboard | stub | [`IP-002-retention-kpi.md`](IP-002-retention-kpi.md) |
| IP-003 | Regulator audit-pack generator | stub | [`IP-003-audit-pack-generator.md`](IP-003-audit-pack-generator.md) |

## Estimated parallelism
3 agents in parallel.

## Symbols-touched
`crates/oya-platform-audit-chain-worker::collect_evidence`, `crates/oya-platform-metering-app::retention_kpi`, `crates/oya-ops-compliance-evidence-pack-app::generate`.

## Agent-handoff
```
icm store -t context-oyatie -c "M04-P04 complete: 8-week retention ≥80%; ≥50K runs/week ≥99.5%; audit-pack regenerates ≤4h; M04 acceptance gate ready" -i critical -k "M04,P04,evidence,retention,audit,M04-complete"
```
