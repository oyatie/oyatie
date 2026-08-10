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

## Wave-5 Seat A follow-through (2026-08-10)

Continued verified remaps `microservices/tenancy/**` → `tenancy/**`, including `slos/**` → `tenancy/observability/slos/**` for OpenSLO files that exist; runbooks/policy/contracts/iac directory cites.

### Deferred

- Journey-local contract/policy/test paths without in-tree counterparts; `src/crates/**` migration narratives; cross-cap cites.
- No hubs, no `Cargo.lock`, no merge.

