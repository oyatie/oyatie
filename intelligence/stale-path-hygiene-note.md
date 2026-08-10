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

## Wave-5 Seat A follow-through (2026-08-10)

Retargeted verified:

- Additional `microservices/cloud-intelligence/**` → `intelligence/**` (k8s/iac/helm + `slos/**` → `observability/slos/**`)
- `microservices/detection/**` → `intelligence/detection/**` for contracts/PRD/ARCHITECTURE/IPs/runbooks/manifest where present

### Deferred

- Detection IP scope stems without file homes; missing historical assets; cross-cap cites.
- No hubs, no `Cargo.lock`, no merge.


## Seat A events tranche (2026-08-10)

Nearest envelope for noun **events** (no `integ/events` rail) = `integ/intelligence` (eventsink adapters + detection). Messaging remains the S0 event-bus substrate (`integ/messaging`).

### Retargeted (verified)

- `microservices/detection` (bare AUDIT evidence) → `intelligence/detection`
- EventSink adapter comment prose `cloud-intelligence OAuth` → `intelligence OAuth` (stream key `cloud-intelligence-receipts:*` retained — runtime identifier)

### Deferred

- IP frontmatter `scope: microservices/detection/<bc>` logical BC names without `intelligence/detection/<bc>` homes
- Cross-cap cloud-iac Argo app paths
