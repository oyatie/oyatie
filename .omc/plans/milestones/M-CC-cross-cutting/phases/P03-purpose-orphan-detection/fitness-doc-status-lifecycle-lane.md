---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P03-IP-FITNESS-DOC-STATUS-LIFECYCLE
title: Fitness lane — doc-status lifecycle (ADR-0109 framework instance #6)
status: scaffolded
execution_unit: ChangeSet
final_shape_compliance: true
dependency_additions:
  - specs/cross-cutting/lifecycle-configs/doc-status-lifecycle.json
  - tools/oya-foundry-fitness-doc-status-lifecycle-app
framework_anchor: docs/decisions/ADR-0109-lifecycle-automation-framework.md
purpose: Detect docs whose `doc_status:` field is missing or whose terminal `superseded` lacks a `superseded_by:` edge.
---

# M-CC-P03-IP-FITNESS-DOC-STATUS-LIFECYCLE — Fitness lane: doc-status lifecycle

## State machine
Stages: drafted → published → stale → archived/superseded.

## Live baseline (Wave A, 2026-05-15)
```
artifacts_observed=714 (every docs/**/*.md file)
violations=714 (all stage_not_declared)
```

Existing `oya-foundry-fitness-doc-freshness-kernel` complements this: freshness detects mtime drift; lifecycle detects schema absence. They share the source corpus but evaluate different state machines.

## Done-criteria (Wave A)
- Baseline of 714 captured as drift ledger.

## Ratchet plan
- Wave A: WARN on every doc lacking `doc_status:`.
- Wave B: delta-BLOCK on new docs without `doc_status:`.
- **Wave C (full-BLOCK, LIVE 2026-05-15):** baseline = 0 violations after BF1 (87 ADRs) + BF2 (9 non-ADR docs) doc-status backfills; full BLOCK retroactive across the 714-doc corpus. Stale-detection integration with freshness kernel still pending as Wave-D follow-up.
