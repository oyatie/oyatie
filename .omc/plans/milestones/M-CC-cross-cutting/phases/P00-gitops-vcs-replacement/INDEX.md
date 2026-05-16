---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M-CC-P00
title: Polyglot GitOps-capable VCS Replacement (Oya VCS)
status: complete
source_plan: ../../../../ralplan-gitops-vcs-replacement-20260514.md
source_spec: ../../../../../specs/cross-cutting/gitops-vcs-replacement.json
purpose: Fold approved ralplan v5 into the cross-cutting master-plan tree as the first prerequisite before broad agent fan-out.
---

# M-CC-P00 — Polyglot GitOps-capable VCS Replacement (Oya VCS)

## M02/M03 Authority gate

**Waiver retired 2026-05-15.** M-CC-P00 (Oya VCS) is the forward closure authority for broad M02/M03 agent fan-out. M02 and M03 may consume existing git/Cargo infrastructure, but promoted Foundry agentic ChangeSets must close through Oya VCS ChangeBundle → Promotion → ReleaseTrain evidence. Legacy grit/icm/rtk/vox/omx/omc surfaces are compatibility/provenance only and do not satisfy promotion authority.

## Verdict

Approved by `$ralplan` consensus v5. Architect verdict: **APPROVE**. Critic verdict: **APPROVE**.

## ImplementationPlan == ChangeSet rule

Every IP in this phase is a ChangeSet-sized execution unit: cohesive, claimable, independently verifiable, bundleable, promotable, and small enough to avoid locking an entire tree without a graph-proven reason.

## Authority boundary

Oya VCS owns the authoritative agent/repo state transition for forward work: claim → work → verify → done → promote. It consumes/projects legacy grit state only through explicit compatibility ports while the command adapters finish landing. Agents still do not call `git`/`gh`; protected-ref mutation, controller rebase, review/fix, merge queue, promotion, and terminal lock release are controller-owned.

## Object chain

`OyaWorkItem -> IssuePlan -> ChangeSet -> VirtualHead / QueueAwareLease / FixupTask -> ChangeBundle -> Promotion / ReleaseTrain`

## Acceptance

- Every promoted ChangeBundle has Oya VCS claim coverage, semantic diff, required multispectrum evidence, required test evidence, package/build/deploy lineage, KG lineage, provenance, and terminal-state lock-release evidence.
- VirtualHead is review/build projection only.
- QueueAwareLease cannot override or release controller-owned locks outside terminal policy.
- GitHub, GitHub Actions, GitHub Issues, Trivy, and Argo CD are replaceable adapters; native fixtures provide the same core states.
- Unit/integration/e2e standards are enforced through `/specs/cross-cutting/test-standard.json` and `/registries/cross-cutting/test-suite-registry.json`.
- `ops.oyatie.com` exposes queue, lock, issue digest, build/cache, package/deploy, promotion, blocker, evidence, and explainability views backed by fresh evidence.
- Long Rust/polyglot compile paths rebuild only affected crates/packages/deployables cold unless a full-gate reason is recorded.

## Implementation Plans

| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Symbol lock domain + ChangeSet kernel | complete | [`IP-001-symbol-lock-domain.md`](IP-001-symbol-lock-domain.md) |
| IP-002 | Remote lock store + event stream | complete | [`IP-002-remote-lock-store-events.md`](IP-002-remote-lock-store-events.md) |
| IP-003 | ChangeBundle attestation + provenance | complete | [`IP-003-change-bundle-attestation.md`](IP-003-change-bundle-attestation.md) |
| IP-004 | GitOps promotion controller + provider seams | complete | [`IP-004-gitops-promotion-controller.md`](IP-004-gitops-promotion-controller.md) |
| IP-005 | Grit-compatible CLI + migration ratchet | complete | [`IP-005-grit-compat-cli-and-migration-ratchet.md`](IP-005-grit-compat-cli-and-migration-ratchet.md) |
| IP-006 | Polyglot AST/indexer adapters | complete | [`IP-006-polyglot-indexers.md`](IP-006-polyglot-indexers.md) |
| IP-007 | Review/fix, rebase, and merge-queue loop | complete | [`IP-007-review-fix-rebase-merge-queue-loop.md`](IP-007-review-fix-rebase-merge-queue-loop.md) |
| IP-008 | Unit/integration/e2e standard enforcement | complete | [`IP-008-test-standard-enforcement.md`](IP-008-test-standard-enforcement.md) |
| IP-009 | AST index contract + impacted-test mapping | complete | [`IP-009-ast-index-contract.md`](IP-009-ast-index-contract.md) |

## Execution order

1. IP-001 + IP-009 first: stable SymbolId/ArtifactPointer/ChangeSet semantics.
2. IP-002 + IP-003 + IP-006 + IP-008 in parallel after contracts stabilize.
3. IP-004 + IP-007 integrate controller promotion, review/fix, rebase, and merge queue.
4. IP-005 ratchets command UX and bans local-only closeout after the promote path is green.
