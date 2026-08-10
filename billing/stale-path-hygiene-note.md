---
doc_class: JudgmentNote
title: Stale microservices/{cloud-billing,finops-portal} path hygiene (wave-3 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - billing/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`billing/**` — Seat A wave-3)

## Scope

Retarget only **verified** in-tree destinations under `billing/**`.
Do not invent missing homes. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- `microservices/cloud-billing/**` → `billing/**` for files that exist (ARCH/README/PRD/contracts/IPs/runbooks/benchmarks/…).
- `microservices/finops-portal/**` → `billing/finops-portal/**` for contracts/runbooks/slos/policy/catalog/IPs that exist.

## Deferred

- Cross-cap observability prometheusrule cites.
- Any remaining missing historical assets without in-tree counterparts.

## Wave-4 Seat A follow-through (2026-08-10)

Continued verified remaps:

- `microservices/cloud-billing/slos/**` → `billing/observability/slos/**`
- Additional `contracts/`, `policies/`, `iac/`, `implementation-plans/` directory cites
- `microservices/finops-portal/**` → `billing/finops-portal/**` where present

### Deferred

- Missing `src/`, `capability-tiers/`, `policies/_tests/`, `runbooks/cell-migration.md`, partial ADR/IP filename stems.
- No hubs, no `Cargo.lock`, no merge.


## Wave-5 Seat A follow-through (2026-08-10)

Retargeted verified remaps:

- microservices/cloud-billing-tax/** -> billing/tax/** (PRD/README/manifest/contracts/IPs/runbooks/benchmarks/faqs/onboarding/tutorials/iac templates)
- Bare microservices/cloud-billing -> billing; microservices/finops-portal -> billing/finops-portal
- FinOps prometheusrule / Chart home URLs -> billing/finops-portal/** where runbook stems exist

### Deferred

- Missing tax ARCHITECTURE/capability-tiers/catalogs/policies/slos/src/tenant-class-behavior and finops-portal-deploy-rollback URL stem
- Historical MISSING findings that record absent homes (no invent)
- No hubs, no Cargo.lock, no merge.
