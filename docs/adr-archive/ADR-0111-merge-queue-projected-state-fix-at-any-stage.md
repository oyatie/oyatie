---
status: Rejected
planning_impact: true
deciders: council-foundry-vcs, council-architecture
date: 2026-05-16
owner: council-foundry-vcs
supersedes: []
superseded_by: []
related:
  - ADR-0110-changeset-state-machine.md
  - ADR-0112-webhook-driven-intelligence-agent-invocation.md
  - ADR-0113-vcs-orchestrator-end-to-end.md
purpose: Define the merge-queue algorithm (projected-merge-state diff validation + fix-at-any-stage adjustment) that prevents divergence + conflicts under heavy agentic concurrency.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0111: Merge queue: projected-merge-state + fix-at-any-stage

## Context

Under agentic load (N changesets in flight concurrently per ADR-0110),
the merge queue is the only point of serialization. Three failure
modes that the naive "FIFO + run-tests-then-merge" model can't
handle:

1. **Divergence**: PR-A admitted (passes CI against dev@v1).
   PR-B admitted right after (passes CI against dev@v1). PR-A
   lands → dev@v2. PR-B's stale-against-v2 tests are now invalid;
   landing it directly may break dev.
2. **Hidden conflict**: PR-A and PR-B both edit `file.rs`. Both
   pass CI independently. Naively landing both produces a broken
   merge (post-A code conflicts with B's edits even though Git's
   3-way merge succeeded).
3. **Fix-at-any-stage**: Agent fixes PR-B mid-queue (new commit
   lands on the PR branch via IP-005 fix-loop). The queue must
   re-validate B's gates against the projected-merge-state, not
   stale validate against the pre-fix state.

The existing IP-006 merge-queue fix-loop family has the scaffolding
(parked-state, retry-budget, fairness, scheduler, tick_log) but
the **projected-merge-state diff validation** algorithm + the
**fix-at-any-stage re-validation protocol** are NOT yet defined.

ADR-0111 locks both algorithms so wave-B implementation can
proceed without per-PR design drift.

## Decision

The merge queue runs a **projected-merge-state** simulation BEFORE
admitting any PR, and re-runs it on every fix-at-any-stage event.

### Projected-merge-state simulation

For each PR in the queue (position i), compute:

```
projected_base_i = squash-merge(dev, PR_0, PR_1, ..., PR_{i-1})
                  (in order; each squash assumes the prior succeeded)
projected_head_i = squash-merge(projected_base_i, PR_i)
```

Then validate THREE invariants on `projected_head_i`:

1. **Diff cleanliness**: `git merge-tree projected_base_i PR_i.head dev`
   produces no conflict markers. Conflict → PR refused admission.
2. **Path-overlap check**: the union of files touched by `PR_i`
   and the union touched by `PR_0..PR_{i-1}` must satisfy the
   "concurrent-safe path" predicate (lane-defined; default = any
   overlap requires explicit `concurrent_safe: true` annotation
   on each PR's metadata, else refuse).
3. **Test re-run against projected base**: pr-tests workflow runs
   against `projected_base_i + PR_i` (synthesized via a transient
   merge commit on a `merge-queue-staging-i` ref). If the
   `projected_base_i` differs from any previously-tested base for
   this PR, tests MUST re-run.

Only after all 3 invariants pass is the PR admitted to position
`i`. On admission, the changeset state transitions from `reviewed`
to `merged_dev` (per ADR-0110), and `dev` advances to
`projected_head_i`.

### Fix-at-any-stage re-validation

When IP-005 (or any agent) pushes a new commit to a PR branch
WHILE that PR is queued, the merge-queue receives a `pr_branch_push`
event (via ADR-0112 webhook). The protocol:

1. **Detect**: webhook payload identifies which queued PR was
   updated. Lookup the PR's position `i` in the queue.
2. **Invalidate**: every queued PR at position ≥ i has its
   pre-computed `projected_head_*` invalidated.
3. **Re-validate**: for positions i, i+1, ..., re-run the
   projected-merge-state simulation in order. Tests re-fire
   against the new projected base.
4. **Re-position**: if PR_i's new diff now path-overlaps a
   pending PR_{i-1} (which previously didn't overlap), PR_i may
   be PUSHED BACK behind PR_{i+1} to preserve admission ordering
   stability. This is per-PR explicit (the fairness.rs scheduler
   handles re-ordering; no PR loses absolute priority — see
   "Fairness invariants" below).
5. **Budget impact**: the re-validation consumes a fresh agent
   invocation slot from the changeset's
   `cost_budget_remaining.agent_invocations_remaining`. Three
   re-validations exhaust most budgets; `cost_exhausted`
   terminal-fail kicks in to prevent infinite fix loops.

### Fairness invariants

Even with fix-at-any-stage re-positioning, the merge queue
preserves:

- **No starvation**: a PR cannot be pushed back more than
  `MAX_REPOSITION = 3` times without triggering a `parked` state
  (human review required to resolve).
- **FIFO modulo conflicts**: PRs without conflicts merge in
  arrival order. PRs with conflicts only move relative to the
  conflicting PRs.
- **Round-robin per-team**: per existing IP-006 fairness.rs, the
  scheduler interleaves PRs from different owning_teams so no
  team monopolizes the queue.

### Conflict-avoidance pre-admit gate (FIRST gate, not last)

Per agentic-pipeline cost calculus (CI is the expensive part),
the conflict-avoidance check runs BEFORE pr-tests, not after:

```
Order today (human-driven):    Order in agentic mode (this ADR):
1. PR opened                    1. PR opened
2. pr-tests runs                2. conflict-avoidance pre-admit (cheap)
3. pr-review fires              3. pr-tests runs (expensive)
4. conflict check at merge      4. pr-review fires
5. merge                        5. merge
```

The pre-admit conflict check is ~1 second (git merge-tree against
projected base); pr-tests is minutes. Failing fast on conflict
saves the CI budget.

## Consequences

### Positive

- Eliminates divergence (PR landed against stale base) at admission
  time, not at merge time. No "passes tests but breaks dev" outcomes.
- Eliminates hidden-conflict landings via projected-merge-state
  simulation.
- Fix-at-any-stage is canonical, not exceptional — agentic
  fix-loop just works.
- Cost-aware: cheap gate first, expensive gate second.

### Negative

- Quadratic-ish scheduling cost: re-validating positions i..n on
  every push touches every queued PR. For queue depth Q,
  worst-case Q*pr-tests runs per fix event. Mitigated by:
  - `MAX_REPOSITION = 3` cap (caps the work per PR)
  - `cost_budget_remaining` per changeset (caps total spend)
  - Per-team round-robin (caps any single team's work)
- Synthesized `merge-queue-staging-i` refs are throwaway — they
  must be cleaned up after each iteration (or queue-deletion task
  for GC'd PRs). The `oya-governance-merge-queue-ref-hygiene`
  lane (new, in wave-C) enforces.
- The "concurrent-safe path" predicate (Decision invariant 2)
  starts conservative (any overlap = refuse). Future relaxation
  via per-product whitelist (e.g., `docs/CHANGELOG.md` is
  always safe to merge concurrently).

### Neutral

- Algorithm is implementable using `git merge-tree` (no working
  tree needed) + `gh api` for ref manipulation. No new external
  dependency.

## Implementation sequencing

- **Wave A** (this ADR Accepted): extend
  the merge-queue fix-loop runner:
  - Add `projected_merge_state` module (per-PR projected base/head
    computation).
  - Add `conflict_avoidance_pre_admit` module.
  - Add `fix_at_any_stage_revalidate` module.
  - Wire webhook listener (per ADR-0112) for `pr_branch_push`.
- **Wave B**: retrofit the gate-validate lanes to allow
  re-runs against projected bases (transient
  `merge-queue-staging-i` refs).
- **Wave C**: `oya-governance-merge-queue-ref-hygiene` lane.
- **Wave D**: `concurrent-safe path` whitelist per-product registry.

## Naming justification

- A new merge-queue conflict kernel (RETIRED per ADR-0363)
  hosts the projected-merge-state algorithm. The existing
  merge-queue fix-loop runner keeps the dispatcher
  role; the new kernel is pure-domain (port-in-kernel per
  ADR-0056).
- Lane id `oya-governance-merge-queue-ref-hygiene` follows
  the `oya-governance-` family.

## Open questions

1. Should `MAX_REPOSITION` be per-PR or per-team? **Decision:
   per-PR** (per-team is gameable; per-PR is the cleanest
   anti-starvation guarantee).
2. Path-overlap predicate v0 = "any overlap = refuse". Is this too
   strict for `docs/`? **Decision: yes, but ship strict first**;
   the whitelist relaxation is wave-D, not wave-A.
3. Re-validation cost model: who pays for the re-tested
   `projected_head_i`? **Decision: the changeset whose push
   triggered the re-validation** (its
   `agent_invocations_remaining` decrements). PRs ahead of it
   incur no cost from the disruption.
