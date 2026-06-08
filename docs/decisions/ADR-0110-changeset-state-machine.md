---
status: Superseded
deciders: council-foundry-vcs, council-architecture
date: 2026-05-16
owner: council-foundry-vcs
supersedes: []
superseded_by: [ADR-0363]
related:
  - ADR-0054-grit-scaffold-claim-pattern.md
  - ADR-0083-tier-1-error-handling-strict.md
  - ADR-0086-event-sourced-evidence-log.md
purpose: Define the canonical changeset state machine and event log shape that the dev > staging > production pipeline orchestrates against.
---

# ADR-0110: Changeset state machine

## Context

The FINAL-FINAL branch pipeline (per
`feedback_branch_pipeline_final_final`, 2026-05-12) implements
`per-agent worktree → local main → dev → staging → production`.
The 2026-05-16 landing (PR #3, PR #4) shipped the branch topology
+ auto-promotion workflows, but the **changeset** — the atomic
intent unit that traverses this pipeline — is still implicit.
Without a canonical state machine:

- IP-005 (CI fix-loop), IP-004 (PR review), IP-006 (merge queue),
  and the new promotion workflows each track partial state in their
  own ledgers — no single source of truth for "where is this
  changeset right now?"
- The agentic pipeline (per ADR-0112 webhook integration) cannot
  determine which agent to dispatch on which event without a
  state-aware router.
- The `oya vcs done` orchestrator (per ADR-0113) cannot decide
  whether a changeset is ready to advance until it can query a
  monotonic state.

ADR-0110 locks the changeset state machine + event log shape so
ADR-0111/0112/0113 can build on a stable contract.

## Decision

A changeset advances through a CLOSED ENUM of 9 states. Transitions
are MONOTONIC (no backwards moves) and EVENT-SOURCED (every
transition appends one row to the `changeset-event-log`).

### The 9 states

| # | State | Owner | Entry trigger | Exit trigger |
|---|---|---|---|---|
| 0 | `opened` | claim agent | `oya vcs claim --intent ...` | first edit lands in worktree |
| 1 | `working` | claim agent | first edit | `oya vcs verify` invoked |
| 2 | `verified` | claim agent | `oya vcs verify` returns OK | `oya vcs done` invoked |
| 3 | `pr_open` | `oya vcs done` orchestrator | PR opened against `dev` | pr-tests workflow_run fires |
| 4 | `ci_running` | IP-005 / pr-tests | pr-tests workflow_run starts | pr-tests workflow_run completes |
| 5 | `ci_passed` | IP-005 | pr-tests `conclusion=success` AND IP-005 fix-loop converges OR no fix needed | IP-004 pr-review fires |
| 6 | `reviewed` | IP-004 | pr-review APPROVE emitted | IP-006 merge-queue admits |
| 7 | `merged_dev` | IP-006 | `dev` ref advances to changeset HEAD | promote-dev-to-staging fast-forwards |
| 8 | `staged` | promote-dev-to-staging | `staging` ref advances | promote-staging-to-production fast-forwards |
| 9 | `produced` (terminal) | promote-staging-to-production | `production` ref advances | (none — terminal) |

Plus **3 terminal-fail states** (also closed enum):

| # | State | Entry trigger |
|---|---|---|
| F1 | `abandoned` | `oya vcs abandon --changeset <id>` (manual) OR pr_open age > 30 days no advance |
| F2 | `rejected` | IP-004 emits REJECT OR human-review verdict rejects |
| F3 | `cost_exhausted` | `cost_budget_remaining` drops below zero on any axis (USD, tokens, or agent_invocations) |

Total state enum: 9 advancing + 3 terminal-fail = 12 closed values.

### Monotonic invariant

For any changeset, the sequence of states observed in its event log
MUST be a non-decreasing subsequence of `[opened, working,
verified, pr_open, ci_running, ci_passed, reviewed, merged_dev,
staged, produced]` OR end at one of `[abandoned, rejected]`.

A transition that violates monotonicity is a fatal error in the
emitting agent and MUST surface via the `changeset-state-monotonicity`
CI fitness lane (new lane in this ADR's wave-A implementation).

### Skip-states

Three states permit being skipped under canonical conditions:

- `working` → `verified` directly when an agent runs `oya vcs verify`
  before making any edits (e.g., dry-run verification).
- `ci_running` → `ci_passed` directly when pr-tests reports success
  on the first run with no IP-005 intervention.
- `staged` → `produced` directly when canary-observability lane
  (FUTURE per ADR-0111) emits an aggregate verdict that covers
  both layers.

Skipping is documented in the event log via an explicit `skipped`
flag on the destination state's event row.

### Event log shape (agentic-tuned)

Every state transition appends one row to
`registry/vcs/changeset-event-log.json` with this shape:

```jsonc
{
  "changeset_id": "cs_2026-05-16T01:39:46Z_abc12345",  // ULID-shaped
  "dedup_key": "cs_..._reviewed_2026-05-16T01:42:18Z",  // idempotency key
  "from_state": "ci_passed",
  "to_state": "reviewed",
  "at": "2026-05-16T01:42:18.443Z",
  "emitted_by": "<ci-pipeline>",
  "cost_budget_remaining": {
    "usd_remaining": 4.73,                  // USD budget left for this changeset
    "tokens_remaining": 1_842_117,          // total tokens left across all retries
    "agent_invocations_remaining": 11        // remaining agent runs in this changeset
  },
  "evidence": {
    "pr_number": 4,
    "head_sha": "779f7fd6...",
    "review_verdict": "APPROVE",
    "review_evidence_path": "evidence/pipeline-maturity-glue/ip-004-pr-review/4/rollup.json"
  },
  "alternates_considered": [
    // For non-deterministic transitions (e.g., IP-005 picks fix-A over
    // fix-B), the rejected alternatives are persisted so audit can
    // replay any branch. Empty array for deterministic transitions.
  ],
  "skipped": false,
  "signature": "<Ed25519 sig over (changeset_id, dedup_key, to_state, at, emitted_by, cost_budget_remaining) keyed by the agent's signing key per ADR-0058>"
}
```

The `dedup_key` is the canonical idempotency anchor — webhook
receivers MUST check the dedup_key against the event log before
appending. Same dedup_key = same transition = no-op. This makes
every transition crash-safe + retry-safe.

`signature` is the canonical anti-tamper field — any consumer of
the event log MUST verify the signature using the agent's published
key before trusting a state transition.

`changeset_id` is a ULID derived from `(timestamp, random)` so it
sorts naturally. Format: `cs_<RFC3339-Z>_<8-hex>`.

### Async event-subscription contract

Agents driving the pipeline MUST NOT poll. The canonical contract is:

1. Caller invokes `oya vcs done --changeset <id> --subscribe <callback-url>`.
2. Orchestrator (per ADR-0113) returns immediately with the
   current state.
3. Every subsequent state transition fires a webhook POST to the
   callback URL with the new event log row as the body.
4. Caller receives a terminal event (`produced`, `abandoned`,
   `rejected`, or `cost_exhausted`) and unsubscribes.

Synchronous wait is an opt-in `--wait` flag for human-driven
testing; agentic callers MUST use the subscription model.

### Backward-compat with IP-006 tick_log

IP-006's existing merge-queue fix-loop tick log
keeps emitting queue-local events. ADR-0110's adoption layer adds
a one-way bridge: every IP-006 admission emits BOTH a tick_log row
(queue-internal) AND a changeset-event-log row (pipeline-spanning).
A future ADR may unify the two; for now they coexist.

## Consequences

### Positive

- Single source of truth for changeset state across IP-004/005/006
  + promotion workflows. Eliminates the "which dispatcher owns this
  state?" ambiguity.
- Event-sourced log allows replay, audit, and timeline
  reconstruction (per ADR-0086 patterns).
- Monotonicity invariant is CI-checkable; drift surfaces as a
  bounded set of violations (not a long-tail debugging session).
- Closed enum (11 values) prevents drift via the
  `oya-governance-changeset-state-enum-closed` lane (new in
  wave-B).

### Negative

- New event log to maintain + new fitness lane to author.
- Every existing dispatcher (IP-004/005/006 + 2 promotion workflows)
  must be retrofitted to emit changeset-event-log rows. Estimated
  10-line addition per emitter; not free.
- The 30-day `abandoned` timeout is a heuristic — long-running
  cross-team feature changesets may need explicit extension. Adding
  an `extended` flag is a future addition.

### Neutral

- The state machine is implementable in either a typed Rust enum
  or a string-keyed YAML registry; the changeset-state kernel uses
  a typed enum to make the closed set a compile-time guarantee.

## Implementation sequencing

- **Wave A** (this ADR is Accepted):
  1. The changeset-state kernel (closed-enum +
     monotonicity validator, pure-domain port-in-kernel per
     ADR-0056).
  2. The changeset-state runner (appends events
     to `registry/vcs/changeset-event-log.json`).
  3. `oya-governance-changeset-state-monotonicity` lane
     (asserts every changeset's event log is monotonic).
  4. `oya-governance-changeset-state-enum-closed` lane
     (asserts every emitted `to_state` is in the closed enum).
- **Wave B**: retrofit IP-004/005/006 + promotion workflows to emit
  changeset-event-log rows. Each emitter gets a ~10-line addition
  invoking the changeset-state-app on every action.
- **Wave C**: dependency for ADR-0111 (merge-queue) + ADR-0112
  (webhook router) + ADR-0113 (`oya vcs done` orchestrator). Those
  ADRs cannot proceed to implementation until Wave A is green on
  `dev`.

## Naming justification

- The changeset-state kernel crate (RETIRED per ADR-0363) followed the
  `kernel` role suffix per ADR-0056 12-value layer enum; same
  kernel-pattern as every other check kernel.
- Lane id `oya-governance-changeset-state-monotonicity` (and
  `-enum-closed`) — `oya-governance-` family prefix per
  registry/quality/lanes.yaml conventions; descriptive suffix.

## Open questions

1. Should `abandoned` and `rejected` be separately rendered in the
   changeset-event-log as terminal events, or should the log
   simply stop appending and a `terminal_reason` field on the last
   row capture the outcome? **Decision: separate `to_state` values**
   (the closed-enum invariant is cleaner if every terminal has its
   own state).
2. Should event signatures use Ed25519 or HMAC-SHA256? **Decision:
   Ed25519** (matches commit signing; agent's signing key is the
   same one used for git commits per ADR-0058).
3. Are there cases where a changeset legitimately needs to
   regress (e.g., `merged_dev` → `pr_open` after a force-revert)?
   **Decision: NO** — a revert is a NEW changeset, not a state
   regression on the original. Monotonicity is preserved.
