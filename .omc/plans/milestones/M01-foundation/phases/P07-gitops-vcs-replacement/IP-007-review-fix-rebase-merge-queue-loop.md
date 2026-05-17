---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P07-IP-007
title: Review/fix, rebase, and merge-queue loop
status: complete
source_plan: ../../../../ralplan-gitops-vcs-replacement-20260514.md
source_spec: ../../../../../specs/gitops-vcs-replacement.json
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
purpose: Model review comments, CI failures, security findings, rebase conflicts, and merge-queue failures as typed FixupTasks tied to ChangeSets.
---
# M01-P07-IP-007 — Review/fix, rebase, and merge-queue loop

## Purpose

Model review comments, CI failures, security findings, rebase conflicts, and merge-queue failures as typed FixupTasks tied to ChangeSets and locks.

## ChangeSet boundary

This IP is one ChangeSet-sized execution unit. It must remain cohesive enough for a single claim/work/verify/done/promote loop. If execution discovers unrelated lock scopes, packages, adapters, or deployables, split the work into child IPs before claiming the broader tree.

## Reused grit behavior

Grit remains authoritative for repo transitions, claims, symbol locks, and compatibility `claim -> work -> done` closeout during cutover.

## New Oya VCS behavior

Oya VCS adds scheduling, projection, evidence, promotion, issue linkage, affected-build planning, package/deploy lineage, and ops explainability around the grit-authoritative transition.

## Test matrix

| Tier | Required proof |
|---|---|
| Unit | Review/fix terminal-state reducer; FixupTask ownership and boundedness. |
| Integration | Fake review provider + fake CI + merge queue adapter with failure injection. |
| E2E | Rejected review creates fix task; accepted fix re-enters queue; independent changes bypass failed item safely. |
| Negative admission | Lock release without terminal state rejected; agent-owned rebase rejected; stale issue digest blocks promotion after SLA. |

## Evidence artifact

`/evidence/gitops-vcs/ip-007-review-mergequeue.json`

## Acceptance-test commands

```bash
cargo test --workspace --all-features --test gitops_vcs_ip_007
oya check test-standard --registry /registry/test-suite-registry.json
```

## Stop condition

Stop if this IP cannot be claimed, verified, bundled, and promoted as one ChangeSet without over-locking unrelated symbols/crates/deployables.

## Done criteria

- [ ] `execution_unit: ChangeSet` remains true; no unrelated scope was folded in.
- [ ] Required evidence is fresh and attached at the evidence artifact path.
- [ ] No direct agent `git`/`gh` path, no unclaimed diff, and no stale evidence enters promotion.
- [ ] ICM context store records packet status before user-facing completion.
