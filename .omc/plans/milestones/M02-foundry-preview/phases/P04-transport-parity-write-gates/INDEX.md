---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M02-P04
title: Transport Parity + Write-Gate Foundations
status: complete
purpose: Same use-case ports across REST / GraphQL / SSE / WebSocket (Phase 00); foundations for gRPC / Webhook / Kafka write-gates (Phase 05).
---

# M02-P04 — Transport Parity + Write-Gate Foundations

## Purpose
Per [`../../../../../.omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md`](../../../../../.omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md) §E. Transport parity is a structural cohesion invariant.

## Acceptance
- Same use-case invoked via REST + GraphQL + SSE + WebSocket produces byte-identical audit events (modulo transport metadata).
- ADR-0XXX `Foundry write-gate foundations (Phase 05 contract)` Accepted.
- Public OpenAPI 3.2 sources at `contracts/openapi/foundry/{account-v1,session-v1,usage-v1,route-v1}.yaml`.
- AsyncAPI at `contracts/asyncapi/foundry/foundry-events-v1.yaml`.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | REST + GraphQL API transports | complete | [`IP-001-rest-graphql-transports.md`](IP-001-rest-graphql-transports.md) |
| IP-002 | SSE + WebSocket subscription transports | complete | [`IP-002-sse-websocket-transports.md`](IP-002-sse-websocket-transports.md) |
| IP-003 | Write-gate foundations ADR + state machine | complete | [`IP-003-write-gate-foundations.md`](IP-003-write-gate-foundations.md) |

## Estimated parallelism
3 agents; transports disjoint per crate.

## Symbols-touched
`crates/oya-foundry-api-{rest,graphql,sse,websocket}-*`, `contracts/openapi/foundry/`, `contracts/asyncapi/foundry/`.

## Agent-handoff
```
icm store -t context-oyatie -c "M02-P04 complete: 4 transports parity-verified; write-gate ADR Accepted" -i critical -k "M02,P04,transport-parity,write-gates,complete"
```
