---
id: ADR-0102
title: Foundry Settings Template Canonical Rendering
status: Superseded
doc_status: published
owner: council-architecture
date: 2026-05-15
superseded_by: []
supersession_note: "Foundry settings render; atomic-render+sref pattern salvaged. Archived per D-DISPOSITIONS-RATIFIED: ARCHIVE-5."
live_plan_authority: false
canonical_authority: /specs/masterplan.json#masterplan_v2.surface_dispositions
read_contract:
  audience:
    - agents
    - humans
  read_timing_class: provenance-archive
  freshness_rule: "Wholly-superseded decision record archived in place (Seed Sub-AC 5.3.1); provenance ledger row lives at /specs/masterplan.json#masterplan_v2.surface_dispositions; never read as live authority — conflicts resolve to the superseding artifacts recorded in that row."
---

# ADR-0102: Foundry Settings Template Canonical Rendering

## Status
Accepted

## Context
Multiple providers and multiple accounts require consistent settings (hooks, skills, MCP servers). Manual maintenance leads to drift.

## Decision
We will use a canonical `SettingsTemplate` value type in `oya-intelligence-settings-template-kernel` and per-provider `SettingsRenderer` implementations in `oya-intelligence-settings-template-adapter`. 
1. Drift is verified at `AccountSnapshotProvider::snapshot()` time.
2. Rendering is atomic (tempfile + rename).
3. Secret references (`sref://`) are resolved at render/spawn time via `SecretStorePort`.

## Drivers
- **Multi-account Consistency:** Ensure N accounts stay in sync.
- **Cross-provider Parity:** Normalize different CLI configuration formats into one workspace template.
- **Drift-free Onboarding:** Automatically configure new accounts on first tick.

## Consequences
- Introduces `oya-intelligence-settings-template-kernel` and `oya-intelligence-settings-template-adapter`.
- 3 template payloads in `templates/foundry-supervisor/`.
- `lean-settings-drift` CI lane to enforce parity.
