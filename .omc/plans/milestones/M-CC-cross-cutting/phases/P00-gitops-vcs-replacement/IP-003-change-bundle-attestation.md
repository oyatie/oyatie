---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P00-IP-003
title: ChangeBundle attestation + provenance
status: complete
source_plan: ../../../../ralplan-gitops-vcs-replacement-20260514.md
source_spec: ../../../../../specs/gitops-vcs-replacement.json
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
---

# M-CC-P00-IP-003 — ChangeBundle attestation + provenance

## Purpose

Define signed ChangeBundle evidence containing grit claim coverage, semantic diff, tests, package/build/deploy lineage, KG edges, provenance, and publication evidence.

## ChangeSet boundary

This IP is one ChangeSet-sized execution unit. It must remain cohesive enough for a single claim/work/verify/done/promote loop. If execution discovers unrelated lock scopes, packages, adapters, or deployables, split the work into child IPs before claiming the broader tree.

## Reused grit behavior

Grit remains authoritative for repo transitions, claims, symbol locks, and compatibility `claim -> work -> done` closeout during cutover.

## New Oya VCS behavior

Oya VCS adds scheduling, projection, evidence, promotion, issue linkage, affected-build planning, package/deploy lineage, and ops explainability around the grit-authoritative transition.

## Test matrix

| Tier | Required proof |
|---|---|
| Unit | ChangeBundle schema/provenance/coverage validation; digest and evidence freshness checks. |
| Integration | Signed bundle fixture with semantic diff, test refs, package/deploy refs, and KG lineage. |
| E2E | done emits bundle without protected-ref mutation; bundle publishes evidence for promotion. |
| Negative admission | Unsigned bundle rejected; unclaimed diff rejected; package/deploy artifact mismatch quarantined. |

## Evidence artifact

`.omc/evidence/gitops-vcs/ip-003-changebundle.json`

## Acceptance-test commands

```bash
cargo test --workspace --all-features --test gitops_vcs_ip_003
oya check test-standard --registry .omc/registries/test-suite-registry.json
```

## Stop condition

Stop if this IP cannot be claimed, verified, bundled, and promoted as one ChangeSet without over-locking unrelated symbols/crates/deployables.

## Done criteria

- [ ] `execution_unit: ChangeSet` remains true; no unrelated scope was folded in.
- [ ] Required evidence is fresh and attached at the evidence artifact path.
- [ ] No direct agent `git`/`gh` path, no unclaimed diff, and no stale evidence enters promotion.
- [ ] ICM context store records packet status before user-facing completion.
