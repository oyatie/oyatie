---
doc_class: JudgmentNote
title: Registry foundry-supervisor tombstone + path hygiene (Seat A wave-5)
status: Accepted
owner_team: council-architecture
date: 2026-08-10
related_artifacts:
  - registry/artifact-capabilities-registry.json
  - registry/knowledge-graph-dynamic.json
  - registry/accounts/schema.json
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`registry/**` — Seat A wave-5)

## Chesterton challenge

`integ/docs` `#1645@2e7dace3b` deleted `templates/foundry-supervisor/**` (hooks pointed at missing `tools/foundry-supervisor-*` binaries) and elevated registry cites as out-of-envelope. This tip owns `registry/**` and must tombstone those rows without resurrecting `#1648` or editing hubs/`specs/**`.

## Wave-5 retargeted / tombstoned (verified)

| Surface | Change |
|---|---|
| `registry/artifact-capabilities-registry.json` | Removed `foundry-supervisor-template-{claude,codex,gemini}` rows (paths deleted on `#1645`); retained `foundry-supervisor-kernel` |
| `registry/knowledge-graph-dynamic.json` | Dropped `lean-settings-drift__enforces__foundry-supervisor-template` edge |
| `registry/accounts/schema.json` `_meta.parser_ref` | `microservices/intelligence/crates/oya-intelligence-settings-template-kernel/src/account_kernel.rs:48` → `intelligence/core/account-kernel/src/lib.rs` |

## Deferred

| Cite class | Why |
|---|---|
| `specs/microservices/intelligence.json#…` | Specs/hubs envelope — **BAN** on this tip (`#1644` sole) |
| Other `microservices/*` catalog/openapi historical paths without verified dest on this tip | Do not invent |

## Non-claims

- ≠ reopen `#1648`. Live rail remains `#1707`.
- No hubs, no `Cargo.lock`, no merge. Observation≠APPROVE. **STOP #1661.**
