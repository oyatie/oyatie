---
doc_class: JudgmentNote
title: Stale microservices/{plugin-app-store,developer-sdk} path hygiene (wave-3 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - marketplace/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`marketplace/**` — Seat A wave-3)

## Scope

Retarget only **verified** in-tree destinations under `marketplace/**`.
Do not invent missing IP/PRD/ARCHITECTURE homes. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- Nested `marketplace/plugin-app-store/**`: `microservices/plugin-app-store/{capabilities,contracts,runbooks}/**` → `marketplace/plugin-app-store/**` where files exist.
- Nested `marketplace/developer-sdk/**`: `microservices/developer-sdk/{capabilities,contracts,packs}/**` → `marketplace/developer-sdk/**` where files exist.

## Deferred (missing legal homes or cross-capability)

- Missing IP/implementation-plan markdown under nested faces (do not invent).
- Cross-cap `microservices/observability/iac/helm/...prometheusrule.yaml#…` hyperscaler_benchmark cites.
- Missing PRD/capacity-model/architecture homes still on legacy paths.
