---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P01
title: Data Use Boundary + Tenancy Kernel
status: complete
purpose: Land the Data Use Boundary ADR (P0 prereq) and the tenant kernel that every other axis depends on.
---

# M01-P01 — Data Use Boundary + Tenancy Kernel

## Purpose
P0 prereq per [`../../../../../docs/PRD.md`](../../../../../docs/PRD.md) §6 constraint 8. Without this phase, no cloud/search/ads/workspace work may begin.

## Acceptance
- ADR-0008 (Data Use Boundary) Accepted; per-consent-tier data-class mapping published.
- `crates/oya-tenancy-kernel` ships immutable tenant identity/region binding plus engine-enforced row-level isolation contracts per ADR-0002/0006/0049; compatibility domain crates remain unaffected.
- `tenant.create` and `tenant.dsr.cascade` SPEC §2 rows green at `stable` tier.
- Cell-isolation evidence collected per cell (audit log + replay test).

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Data Use Boundary ADR-0008 authoring | complete | [`IP-001-data-use-boundary-adr.md`](IP-001-data-use-boundary-adr.md) |
| IP-002 | `oya-tenancy-kernel` final-shape contracts | complete | [`IP-002-tenant-kernel-contracts.md`](IP-002-tenant-kernel-contracts.md) |
| IP-003 | `dsr.cascade.execute` ≤30d cascade engine | complete | [`IP-003-dsr-cascade-engine.md`](IP-003-dsr-cascade-engine.md) |

## Estimated parallelism
3 agents in parallel after IP-001 ADR merge (ADR is doc-only, gates IP-002 + IP-003 which both consume the data-class mapping).

## Symbols-touched (high level)
`crates/oya-tenancy-kernel`, `crates/oya-tenancy-domain`, `crates/oya-data-boundary-kernel`, `docs/decisions/ADR-0008-data-use-boundary.md`, `docs/PRIVACY-PROGRAM.md` (consent-tier mapping table).

## Completion evidence
- IP-001: [`.omc/evidence/foundation/m01-p01-ip-001-data-use-boundary-adr.json`](../../../../../evidence/foundation/m01-p01-ip-001-data-use-boundary-adr.json)
- IP-002: [`.omc/evidence/foundation/m01-p01-ip-002-tenant-kernel-contracts.json`](../../../../../evidence/foundation/m01-p01-ip-002-tenant-kernel-contracts.json)
- IP-003: [`.omc/evidence/foundation/m01-p01-ip-003-dsr-cascade-engine.json`](../../../../../evidence/foundation/m01-p01-ip-003-dsr-cascade-engine.json)

## Agent-handoff
On phase complete, emit:
```
icm store -t context-oyatie -c "M01-P01 complete: Data Use Boundary ADR-0008 Accepted; tenant-kernel stable; tenant.dsr.cascade ≤30d demonstrated" -i critical -k "M01,P01,data-use-boundary,tenant-kernel,complete"
```
