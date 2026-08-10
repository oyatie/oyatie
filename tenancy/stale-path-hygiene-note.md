---
doc_class: JudgmentNote
title: Stale microservices/tenancy path hygiene (wave-3 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - tenancy/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`tenancy/**` — Seat A wave-3)

## Scope

Retarget only **verified** in-tree destinations under `tenancy/**`.
Do not invent missing homes. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- Broad interior retarget: `microservices/tenancy/**` → `tenancy/**` across IPs/runbooks/contracts/policy/iac/docs where destinations exist (~100+ files).

## Deferred

- Cross-capability cites (`microservices/observability/…`, `microservices/cloud-iac/…`, foreign PRDs).
- Historical inventory narratives that name retired dump paths as audit evidence (leave until shrink rail).
