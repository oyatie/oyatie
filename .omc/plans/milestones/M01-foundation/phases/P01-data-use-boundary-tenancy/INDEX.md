---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P01
title: Data Use Boundary + Tenancy Kernel
status: stub
purpose: Land the Data Use Boundary ADR (P0 prereq) and the tenant kernel that every other axis depends on.
---

# M01-P01 — Data Use Boundary + Tenancy Kernel

## Purpose
P0 prereq per [`../../../../../docs/PRD.md`](../../../../../docs/PRD.md) §6 constraint 8. Without this phase, no cloud/search/ads/workspace work may begin.

## Acceptance
- ADR-0008 (Data Use Boundary) Accepted; per-consent-tier data-class mapping published.
- `crates/oya-platform-tenant-kernel` + `-domain` + `-app` + `-api` ship with engine-enforced row-level isolation per ADR-0006.
- `tenant.create` and `tenant.dsr.cascade` SPEC §2 rows green at `stable` tier.
- Cell-isolation evidence collected per cell (audit log + replay test).

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Data Use Boundary ADR-0008 authoring | stub | [`IP-001-data-use-boundary-adr.md`](IP-001-data-use-boundary-adr.md) |
| IP-002 | `oya-platform-tenant-kernel` final-shape contracts | stub | [`IP-002-tenant-kernel-contracts.md`](IP-002-tenant-kernel-contracts.md) |
| IP-003 | `tenant.dsr.cascade` ≤30d cascade engine | stub | [`IP-003-dsr-cascade-engine.md`](IP-003-dsr-cascade-engine.md) |

## Estimated parallelism
3 agents in parallel after IP-001 ADR merge (ADR is doc-only, gates IP-002 + IP-003 which both consume the data-class mapping).

## Symbols-touched (high level)
`crates/oya-platform-tenant-{kernel,domain,app,api}-*`, `docs/decisions/ADR-0008-data-use-boundary.md`, `docs/PRIVACY-PROGRAM.md` (consent-tier mapping table).

## Agent-handoff
On phase complete, emit:
```
icm store -t context-oyatie -c "M01-P01 complete: Data Use Boundary ADR-0008 Accepted; tenant-kernel stable; tenant.dsr.cascade ≤30d demonstrated" -i critical -k "M01,P01,data-use-boundary,tenant-kernel,complete"
```
