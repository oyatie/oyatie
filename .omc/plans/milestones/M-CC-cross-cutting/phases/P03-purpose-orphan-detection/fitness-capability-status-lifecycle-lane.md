---
purpose: Lifecycle-automation lane for capability status (proposed → granted → revoked/expired) via ADR-0109 framework.
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P03-IP-FITNESS-CAPABILITY-STATUS-LIFECYCLE
title: Fitness lane — capability-status lifecycle (ADR-0109 framework instance #4)
status: scaffolded
execution_unit: ChangeSet
final_shape_compliance: true
dependency_additions:
  - specs/cross-cutting/lifecycle-configs/capability-status-lifecycle.json
  - tools/oya-foundry-fitness-capability-status-lifecycle-app
framework_anchor: docs/decisions/ADR-0109-lifecycle-automation-framework.md
purpose: Detect capability grants whose status/expiry is unknown or overdue.
---

# M-CC-P03-IP-FITNESS-CAPABILITY-STATUS-LIFECYCLE — Fitness lane: capability-status lifecycle

## State machine
Stages: proposed → granted → {revoked | expired}.

## Live baseline (Wave A, 2026-05-15)
```
artifacts_observed=0 (no `specs/**/*.capability.json` files yet)
violations=0
```

Fresh kernel; capability files appear once `oya-foundry-capability-registry-*` ships its first grant manifests.

## Done-criteria (Wave A)
- `cargo run -q -p oya-foundry-fitness-capability-status-lifecycle-app -- --warn-only` succeeds (vacuous baseline).

## Ratchet plan
- Wave A: kernel ready; vacuous baseline.
- Wave B: blocks any capability manifest with missing `status:` or `expires_at`.
- Wave C: blocks any granted capability past `expires_at`.
