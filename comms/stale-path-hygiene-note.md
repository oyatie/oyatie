---
doc_class: JudgmentNote
title: Stale microservices/{messenger,mail,meet,contact-center,comms-email,emergency} path hygiene (wave-5 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - comms/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`comms/**` — Seat A wave-5)

## Scope

Retarget only **verified** in-tree destinations under nested comms faces.
Do not invent missing homes. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- `microservices/messenger/**` → `comms/messenger/**`
- `microservices/mail/**` → `comms/mail/**`
- `microservices/meet/**` → `comms/meet/**`
- `microservices/contact-center/**` → `comms/contact-center/**`
- `microservices/comms-email/**` → `comms/comms-email/**`
- `microservices/emergency/**` → `comms/emergency/**`
- Including contracts/policy/runbooks/iac/IPs and `slos/**` → nested `observability/slos/**` when present

## Deferred

- Missing PRD/ARCHITECTURE/`src/` homes; journey-local eval fixtures; cross-cap cites
