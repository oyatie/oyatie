---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P07-IP-004
title: GitOps promotion controller + provider seams
status: complete
source_plan: ../../../../ralplan-gitops-vcs-replacement-20260514.md
source_spec: ../../../../../specs/gitops-vcs-replacement.json
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
purpose: Build the controller-owned promotion state machine for CI/CD admission, security scans, GitOps reconciliation.
---
# M01-P07-IP-004 — GitOps promotion controller + provider seams

## Purpose

Build the controller-owned promotion state machine for CI/CD admission, security scans, GitOps reconciliation, dev-to-staging-to-production release trains, rollback, and environment health.

## ChangeSet boundary

This IP is one ChangeSet-sized execution unit. It must remain cohesive enough for a single claim/work/verify/done/promote loop. If execution discovers unrelated lock scopes, packages, adapters, or deployables, split the work into child IPs before claiming the broader tree.

## Reused grit behavior

Grit remains authoritative for repo transitions, claims, symbol locks, and compatibility `claim -> work -> done` closeout during cutover.

## New Oya VCS behavior

Oya VCS adds scheduling, projection, evidence, promotion, issue linkage, affected-build planning, package/deploy lineage, and ops explainability around the grit-authoritative transition.

## Test matrix

| Tier | Required proof |
|---|---|
| Unit | Promotion transition reducer, idempotency keys, rollback/readiness states. |
| Integration | Fake CI + fake GitOps reconciler + GitHub/GHA/Trivy/Argo adapter contract fixtures. |
| E2E | Bundle promotes through admission to published/reconciled and records release-train evidence. |
| Negative admission | Duplicate promotion collapses; stale index rejected; provider outage falls back to native mode when policy allows. |

## Evidence artifact

`/evidence/gitops-vcs/ip-004-controller.json`

## Acceptance-test commands

```bash
cargo test --workspace --all-features --test gitops_vcs_ip_004
oya check test-standard --registry /registry/test-suite-registry.json
```

## Stop condition

Stop if this IP cannot be claimed, verified, bundled, and promoted as one ChangeSet without over-locking unrelated symbols/crates/deployables.

## Done criteria

- [ ] `execution_unit: ChangeSet` remains true; no unrelated scope was folded in.
- [ ] Required evidence is fresh and attached at the evidence artifact path.
- [ ] No direct agent `git`/`gh` path, no unclaimed diff, and no stale evidence enters promotion.
- [ ] ICM context store records packet status before user-facing completion.
