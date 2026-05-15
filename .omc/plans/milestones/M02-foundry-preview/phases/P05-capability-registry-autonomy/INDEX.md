---
purpose: Ship the capability registry, Cedar-policy autonomy ceiling, audit-chain emission per invocation, and RAG endpoint exposure.
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M02-P05
title: Capability Registry + Autonomy Ceiling + RAG Endpoint
status: complete
purpose: Ship the capability registry, Cedar-policy autonomy ceiling, audit-chain emission per invocation, and RAG endpoint exposure.
---

# M02-P05 — Capability Registry + Autonomy Ceiling + RAG

## Purpose
Per [`../../../../../docs/ROADMAP.md`](../../../../../docs/ROADMAP.md) §2.2 W-Foundry-Preview gate. ≥ 50 capabilities published; T1-T4 autonomy tiers enforced.

## Acceptance
- Capability registry online with ≥ 50 capabilities published.
- Autonomy ceiling Cedar policy + runtime check operational; T4 disabled by default for actuation.
- Evidence emission per capability invocation (ADR-0003).
- RAG endpoint exposed to Foundry-internal capabilities.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Capability registry kernel + publish surface | complete | [`IP-001-capability-registry.md`](IP-001-capability-registry.md) |
| IP-002 | Autonomy ceiling Cedar policy + runtime check | complete | [`IP-002-autonomy-ceiling.md`](IP-002-autonomy-ceiling.md) |
| IP-003 | RAG endpoint exposed to Foundry-internal capabilities | complete | [`IP-003-rag-endpoint.md`](IP-003-rag-endpoint.md) |

## Estimated parallelism
3 agents in parallel; disjoint crate suffix.

## Symbols-touched
`crates/oya-foundry-capability-registry-*`, `crates/oya-foundry-autonomy-ceiling-*`, `crates/oya-foundry-rag-endpoint-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M02-P05 complete: capability registry ≥50; autonomy ceiling enforced; RAG endpoint live; M02 acceptance gate ready" -i critical -k "M02,P05,capability-registry,autonomy,rag,M02-complete"
```
