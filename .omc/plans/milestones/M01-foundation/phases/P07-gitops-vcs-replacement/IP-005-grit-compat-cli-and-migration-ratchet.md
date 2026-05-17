---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P07-IP-005
title: Grit-compatible CLI + migration ratchet
status: complete
migration_status: cleanup
source_plan: ../../../../ralplan-gitops-vcs-replacement-20260514.md
source_spec: ../../../../../specs/gitops-vcs-replacement.json
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
purpose: Provide claim/work/done/status/symbols/queue/watch/promote CLI while ratcheting agents away from direct git/gh and local-only closeout.
---
# M01-P07-IP-005 — Grit-compatible CLI + migration ratchet

## Purpose

Provide claim/work/done/status/symbols/queue/watch/promote ergonomics while ratcheting agents away from direct git/gh and local-only closeout.

## ChangeSet boundary

This IP is one ChangeSet-sized execution unit. It must remain cohesive enough for a single claim/work/verify/done/promote loop. If execution discovers unrelated lock scopes, packages, adapters, or deployables, split the work into child IPs before claiming the broader tree.

## Reused grit behavior

Grit remains authoritative for repo transitions, claims, symbol locks, and compatibility `claim -> work -> done` closeout during cutover.

## New Oya VCS behavior

Oya VCS adds scheduling, projection, evidence, promotion, issue linkage, affected-build planning, package/deploy lineage, and ops explainability around the grit-authoritative transition.

## Test matrix

| Tier | Required proof |
|---|---|
| Unit | CLI command parsing; forbidden-operation detection; compatibility alias mapping. |
| Integration | Grit-compatible command shim against fixture repo and provider fakes. |
| E2E | Agent uses claim/work/done/promote without git/gh while evidence and locks flow through controller. |
| Negative admission | Direct git/gh evidence rejected; local-only closeout blocked after ratchet arm. |

## Evidence artifact

`/evidence/gitops-vcs/ip-005-cli-ratchet.json`

## Acceptance-test commands

```bash
cargo test --workspace --all-features --test gitops_vcs_ip_005
oya check test-standard --registry /registry/test-suite-registry.json
```

## Stop condition

Stop if this IP cannot be claimed, verified, bundled, and promoted as one ChangeSet without over-locking unrelated symbols/crates/deployables.

## Done criteria

- [ ] `execution_unit: ChangeSet` remains true; no unrelated scope was folded in.
- [ ] Required evidence is fresh and attached at the evidence artifact path.
- [ ] No direct agent `git`/`gh` path, no unclaimed diff, and no stale evidence enters promotion.
- [ ] ICM context store records packet status before user-facing completion.
