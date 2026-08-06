---
status: Superseded
deciders: council-foundry-vcs, council-architecture
date: 2026-05-16
owner: council-foundry-vcs
supersedes: []
superseded_by: [ADR-0363]
related:
  - ADR-0110-changeset-state-machine.md
  - ADR-0111-merge-queue-projected-state-fix-at-any-stage.md
  - ADR-0112-webhook-driven-intelligence-agent-invocation.md
purpose: Define the `oya vcs done` orchestrator that drives a changeset through the full agentic pipeline (PR-open → CI → review → merge → promote) and the agentic subscription contract for callers.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0113: VCS orchestrator (`oya vcs done`) end-to-end

## Context

ADR-0110 (state machine), ADR-0111 (merge queue), and ADR-0112
(webhook router) lock the substrate. They do NOT define the
ENTRY POINT — the single command an agent invokes to say "this
changeset is ready; drive it to terminal state". Today
`oya vcs done` does the rtk-ai/grit-equivalent local merge; in
the agentic pipeline it must orchestrate:

1. PR-open against `dev` (the new default per the 2026-05-16
   branch-pipeline implementation).
2. Subscription to changeset events (per ADR-0110 async contract).
3. Returning immediately with a `changeset_id` (no synchronous
   block on CI/review/merge).
4. Resilience: if the orchestrator process crashes, the changeset
   keeps moving (the state machine + webhooks own the actual
   advance; the orchestrator is only the kickoff).

ADR-0113 locks the orchestrator's contract and the
caller-subscription protocol.

## Decision

`oya vcs done` becomes the canonical kickoff for the agentic
pipeline. New invocation shape:

```
oya vcs done
  --changeset <id>           # ULID from `oya vcs claim`
  [--subscribe <url>]        # webhook URL for state-change events
  [--wait]                   # opt-in synchronous wait (default: async)
  [--cost-budget-usd <n>]    # per-changeset USD cap (default: $10)
  [--cost-budget-tokens <n>] # per-changeset token cap (default: 2M)
  [--max-agent-invocations <n>] # per-changeset invocation cap (default: 50)
  [--draft]                  # open PR as draft
  [--title <text>]
  [--body <text>]
```

### Async-by-default contract

1. Agent invokes `oya vcs done --changeset <id> --subscribe <url>`.
2. Orchestrator runs `oya verify` locally (per existing
   `oya submit` flow). On non-zero, return failure immediately
   (changeset never enters `pr_open`).
3. Orchestrator pushes the changeset's commits to a feature
   branch and opens a PR against `dev` via `gh pr create --fill`
   (or with explicit `--title/--body`).
4. Orchestrator writes the initial `changeset-event-log` row:
   `opened → working → verified → pr_open` (skipping intermediate
   states it directly observed; `skipped: true` flag set).
5. Orchestrator returns 0 IMMEDIATELY with `{ "changeset_id": ...,
   "current_state": "pr_open", "pr_number": ... }` on stdout (JSON).
6. The rest of the pipeline (CI → review → merge → promote) runs
   asynchronously, driven by ADR-0112 webhooks emitting
   `changeset-event-log` rows.
7. The `--subscribe <url>` callback receives every state
   transition; terminal events (`produced`, `abandoned`,
   `rejected`, `cost_exhausted`) are the unsubscribe signal.

### Synchronous `--wait` (opt-in for humans + tests)

With `--wait`, the orchestrator polls
`registry/vcs/changeset-event-log.json` for the changeset_id
every 30 s and exits when a terminal state is observed. Bounded
total wait = 2 hours (configurable via
`--wait-timeout-seconds`); exceeded → returns the current state
and exit code 3 (still-in-flight). This is for human-driven
testing only; agentic callers MUST use `--subscribe`.

### Crash resilience

If the orchestrator process is killed mid-execution:
- The PR may or may not have been opened (depends on crash point).
- The changeset-event-log either has `pr_open` (PR exists) or
  doesn't (PR doesn't).
- A subsequent `oya vcs done --changeset <id>` is IDEMPOTENT:
  - If `pr_open` already in log → no-op for the PR-create step.
  - If the feature branch already exists on remote → reuse it.
  - If `oya verify` already ran (verified state in log) →
    skip the re-verify.
  - The orchestrator can pick up wherever the previous instance
    crashed.

The idempotency anchor is the `changeset_id`, NOT the PR number
(which may not yet exist).

### Cost-budget enforcement

The three caps (USD, tokens, agent_invocations) are written into
the FIRST `changeset-event-log` row at `pr_open` time. Every
subsequent agent invocation (IP-004 review, IP-005 fix, merge-queue
re-validation per ADR-0111) decrements the appropriate
counters. When any counter hits zero or below, the next agent
invocation transitions the changeset to `cost_exhausted` (terminal
fail per ADR-0110 update).

The caps are CONFIGURABLE per changeset but bounded by the team's
monthly budget (per `feedback_quality_performance_scalability_bar`
hyperscaler economics) — the
`oya-governance-changeset-cost-budget-monthly` lane
(crate at `crates/oya-governance-changeset-cost-budget-monthly/`)
asserts cumulative monthly spend per team stays under the cap. The
lane sums `cost_usd` across all changeset event-log rows scoped to
team-id for the current calendar month and fails the gate if the
total exceeds the team's `monthly_budget_usd` row in
`registry/team-budgets.yaml`.

### Override surface

For the rare case where an agent's verdict is wrong (false
APPROVE or false REJECT), `oya vcs override` provides a human
escape hatch:

```
oya vcs override
  --changeset <id>
  --to-state <reviewed|rejected|abandoned>
  --justification <text>      # REQUIRED, ≥40 chars
  --reviewer-name <text>      # REQUIRED, human reviewer identity
```

The override appends a special event log row with
`emitted_by: human-override:<reviewer-name>`. Per
ADR-0083, the override's signature is the reviewer's git signing
key (Ed25519). The `oya-governance-override-justification`
lane (crate at `crates/oya-governance-override-justification/`)
asserts every override row has a justification ≥40 chars + a
signature; lane reads the event log rows where
`emitted_by like 'human-override:%'` and fails the gate if any row
has `len(justification) < 40` OR `signature_verified = false`.

Override frequency is observable via the
`oya-governance-override-frequency-alarming` lane (crate at
`crates/oya-governance-override-frequency-alarming/`); lane
counts overrides per team per calendar month and fires alert when
count > 5 (default threshold; per-team override in
`registry/team-budgets.yaml#override_alert_threshold`) — sign
that the agent verdicts are unreliable + needs human triage.

### Outputs (stdout JSON shape)

Async mode (default):
```json
{
  "changeset_id": "cs_2026-05-16T01:39:46Z_abc12345",
  "current_state": "pr_open",
  "pr_number": 4,
  "pr_url": "https://github.com/jason931225/oyatie/pull/4",
  "subscribe_url": "https://example.com/webhook/changeset-events",
  "cost_budget_remaining": {
    "usd_remaining": 10.0,
    "tokens_remaining": 2000000,
    "agent_invocations_remaining": 50
  }
}
```

Sync mode (`--wait`):
```json
{
  "changeset_id": "...",
  "final_state": "produced",          // or abandoned/rejected/cost_exhausted
  "pr_number": 4,
  "merged_sha": "...",
  "staged_sha": "...",
  "produced_sha": "...",
  "total_duration_seconds": 1834,
  "cost_consumed": {
    "usd": 3.17,
    "tokens": 1842117,
    "agent_invocations": 11
  }
}
```

## Consequences

### Positive

- Agentic callers get a one-line invocation
  (`oya vcs done --changeset <id> --subscribe <url>`) that
  starts the full pipeline and returns immediately. No
  synchronous wait in agentic mode.
- The orchestrator owns ZERO state — all state is in the
  changeset-event-log. Crash-restart-replay is canonical.
- Cost budgets are first-class; runaway loops are bounded.
- Override surface exists for human triage but is alarmed when
  overused.

### Negative

- Async-by-default is a behavior shift from the current
  `oya submit` synchronous flow. Callers must subscribe; sync
  becomes opt-in.
- New event log to maintain; subscriber endpoint to host
  (could be the same Foundry control-plane endpoint as ADR-0112
  webhook receiver).
- The `--wait` polling at 30 s is a fallback; a long-lived
  changeset with `--wait` will hold the calling process for up
  to 2 hours.

### Neutral

- The orchestrator app (new in wave-A) is small
  (~400 LOC) and delegates almost everything to the existing
  IP-004/005/006 + promotion workflows + webhook receiver.
  It's a thin coordination layer, not a new substrate.

## Implementation sequencing

- **Wave A** (this ADR Accepted):
  1. The orchestrator kernel — pure-domain
     state-validator + idempotency-key generator.
  2. The orchestrator app — runner; integrates
     `oya verify` + `git push` + `gh pr create` + event-log
     appending.
  3. `oya vcs done` subcommand in `oya-dev-cli` delegates to
     the orchestrator app.
  4. `oya vcs override` subcommand (gated on human-signing-key).
- **Wave B**: webhook subscription endpoint (lives in the same
  receiver as ADR-0112, exposed at
  `/webhook/changeset-events/<changeset_id>`).
- **Wave C**: cost-budget enforcement lanes
  (`oya-governance-changeset-cost-budget-monthly`,
  `-override-justification`, `-override-frequency-alarming`).

## Naming justification

- The orchestrator kernel + app (RETIRED per ADR-0363) followed
  the canonical role-suffix convention.
- Subcommand `oya vcs done` retains its name (canonical CLI
  surface; semantics are extended, not renamed).
- `oya vcs override` is the human-escape subcommand.

## Open questions

1. Should `--subscribe` URL be required for agentic mode (refuse
   to start without one)? **Decision: NO** — the changeset-event-log
   is the source of truth even without subscription. Callers can
   poll the log directly if they prefer. Subscription is an
   optimization, not a contract.
2. Should the orchestrator support multi-changeset transactions
   (atomic landing of N changesets that must succeed together)?
   **Decision: NO in v1** — the merge queue serializes; multi-CS
   atomicity is composed via a coordinator changeset that depends
   on the N children.
3. What's the contract if a changeset is `produced` but the
   production refs are later force-rewound? **Decision: a
   regression triggers a NEW changeset with intent "revert <id>"**;
   the original changeset stays `produced` (per ADR-0110
   monotonicity). The revert changeset is its own pipeline run.
