---
doc_class: JudgmentNote
title: Stale microservices/{cloud-network,cloud-network-dns} path hygiene (wave-5 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - network/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`network/**` — Seat A wave-5)

## Scope

Retarget only **verified** in-tree destinations under `network/**` / `network/dns/**`.
Do not invent missing homes. No hubs, no `Cargo.lock`, no merge.

## Retargeted (verified)

- `microservices/cloud-network/**` → `network/**` (root/runbooks/iac directory cites)
- `microservices/cloud-network-dns/**` → `network/dns/**` (root/iac directory cites)

## Deferred

- `retired tenant_class` artifact cites; missing `ARCHITECTURE.md` / `src/`; guest-on-oci always-free path without in-tree home
