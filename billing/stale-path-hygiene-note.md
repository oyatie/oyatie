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
