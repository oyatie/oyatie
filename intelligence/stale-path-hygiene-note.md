---
doc_class: JudgmentNote
title: Stale microservices/cloud-intelligence path hygiene (wave-3 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - intelligence/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`intelligence/**` — Seat A wave-3)

## Scope

Retarget only **verified** in-tree destinations under `intelligence/**`.
Do not invent missing homes. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- `microservices/cloud-intelligence/**` → `intelligence/**` for Dockerfile/PRD/IP-001/contracts/design/iac/helm/k8s/runbooks/policy files that exist.

## Deferred

- Missing historical homes without in-tree counterparts.
- Cross-cap dump cites outside `intelligence/**`.
