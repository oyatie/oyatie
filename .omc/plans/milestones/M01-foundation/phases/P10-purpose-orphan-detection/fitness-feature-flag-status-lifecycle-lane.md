---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P10-IP-FITNESS-FEATURE-FLAG-STATUS-LIFECYCLE
title: Fitness lane — feature-flag-status lifecycle (ADR-0109 framework instance #8)
status: scaffolded
execution_unit: ChangeSet
final_shape_compliance: true
dependency_additions:
  - specs/lifecycle-configs/feature-flag-status-lifecycle.json
  - tools/oya-governance-feature-flag-status-lifecycle-app
framework_anchor: docs/decisions/ADR-0109-lifecycle-automation-framework.md
purpose: Detect feature flags whose ramp/sunset has stalled past its declared deadline.
---

# M01-P10-IP-FITNESS-FEATURE-FLAG-STATUS-LIFECYCLE — Fitness lane: feature-flag-status lifecycle

## State machine
Stages: proposed → live → ramped → deprecated → removed.

## Live baseline (Wave A, 2026-05-15)
```
artifacts_observed=0 (no docs/feature-flags/**/*.md files yet)
violations=0
```

Fresh kernel; flag-status docs populate as `#[cfg(feature` usages grow beyond the current 6 occurrences.

## Done-criteria (Wave A)
- `cargo run -q -p oya-governance-feature-flag-status-lifecycle-app -- --warn-only` succeeds.

## Ratchet plan
- Wave A: kernel ready.
- Wave B: delta-BLOCK on new feature-flag manifests without `status:`.
- **Wave C (full-BLOCK, LIVE 2026-05-15):** baseline = 0 violations (no `docs/feature-flags/**/*.md` files yet). Wave-C in force so future flag manifests MUST carry valid stage + removal_deadline metadata or fail-build.
