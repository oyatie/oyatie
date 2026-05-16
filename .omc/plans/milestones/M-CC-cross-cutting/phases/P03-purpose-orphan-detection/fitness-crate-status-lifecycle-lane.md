---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P03-IP-FITNESS-CRATE-STATUS-LIFECYCLE
title: Fitness lane — crate-status lifecycle (ADR-0109 framework instance #3)
status: scaffolded
execution_unit: ChangeSet
final_shape_compliance: true
dependency_additions:
  - specs/lifecycle-configs/crate-status-lifecycle.json
  - tools/oya-foundry-fitness-crate-status-lifecycle-app
framework_anchor: docs/decisions/ADR-0109-lifecycle-automation-framework.md
purpose: Detect crates whose lifecycle stage is not declared in `package.metadata.oyatie.lifecycle_stage`.
---

# M-CC-P03-IP-FITNESS-CRATE-STATUS-LIFECYCLE — Fitness lane: crate-status lifecycle

## State machine
Stages: scaffolded → live → quiescent → archived.

## Live baseline (Wave A, 2026-05-15)
```
artifacts_observed=0 (no crates yet declare package.metadata.oyatie.lifecycle_stage)
violations=0
```

Fresh kernel; awaits the backfill pass that adds the metadata block to each crate's Cargo.toml. Orphan detection already covered by `oya-foundry-fitness-archive-orphan-kernel`; this lane is the canonical state-machine record.

## Done-criteria (Wave A)
- `cargo run -q -p oya-foundry-fitness-crate-status-lifecycle-app -- --warn-only` returns 0 violations (vacuous baseline; population grows via backfill).

## Ratchet plan
- Wave A: kernel in place; vacuous baseline.
- Wave B: delta-BLOCK on new artifacts.
- **Wave C (full-BLOCK, LIVE 2026-05-15):** baseline = 0 violations (vacuous corpus — no `crates/*-domain/Cargo.toml` files declare `package.metadata.oyatie.lifecycle_stage` yet). Wave-C in force so future backfill commits MUST land schema-correct or fail-build.
