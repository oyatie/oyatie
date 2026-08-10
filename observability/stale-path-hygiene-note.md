---
doc_class: JudgmentNote
title: Stale microservices/observability path hygiene (wave-3 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - observability/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`observability/**` — Seat A wave-3)

## Scope

Retarget only **verified** in-tree destinations under `observability/**`.
Do not invent missing homes. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- `observability/IPs/IP-ADR-0339-Shared-IaC-Modules.md` dossier contract/capability cites:
  `microservices/observability/contracts/**` + `capabilities/**` → `observability/**`.

## Deferred

- Missing IP-001..015 / PRD / ARCHITECTURE / MIGRATION homes still on legacy cites in root manifest (do not invent).

## Wave-5 Seat A follow-through (2026-08-10)

Retargeted verified:

- `microservices/observability/iac/` → `observability/iac/`
- `microservices/observability/contracts/openapi.yaml` → `observability/diagnostics/contracts/openapi.yaml` (diagnostics face)
- `microservices/diagnostics/**` → `observability/diagnostics/**` where present

### Deferred

- Missing IP-001..015 / PRD / ARCHITECTURE / MIGRATION homes; tenant-isolation policy docs; dashboard path cites without in-tree counterparts.
- No hubs, no `Cargo.lock`, no merge.

