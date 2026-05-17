---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P18-IP-002
title: Merge-queue projected-state + fix-at-any-stage re-validate
status: scaffolded
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_adr: ../../../../../../docs/decisions/ADR-0111-merge-queue-projected-state-fix-at-any-stage.md
depends_on:
  - M01-P18-IP-001
purpose: Extend IP-006 merge-queue with projected-merge-state diff validation (pre-admit conflict-avoidance gate) + fix-at-any-stage re-validation protocol so the queue handles agentic load without divergence.
---

# M01-P18-IP-002 — Merge-queue projected-state + fix-at-any-stage

## Scope

Implement ADR-0111 wave-A as an extension of IP-006-from-P10's
`oya-foundry-vcs-merge-queue-fix-loop-app`:

- New kernel `oya-foundry-vcs-merge-queue-conflict-kernel` — pure
  algorithm: projected-merge-state computation +
  `git merge-tree` conflict-detection + path-overlap check.
- New module in the app: `projected_merge_state` + `conflict_avoidance_pre_admit`
  + `fix_at_any_stage_revalidate`.
- New CI lane `oya-foundry-fitness-merge-queue-ref-hygiene` — GCs
  stale `merge-queue-staging-i` refs.

Algorithm: for queued PR at position i, compute
`squash-merge(dev, PR_0..PR_{i-1}, PR_i)`, validate 3 invariants
(diff cleanliness, path-overlap, re-run pr-tests against projected
base) BEFORE admission. On `pr_branch_push` webhook event,
invalidate + re-validate positions ≥ i; re-position bounded by
`MAX_REPOSITION = 3`.

## Dependencies

- M01-P18-IP-001 (changeset-state kernel) — for emitting
  re-validation events.

## Acceptance

- `validate_projected_merge_state(dev_head, queued_prs, candidate)`
  kernel function exists, returns
  `Result<ProjectedStateReport, ProjectedStateError>`.
- 3-invariant validator wired into the existing merge-queue
  app's admit path.
- Fix-at-any-stage handler invoked on `pull_request.synchronize`
  webhook event (per ADR-0112 — stubbed if IP-003 not yet landed).
- Concurrent-safe path predicate defaults to refuse-on-overlap;
  per-product whitelist YAML at
  `registry/vcs/concurrent-safe-paths.yaml` initialized empty.
- Hygiene lane GCs `merge-queue-staging-i` refs older than 1
  hour.
- Smoke test: 2 PRs that touch the same file are queued; the
  later one is refused admission with conflict-avoidance error
  surface; both PRs see the report in their PR comment.

## Symbols to grit-claim

- `crates/oya-foundry-vcs-merge-queue-conflict-kernel/src/lib.rs::*`
- `tools/oya-foundry-vcs-merge-queue-fix-loop-app/src/projected_merge_state.rs::*`
- `tools/oya-foundry-vcs-merge-queue-fix-loop-app/src/conflict_avoidance_pre_admit.rs::*`
- `tools/oya-foundry-vcs-merge-queue-fix-loop-app/src/fix_at_any_stage_revalidate.rs::*`
- `registry/vcs/concurrent-safe-paths.yaml::*`

## Exit evidence

- `/evidence/agentic-vcs-pipeline/ip-002-merge-queue-conflict.json`
- `/evidence/agentic-vcs-pipeline/ip-002-fix-at-any-stage-smoke.json`
