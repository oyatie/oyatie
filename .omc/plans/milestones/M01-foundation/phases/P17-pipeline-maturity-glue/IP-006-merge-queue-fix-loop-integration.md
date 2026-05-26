---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P17-IP-006
title: Merge-queue fix-loop integration (parked-PR semantics + bounded retry + fairness + revalidation)
status: scaffolded
tier: L
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_audit: ../../../../../../evidence/audits/pipeline-maturity-audit-2026-05-15.md
audit_amendment_ref: "Amendment 2026-05-15 §B: Merge-queue fix-loop integration"
upstream_kernel: oya-vcs-review-mergequeue-kernel
purpose: When a parked PR's fix lands, queue position is preserved, speculative rebase + re-CI runs against current queue HEAD, other PRs in queue keep flowing, concurrent fix-loops converge, bounded retry per PR enforces eviction on exhaustion.
---

# M01-P17-IP-006 — Merge-queue fix-loop integration

## Scope

New constraint from 2026-05-15 amendment §B. Today's merge-queue semantics are insufficient for autonomous fix-loop integration: a CI-failing PR blocks the queue head, and there's no formal "parked" state. This IP introduces:

- **Parked-PR state**: a queue position that is reserved but skipped on each cycle. Held by PR id, not branch SHA.
- **Speculative rebase**: when fix-loop output (from IP-005) lands on a parked PR's branch, the queue scheduler rebases the new tip against *current queue HEAD* (not the original rebase target — queue may have advanced) and re-runs admission CI.
- **Fairness**: while one PR is parked, other PRs in the queue continue processing. Parked PRs don't block head; they re-enter at their original position on next successful admission.
- **Concurrent fix-loops**: multiple parked PRs may have fix-loop agents running concurrently; the queue scheduler serializes only the final landing (one PR merges per cycle, others stay parked).
- **Bounded retry per PR**: 5 attempts (parallel to IP-005's budget); on exhaustion, PR is evicted from queue and the stuck-PR issue is escalated.
- **Convergence proof**: scheduler emits an admission-log entry per cycle so an external observer can verify forward progress (no livelock).

## Dependencies

- IP-002 (`oya` CLI) — scheduler is implemented as `oya merge-queue scheduler-tick` subcommand.
- IP-004 (reviewer-agent) — `pr-review-approved` events feed initial queue admission.
- IP-005 (CI-failure fix-loop) — produces the fix commits that trigger parked-PR revalidation.
- IP-007 (surface-all-failures CI) — each cycle's CI run reports the full failure surface; retry counter decrements once per cycle, not once per failure.

## Acceptance

- Three test PRs (A, B, C) admitted in that order; PR A fails CI; PRs B and C are NOT blocked behind A and continue processing.
- PR A enters parked state; IP-005 fix-loop pushes a fix; scheduler rebases against current queue HEAD (which may now include B or C if they merged) and re-runs CI.
- If PR A succeeds, it merges at its preserved queue position (or the next-available position if its original was filled — semantic decision documented in `/specs/merge-queue-parked-pr.json`).
- If PR A fails 5 times, it is evicted with a stuck-PR issue labeled `human-escalation`.
- Concurrent fix-loops: PRs A and D both parked simultaneously; both fix-loops run; one merges, the other re-parks. Scheduler does not deadlock.
- Per-tick evidence at `/evidence/pipeline-maturity-glue/ip-006-merge-queue/<tick-N>.json`.
- Rollup evidence at `/evidence/pipeline-maturity-glue/ip-006-merge-queue-fix-loop.json`.

## Symbols to grit-claim

- `crates/oya-foundry-vcs-merge-queue-scheduler-kernel/src/lib.rs::Scheduler`
- `crates/oya-foundry-vcs-merge-queue-scheduler-kernel/src/parked_state.rs::ParkedPr`
- `crates/oya-foundry-vcs-merge-queue-scheduler-kernel/src/speculative_rebase.rs::rebase_against_head`
- `crates/oya-foundry-vcs-merge-queue-scheduler-kernel/src/fairness.rs::pick_next_pr`
- `crates/oya-foundry-vcs-merge-queue-scheduler-kernel/src/retry_budget.rs::PrBudget`
- `tools/oya-cli/src/subcommands/merge_queue.rs::scheduler_tick`
- `specs/merge-queue-parked-pr.json::*` (state-machine spec)
- `registry/merge-queue-tick-log.json::*` (per-tick admission log)

## Exit evidence

- `/evidence/pipeline-maturity-glue/ip-006-merge-queue-fix-loop.json`
- `/evidence/pipeline-maturity-glue/ip-006-merge-queue/<tick-N>.json` (per-tick scheduler traces)
- `/evidence/pipeline-maturity-glue/ip-006-concurrent-fix-loop-convergence.json` (the A+D concurrent test)
