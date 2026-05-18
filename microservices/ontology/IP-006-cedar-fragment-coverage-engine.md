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
  - oya-foundry-fitness-cedar-coverage
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-cedar-fragment-coverage-{kernel,domain,usecase,api,adapter}/
  - microservices/ontology/policy/*.cedar
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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
- CI lane `oya-foundry-fitness-cedar-coverage`: refuses PR if any registered Action Type lacks a permit fragment + default-deny.

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
