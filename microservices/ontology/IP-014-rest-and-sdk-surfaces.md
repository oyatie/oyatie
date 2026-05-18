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
  - oya-foundry-fitness-api-semver
  - oya-foundry-fitness-sdk-regen-conformance
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-*-rest/
  - microservices/ontology/src/crates/oya-ontology-*-sdk/
  - microservices/ontology/contracts/openapi/ontology.yaml
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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
