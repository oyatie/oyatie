---
doc_class: ImplementationPlan
ip_id: IP-015
title: App composition-root binaries + branch-protection update + HG-ONT registration + OpenSLO manifests
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology + axis-foundry
date: 2026-05-17
depends_on: [IP-001, IP-002, IP-003, IP-004, IP-005, IP-006, IP-007, IP-008, IP-009, IP-010, IP-011, IP-012, IP-013, IP-014]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-foundry-fitness-per-microservice-layout
  - oya-foundry-fitness-authority-cohesion
  - oya-foundry-fitness-hyperscaler-maturity-claims
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-*-app/
  - .github/branch-protection.yaml
  - /specs/hyperscaler-gates.json
  - microservices/ontology/slos/
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: App binaries + branch-protection + HG-ONT + OpenSLO

## Intent

Final IP of the phase. Ship composition-root `*-app` binaries that wire usecase + adapter + rest + worker; update `.github/branch-protection.yaml` with the 5 new required lanes; register HG-ONT (hyperscaler maturity claim) in `/specs/hyperscaler-gates.json`; author the µservice's own OpenSLO manifests at `slos/`.

## Scope

In-scope:
- Composition-root `*-app` binaries per BC (10+ apps).
- `.github/branch-protection.yaml` diff per `PHASE-01-TYPED-ENTITY-SUBSTRATE.md` §"branch-protection.yaml diff preview".
- `release/ontology/{staging,production}` pattern protection rules.
- `/specs/hyperscaler-gates.json` HG-ONT entry.
- OpenSLO manifests at `microservices/ontology/slos/`:
  - `function-read-availability.openslo.yaml`
  - `function-read-latency.openslo.yaml`
  - `action-invocation-availability.openslo.yaml`
  - `audit-chain-emission-completeness.openslo.yaml`
  - `dynamic-layer-freshness.openslo.yaml`

## Implementation

| Step | Action |
|---|---|
| 1 | For each BC: scaffold `*-app` composition root binary; wire usecase + adapters + rest + worker |
| 2 | Author OpenSLO manifests (5 SLIs) |
| 3 | Update `.github/branch-protection.yaml` with 5 new lanes |
| 4 | Register HG-ONT in `/specs/hyperscaler-gates.json` per ADR-0123 |
| 5 | Register catalog records for all `*-app` crates |
| 6 | End-to-end drill: deploy to dev; AC-01..AC-14 of PRD pass |

## Verification

- `cargo build --workspace --all-features` — exit 0.
- All app smoke tests pass (startup + healthcheck + shutdown).
- `oya gate validate per-microservice-layout --microservice ontology` — exit 0.
- `oya gate validate authority-cohesion` — exit 0 (HG-ONT registers green).
- `oya gate validate hyperscaler-maturity-claims` — exit 0.
- All 5 OpenSLO manifests validate against OpenSLO v1.0 schema.
- All 5 new branch-protection lanes are required on dev + staging + release/ontology/{staging,production}.

## References

- ADR-0123 (hyperscaler maturity claim gate).
- ADR-0139 (SLO gate); ADR-0131 (per-microservice flat layout).
- `microservices/ontology/PHASE-01-TYPED-ENTITY-SUBSTRATE.md` §"branch-protection.yaml diff preview".
- `microservices/observability/PRD.md` §"OpenSLO manifest convention" (sibling pattern).
