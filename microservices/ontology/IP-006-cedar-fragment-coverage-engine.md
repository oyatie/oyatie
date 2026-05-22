---
doc_class: ImplementationPlan
ip_id: IP-006
title: cedar-fragment-coverage (Cedar v4 policy fragments + default-deny + autonomy-tier ceiling)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology + ops-security
date: 2026-05-17
depends_on: [IP-002]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-governance-cedar-coverage
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-cedar-fragment-coverage-{kernel,domain,usecase,api,adapter}/
  - microservices/ontology/policy/*.cedar
doc_status: published
---


# IP-006: cedar-fragment-coverage

## Intent

Wire Cedar v4 policy fragments (`policy/tenant-scope.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`, `public-read.cedar`, `pillar.cedar`, per-Action fragments) into the µservice. Enforce default-deny baseline + per-Action permit + autonomy-tier ceiling on every gate.

## Scope

In-scope:
- `oya-ontology-cedar-fragment-coverage-kernel`: `CedarPolicyEvaluator` port; `CedarFragment`, `PolicyDecision`, `AutonomyTierCeiling`, `CedarDecisionRef`.
- `oya-ontology-cedar-fragment-coverage-domain`: pure decision logic; fragment merge; default-deny fallback.
- `oya-ontology-cedar-fragment-coverage-usecase`: orchestrator (fragment lookup → evaluate → emit decision).
- `oya-ontology-cedar-fragment-coverage-api`: typed I/O contracts.
- `oya-ontology-cedar-fragment-coverage-adapter`: Cedar v4 SDK bindings (cedar-policy crate); fragment hot-reload via inotify or schema-propagation-worker event.
- CI lane `oya-governance-cedar-coverage`: refuses PR if any registered Action Type lacks a permit fragment + default-deny.

Out-of-scope:
- Cedar evaluation in agent-gateway (IP-012 wires it).

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold 5 crates |
| 2 | Integrate `cedar-policy` crate (v4); pin LTS version per docs/standards/observability-slo.md |
| 3 | Author kernel port + entities |
| 4 | Author domain default-deny baseline + permit-merge logic |
| 5 | Author usecase orchestrator |
| 6 | Author adapter with hot-reload + fragment validation |
| 7 | Author LEAN coverage lane logic + fuzz tests |
| 8 | Integration test: synthetic permit grant → expected decision; forbid → 403 |
| 9 | Cedar evaluation perf budget: p99 ≤ 10 ms hard cap (timeout) |

## Verification

- `cargo nextest run -p oya-ontology-cedar-fragment-coverage-adapter` — exit 0.
- `oya gate validate cedar-coverage --microservice ontology` — exit 0; every Action Type has permit + default-deny.
- Cedar fuzz tests in CI exit 0.
- Perf bench: p99 evaluation ≤ 10 ms.

## References

- ADR-0140 (retired per ADR-0145) (Cedar policy enforcement).
- AWS Cedar v4 — `cedarpolicy.com`.
- `cedar-policy` Rust crate — `docs.rs/cedar-policy`.
- `microservices/ontology/policy/*.cedar`.


## A. Problem
`IP-006: cedar-fragment-coverage` is not a generic implementation packet; it closes the `006 cedar fragment coverage engine` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

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
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `006 cedar fragment coverage engine` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
