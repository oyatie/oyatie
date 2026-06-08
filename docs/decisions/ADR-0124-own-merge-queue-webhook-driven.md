---
adr_id: ADR-0124
title: Own merge-queue policy — webhook-driven, GitHub-merge-queue-free
status: Superseded
date: 2026-05-17
owner: jason931225
deciders: jason931225
supersedes: none
superseded_by: [ADR-0515]
related:
  - ADR-0111-merge-queue-projected-state-fix-at-any-stage.md
  - ADR-0112-webhook-driven-intelligence-agent-invocation.md
  - ADR-0113-vcs-orchestrator-end-to-end.md
  - ADR-0116-retire-external-agent-coordination-tooling.md
---

# ADR-0124: Own merge-queue policy — webhook-driven, GitHub-merge-queue-free

## Status

Superseded by ADR-0515 (unified Rust-native CI/CD) — 2026-06-06: merge-queue intent + the 20-row blocker taxonomy fold into oya-ci Tide; file-overlap clustering → graph-exact conflicts(a,b).

## Context

Dev branch protection had `required_status_checks.strict: true` ("Require
branches to be up to date before merging"). Combined with N open PRs, this
forced an O(N²) rebase cascade: every merge to dev invalidated every other
PR, requiring re-rebase + full re-CI on each one.

GitHub's native **merge queue** would solve this, but is **not available**
on this repository plan.

We already own the substrate to do this ourselves:

- The review/merge-queue kernel — scheduler, parked-state,
  fairness, speculative rebase, per-PR retry budget (ADR-0111).
- The merge-queue conflict kernel — conflict detection
  on projected merge state.
- ADR-0112 — webhook-driven Foundry agent invocation (event-driven, not
  cron-driven).
- ADR-0113 — end-to-end VCS orchestrator (PR open → admission → queue →
  merge → release).

## Decision

1. **Branch protection on `dev` runs `strict: false` permanently.** This
   removes the up-to-date requirement from GitHub's side. Required status
   checks, signed commits, conversation-resolution, and `enforce_admins`
   all remain ON.

2. **Branch protection is encoded as IaC** at `infra/branch-protection/dev.json`
   and applied via `scripts/branch-protection-apply.sh`. A daily drift
   workflow (`.github/workflows/branch-protection-drift.yml`) fails CI if
   the live config diverges from the canonical config.

3. **The merge queue is implemented by us, webhook-driven.** It runs as a
   GitHub Actions workflow whose triggers ARE webhook events emitted by
   GitHub:

   | Event | Queue tick action |
   |---|---|
   | `pull_request: opened/reopened/ready_for_review` | enqueue |
   | `pull_request_review: submitted` (state=approved) | mark approved |
   | `check_suite: completed` | mark CI verdict; if head-of-queue + green → merge |
   | `push` to `dev` | head merged → re-rebase next-in-queue ONLY if file-overlap with merged head |
   | `workflow_run: completed` (CI fix-loop dispatcher) | requeue head at original position |
   | `pull_request: synchronize` (force-push) | revalidate; if still approved + CI green → re-eligible |

   **No cron.** Each scheduler tick is triggered by exactly one webhook event.

4. **Queue ordering is by `pull_request.createdAt`**, with the fairness
   primitives from the review/merge-queue kernel's `fairness` module
   applied to prevent starvation of any agent lane.

5. **File-overlap clustering** — the scheduler computes the set of
   touched files per PR and only re-rebases the next-in-queue PR when its
   file-set intersects the merged head's file-set. PRs touching disjoint
   crates are NOT re-rebased and merge independently. This is the key
   property that breaks the O(N²) cascade.

## Merge-blocker taxonomy and queue handling

Every category of merge blocker is enumerated below with its queue-side
response. The scheduler MUST handle each row; missing handlers are
treated as parking with a fixuptask filed.

| # | Blocker | Detection event | Queue action | Recovery |
|---|---|---|---|---|
| 1 | Rebase cascade (PR is BEHIND dev) | `push` to dev | re-rebase ONLY if file-overlap with merged head, else no-op | one re-rebase per cluster head; PRs in disjoint clusters skip |
| 2 | Required CI red | `check_suite: completed` failure | requeue at original position; trigger M01-P17-IP-005 fix-loop dispatcher | fix-loop opens fix commits; on next `pull_request: synchronize` queue re-evaluates |
| 3 | Reviewer-agent rejection | `pull_request_review: submitted` state=CHANGES_REQUESTED | requeue at original position; trigger reviewer-agent fix-loop | same as #2 |
| 4 | Unresolved review thread | live query `pullRequest.reviewThreads.isResolved=false` | park with `ParkedReason::ReviewThreadOpen`; emit thread-id list | per-thread audit + fix OR rebuttal; never bulk-resolve (see [[codex-bulk-resolve-antipattern]]) |
| 5 | Missing signed-commit | `pull_request: synchronize` + commit verification API | park with `ParkedReason::UnsignedCommit` | author re-signs and force-pushes; `synchronize` event re-validates |
| 6 | Required status check stuck pending > 30 min | scheduled internal tick + check_run age inspection | force-retry the workflow_run via `gh api workflow_runs/{id}/rerun-failed-jobs` once, then park | second timeout → fixuptask + page |
| 7 | Textual merge conflict, known pattern (audit-chain.jsonl, lib.rs pub mod) | rebase fails with conflict markers | auto-resolve via `scripts/jsonl-union-merge.py` / `scripts/libs-pubmod-union.py`; commit + push | conflict-kernel records the resolve in projected state |
| 8 | Textual merge conflict, unknown pattern | rebase fails | park with `ParkedReason::TextualConflict`; emit conflict file list | human or agent fixes; `synchronize` event un-parks |
| 9 | PR in draft | `pull_request: opened/converted_to_draft` | exclude from queue | `ready_for_review` event enqueues |
| 10 | Force-push during queue residency | `pull_request: synchronize` | invalidate prior CI verdict; remain in queue at original position | new CI run drives the next tick |
| 11 | PR closed/withdrawn | `pull_request: closed` | remove from queue; promote next | n/a |
| 12 | Squash-merge denied by protection drift | merge API returns 405/422 | park with `ParkedReason::ProtectionDrift`; trigger drift workflow | drift workflow re-applies IaC; queue resumes |
| 13 | GitHub API rate limit | API returns 403 with `X-RateLimit-Remaining: 0` | exponential backoff per retry-budget primitive (`pr_retry_budget.rs`) | budget exhausted → fixuptask |
| 14 | Same-file logical collision (no textual conflict, but two PRs both add `pub mod X`) | rebase succeeds but post-rebase build fails | treated as blocker #2 (CI red); fix-loop dispatched | next-in-cluster gets re-rebased after head merges |
| 15 | Bot author (Renovate/Dependabot) without signed-commit | commit verification API | allowlist check; if allowlisted skip blocker #5 | denylist → blocker #5 |
| 16 | Merge-queue workflow itself fails | `workflow_run: completed` (queue workflow) failure | emit `oya-governance-merge-queue-health` lane red on dev | queue health lane pages oncall; manual recovery |
| 17 | Required reviewer-agent has not run | reviewer-agent status check missing on PR | enqueue with `ParkedReason::AwaitingReviewerAgent`; trigger reviewer-agent invocation | reviewer-agent completion event re-evaluates |
| 18 | Branch protection requires N approvals, has N-1 | live PR query | park with `ParkedReason::AwaitingApprovals` | `pull_request_review: submitted` event re-evaluates |
| 19 | PR base branch is not `dev` | enqueue-time validation | reject; do NOT add to queue | author retargets the PR |
| 20 | PR exceeds size budget (≥1500 lines or ≥25 files) | enqueue-time diff size check | park with `ParkedReason::OversizeChange`; require explicit unparking ADR | per-PR justification doc or split |

Categories #1-3 are the high-frequency blockers; the rest are tail-risk
handling. Any blocker without a handler is treated as park-with-fixuptask
so nothing silently drops out of the queue.

## Consequences

### Positive
- Eliminates the rebase cascade. 25 open PRs (current state) drain on
  first-CI-green per PR.
- Semantic-merge-conflict risk is bounded: required CI on dev catches the
  rare case where two PRs are individually green but break together;
  auto-revert via existing fix-loop dispatcher (M01-P17-IP-005).
- File-overlap clustering preserves serialization where it matters
  (same-file edits) without paying O(N²) elsewhere.
- Webhook-driven, not cron-driven — sub-second latency from PR event to
  queue tick. No wasted polling cycles.
- All policy is IaC; drift detection prevents silent regression to
  `strict: true`.

### Negative
- Semantic merge conflicts CAN land before dev CI catches them. Mitigated
  by the existing CI fix-loop dispatcher.
- We carry the maintenance cost of the queue logic ourselves vs delegating
  to GitHub. Mitigated by the existing kernel crates already covering
  the hard parts (fairness, retry budget, speculative rebase, projected
  state).

### Risk
- If the merge-queue workflow itself fails silently, PRs stop merging.
  Mitigation: `oya-governance-merge-queue-health` lane checks queue
  depth + last-merge-timestamp and pages on staleness > 1 hour (follow-on,
  tracked as `F-MERGE-QUEUE-HEALTH-LANE` in `registry/fixuptasks.jsonl`).

## Implementation phases

- **Phase 1 — IMMEDIATE (this PR)**: flip `strict: false`, encode IaC,
  drift workflow, this ADR. Done in commit accompanying this ADR.
- **Phase 2 — POLLER WIRING (follow-on PR)**: a new `oya vcs merge-queue
  tick --event <event.json>` subcommand on `oya-dev-cli` that reads a
  GitHub webhook payload from stdin/path, calls
  `oya_foundry_vcs_review_mergequeue_kernel::Scheduler::tick()`, and
  acts on the returned `TickAction` (merge PR, rebase next, park PR,
  trigger fix-loop). Workflow YAML at
  `.github/workflows/oya-merge-queue-tick.yml` invokes the subcommand
  on each listed webhook event. Tracked as
  `F-MERGE-QUEUE-WEBHOOK-POLLER-WIRING` in `registry/fixuptasks.jsonl`.
- **Phase 3 — HEALTH LANE (follow-on PR)**: `oya-governance-merge-queue-health`
  lane as a required status check on dev. Tracked as
  `F-MERGE-QUEUE-HEALTH-LANE`.

## Sources

- ADR-0111 — merge-queue projected state, fix-at-any-stage
- ADR-0112 — webhook-driven Foundry agent invocation
- ADR-0113 — VCS orchestrator end-to-end
- GitHub branch protection REST API — `required_status_checks.strict`
