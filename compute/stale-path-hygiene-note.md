---
doc_class: JudgmentNote
title: Stale microservices/cloud-compute path hygiene (wave-5 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - compute/manifest.json
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (compute/** — Seat A wave-5)

## Scope

Retarget only verified in-tree destinations under compute/**.
Do not invent missing homes. No hubs, no Cargo.lock, no merge.

## Wave-5 Seat A scan (2026-08-10)

Verified interior scan of compute/**:

- Remappable microservices/cloud-compute/** / microservices/compute/** cites: 0
- Tree is crate-facing (core/, adapters/, facade/) + manifest.json / OWNERS
- OpenSLO: live exemption retained (slo_exemption); no compute/observability/slos/ invented

### Deferred

- OpenSLO authoring until a grounded runtime SLI exists (manifest exemption)
- No hubs, no Cargo.lock, no merge.
