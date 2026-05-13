---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P03
title: Audit Chain + Evidence Emission
status: stub
purpose: Implement ADR-0003 Merkle-sealed audit chain (Ed25519 signed) and the evidence emission contract every regulated capability invocation must satisfy.
---

# M01-P03 — Audit Chain + Evidence Emission

## Purpose
Per [`../../../../../docs/PRD.md`](../../../../../docs/PRD.md) §6 constraint 2: audit-chain immutability is required or the cohesion thesis fails on first audit.

## Acceptance
- `audit.event.emit` SPEC §2 row green at `stable`; append-only + hash-chained + per-tenant-shard.
- AsyncAPI source at `contracts/asyncapi/platform/audit-events-v1.yaml`; Proto at `contracts/proto/platform/audit/v1/`.
- Tamper-evident verification: a Sev-1 drill confirms a tampered event is detected within one verification cycle.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | `oya-platform-audit-chain-kernel` Merkle + Ed25519 | stub | [`IP-001-merkle-ed25519-kernel.md`](IP-001-merkle-ed25519-kernel.md) |
| IP-002 | Audit event AsyncAPI + Proto contract | stub | [`IP-002-audit-asyncapi-proto.md`](IP-002-audit-asyncapi-proto.md) |
| IP-003 | Tamper-evidence Sev-1 drill runbook | stub | [`IP-003-tamper-evidence-drill.md`](IP-003-tamper-evidence-drill.md) |

## Estimated parallelism
3 agents; IP-001 + IP-002 disjoint; IP-003 follows once kernel exists.

## Symbols-touched
`crates/oya-platform-audit-chain-{kernel,domain,app,api,worker}-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P03 complete: ADR-0003 audit chain shipped; tamper drill green" -i critical -k "M01,P03,audit-chain,complete"
```
