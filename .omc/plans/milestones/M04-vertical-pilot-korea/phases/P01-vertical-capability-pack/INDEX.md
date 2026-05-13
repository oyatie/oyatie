---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M04-P01
title: Vertical Election + Capability Pack Authoring
status: stub
purpose: Council elects pilot vertical; Foundry-authored capability pack ships for that vertical.
---

# M04-P01 — Vertical Capability Pack

## Purpose
Per [`../../../../../docs/PRD.md`](../../../../../docs/PRD.md) §8 question 4 (vertical-corporate likely). Foundry authors the per-vertical capability pack using Phase 02 self-hosting loop.

## Acceptance
- Council resolution on vertical election logged in `.omc/plans/open-questions.md`.
- Vertical capability pack contains: per-vertical entities (`oya-vertical-<elected>-*-kernel`), workflows (`workflow.definition.publish` rows per vertical), regulatory binding stubs.
- Pack signs via Cosign per ADR-0039.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Council resolution + vertical election | stub | [`IP-001-council-resolution.md`](IP-001-council-resolution.md) |
| IP-002 | Capability pack kernel (`oya-vertical-<elected>-pack-kernel`) | stub | [`IP-002-capability-pack-kernel.md`](IP-002-capability-pack-kernel.md) |
| IP-003 | Per-vertical workflows + entity definitions | stub | [`IP-003-vertical-workflows.md`](IP-003-vertical-workflows.md) |

## Estimated parallelism
After IP-001 resolution, IP-002 + IP-003 run in parallel (2 agents).

## Symbols-touched
`crates/oya-vertical-<elected>-pack-kernel`, `crates/oya-vertical-<elected>-{<sub-domain>}-*`, `docs/products/vertical-<elected>/PRD.md`.

## Agent-handoff
```
icm store -t context-oyatie -c "M04-P01 complete: vertical <elected> capability pack shipped; Cosign-signed" -i critical -k "M04,P01,vertical-pack,complete"
```
