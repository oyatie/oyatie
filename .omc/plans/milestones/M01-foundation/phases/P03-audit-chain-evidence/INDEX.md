---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P03
title: Audit Chain + Evidence Emission
status: complete
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
| IP-001 | `oya-audit-chain-domain` Merkle + Ed25519 | complete | [`IP-001-merkle-ed25519-kernel.md`](IP-001-merkle-ed25519-kernel.md) |
| IP-002 | Audit event AsyncAPI + Proto contract | complete | [`IP-002-audit-asyncapi-proto.md`](IP-002-audit-asyncapi-proto.md) |
| IP-003 | Tamper-evidence Sev-1 drill runbook | complete | [`IP-003-tamper-evidence-drill.md`](IP-003-tamper-evidence-drill.md) |

## Estimated parallelism
IP-001, IP-002, and IP-003 complete; M01-P04-IP-001 is the next ready foundation slice.

## Symbols-touched
`crates/oya-audit-chain-domain`, `crates/oya-audit-chain-application`, `crates/oya-audit-chain-file-adapter`, and audit contract files under `contracts/{asyncapi,proto}/platform/audit/`.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P03 complete: ADR-0003 audit chain shipped; tamper drill green" -i critical -k "M01,P03,audit-chain,complete"
```
