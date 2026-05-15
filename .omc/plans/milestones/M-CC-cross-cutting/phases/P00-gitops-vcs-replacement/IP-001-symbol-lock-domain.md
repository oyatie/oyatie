---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P00-IP-001
title: Symbol lock domain + ChangeSet kernel
status: complete
source_plan: ../../../../ralplan-gitops-vcs-replacement-20260514.md
source_spec: ../../../../../specs/gitops-vcs-replacement.json
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
purpose: Auto-backfilled purpose for IP-001-symbol-lock-domain.md
---
# M-CC-P00-IP-001 — Symbol lock domain + ChangeSet kernel

## Purpose

Define SymbolId, ArtifactPointer, Claim, Lease, QueueAwareLease, ChangeSet, and terminal-state invariants that preserve grit-style semantic locking while giving Oya VCS schedulable work units.

## ChangeSet boundary

This IP is one ChangeSet-sized execution unit. It must remain cohesive enough for a single claim/work/verify/done/promote loop. If execution discovers unrelated lock scopes, packages, adapters, or deployables, split the work into child IPs before claiming the broader tree.

## Reused grit behavior

Grit remains authoritative for repo transitions, claims, symbol locks, and compatibility `claim -> work -> done` closeout during cutover.

## New Oya VCS behavior

Oya VCS adds scheduling, projection, evidence, promotion, issue linkage, affected-build planning, package/deploy lineage, and ops explainability around the grit-authoritative transition.

## Test matrix

| Tier | Required proof |
|---|---|
| Unit | Claim compatibility; lease/queue state machine; object transition rules. |
| Integration | Same-file/different-symbol and same-symbol collision fixtures against grit-port fakes. |
| E2E | Two agents claim unrelated functions and close through claim/work/done/promote. |
| Negative admission | Conflicting write claim rejected; QueueAwareLease cannot override grit lock; VirtualHead cannot mutate repo state. |

## Evidence artifact

`/evidence/gitops-vcs/ip-001-claim-kernel.json`

## Acceptance-test commands

```bash
cargo test --workspace --all-features --test gitops_vcs_ip_001
oya check test-standard --registry /registries/cross-cutting/test-suite-registry.json
```

## Stop condition

Stop if this IP cannot be claimed, verified, bundled, and promoted as one ChangeSet without over-locking unrelated symbols/crates/deployables.

## Done criteria

- [ ] `execution_unit: ChangeSet` remains true; no unrelated scope was folded in.
- [ ] Required evidence is fresh and attached at the evidence artifact path.
- [ ] No direct agent `git`/`gh` path, no unclaimed diff, and no stale evidence enters promotion.
- [ ] ICM context store records packet status before user-facing completion.
