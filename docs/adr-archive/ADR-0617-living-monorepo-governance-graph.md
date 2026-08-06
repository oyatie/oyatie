---
id: ADR-0617
title: "The Living Monorepo Governance Graph — monorepo management + project lifecycle as one governed, federated, content-addressed graph (amends the ADR-0516 fabric apex)"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-07-10
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-701]
amends: [ADR-0516]
depends_on: [ADR-0516, ADR-0517, ADR-0520, ADR-0522, ADR-0541, ADR-0562, ADR-0580, ADR-0280]
related: [ADR-0518, ADR-0519, ADR-0521, ADR-0530, ADR-0551, ADR-0552, ADR-0563, ADR-0615]
related_specs:
  - /specs/capability-registry.json
  - /specs/substrate-dependency-dag.json
milestone: W0
---

> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Living monorepo governance graph

# ADR-0617: The Living Monorepo Governance Graph

## Status

**Proposed — 2026-07-10** (ratified under the founder's 2026-07-08 autonomous-drive delegation and an
explicit 2026-07-10 PM interview that resolved every fork to ambiguity 0.10; door: one-way — the
graph-as-management-substrate + fail-closed governance is a placement/enforcement commitment the fabric
then implements). Lifecycle status stays **Proposed** until formal Accept rides cross-artifact
propagation; this ADR records the founder's decisions and **amends ADR-0516** (extends the fabric apex
with the monorepo-management + project-lifecycle dimension). It builds no substrate here; it is the
design the Living-Monorepo work implements. Graduated from the PM `Living Monorepo Governance Graph`
(harvested to `.omc/ultragoal/living-monorepo-pm-2026-07-10.md`).

## Context

ADR-0516 set the agentic-delivery-fabric apex; ADR-0517 ratified the owned content-addressed AST
substrate *doctrine*, realized so far as the ADR-0580 `governance/corpus/` Phase -1 spike; ADR-0522
established "one graph, four runners" for the buck2 *target* graph. What was still missing is the
unifying operating model for a *different* graph — the corpus/knowledge graph: **how the whole monorepo
and its full development lifecycle are managed**.

Today the repo bleeds **ungoverned ephemeral state** with no single source of truth and no lifecycle,
surfacing as **contradiction, sprawl, staleness** across five surfaces that look separate but are one
problem: (1) code (the AST), (2) docs (PRD/RFC/ADR, prone to cross-document contradiction), (3)
capabilities + progress, (4) **agentic-tooling ephemera** — OMC (`.omc/`) + OMX (`.omx/`) markdown, GJC
auto-checkpoint commits, `/tmp` scratch — and (5) **git artifacts**: measured this session at **369 local
worktrees, 550 local branches, 91 remote branches**, most never harvested and never reaped. This is not
five problems; it is one — *ungoverned ephemeral state* — showing up five ways.

## Decision

### §1 The thesis — the monorepo IS one live governed graph

The monorepo and its lifecycle are **one live, federated (per-cell), content-addressed graph** —
`governance/corpus/` extended into the single management + lifecycle substrate for the whole repo. This
is **not a new system**: it is the productized development pipeline (the fabric) with the corpus as its
substrate; docs-as-code, git-hygiene, and tool-ephemera harvest are all **facets** of it. Every artifact
is exactly one of:

- **A node** — code-symbol (AST, content-addressed via `core::Function::signature_hash`), capability,
  doc (PRD/RFC/ADR/DD), decision, requirement, progress-item — OR an **ephemeral work-node** (git
  worktree, branch, checkpoint, `.omc`/`.omx` marker, `/tmp` scratch) carrying a **TTL + a harvest edge**
  to its durable node.
- **A projection** — rendered docs, handoffs, roadmaps, dashboards, the ADR index, the JSON SSOT view —
  compiled OUT of the graph, never hand-maintained.

Development-lifecycle stages (illustrative — research→design→plan→RED/GREEN→code→review→harden→perf→
simplify→full-test-suite→slop→doubt-driven+verify→ci/cd→ship→observe→self-improve; the canonical stage
list is the fabric pipeline SSOT, not minted by this ADR) are node **state-transitions**, each gated by
**deterministic invariants (fail-closed)**. An LLM/NLI pass is **advisory-only** (files an issue, never a
merge verdict — evidence-admissibility bar). Contradiction/sprawl/staleness become *structurally
impossible on the durable/merge surface* — projections are compiled from one source so they cannot
disagree, and fail-closed invariants make new ungoverned sprawl **un-mergeable** — while local ephemera
are caught by the §3 detector and auto-expired rather than blocked at creation, because there is ONE
single-source graph anchored to code-AST reality.

### §2 The write-authority stays git-native; the query DB is a derived projection

The **git-native corpus + governed JSON is the write-authority source of truth** (commit-only writes,
deterministic canonical form, provenance, PR-review, portability). A **rebuildable, de-committed,
materialized query index** is a DERIVED projection reconciled FROM the committed corpus (the
materialized-projection-of-a-source-of-truth pattern; Palantir Foundry Ontology is a reference-only
exemplar). The invariant evaluator's traversals, the operator console, and dashboards query the **index**;
writes never touch it. Owned-Rust query engine is the W5 destination; a best-in-class embedded Rust DB
(SQLite/DuckDB — relational; CozoDB — Datalog/graph; oxigraph — RDF) behind an **owned query port** is the
transient adapter (ADR-0520). Sharded per-cell, one logical query surface. **Rejected: DB-as-SSOT** — it
breaks commit-only-writes/provenance/portability.

### §3 The ephemera lifecycle — harvest then reap (founder-ratified forks)

Every ephemeral work-node (git artifacts + tool markers + scratch) follows one uniform mechanism:

- **Scope:** covers the developer's **local working set** (every scratch worktree/branch), not only the
  PR/merge-durable set — the 369/550 local artifacts are the target.
- **Creation:** **allowed-but-auto-expired.** A sanctioned governed-creation path exists; raw
  `git worktree add`/branch is never blocked (the agentic tooling — OMC/OMX/workflow worktrees — must not
  be blocked at creation). Fail-closed lands on the **durable invariant** (a stray past-TTL with no harvest
  edge is a violation), never on blocking creation, and stays non-blocking at velocity.
- **Disposition:** **reap-by-default; harvest only on a deterministic durable signal** (keeps the 900+
  backlog a cheap sweep, not a manual salvage). A reap **archives, never `rm`s** — it moves the work-node
  into the harvest sink and is **gated by a second verifier**; the destructive step is the one place the
  system can lose work, so it is fail-safe by construction, never a bare delete.
- **Durable signal:** commits **not reachable from `origin/dev`** (covers unpushed-local AND
  pushed-but-unmerged) OR **dirty/untracked working-tree state in a worktree** (uncommitted edits are not
  commits, so commit-reachability alone would miss them) OR an explicit harvest/promote marker. A
  work-node is reap-eligible ONLY when it is fully-merged, clean, AND carries no marker. Never loses
  unmerged, uncommitted, or otherwise local-only work.
- **TTL clock:** **per work-node kind** (worktree / branch / `.omc`-`.omx` marker / `/tmp`); **last-touch
  (commit / checkout / write) resets** it. **TTL expiry gates *eligibility*; the durable signal gates
  *disposition*** — nothing is reaped before its TTL expires (an active long-lived worktree is therefore
  never reaped), so the sink only ever receives *abandoned* strays; exact durations are policy-pack DATA
  tuned post-cleanup (decide-later).
- **Harvest sink:** **park as a durable content-addressed node** (a graph node + harvest edge, quarantined
  for later human/agent triage that promotes valuable ones to PRs/ADRs) — NOT auto-PR (floods the queue),
  NOT ref-only (ungoverned). **Uniform sink** for both the one-time backlog cleanup and the steady-state
  sweep.
- **Detector (forward-looking):** a continuous reconciler CATCHES newly-created ungoverned sprawl (any
  worktree/branch/scratch with no governance node and no harvest edge), so the clean baseline stays clean.

### §4 Concurrency — multi-developer × multi-agent × extreme velocity (first-class)

The graph supports **high-concurrency writes**: content-addressed nodes are append-only/idempotent; typed
edges are **declared from one end** (reverse derived), so two ends physically cannot disagree (eliminating
the half-edge class by construction). Sprawl volume scales with devs×agents, so harvest/reap is a
**continuous background reconciler**, never a batch job. Extreme-velocity merges run through an
**auto-rebase queue + projected merge-state** (zero manual shepherding — manual shepherding is a process
failure). Concurrent conflicting edits surface as explicit `conflicts_with` edges, not silent divergence.

### §5 Management surface + enforcement

Management is BOTH the **deterministic gates** (block bad states) AND an **operator console**
(manage-through-the-graph) — delivered together. The console is a **pure projection** (read) + **governed
commit-only mutations** (never a parallel write path). The invariant set is the enforcement backbone: it
extends the existing `cross-artifact-agreement` evaluator (do not rebuild) with an `ungoverned_artifact`
code plus the docs-as-code invariant battery (functional-regression / full-test-ladder, architectural
conformance, no-silent-fallback, dead-code reachability, `anchor_fingerprint_drift`, and amends/supersede
reciprocity); fail-closed on the R0 path after a one-time cleanup to a clean baseline. Neutral engine +
policy-as-data (ontology/rulepack/anchor-extractors as DATA); a stranger's-repo fixture is a required gate
so no repo-specific nuance leaks into the engine. The whole substrate is **owned-Rust, buck2-built,
zero-shell / zero-CLI, adds no new python/shell/CLI surface, and takes on no new dependency** — the sole
exception is the §2 embedded-DB adapter, a sanctioned **ADR-0520** transient-stack pick (best-in-class,
MIT/Apache) behind the owned query port, subject to the transient-stack bar and replaced at owned-stack
cutover.

### §6 What this amends / relates

- **Amends ADR-0516** (fabric apex) — adds the monorepo-management + lifecycle-as-graph operating model.
- **Depends on ADR-0517** (the owned AST/content-addressed substrate *doctrine* = what the corpus
  realizes); **ADR-0580** (the `governance/corpus/` Phase -1 spike this ADR **hardens** from a
  two-way-door tree into the management substrate); **ADR-0541** (the corpus liveness / decay-invariant
  graph — the nearest prior art; ADR-0617 **owns the decay mechanism** it began); **ADR-0522** (the
  buck2 *target* graph — a *different* object; the living graph is the corpus graph, sharing the
  "one graph" discipline, not the graph); **ADR-0562** (capability registry + membership lint =
  capability nodes); **ADR-0520** (the transient-substrate-behind-a-stable-interface rule governing the
  §2 embedded-DB adapter); **ADR-0280** (dependency DAG = dependency edges).
- Consistent with ADR-0551/0552/0563 (merge-base ratchet + scm-facts + rename-aware relabel), ADR-0615
  (substrate/product placement).

## Consequences

- One substrate governs code + docs + capabilities + progress + git-artifacts + tool-ephemera; the five
  sprawl surfaces collapse to one lifecycle mechanism.
- The 369-worktree / 550-branch / 91-remote sprawl + `.omc`/`.omx`/`/tmp` ephemera get harvested to a clean
  baseline (one-time) then held clean (steady-state); fail-closed makes new ungoverned sprawl un-mergeable.
- Docs/handoffs/roadmaps become projections compiled out of the graph — cross-document contradiction is
  structurally impossible and live md/json count drops (advances the masterplan-v2 reduction goal).
- Queries become first-class (operator console, graph invariants) without a DB-as-truth; provenance +
  portability preserved.

## Alternatives considered

- **DB as the source of truth.** Rejected: breaks commit-only-writes, provenance, PR-review, determinism,
  and the stranger's-repo portability test — reintroduces ungoverned mutation.
- **Block ungoverned creation up front.** Rejected: would refuse the agentic tooling that legitimately
  spawns worktrees; fail-closed lands on the durable invariant + auto-expire instead.
- **A separate GitOps reconciler for git hygiene.** Rejected: git artifacts are nodes in the ONE graph —
  one lifecycle mechanism, not two.
- **Ratchet-from-debt enforcement.** Rejected by the founder in favor of fail-closed after a one-time
  cleanup (bigger upfront cost, accepted).
- **One global (non-federated) graph.** Rejected: hub-fan-in bottleneck at repo scale.

## Open questions (decide-later; policy-pack DATA)

- Exact per-kind TTL durations (tuned post-cleanup).
- Fail-closed *timing* (within the committed cleanup-first posture — NOT whether to ratchet-from-debt,
  which Alternatives rejects): flip immediately once the one-time reap reaches a clean baseline, vs a
  short frozen grandfather-window that must burn to zero.
- Node home for the new kinds (corpus/core vs doc-parser) + where ephemeral-work-nodes live.
- Federation shard key + cross-shard query spanning.
- Docs→projection migration path; console tech (greenfield Leptos/multi-platform).
