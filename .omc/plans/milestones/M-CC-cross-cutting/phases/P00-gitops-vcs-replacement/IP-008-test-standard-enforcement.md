---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P00-IP-008
title: Unit/integration/e2e standard enforcement
status: complete
source_plan: ../../../../ralplan-gitops-vcs-replacement-20260514.md
source_spec: ../../../../../specs/gitops-vcs-replacement.json
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
purpose: Enforce documented machine-readable unit, integration, contract, e2e, property, and fuzz standards through semantic diff and CI/CD.
---
# M-CC-P00-IP-008 — Unit/integration/e2e standard enforcement

## Purpose

Enforce documented machine-readable unit, integration, contract, e2e, property, and fuzz standards through semantic diff and CI/CD admission.

## ChangeSet boundary

This IP is one ChangeSet-sized execution unit. It must remain cohesive enough for a single claim/work/verify/done/promote loop. If execution discovers unrelated lock scopes, packages, adapters, or deployables, split the work into child IPs before claiming the broader tree.

## Reused grit behavior

Grit remains authoritative for repo transitions, claims, symbol locks, and compatibility `claim -> work -> done` closeout during cutover.

## New Oya VCS behavior

Oya VCS adds scheduling, projection, evidence, promotion, issue linkage, affected-build planning, package/deploy lineage, and ops explainability around the grit-authoritative transition.

## Test matrix

| Tier | Required proof |
|---|---|
| Unit | Semantic diff to required tier resolver; freshness and blocking policy decisions. |
| Integration | Registry-driven test selection by surface/language/package/deploy edge. |
| E2E | Failing required integration/e2e blocks promotion and emits typed FixupTask. |
| Negative admission | Stale evidence rejected; advisory-only evidence rejected; unaccounted generated client or contract change blocks promotion. |

## Evidence artifact

`/evidence/gitops-vcs/ip-008-test-enforcement.json`

## Acceptance-test commands

```bash
cargo test --workspace --all-features --test gitops_vcs_ip_008
oya check test-standard --registry /registries/cross-cutting/test-suite-registry.json
```

## Stop condition

Stop if this IP cannot be claimed, verified, bundled, and promoted as one ChangeSet without over-locking unrelated symbols/crates/deployables.

## Done criteria

- [ ] `execution_unit: ChangeSet` remains true; no unrelated scope was folded in.
- [ ] Required evidence is fresh and attached at the evidence artifact path.
- [ ] No direct agent `git`/`gh` path, no unclaimed diff, and no stale evidence enters promotion.
- [ ] ICM context store records packet status before user-facing completion.
