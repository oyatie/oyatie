---
purpose: Lifecycle-automation lane for migration status (pre-cutover → in-cutover → cleanup → done) via ADR-0109 framework.
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P03-IP-FITNESS-MIGRATION-STATUS-LIFECYCLE
title: Fitness lane — migration-status lifecycle (ADR-0109 framework instance #5)
status: scaffolded
execution_unit: ChangeSet
final_shape_compliance: true
dependency_additions:
  - specs/cross-cutting/lifecycle-configs/migration-status-lifecycle.json
  - tools/oya-foundry-fitness-migration-status-lifecycle-app
framework_anchor: docs/decisions/ADR-0109-lifecycle-automation-framework.md
purpose: Detect cutover/migration plans whose stage hasn't advanced past its declared milestone gate.
---

# M-CC-P03-IP-FITNESS-MIGRATION-STATUS-LIFECYCLE — Fitness lane: migration-status lifecycle

## State machine
Stages: pre-cutover → in-cutover → cleanup → done. Stage transitions are gated by milestone anchors (e.g. `M-CC-P01-merge`).

## Live baseline (Wave A, 2026-05-15)
```
artifacts_observed=326 (all plan files; lane scans for migration_status: in front-matter)
violations=326 (all stage_not_declared — only true migration plans need this field; the lane reports drift for plans that should but don't carry it)
```

Wave B will narrow the glob to `**/cutover-*.md` / `**/migration-*.md` to reduce false positives.

## Done-criteria (Wave A)
- Baseline of 326 captured as drift ledger (most non-migration plans will gain `migration_status: not-applicable` in Wave B backfill OR the glob narrows).

## Ratchet plan
- Wave A: lane in place; WARN on every plan lacking migration_status.
- Wave B: glob narrows to cutover/migration-only plans; BLOCK new migration plans without status.
- Wave C: full BLOCK + milestone-overdue enforcement.
