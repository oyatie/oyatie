---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P10-IP-005
title: CI-failure webhook → fix-loop agent dispatch
status: scaffolded
tier: M
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_audit: ../../../../../../evidence/audits/pipeline-maturity-audit-2026-05-15.md
audit_amendment_ref: "Amendment 2026-05-15 §A: CI-failure → webhook → fix-loop"
purpose: On `workflow_run.conclusion == failure`, dispatch a fix-loop agent with failing-job log + PR diff + prior commit history. Agent diagnoses, writes fix, commits + pushes to same PR branch. Bounded retry budget before human escalation.
---

# M-CC-P10-IP-005 — CI-failure webhook → fix-loop agent dispatch

## Scope

New constraint from 2026-05-15 amendment §A. PR #3's cascade demonstrated that CI failures today require a human (or Skill-tool dispatch) to read logs, diagnose, and push a fix. This IP closes the loop:

- `.github/workflows/ci-failure-fix-loop.yml` triggers on `workflow_run` event where `conclusion == failure` AND `event == pull_request`.
- Workflow invokes `tools/oya-ci-fix-loop-dispatcher` which gathers (failing-job log, full PR diff vs base, last N=5 commits on branch, mistakes-ledger candidates from IP-003) and posts a fix-loop task into the agent dispatch queue.
- A fix-loop agent claims the task via `oya claim --agent ci-fix-loop --intent "fix-CI-failure-PR-<N>"`, executes `oya work` → fix → `oya verify` → `oya done` → push to PR branch.
- Bounded retry budget: **N=5 per PR per CI-failure class**; sixth occurrence on same PR escalates to human via a "stuck-PR" GitHub issue.
- Integrates with IP-003 mistakes-ledger: every fix-loop iteration writes a ledger row.

## Dependencies

- IP-002 (`oya` CLI) — fix-loop agent uses `oya claim/work/verify/done` rather than `git`/`gh`.
- IP-003 (mistakes-ledger) — every fix-loop iteration emits a ledger row.
- IP-007 (surface-all-failures CI) — without exhaustive failure surface, fix-loop wastes retry budget on one-failure-per-cycle whack-a-mole.
- IP-006 (merge-queue fix-loop integration) — fix-loop output feeds IP-006's parked-PR re-CI logic.

## Acceptance

- `workflow_run failure` on a test PR triggers `ci-failure-fix-loop.yml` within 60s.
- Dispatcher posts a task to the agent dispatch queue with full context bundle.
- A fix-loop agent (claude/codex/gemini-class) consumes the task, pushes a fix commit; CI re-runs.
- After 5 retries on the same PR with no convergence, dispatcher opens a stuck-PR issue, labels `human-escalation`, and stops dispatching further attempts.
- Per-attempt evidence at `/evidence/pipeline-maturity-glue/ip-005-fix-loop/<pr-number>/<attempt-N>.json`.
- Rollup evidence at `/evidence/pipeline-maturity-glue/ip-005-ci-fix-loop.json`.

## Symbols to grit-claim

- `.github/workflows/ci-failure-fix-loop.yml::*`
- `tools/oya-ci-fix-loop-dispatcher/Cargo.toml::package`
- `tools/oya-ci-fix-loop-dispatcher/src/main.rs::main`
- `tools/oya-ci-fix-loop-dispatcher/src/context_bundle.rs::gather`
- `tools/oya-ci-fix-loop-dispatcher/src/retry_budget.rs::Budget`
- `tools/oya-ci-fix-loop-dispatcher/src/escalation.rs::open_stuck_pr_issue`
- `specs/cross-cutting/ci-fix-loop-context-bundle.json::*` (new schema for the agent task payload)
- `registries/cross-cutting/ci-fix-loop-retry-budget.json::*` (per-PR retry counters)

## Exit evidence

- `/evidence/pipeline-maturity-glue/ip-005-ci-fix-loop.json`
- `/evidence/pipeline-maturity-glue/ip-005-fix-loop/<pr-number>/` (per-PR attempt traces)
