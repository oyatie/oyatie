---
doc_class: ImplementationPlan
ip_id: IP-014
title: REST + SDK surfaces (OpenAPI 3.2 + Rust client + future TS/Python via bindgen)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology + dx-sdk
date: 2026-05-17
depends_on: [IP-002, IP-003, IP-004, IP-005, IP-007, IP-008, IP-010, IP-011, IP-012, IP-013]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-governance-api-semver
  - oya-governance-sdk-regen-conformance
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-*-rest/
  - microservices/ontology/src/crates/oya-ontology-*-sdk/
  - microservices/ontology/contracts/openapi/ontology.yaml
doc_status: published
---


# IP-014: REST + SDK surfaces

## Intent

Author the protocol-neutral `*-api` crates + the `*-rest` HTTP surfaces + the Rust `*-sdk` clients for all 8 outbound BCs (object-type-registry, link-type-registry, action-type-registry, function-type-registry, entity-store, link-store, action-engine, function-engine, agent-gateway, audit-chain). Ship OpenAPI 3.2 spec + Proto via tonic-build.

## Scope

In-scope:
- `oya-ontology-<bc>-api` crates: protocol-neutral typed I/O contracts.
- `oya-ontology-<bc>-rest` crates: HTTP handlers; OpenAPI spec generation; OIDC + Cedar middleware.
- `oya-ontology-<bc>-sdk` crates: Rust client; type-safe; OpenTelemetry instrumentation; retry + circuit breaker.
- `microservices/ontology/contracts/openapi/ontology.yaml` consolidated spec.
- `microservices/ontology/contracts/proto/ontology.proto` consolidated spec.

## Implementation

| Step | Action |
|---|---|
| 1 | For each BC: scaffold `-api`, `-rest`, `-sdk` crates |
| 2 | Author OpenAPI 3.2 spec via `utoipa` annotations in `-rest` |
| 3 | Author Cedar middleware in `-rest` (per-request evaluation) |
| 4 | Author Rust SDK with type-safe builder pattern |
| 5 | Wire `tonic-build` for gRPC client codegen (used by Workflow µservice) |
| 6 | Per-route tests: happy + auth-fail + tenant-mismatch + Cedar-deny |
| 7 | Contract tests: OpenAPI spec ↔ SDK match (LEAN lane validates) |

## Verification

- `cargo nextest run --workspace --all-features` — exit 0.
- OpenAPI spec lint: `spectral lint contracts/openapi/ontology.yaml` exit 0.
- `oya gate validate api-semver --microservice ontology` — exit 0 (no breaking change).
- Per-route coverage: 1 happy + 1 auth-fail + 1 tenant-mismatch per endpoint.

## References

- ADR-0056 (BNF v4.1); ADR-0105 (13-layer enum).
- `microservices/ontology/contracts/openapi/ontology.yaml`.
- `microservices/ontology/contracts/proto/ontology.proto`.
- `microservices/ontology/sdk-plan.md`.


## A. Problem
`IP-014: REST + SDK surfaces` is not a generic implementation packet; it closes the `014 rest and sdk surfaces` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

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
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `014 rest and sdk surfaces` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
