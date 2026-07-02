---
id: ADR-0101
title: Foundry Supervisor Mountpoint (Direct Hyper)
status: Superseded
doc_status: published
owner: council-architecture
date: 2026-05-15
superseded_by: []
supersession_note: "Temporary bypass shortcut promoted to architecture; foundry context retired. Archived per D-DISPOSITIONS-RATIFIED: ARCHIVE-5."
live_plan_authority: false
canonical_authority: /specs/masterplan.json#masterplan_v2.surface_dispositions
read_contract:
  audience:
    - agents
    - humans
  read_timing_class: provenance-archive
  freshness_rule: "Wholly-superseded decision record archived in place (Seed Sub-AC 5.3.1); provenance ledger row lives at /specs/masterplan.json#masterplan_v2.surface_dispositions; never read as live authority — conflicts resolve to the superseding artifacts recorded in that row."
---

# ADR-0101: Foundry Supervisor Mountpoint (Direct Hyper)

## Status
Accepted

## Context
The Foundry Supervisor requires a webhook surface for inbox ingestion and health checks. `oya-intelligence-api-rest-adapter` is currently a stub.

## Decision
The Foundry Supervisor will mount its webhook surface directly via `oya-http-runtime-hyper-adapter` (Hyper/Tokio) instead of going through the `api-rest-adapter`.

## Drivers
- **Implementation Speed:** Avoid unblocking the `api-rest-adapter` stub which is part of a different phase.
- **Performance:** Direct Hyper mounting is lower overhead for high-cadence supervisor ticks.

## Consequences
- The supervisor bypasses the project's standard REST adapter temporarily.
- Once M02-P04 lands a real router in `api-rest-adapter`, the supervisor may be migrated.
