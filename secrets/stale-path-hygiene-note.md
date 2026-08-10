---
doc_class: JudgmentNote
title: Stale microservices/{cloud-secrets,cloud-kms} path hygiene (wave-4 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - secrets/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`secrets/**` — Seat A wave-4)

## Scope

Retarget only **verified** in-tree destinations under `secrets/**` / `secrets/kms/**`.
Do not invent missing homes. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- `microservices/cloud-secrets/**` → `secrets/**` when present
- `microservices/cloud-kms/**` → `secrets/kms/**` when present (PRD/README/manifest/faqs/benchmarks/runbooks/contracts)

## Deferred

- Journey-local contract/policy/test paths without in-tree counterparts; `legal/**`; `retired tenant_class` artifacts; missing `byok-ceremony.md`.


## Wave-5 Seat A follow-through (2026-08-10)

Retargeted verified remaps:

- microservices/cloud-kms/slos/ -> secrets/observability/slos/cloud-kms/
- Bare microservices/cloud-secrets -> secrets (reorg-unit judgments)

### Deferred

- Journey-local policy/eval/asyncapi fixtures without in-tree counterparts; missing KMS ARCHITECTURE/src/retired; control-plane.openslo.yaml filename not present under observability alias
- No hubs, no Cargo.lock, no merge.
