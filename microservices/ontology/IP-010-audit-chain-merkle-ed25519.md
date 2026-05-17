---
doc_class: ImplementationPlan
ip_id: IP-010
title: audit-chain (Merkle tree + Ed25519 sealing via OpenBao)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology + audit-chain µservice owner
date: 2026-05-17
depends_on: [IP-007]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-foundry-fitness-audit-chain-emission
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-audit-chain-{kernel,domain,usecase,adapter,worker}/
doc_status: published
---

# IP-010: audit-chain (Merkle + Ed25519)

## Intent

Author the audit-chain BC: per-tenant Merkle tree; per-period segment (60 s rolling OR 10⁴ events whichever first); Ed25519 sealing via OpenBao Transit; chain-of-chains mirror to platform `audit-chain` µservice per Bominal ADR-0028.

## Scope

In-scope:
- `oya-ontology-audit-chain-{kernel,domain,usecase,adapter,worker}` crates.
- Merkle tree builder (deterministic; concatenated event hashes; sha256 leaves; sha256 internal).
- Ed25519 signing via OpenBao Transit API; per-tenant key bound at issuance.
- Outbox-consumer worker reads `ontology.events.*` topics; appends to Merkle; seals on cadence.
- HSM-backed Ed25519 for pack-us-healthcare + pack-eu where available.

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold 5 crates |
| 2 | Author Merkle tree builder + tests (deterministic; tamper-evident) |
| 3 | Wire OpenBao Transit signing |
| 4 | Author worker: consume Kafka outbox; append events; seal segments |
| 5 | Postgres audit-chain table (append-only via trigger) |
| 6 | Chain-of-chains mirror to platform audit-chain µservice |
| 7 | Tests: tampered Merkle node detected; replay from outbox reproduces same root |

## Verification

- `cargo nextest run -p oya-ontology-audit-chain-worker --test merkle_verify` — exit 0.
- `oya gate validate audit-chain-emission --microservice ontology` — exit 0; emission rate = 1.0.
- Synthetic tamper → verification fails (expected).

## References

- Bominal ADR-0028 (audit-chain Merkle/Ed25519).
- OpenBao Transit API — `openbao.org/docs/secrets/transit`.
- ADR-0117 (residency; per-pack key custody).
