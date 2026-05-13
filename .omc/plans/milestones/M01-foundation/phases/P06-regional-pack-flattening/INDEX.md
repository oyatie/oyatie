---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P06
title: Regional Pack Architecture + Flattening Ratchet
status: stub
purpose: Ship the regional-pack contract (canonical seams + per-pack plug-in) and the architectural-flattening-target ratchet per ADR-0015.
---

# M01-P06 — Regional Pack Architecture + Flattening Ratchet

## Purpose
Per [`../../../../../docs/DESIGN.md`](../../../../../docs/DESIGN.md) §12 (regional packs) and ADR-0015 (flat-crates target). Every regulated surface declares its `regulatory_packs:` set; a regional pack supplies the per-jurisdiction implementation.

## Acceptance
- Regional pack ADR Accepted; `crates/oya-platform-regional-pack-kernel` shipped.
- Initial pack roster (per DESIGN §12.4): KR + JP + US + EU + IN + BR + KSA + UAE + ANZ + SG seam contracts published.
- `oya-foundry-fitness-flat-crates-guard` lane green: every workspace crate under `crates/oya-*`, every workspace crate has `registry/catalog/<crate>.yaml`, retired top-level roots stay absent, role-boundary graph validates.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Regional Pack ADR + kernel | stub | [`IP-001-regional-pack-adr-kernel.md`](IP-001-regional-pack-adr-kernel.md) |
| IP-002 | Flat-crates guard ratchet | stub | [`IP-002-flat-crates-guard.md`](IP-002-flat-crates-guard.md) |

## Estimated parallelism
2 agents in parallel; disjoint surface.

## Symbols-touched
`crates/oya-platform-regional-pack-kernel`, `crates/oya-foundry-fitness-flat-crates-guard-kernel`.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P06 complete: regional-pack kernel + flat-crates guard lane green; M01 acceptance gate ready" -i critical -k "M01,P06,regional-pack,flat-crates,M01-complete"
```
