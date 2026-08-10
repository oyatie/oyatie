---
doc_class: JudgmentNote
title: Stale eventing/messaging path hygiene (Seat A events+messaging tranche)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - messaging/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`messaging/**` — Seat A events+messaging)

## Scope

Capability rail for noun **messaging** = `integ/messaging` → `messaging/**`.
Nearest rail for noun **events** (no `integ/events`) for the S0 event-bus substrate = also `integ/messaging` (ADR-0536 D-13). Intelligence owns EventSink adapters separately.

Retarget only **verified** in-tree destinations under `messaging/**`. Do not invent OpenSLO/contracts. No hubs, no `Cargo.lock`, no merge, no specs.

## Retargeted (verified — prior tips)

- Capability-root `messaging/manifest.json` S0/substrate accounting cites `messaging/**` only (`#1658@2d1c81693` lineage)
- Legacy `libs/oya-messaging-*` / `oya/eventing` dump cites retired in manifest comments

## This tranche

- Hygiene note lands; tree scan finds zero residual `microservices/` path cites under `messaging/**`
- Event-bus purpose remains documented in manifest (`Event bus / messaging substrate`)

## Deferred

- Invented OpenSLO / OpenAPI / AsyncAPI under `messaging/` (none in-tree — omit rather than invent)
- Cross-cap broker runtime / Pulsar deploy surfaces
