---
doc_class: JudgmentNote
title: Stale microservices/cloud-data path hygiene (wave-4 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - data/cloud-data/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`data/cloud-data/**` — Seat A wave-4)

## Scope

Retarget only **verified** in-tree destinations under `data/cloud-data/**`.
Do not invent missing homes. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- `microservices/cloud-data/**` → `data/cloud-data/**` for PRD/README/manifest/onboarding/migration-playbooks/feature-parity/remediation/contracts/openapi/faqs/reference-implementations/tutorials/iac where files exist.

## Deferred

- `src/` dump ownership narrative; any remaining missing historical assets.

## Wave-5 Seat A follow-through (2026-08-10)

Retargeted verified nested faces:

- `microservices/data-warehouse/**` → `data/data-warehouse/**`
- `microservices/data-pipeline/**` → `data/data-pipeline/**`
- `microservices/analytics/**` → `data/analytics/**` (iac/IPs)

### Deferred

- Missing PRD/ARCHITECTURE/`src/` dump cites; remaining historical IP stems
- No hubs, no `Cargo.lock`, no merge.
