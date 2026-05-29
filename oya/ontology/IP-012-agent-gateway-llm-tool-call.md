---
doc_class: ImplementationPlan
ip_id: IP-012
title: agent-gateway (LLM tool-call dispatch + OpenAI tool-spec generation + autonomy-tier ceiling)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology + ops-security
date: 2026-05-17
depends_on: [IP-008, IP-006]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-governance-cedar-coverage
  - oya-governance-perf-budget
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-agent-gateway-{kernel,domain,usecase,adapter,rest}/
doc_status: published
---


# IP-012: agent-gateway

## Intent

Author the Palantir-AIP-parity LLM tool-call ingress. Auto-generates OpenAI-compatible tool-specs from Function Type schemas; dispatches LLM tool-calls; enforces Cedar autonomy-tier ceiling; tier-filters results.

## Scope

In-scope:
- `oya-ontology-agent-gateway-{kernel,domain,usecase,adapter,rest}` crates.
- OpenAI tool-spec generator: reads Function Type schema; emits `tool_specs.json` per autonomy_tier scope.
- LLM JWT verification: extracts `autonomy_tier` claim; cross-checks against Function Type's `required_tier`.
- Per-session rate limit (token bucket; default 100 calls/min/session).
- Circuit breaker on per-LLM error rate > 50%.
- Tier-filtered result projection before returning to LLM.
- Audit-chain emit per tool-call.

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold 5 crates |
| 2 | Author tool-spec generator (Function Type → OpenAI tool-spec schema) |
| 3 | Wire LLM JWT verification (cedar autonomy_tier enforcement) |
| 4 | Author session-scoped rate limiter |
| 5 | Author circuit breaker |
| 6 | Wire tier-filter on tool-call return payload |
| 7 | REST endpoints: `/agent/tool-specs` (list), `/agent/tool-call` (dispatch) |
| 8 | Tests: autonomy_tier insufficient → 403; rate-limit hit → 429; runaway loop → circuit broken |

## Verification

- `cargo nextest run -p oya-ontology-agent-gateway-usecase --test autonomy_tier_check` — exit 0.
- Tool-call round-trip p99 ≤ 200 ms.
- LEAN lanes green.

## References

- Bominal ADR-0107 (agent gateway).
- ADR-0140 (retired per ADR-0145) (Cedar autonomy_tier).
- OpenAI tool-call spec — `platform.openai.com/docs/guides/function-calling`.
- EU AI Act 2024/1689 Arts. 9–15 (compliance).


## A. Problem
`IP-012: agent-gateway` is not a generic implementation packet; it closes the `012 agent gateway llm tool call` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

## B. Approach
Typed registry evolution with monotonic data-class/pillar rules, versioned object/link/action/function schemas, and migration receipts for caller-side read libraries. The implementation must keep the µservice boundary intact: contracts remain under `microservices/ontology/contracts/openapi/ontology.yaml` / `microservices/ontology/contracts/proto/ontology.proto`, policy decisions remain in `microservices/ontology/policy/tenant-scope.cedar`, operational proof remains in `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and the parity claim is checked against `microservices/ontology/competitor-parity-matrix.md`.

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
- `microservices/ontology/catalog/oya-ontology-object-type-registry-domain.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/capabilities/type-register.yaml` — verify/update as the authoritative artifact for this IP.
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
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `012 agent gateway llm tool call` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
