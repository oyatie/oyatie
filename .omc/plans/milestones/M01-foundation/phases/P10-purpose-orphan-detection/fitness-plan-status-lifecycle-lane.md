---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P10-IP-FITNESS-PLAN-STATUS-LIFECYCLE
title: Fitness lane — plan-status lifecycle (ADR-0109 framework instance #2)
status: scaffolded
execution_unit: ChangeSet
final_shape_compliance: true
dependency_additions:
  - specs/lifecycle-configs/plan-status-lifecycle.json
  - tools/oya-governance-plan-status-lifecycle-app
framework_anchor: docs/decisions/ADR-0109-lifecycle-automation-framework.md
purpose: Detect plan files whose `status:` is missing or uses an undeclared stage.
---

# M01-P10-IP-FITNESS-PLAN-STATUS-LIFECYCLE — Fitness lane: plan-status lifecycle

## State machine
Stages: stub → scaffolded → proposed → pending → open → in-progress → partial → complete → merged → archived. Branches: approved-folded, split-required-too-broad-for-single-changeset, deferred-to-followon-wave.

## Live baseline (Wave A, 2026-05-15)
```
artifacts_observed=326
stage_counts=[("approved-folded", 1), ("complete", 16), ("merged", 2), ("pending", 27), ("proposed", 35)]
violations=245 (all stage_not_declared)
```

## Done-criteria (Wave A)
- `cargo run -q -p oya-governance-plan-status-lifecycle-app -- --warn-only` reports the 245 baseline.

## Ratchet plan
- Wave A: WARN. Wave B: BLOCK new plans without `status:`. **Wave C (full-BLOCK, LIVE 2026-05-15):** baseline = 0 violations after BF1/BF2/BF3 plan-status backfills; lane fails-build retroactively on any plan without a declared stage.
