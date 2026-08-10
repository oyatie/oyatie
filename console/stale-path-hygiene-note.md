---
doc_class: JudgmentNote
title: Stale microservices/ops-dashboard-control-center path hygiene (wave-2 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - console/manifest.json
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`console/**` slice)

## Scope

Wave-2 Seat A keep_forever prep: retarget only **verified** in-tree destinations under `console/**`.
Do not invent missing IP/PRD/ARCHITECTURE homes; defer with this note.

## Retargeted (verified)

- `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml` → `console/contracts/asyncapi/ops-dashboard-control-center-events.yaml`
- `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml` → `console/contracts/openapi/ops-dashboard-control-center.yaml`

## Deferred (missing legal homes or cross-capability)

- (none — manifest slice complete)
