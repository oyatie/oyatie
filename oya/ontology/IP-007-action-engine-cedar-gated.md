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
  - oya-governance-cedar-coverage
  - oya-governance-audit-chain-emission
  - oya-governance-shardability
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

- ADR-0006 (Ontology typed-entity layer); ADR-0140 (retired per ADR-0145) (Cedar).
- Bominal ADR-0028 (audit-chain); ADR-0050 (outbox); ADR-0106 (Ontology); ADR-0107 (agent gateway).
- `microservices/ontology/PRD.md` §"action-engine".


## A. Problem
`IP-007: action-engine` is not a generic implementation packet; it closes the `007 action engine cedar gated` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

## B. Approach
Cedar-first authorization with action/function schemas, default-deny fragments, and signed denial evidence before any entity mutation or function read. The implementation must keep the µservice boundary intact: contracts remain under `microservices/ontology/contracts/openapi/ontology.yaml` / `microservices/ontology/contracts/proto/ontology.proto`, policy decisions remain in `microservices/ontology/policy/tenant-scope.cedar`, operational proof remains in `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and the parity claim is checked against `microservices/ontology/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/ontology/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/openapi/ontology.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/proto/ontology.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/asyncapi/ontology-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/policy/tenant-scope.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/slos/read-path-library-freshness.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/runbooks/type-registry-migration.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/policy/cross-tenant-refusal.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/capabilities/cedar-evaluate.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/ontology/PRD.md` and `microservices/ontology/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `ontology`.
2. Diff the declared contract in `microservices/ontology/contracts/openapi/ontology.yaml` and `microservices/ontology/contracts/proto/ontology.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/ontology/policy/tenant-scope.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/ontology/slos/read-path-library-freshness.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/ontology/PRD.md`, `microservices/ontology/ARCHITECTURE.md`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/policy/tenant-scope.cedar`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and `microservices/ontology/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/ontology/PRD.md`
- `microservices/ontology/ARCHITECTURE.md`
- `microservices/ontology/contracts/openapi/ontology.yaml`
- `microservices/ontology/contracts/proto/ontology.proto`
- `microservices/ontology/contracts/asyncapi/ontology-events.yaml`
- `microservices/ontology/policy/tenant-scope.cedar`
- `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`
- `microservices/ontology/runbooks/type-registry-migration.md`
- `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml`
- `microservices/ontology/competitor-parity-matrix.md`
- `microservices/ontology/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `007 action engine cedar gated` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
