---
doc_class: JudgmentNote
title: Stale microservices/{cloud-storage,drive,recordings,imaging} path hygiene (wave-5 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - storage/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`storage/**` — Seat A wave-5)

## Scope

Retarget only **verified** in-tree destinations under `storage/**` (incl. nested drive/recordings/imaging).
Do not invent missing homes. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- `microservices/cloud-storage/**` → `storage/**` (PRD/README/manifest/faqs/benchmarks/runbooks/migration-playbooks/iac/coherence artifacts when present)
- Nested faces already advanced in wave-3 remain; additional verified nested remaps when destinations exist

## Deferred

- Missing `src/` / `tests/` / `ARCHITECTURE.md` / historical IP homes under drive/recordings/imaging
- Cross-cap observability/governance/cloud-iac cites
