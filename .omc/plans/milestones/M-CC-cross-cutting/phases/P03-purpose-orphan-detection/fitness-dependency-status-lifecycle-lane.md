---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P03-IP-FITNESS-DEPENDENCY-STATUS-LIFECYCLE
title: Fitness lane — dependency-status lifecycle (ADR-0109 framework instance #7)
status: scaffolded
execution_unit: ChangeSet
final_shape_compliance: true
dependency_additions:
  - specs/cross-cutting/lifecycle-configs/dependency-status-lifecycle.json
  - tools/oya-foundry-fitness-dependency-status-lifecycle-app
framework_anchor: docs/decisions/ADR-0109-lifecycle-automation-framework.md
purpose: Detect third-party deps whose deprecation has not advanced to removal by the declared deadline.
---

# M-CC-P03-IP-FITNESS-DEPENDENCY-STATUS-LIFECYCLE — Fitness lane: dependency-status lifecycle

## State machine
Stages: added → in-use → deprecated → removed. `deprecated` requires `replaced_by:` edge.

## Live baseline (Wave A, 2026-05-15)
```
artifacts_observed=0 (no docs/dependencies/**/*.md files yet)
violations=0
```

Fresh kernel; dependency-status docs populate as ADR-0064 (LTS dependency policy) compliance lands.

## Done-criteria (Wave A)
- `cargo run -q -p oya-foundry-fitness-dependency-status-lifecycle-app -- --warn-only` succeeds (vacuous baseline).

## Ratchet plan
- Wave A: kernel ready; vacuous baseline.
- Wave B: delta-BLOCK on new dependency-status docs without `status:` or `replaced_by:` edge.
- **Wave C (full-BLOCK, LIVE 2026-05-15):** baseline = 0 violations (no `docs/dependencies/**/*.md` files yet). Wave-C in force so future dependency docs MUST carry valid stage + replacement metadata or fail-build.
