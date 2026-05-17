---
doc_class: ImplementationPlan
ip_id: IP-005
title: link-store (Postgres + cross-tenant Cedar gate + traversal)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology
date: 2026-05-17
depends_on: [IP-004]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-foundry-fitness-ontology-tenancy-isolation
  - oya-foundry-fitness-cedar-coverage
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-link-store-{kernel,domain,usecase,adapter,adapter-postgres}/
doc_status: published
---

# IP-005: link-store (typed Link Type persistence + traversal)

## Intent

Author the Link Type instance persistence + traversal layer. Both endpoints checked against `app.tenant_id`; cross-tenant link refused unless explicit Cedar `CrossTenantLinkGrant` present (per `policy/tenant-scope.cedar`).

## Scope

In-scope:
- `oya-ontology-link-store-kernel`: `LinkTypeStore` port, `LinkInstance`, `LinkId`, `TraversalQuery`.
- `oya-ontology-link-store-domain`: cardinality enforcement (1:1 / 1:N / M:N); same-tenant default; cross-tenant Cedar pre-check.
- `oya-ontology-link-store-usecase`: orchestrator (Cedar → write/traverse → emit).
- `oya-ontology-link-store-adapter`: protocol-neutral wrapper.
- `oya-ontology-link-store-adapter-postgres`: Postgres + Citus impl; RLS-scoped; traversal API with depth limit (≤ 5).

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold 5 crates per ADR-0105 |
| 2 | Cardinality enforcement at domain layer (refuse 1:1 write where pair already exists) |
| 3 | Cross-tenant Cedar pre-check: both endpoints' tenant_id verified; mismatch → 403 unless grant present |
| 4 | Traversal API: depth-bounded BFS; tenant-scoped at every hop |
| 5 | Tests: cross-tenant write refused; cardinality enforced; depth limit honoured |

## Verification

- `cargo nextest run -p oya-ontology-link-store-adapter-postgres --test cross_tenant_refused` — exit 0.
- `oya gate validate ontology-cross-tenant-link --tenant-a <id> --tenant-b <id>` — synthetic attempt refused.
- LEAN lanes green.

## References

- ADR-0006 (Ontology typed-entity layer).
- Bominal ADR-0106 + ADR-0107.
- `microservices/ontology/policy/tenant-scope.cedar` (cross-tenant link forbid clause).
