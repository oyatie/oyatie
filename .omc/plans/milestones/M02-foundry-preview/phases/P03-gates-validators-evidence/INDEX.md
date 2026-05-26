---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M02-P03
title: Gates / Validators / Evidence Templates
status: complete
purpose: Ship the foundry-fitness lane suite + P00-08 evidence validator that gates every Phase 00 merge.
---

# M02-P03 — Gates / Validators / Evidence

## Purpose
Per [`../../../../../.omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md`](../../../../../.omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md) §D. The lane suite enforces every other principle.

## Acceptance
- `scripts/validate-foundry-phase00-evidence.mjs` green; rejects missing crates/tests/adapters, raw-secret exposure, Clean-Architecture boundary violation.
- ≥ 7 fitness lanes green on `main`: claim-ceiling, authority-cohesion, bypass, pr-traceability, pre-push, quality-lane, cohesion-fitness.
- Claim-ceiling ratchet log: one WARN → BLOCK promotion per wave; row in [`../../../../../docs/CHANGELOG.md`](../../../../../docs/CHANGELOG.md).

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Phase 00 evidence validator (`validate-foundry-phase00-evidence.mjs`) | complete | [`IP-001-phase00-evidence-validator.md`](IP-001-phase00-evidence-validator.md) |
| IP-002 | Foundry-fitness lane ratchet (7 lanes WARN→BLOCK) | complete | [`IP-002-foundry-fitness-lane-ratchet.md`](IP-002-foundry-fitness-lane-ratchet.md) |
| IP-003 | ADR template + foundation-bypass ledger | complete | [`IP-003-adr-template-bypass-ledger.md`](IP-003-adr-template-bypass-ledger.md) |

## Estimated parallelism
3 agents; IP-001/002/003 share zero source files.

## Symbols-touched
`scripts/validate-foundry-phase00-evidence.mjs`, `crates/oya-governance-{claim-ceiling,authority-cohesion,bypass,pr-traceability,pre-push,quality-lane,cohesion-fitness}-kernel`.

## Agent-handoff
```
icm store -t context-oyatie -c "M02-P03 complete: validators + 7 lanes ratcheted; foundation-bypass ledger live" -i critical -k "M02,P03,gates,validators,evidence,complete"
```
