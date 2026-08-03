# Living Monorepo Governance Graph

*Created At: 2026-07-10T07:24:18.982694+00:00*

## Goal

Manage the monorepo and full dev lifecycle as one governed, federated (per-cell), content-addressed graph by extending the existing owned-Rust governance/corpus AST graph, so that every artifact is either a governed node or a compiled projection, and contradiction, sprawl, and staleness become structurally impossible rather than merely detected.

## User Stories

1. **As a** Platform developer, **I want to** create worktrees/branches via raw git without being blocked, **so that** agentic tooling (OMC/OMX/workflow) is never blocked at creation while strays are still lifecycle-managed.
2. **As a** Platform developer, **I want to** have my unmerged/local-only work preserved before any reap, **so that** nothing salvageable is ever lost during cleanup or steady-state sweeps.
3. **As a** Governance operator, **I want to** run one continuous background reconciler that harvests then reaps ungoverned strays, **so that** the clean baseline stays clean without manual shepherding.
4. **As a** Governance operator, **I want to** triage parked durable nodes and promote valuable ones to PRs/ADRs later, **so that** abandoned-but-valuable work re-enters the governed flow on demand rather than flooding the merge queue.
5. **As a** Multiple developers each running multiple agents, **I want to** commit at extreme velocity against a concurrency-safe graph, **so that** high-throughput agentic work does not corrupt truth or serialize on local actions.
6. **As a** External adopter (stranger's repo), **I want to** swap in a different policy pack against the neutral engine, **so that** the governance system is portable to any repo without oyatie-specific nuance.

## Constraints

- Owned-Rust only; no new python/shell/CLI/dependencies
- Neutral engine + policy-as-data: no oyatie-nuance embedded in the engine
- Git artifacts modeled as ephemeral TTL'd work-nodes inside the ONE graph, not a separate reconciler
- Extend the existing corpus + cross-artifact-agreement evaluator; do not rebuild
- Operator console and gates must be delivered together
- Fail-closed enforcement lands on durable governance invariants (stray past-TTL with no harvest edge), never on blocking creation itself
- Enforcement must remain non-blocking at velocity: gate at the durable/merge boundary + async detector, not a serial lock on every local action
- Ungoverned creation is allowed-but-auto-expired; a sanctioned governed-creation path also exists
- Deterministic invariants gate every state transition; LLM is advisory-only
- Graph must support high-concurrency writes: append-only/idempotent content-addressed nodes, edges declared-from-one-end to avoid write conflicts, federated per-cell sharding
- Graduates to a Design Doc amending the ADR-0516..0535 agentic-delivery-fabric cluster

## Success Criteria

1. Contradiction, sprawl, and staleness are structurally impossible, not merely detected
2. Existing sprawl (369 local worktrees, 550 local branches, 91 remote, plus .omc/.omx/tmp) is harvested and reaped to a clean baseline and prevented thereafter
3. Docs, handoffs, roadmaps, dashboards, and SSOT are projections compiled from the graph, never hand-maintained
4. Portable to any repo: the stranger's-repo test passes with a swapped policy pack
5. Reap-by-default sweep keeps the 900+ backlog cheap and deterministic (no manual salvage review)
6. No unmerged or local-only work is ever lost (durable signal = commits unreachable from origin/dev OR explicit harvest/promote marker)
7. Newly-created ungoverned sprawl is continuously caught by a detector (no governance node and no harvest edge) and auto-expired on TTL
8. Harvest/reap runs as a continuous background reconciler, never a batch job, scaling with devs×agents
9. Extreme-velocity merges run through an auto-rebase queue + projected merge-state with zero manual shepherding
10. Concurrent conflicting edits surface as explicit conflicts_with edges rather than silent divergence
11. Full anti-pattern set is each prevented by a deterministic graph invariant: sprawl (ungoverned-artifact detector + TTL), functional regression (full test ladder + regression gate), quality degradation (quality gates on R0 path), architectural regression/drift (ADR-0280 dependency DAG + capability membership lint + ArchUnit/Axivion/FINOS-CALM-class conformance), silent regressions/failures (fail-closed + no-silent-fallback, every deny/error/degraded path is loud), dead code/docs (reachability/orphan detection + report→archive reap), staleness (anchor_fingerprint_drift: recorded content-address vs live code-AST hash), contradictions (conflicts_with edges + cross-artifact-agreement invariants)

## Assumptions

- Because last-touch (commit, checkout, or file write) resets the TTL, the harvest sink only ever receives abandoned strays, so no active-lane special case is needed
- The 900+ item backlog cleanup is simply the first sweep of the same uniform steady-state rule (one sink, one mechanism)
- A one-time cleanup precedes steady-state fail-closed enforcement
- Content-addressed nodes being append-only/idempotent and edges declared-from-one-end is sufficient to avoid write-conflicts by construction under concurrency

## Decide Later

The following items were deferred or identified as premature at this stage. They should be revisited when more context is available:

- The exact TTL durations per work-node kind (worktree / branch / .omc-.omx marker / tmp scratch) — dependent on policy-pack defaults tuned post-cleanup
- Docs→projection cutover / projection migration path (breadth-keeper track, not yet specified)
- Policy-pack/engine split for the stranger's-repo test (breadth-keeper track, not yet specified)
- Console + gates MVP surface definition (breadth-keeper track, not yet specified)
- Later triage that promotes valuable parked durable nodes to PRs/ADRs

## Existing Codebase Context

- **oyatie** (`/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_5a241375/oyatie`)
- **oyatie** (`/Users/jasonlee/Developer/oyatie`)

---
*PM ID: pm_seed_interview_20260710_070532*
*Interview ID: interview_20260710_070532*
