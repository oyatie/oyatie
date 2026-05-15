---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P00-IP-006
title: Polyglot AST/indexer adapters
status: complete
source_plan: ../../../../ralplan-gitops-vcs-replacement-20260514.md
source_spec: ../../../../../specs/gitops-vcs-replacement.json
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
---

# M-CC-P00-IP-006 — Polyglot AST/indexer adapters

## Purpose

Graduate semantic claim/index coverage beyond Rust to TypeScript/JavaScript, Swift, Kotlin, C#, C/C++, WinUI/XAML, schemas, contracts, and config.

## ChangeSet boundary

This IP is one ChangeSet-sized execution unit. It must remain cohesive enough for a single claim/work/verify/done/promote loop. If execution discovers unrelated lock scopes, packages, adapters, or deployables, split the work into child IPs before claiming the broader tree.

## Reused grit behavior

Grit remains authoritative for repo transitions, claims, symbol locks, and compatibility `claim -> work -> done` closeout during cutover.

## New Oya VCS behavior

Oya VCS adds scheduling, projection, evidence, promotion, issue linkage, affected-build planning, package/deploy lineage, and ops explainability around the grit-authoritative transition.

## Test matrix

| Tier | Required proof |
|---|---|
| Unit | Symbol extraction normalization per language adapter and pointer fallback rules. |
| Integration | Fixture corpus for Rust/TS/Swift/Kotlin/C#/C/C++/XAML/schema/config. |
| E2E | Polyglot diff maps to graduated claims, dependency closures, required tests, and promotion blockers. |
| Negative admission | Unsupported production surface blocks promotion; parser failure without explicit pointer scope blocks production. |

## Evidence artifact

`/evidence/gitops-vcs/ip-006-polyglot-indexers.json`

## Acceptance-test commands

```bash
cargo test --workspace --all-features --test gitops_vcs_ip_006
oya check test-standard --registry /registries/cross-cutting/test-suite-registry.json
```

## Stop condition

Stop if this IP cannot be claimed, verified, bundled, and promoted as one ChangeSet without over-locking unrelated symbols/crates/deployables.

## Done criteria

- [ ] `execution_unit: ChangeSet` remains true; no unrelated scope was folded in.
- [ ] Required evidence is fresh and attached at the evidence artifact path.
- [ ] No direct agent `git`/`gh` path, no unclaimed diff, and no stale evidence enters promotion.
- [ ] ICM context store records packet status before user-facing completion.
