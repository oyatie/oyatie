---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P18-IP-004
title: VCS orchestrator + async-by-default `oya vcs done`
status: scaffolded
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_adr: ../../../../../../docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md
depends_on:
  - M01-P18-IP-001
purpose: Wire `oya vcs done` as the canonical async-by-default agentic kickoff that drives a changeset through the full pipeline (PR-open → CI → review → merge → promote).
---

# M01-P18-IP-004 — VCS orchestrator + async-by-default `oya vcs done`

## Scope

Implement ADR-0113 wave-A:

- New kernel `oya-foundry-vcs-orchestrator-kernel` — pure-domain
  state-validator + idempotency-key generator + cost-budget
  enforcer.
- New app `oya-foundry-vcs-orchestrator-app` — runner: invokes
  `oya verify` → pushes to feature branch → opens PR against
  `dev` via `gh pr create --fill` → appends initial
  changeset-event-log row → returns immediately with JSON
  payload `{ changeset_id, current_state: pr_open, pr_number,
  pr_url, subscribe_url, cost_budget_remaining }`.
- Extend `oya vcs done` subcommand in `oya-dev-cli` to delegate
  to the orchestrator app. Add flags: `--changeset`,
  `--subscribe`, `--wait`, `--cost-budget-{usd,tokens}`,
  `--max-agent-invocations`, `--draft`, `--title`, `--body`.
- New subcommand `oya vcs override --changeset --to-state
  --justification --reviewer-name` for human-escape verdicts.
- Crash-safe idempotency: orchestrator picks up where prior
  crash left off, keyed on changeset_id (not PR number).

## Dependencies

- M01-P18-IP-001 (changeset-state kernel) — for state writes.
- IP-003 (webhook receiver) for `--subscribe` callbacks
  (orchestrator works without it; subscription is the optimization).

## Acceptance

- `oya vcs done --changeset <id> --subscribe <url>` returns within
  10 seconds with valid JSON containing `changeset_id`,
  `current_state: pr_open`, `pr_number`, `pr_url`,
  `cost_budget_remaining`.
- Crash-restart test: kill orchestrator after `oya verify` but
  before push; re-run; orchestrator picks up at the push step
  without re-verifying.
- Cost-budget exhaustion: synthetic changeset with
  `--cost-budget-usd 0.01` transitions to `cost_exhausted` on
  first IP-005 invocation; orchestrator surfaces terminal state
  in the next subscription event.
- `oya vcs override` requires `--justification` ≥40 chars AND a
  human signing key; rejects on either missing.
- Override events alarm via the new
  `oya-foundry-fitness-override-justification` lane.

## Symbols to grit-claim

- `crates/oya-foundry-vcs-orchestrator-kernel/src/lib.rs::*`
- `tools/oya-foundry-vcs-orchestrator-app/src/main.rs::main`
- `tools/oya-foundry-vcs-orchestrator-app/src/{state_validator,idempotency,cost_budget}.rs::*`
- `crates/oya-dev-cli/src/commands/vcs.rs::run` (extend `done`
  subcommand; add `override` subcommand)

## Exit evidence

- `/evidence/agentic-vcs-pipeline/ip-004-orchestrator-async-smoke.json`
- `/evidence/agentic-vcs-pipeline/ip-004-crash-restart.json`
- `/evidence/agentic-vcs-pipeline/ip-004-cost-exhausted-terminal.json`
- `/evidence/agentic-vcs-pipeline/ip-004-override-justification.json`
