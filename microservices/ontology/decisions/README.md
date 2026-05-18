---
doc_class: ADRIndex
title: ADR index for the ontology µservice
microservice: ontology
date: 2026-05-17
doc_status: published
---

# ADR index — ontology µservice

ADRs for the ontology µservice live in the global ADR registry at
`docs/decisions/`. This index lists the ADRs that drive the Ontology
substrate, in chronological order, with their relevance to this µservice.

## Foundational

| ADR | Title | Relevance |
|---|---|---|
| ADR-0006 | Ontology as the engine-enforced typed-entity layer with per-property tier classification | Foundational design |
| ADR-0028 (Bominal) | Audit chain (Merkle + Ed25519) | Inherited; emitted by audit-chain BC |
| ADR-0055 | Object Graph renamed to Ontology | Naming authority |
| ADR-0056 | BNF v4.1 | Naming authority |
| ADR-0059 | Workflow + Ontology = ecosystem adapter layer | THE load-bearing architectural rule |
| ADR-0106 (Bominal) | Ontology architecture | Inherited 1:1 (with name override per ADR-0122) |
| ADR-0107 (Bominal) | Ontology agent gateway | Inherited; LLM tool-call ingress |
| ADR-0108–0112 (Bominal) | Property types (vector, geo, timeseries, ciphertext, struct) | Inherited |
| ADR-0122 | Ontology crate rename from Object Graph | Locks naming |
| ADR-0132 (Bominal) | Data-ownership pillars (org / person) | Inherited; pillar BC |
| ADR-0140 (retired per ADR-0145) | Cedar policy enforcement | Enforces every Action Type |

## Substrate

| ADR | Title | Relevance |
|---|---|---|
| ADR-0050 (Bominal) | Outbox pattern | Inherited; outbox → Kafka → audit-chain |
| ADR-0101 (Bominal) | Hexagonal sealed traits | Inherited; sealed port trait pattern |
| ADR-0105 | 13-layer canonical enum + Amendment 3 (`*-adapter-<backend>`) | Layer authority |
| ADR-0117 | Cloud-native infrastructure (residency) | Data residency authority |

## Phase + Governance

| ADR | Title | Relevance |
|---|---|---|
| ADR-0110 | ChangeSet state machine | Each IP is one ChangeSet |
| ADR-0123 | Hyperscaler maturity claim gate | HG-ONT registers here |
| ADR-0139 | Agentic SLO-gated promotion | Function-read SLO authored under it |
| ADR-0131 | Per-microservice flat layout | This pack authored natively under it |

## Open ADR successor-IPs

| Question | Owner | Target |
|---|---|---|
| ClickHouse history-mirror M02b vs M03 (resolved: M02b per IP-009) | council-architecture | Resolved |
| Function DSL: embedded Rust vs JSON-IR | council-architecture | M02b/P02 |
| Plugin SDK distribution: WASM (Wasmtime) vs native dylib | council-architecture | subsequent-to-M02b-completion ADR |
| Sequential agent autonomy ceiling: per-tool-call vs per-session | council-privacy + axis-ontology | M03 |

## References

- `docs/decisions/` — global ADR registry.
- `microservices/ontology/PRD.md` §"Related ADRs".
- `microservices/ontology/PHASE-01-TYPED-ENTITY-SUBSTRATE.md`.
