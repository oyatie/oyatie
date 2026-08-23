---
doc_class: MasterPlan
shape: compatibility_projection_non_authoritative
length_cap: 800
authority_tier: 4
status: Accepted
date: 2026-05-19
owners:
- council-architecture
canonical_authority: /specs/masterplan.json
live_plan_authority: false
read_contract:
  audience:
    - humans
  read_timing_class: on-demand
  freshness_rule: "Projection only; conflicts resolve to /specs/masterplan.json#masterplan_v2."
companion_docs:
- /specs/root-hub-pointers.json
- /specs/master-plan-sequencing.json
- /specs/planning-closure-contract.json
- /specs/planning-closure-status-closure-ledger.json
- docs/decisions/ADR-0709-general-live-apex.md
authority_chain_declaration: |
  system / developer / user instructions
    > /specs/root-hub-pointers.json
    > docs/AGENTS.md (operating contract until explicit /specs/agent-operating-contract.json PHASE-5 promotion evidence)
    > installed agent-runtime skill and role catalog (for Codex: ~/.codex/skills + ~/.codex/agents; project .codex overlays only when intentionally checked in)
    > /specs/masterplan.json#masterplan_v2 (sole live plan authority and work-item ID namespace)
    > machine-readable specs and registries under /specs, /registry, /evidence, and /templates (supporting evidence/provenance only unless directly cited by masterplan v2)
    > external/upstream skill documentation (informational only; not vendored into this repo)
    > repo-root Redirect-class files (non-authoritative; lane-thin)
    > working drafts (never authoritative)
purpose: "Human compatibility projection for the machine-readable Oyatie master plan."
doc_status: published
---
# Oyatie Master Plan

This file is a human compatibility projection only. It is not a live plan authority, does not mint work-item IDs, and does not carry status claims. The canonical master plan, live work-item ID space, dependency DAG, surface dispositions, and read contracts live in `/specs/masterplan.json#masterplan_v2`.

## Current Authority

- Canonical plan authority: `/specs/masterplan.json`
- Canonical fragment for this consolidation: `/specs/masterplan.json#masterplan_v2`
- Live work-item ID namespace: `MPV2-####`, validated by the pipeline cross-artifact agreement masterplan-v2 authority check.
- Former plan surfaces (`/specs/master-plan-sequencing.json`, `/specs/planning-closure-contract.json`, `/specs/planning-closure-status-closure-ledger.json`, `docs/ROADMAP.md`, and legacy agent-harness runtime artifacts) are absorbed provenance or runtime data, not live plan authorities.

Historical `.omc`/`.omx` planning prompts and local runtime stores may be forensically read only when a gate or masterplan v2 evidence reference asks for them. They never override `/specs/masterplan.json`.

## Projection Contract

This projection intentionally avoids duplicating sequence, scope, status, or dependency detail. Humans use it as a pointer; agents and gates read `/specs/masterplan.json#masterplan_v2` directly.

Any update that adds roadmap content, work-item IDs, readiness status, or sequencing here without a generated-projection freshness gate is stale on arrival and must be rejected.