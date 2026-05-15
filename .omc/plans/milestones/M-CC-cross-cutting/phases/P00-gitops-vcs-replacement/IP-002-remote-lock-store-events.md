---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P00-IP-002
title: Remote lock store + event stream
status: complete
source_plan: ../../../../ralplan-gitops-vcs-replacement-20260514.md
source_spec: ../../../../../specs/gitops-vcs-replacement.json
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
purpose: Auto-backfilled purpose for IP-002-remote-lock-store-events.md
---
# M-CC-P00-IP-002 — Remote lock store + event stream

## Purpose

Implement LockStorePort semantics for local and distributed leases, watch events, TTL, heartbeats, stale recovery, and queue state projection.

## ChangeSet boundary

This IP is one ChangeSet-sized execution unit. It must remain cohesive enough for a single claim/work/verify/done/promote loop. If execution discovers unrelated lock scopes, packages, adapters, or deployables, split the work into child IPs before claiming the broader tree.

## Reused grit behavior

Grit remains authoritative for repo transitions, claims, symbol locks, and compatibility `claim -> work -> done` closeout during cutover.

## New Oya VCS behavior

Oya VCS adds scheduling, projection, evidence, promotion, issue linkage, affected-build planning, package/deploy lineage, and ops explainability around the grit-authoritative transition.

## Test matrix

| Tier | Required proof |
|---|---|
| Unit | Idempotent claim/release; TTL and heartbeat state machine; duplicate event collapse. |
| Integration | Local + remote-adapter fake with TTL/watch replay and stale-lock recovery. |
| E2E | Stale agent expires, queued agent receives claim, dependent ChangeSets requeue safely. |
| Negative admission | Stale lock release by non-owner rejected; stale recovery without grit evidence rejected. |

## Evidence artifact

`/evidence/gitops-vcs/ip-002-lockstore.json`

## Acceptance-test commands

```bash
cargo test --workspace --all-features --test gitops_vcs_ip_002
oya check test-standard --registry /registries/cross-cutting/test-suite-registry.json
```

## Stop condition

Stop if this IP cannot be claimed, verified, bundled, and promoted as one ChangeSet without over-locking unrelated symbols/crates/deployables.

## Done criteria

- [ ] `execution_unit: ChangeSet` remains true; no unrelated scope was folded in.
- [ ] Required evidence is fresh and attached at the evidence artifact path.
- [ ] No direct agent `git`/`gh` path, no unclaimed diff, and no stale evidence enters promotion.
- [ ] ICM context store records packet status before user-facing completion.
