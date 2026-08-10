---
doc_class: JudgmentNote
title: Stale microservices/docs path hygiene (Seat A documents tranche)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - app/docs/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`app/docs/**` — Seat A documents)

## Scope

Nearest envelope for noun **documents** = `integ/app-docs` → `app/docs/**` (product forever home; not `integ/docs` plane).
Retarget only **verified** in-tree destinations under `app/docs/**`. Do not invent missing homes. No hubs, no `Cargo.lock`, no merge, no specs.

## Retargeted (verified)

- `microservices/docs/<path>` → `app/docs/<path>` when the destination file or directory exists (AUDIT, IPs, runbooks, prometheusrule runbook cites, contracts, catalog, slos, iac).

## Deferred

- Missing historical IP-* / PRD / capacity-model / threat-model / failure-modes homes not present under `app/docs/`
- Cross-cap `microservices/observability/**`, `microservices/governance/**`, sibling µservice runbook cites
