---
doc_class: JudgmentNote
title: Stale path hygiene (wave-5 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - workflow/
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`workflow/**` — Seat A wave-5)

## Wave-5 Seat A follow-through (2026-08-10)

Retargeted verified — **for the high-value paths enumerated in the wave-6 section below, NOT for
the whole namespace**:

- `microservices/workflow-engine/**` → `workflow/workflow-engine/**`
- `microservices/workflow-engine/slos/**` → `workflow/observability/slos/workflow-engine/**`

### Scope of the claim

This note records *which cites were retargeted*, not that the old root is retired. As of
2026-08-18 the old prefix `microservices/workflow-engine` is still referenced by **186 tracked
files**, and some of those references are operational rather than historical — for example
`workflow/workflow-engine/runbooks/deadlock-resolution.md:164` still instructs
`rg "…" crates microservices/workflow-engine`, a command against a root that no longer exists,
and changed catalog entries retain the old phase-document path.

Stated unscoped, the claim reads as "the migration is complete", which would let a consumer
treat those live references as already handled. They are not. The unretargeted remainder is
deferred work, not verified work.

### Deferred

- The remaining `microservices/workflow-engine` references (186 files), including operational
  runbook commands and catalog phase-document paths.
- Historical numbered IPs not present under nested face; missing runbooks/testing-strategy
- No hubs, no `Cargo.lock`, no merge.

## Seat A wave-6 dep-ordered (2026-08-10)

- Verified remaps applied: **118** cite(s) across **43** file(s).
- Scope: path/manifest/SLO/contract/capability/catalog high-value only; missing homes deferred.
- Product unblock: forever cites for nested faces + observability prometheusrule alias.
- No hubs / Cargo.lock / merge / #1661 / cloud-os absorb.
