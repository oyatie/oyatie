---
doc_class: ImplementationPlan
ip_id: IP-007
title: action-engine (Cedar-gated + idempotent + transaction-receipt + audit-emit)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology
date: 2026-05-17
depends_on: [IP-004, IP-006]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-foundry-fitness-cedar-coverage
  - oya-foundry-fitness-audit-chain-emission
  - oya-foundry-fitness-shardability
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-action-engine-{kernel,domain,usecase,adapter,worker}/
doc_status: published
---

# IP-007: action-engine

## Intent

Author the Action invocation engine that:
1. Gates every invocation through Cedar (default-deny baseline + per-Action permit).
2. Enforces idempotency (key required for production-tier Actions).
3. Emits a transaction receipt per Bominal ADR-0028 (object_ids + link_ids + audit_chain_ref).
4. Emits ObjectInstanceMutated + ActionTypeInvoked events to Kafka outbox.

## Scope

In-scope:
- `oya-ontology-action-engine-{kernel,domain,usecase,adapter,worker}` crates.
- Cedar gate integration from IP-006.
- Idempotency journal table in Postgres (deduplicated by idempotency_key + tenant_id).
- Transaction receipt emission via outbox.
- Action receipt audit-chain submit to audit-chain worker.
- Worker: async retry on transient Postgres failures; outbox-poller.

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold 5 crates |
| 2 | Author idempotency journal schema + adapter |
| 3 | Wire Cedar gate from IP-006 |
| 4 | Author transaction receipt builder |
| 5 | Wire outbox emit |
| 6 | Worker: process pending action invocations; retry; backpressure |
| 7 | Tests: Cedar deny → 403 + no write; idempotency repeat → same receipt; transient failure → retry |

## Verification

- `cargo nextest run -p oya-ontology-action-engine-usecase --test cedar_gate` — exit 0.
- Idempotency test: same key twice → same receipt.
- Action receipt audit-chain seal emitted.
- LEAN lanes green.

## References

- ADR-0006 (Ontology typed-entity layer); ADR-0140 (Cedar).
- Bominal ADR-0028 (audit-chain); ADR-0050 (outbox); ADR-0106 (Ontology); ADR-0107 (agent gateway).
- `microservices/ontology/PRD.md` §"action-engine".
