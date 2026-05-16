---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P03-IP-FITNESS-ADR-STATUS-LIFECYCLE
title: Fitness lane — adr-status lifecycle (ADR-0109 framework instance #1)
status: scaffolded
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions:
  - specs/cross-cutting/lifecycle-configs/adr-status-lifecycle.json
  - tools/oya-foundry-fitness-adr-status-lifecycle-app
framework_anchor: docs/decisions/ADR-0109-lifecycle-automation-framework.md
adr_anchor: docs/decisions/ADR-0109-lifecycle-automation-framework.md
naming_justification:
  oya-foundry-fitness-adr-status-lifecycle-app: |
    v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:adr-status-lifecycle>-<layer:app>`;
    canonical `app` suffix per ADR-0105/0107 amendment 2026-05-15.
purpose: Detect ADRs whose `status:` field is missing, unknown, or terminally `superseded` without `superseded_by:` — and fail the gate so ADR-status drift cannot accumulate.
---

# M-CC-P03-IP-FITNESS-ADR-STATUS-LIFECYCLE — Fitness lane: adr-status lifecycle

## Purpose
Per ADR-0109, the canonical ADR lifecycle is `proposed → accepted → superseded → archived`. Today 87 ADR files exist; only 18 declare `status: Accepted` and 1 declares `Superseded`. The remaining 68 carry no machine-readable status. This lane closes that drift.

## State machine
- `proposed` — non-terminal
- `accepted` — non-terminal
- `superseded` — terminal; requires `superseded_by:` edge
- `archived` — terminal

Transitions: `proposed → accepted`, `proposed → archived`, `accepted → superseded`, `accepted → archived`, `superseded → archived`.

## Live baseline (Wave A, 2026-05-15)
```
artifacts_observed=87
stage_counts=[("accepted", 18), ("superseded", 1)]
violations=68
breakdown:
  stage_not_declared: 68 (ADRs 0001-0033, 0034-0099 lacking front-matter `status:` field)
  unknown_stage: 0
  missing_supersession: 0
  illegal_transition: 0
```

## Acceptance-test-commands
```
cargo test -p oya-foundry-fitness-lifecycle-kernel
cargo test -p oya-foundry-fitness-adr-status-lifecycle-app
cargo run -q -p oya-foundry-fitness-adr-status-lifecycle-app -- --warn-only
```

## Done-criteria (Wave A)
- Kernel + dev-CLI tests green.
- `cargo check --workspace` green.
- Lane runs against the live workspace and reports the 68-violation baseline as the WARN ledger.

## Ratchet plan (WARN → BLOCK)
- **Wave A (initial):** lane runs WARN-only; CI captures the 68 baseline.
- **Wave B (delta-BLOCK):** lane blocks any NEW ADR without `status:` (delta-gate against baseline).
- **Wave C (full-BLOCK, LIVE 2026-05-15):** full retroactive BLOCK; baseline = 0 violations after BF1/BF2/BF3 backfills closed every ADR's `status:` field.

## Rollback-procedure
Lane is purely additive — reverting the merge removes config + dev-CLI; no other crate depends.

## Decision-log (Linus good-taste row)
Special cases eliminated: 68 ADRs with implicit "I guess this is accepted" status become mechanically auditable. No more silent ADR-status drift.
