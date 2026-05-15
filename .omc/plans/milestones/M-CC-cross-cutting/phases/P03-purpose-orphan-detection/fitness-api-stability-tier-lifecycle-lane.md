---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P03-IP-FITNESS-API-STABILITY-TIER-LIFECYCLE
title: Fitness lane — api-stability-tier lifecycle (ADR-0109 framework instance #9)
status: scaffolded
execution_unit: ChangeSet
final_shape_compliance: true
dependency_additions:
  - specs/cross-cutting/lifecycle-configs/api-stability-tier-lifecycle.json
  - tools/oya-foundry-fitness-api-stability-tier-lifecycle-app
framework_anchor: docs/decisions/ADR-0109-lifecycle-automation-framework.md
adr_anchor: docs/decisions/ADR-0037-public-api-stability-tiers-and-deprecation.md
purpose: Detect public-API surfaces whose stability tier is missing or whose deprecation has stalled past its removal deadline.
---

# M-CC-P03-IP-FITNESS-API-STABILITY-TIER-LIFECYCLE — Fitness lane: api-stability-tier lifecycle

## State machine
Stages: experimental → stable → deprecated → removed. `deprecated` requires `replaced_by_api:` edge.

## Live baseline (Wave A, 2026-05-15)
```
artifacts_observed=0 (no docs/api-stability/**/*.md files yet; #[deprecated] attrs exist on 10 sites but no per-surface stability manifest)
violations=0
```

Fresh kernel; pairs with ADR-0037 (stability tier doctrine). Existing 10 `#[deprecated]` attribute sites become manifest-tracked in the backfill pass.

## Done-criteria (Wave A)
- `cargo run -q -p oya-foundry-fitness-api-stability-tier-lifecycle-app -- --warn-only` succeeds.

## Ratchet plan
- Wave A: kernel ready.
- Wave B: stability-manifest per public API; WARN on missing tier.
- Wave C: BLOCK on overdue deprecated → removed transitions per ADR-0037 schedule.
