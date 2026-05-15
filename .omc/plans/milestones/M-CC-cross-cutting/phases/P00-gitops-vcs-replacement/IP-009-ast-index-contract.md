---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P00-IP-009
title: AST index contract + impacted-test mapping
status: complete
source_plan: ../../../../ralplan-gitops-vcs-replacement-20260514.md
source_spec: ../../../../../specs/gitops-vcs-replacement.json
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
purpose: Auto-backfilled purpose for IP-009-ast-index-contract.md
---
# M-CC-P00-IP-009 — AST index contract + impacted-test mapping

## Purpose

Define stable language-neutral AST/range/pointer/dependency contracts used by claims, review mapping, impacted tests, semantic conflict detection, and cache invalidation.

## ChangeSet boundary

This IP is one ChangeSet-sized execution unit. It must remain cohesive enough for a single claim/work/verify/done/promote loop. If execution discovers unrelated lock scopes, packages, adapters, or deployables, split the work into child IPs before claiming the broader tree.

## Reused grit behavior

Grit remains authoritative for repo transitions, claims, symbol locks, and compatibility `claim -> work -> done` closeout during cutover.

## New Oya VCS behavior

Oya VCS adds scheduling, projection, evidence, promotion, issue linkage, affected-build planning, package/deploy lineage, and ops explainability around the grit-authoritative transition.

## Test matrix

| Tier | Required proof |
|---|---|
| Unit | SymbolId stability, AST range normalization, pointer fallback, cache-key invalidation predicates. |
| Integration | Parser-backed fixture corpus plus schema/config pointer fixtures and generated-client dependency edges. |
| E2E | Post-rebase semantic diff recomputes impacted claims/tests/build closures without whole-tree rebuild unless declared. |
| Negative admission | Parser failure without explicit pointer scope blocks production; stale cache key blocks promotion. |

## Evidence artifact

`/evidence/gitops-vcs/ip-009-ast-contract.json`

## Acceptance-test commands

```bash
cargo test --workspace --all-features --test gitops_vcs_ip_009
oya check test-standard --registry /registries/cross-cutting/test-suite-registry.json
```

## Stop condition

Stop if this IP cannot be claimed, verified, bundled, and promoted as one ChangeSet without over-locking unrelated symbols/crates/deployables.

## Done criteria

- [ ] `execution_unit: ChangeSet` remains true; no unrelated scope was folded in.
- [ ] Required evidence is fresh and attached at the evidence artifact path.
- [ ] No direct agent `git`/`gh` path, no unclaimed diff, and no stale evidence enters promotion.
- [ ] ICM context store records packet status before user-facing completion.
