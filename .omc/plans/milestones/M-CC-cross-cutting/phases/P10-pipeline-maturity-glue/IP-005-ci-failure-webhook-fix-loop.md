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

**Dual-source fix-loop** — the same dispatcher serves two failure surfaces in the canonical state machine (`push → CI → fix-loop until green → review → fix-loop until APPROVE → merge`):

  - **CI-source** (this IP's primary trigger): `.github/workflows/ci-failure-fix-loop.yml` triggers on `workflow_run` event where `conclusion == failure` AND `event == pull_request`. Context bundle: (failing-job log, full PR diff vs base, last N=5 commits, mistakes-ledger candidates from IP-003).
  - **Review-source** (added by IP-004 `pr-review-fix-requested` event): same workflow listens for `repository_dispatch: pr-review-fix-requested` events emitted by IP-004's dispatcher on REJECT / CHANGES_REQUESTED. Context bundle: (per-facet review findings, full PR diff, last N=5 commits, mistakes-ledger candidates).
  - Both sources funnel into the same `tools/oya-ci-fix-loop-dispatcher` → posts a fix-loop task into the agent dispatch queue.

A fix-loop agent claims the task via `oya claim --agent <ci|review>-fix-loop --intent "fix-<source>-PR-<N>"`, executes `oya work` → fix → `oya verify` → `oya done` → push to PR branch.

Bounded retry budget: **shared pool of N=5 attempts per PR across BOTH sources** (a PR doesn't get 5 CI attempts AND 5 review attempts — total 5 fix-loop iterations regardless of which source triggered). Sixth occurrence on same PR escalates to human via a "stuck-PR" GitHub issue. Shared budget prevents runaway loops where CI fix triggers review reject triggers CI fix etc.

Integrates with IP-003 mistakes-ledger: every fix-loop iteration writes a ledger row (source + retry-count + context-bundle hash + outcome).

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
- `specs/ci-fix-loop-context-bundle.json::*` (new schema for the agent task payload)
- `registry/ci-fix-loop-retry-budget.json::*` (per-PR retry counters)

## Exit evidence

- `/evidence/pipeline-maturity-glue/ip-005-ci-fix-loop.json`
- `/evidence/pipeline-maturity-glue/ip-005-fix-loop/<pr-number>/` (per-PR attempt traces)
