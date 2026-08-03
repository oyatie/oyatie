# The Living Monorepo — hyperscaler monorepo management + project lifecycle as one governed graph

> Staging draft (idea-refine output, 2026-07-10). Per its own thesis this is STAGING — to be harvested into a governed DD/ADR node, not left as a loose markdown file.

## Problem Statement
How might we make the monorepo AND its entire development lifecycle **one live, federated, content-addressed graph** — so every artifact (code, docs, capabilities, decisions, requirements, progress, AND all ephemera: worktrees, branches, checkpoints, `.omc`/`.omx` markdown, `/tmp` scratch) is a governed **node** or a **projection** — making contradiction, sprawl, and staleness *structurally impossible*, not merely detectable?

## Recommended Direction — The Living Monorepo Graph
Extend the existing `governance/corpus/` AST graph (5 crates already) into the single **management + lifecycle substrate for the whole monorepo**. One logical graph, **federated per-cell** (sharded; no global hub bottleneck; matches the cellular-hub-aware repo direction). Everything is exactly one of:
- **A node** — code-symbol (AST, content-addressed via `signature_hash`), capability, doc (PRD/RFC/ADR/DD), decision, requirement, progress-item — OR an **ephemeral work-node** (worktree, branch, checkpoint, `.omc`/`.omx`/`/tmp` scratch) carrying a **TTL + a harvest edge** to its durable node (PR/decision/observation).
- **A projection** — rendered docs, handoffs, roadmaps, dashboards, the ADR index, the JSON SSOT — compiled OUT of the graph, never hand-maintained.

Development-pipeline stages (research→…→ship→observe) are node **state-transitions**, each gated by **deterministic invariants (FAIL-CLOSED)** — LLM advisory-only. Ephemeral nodes **auto-reap** on TTL/reachability *after harvest*. Management is BOTH the **gates** (block bad states) AND an **operator console** (manage-through-the-graph) — both from day one. This is **not a new system: it IS the productized development pipeline** (Workstream P) with the corpus (Workstream D) as its substrate; git-hygiene, docs-as-code, and tool-ephemera harvest are all facets of it.

## Query substrate (founder-confirmed 2026-07-10)

JSON/corpus files stay the **write-authority SSOT** (commit-only, deterministic, portable). A **rebuildable, de-committed, materialized query index** is a DERIVED projection reconciled FROM the committed corpus (materialized-projection-of-a-source-of-truth pattern (Palantir Foundry Ontology is the reference-only exemplar) — already the materializer doctrine). The invariant evaluator's traversals, the operator console, and dashboards query the **index**, not raw JSON; blow it away and rebuild from source anytime — never the write path. Owned-Rust query engine is the W5 destination; a best-in-class embedded Rust DB behind an owned query port is the transient adapter (references: CozoDB — Datalog/graph; oxigraph — RDF/SPARQL; DuckDB/SQLite — relational; all embedded, no daemon). Sharded per-cell, one logical query surface. This preserves provenance + PR-review + the stranger's-repo test while giving graph queries. (Rejected: DB-as-SSOT — breaks commit-only-writes/provenance/portability.)

## Key Assumptions to validate
- [ ] The corpus content-address + doc-parser + extract scale to code+docs+capabilities+git-artifacts as one federated graph (probe: load the current tree, measure).
- [ ] Fail-closed-now is reachable: the existing **369 worktrees / 550 local branches / 91 remote** + `.omc/.omx/tmp` ephemera can be harvested+reaped to a CLEAN baseline before the gate flips (probe: a one-time reap pass).
- [ ] The lifecycle-stage gates + ephemeral-TTL + git-hygiene rules express as DATA (rulepack), portable (stranger's-repo test).
- [ ] The console is a pure projection (read) + governed mutations (commit/PR only), never a parallel write path.

## MVP Scope (extend, don't rebuild)
- Corpus node/edge model: + ephemeral-work-node kind (worktree/branch/checkpoint/scratch, TTL + harvest edge) + the PRD/RFC/ADR/progress nodes (Workstream D).
- Deterministic gate (extend `cross-artifact-agreement`): fail-closed on `ungoverned_artifact` (a worktree/branch/scratch past TTL with no harvest edge) + the docs-as-code invariants.
- The reaper: TTL/reachability → harvest-check → report→archive (git worktree remove / branch archive / `git mv`), second-verifier-gated, **never rm**.
- **One-time cleanup**: harvest durable work from the 369/550/91 + ephemera → reap to a clean baseline (precondition for fail-closed).
- Operator console: a projection (monorepo state, capability map, lifecycle progress, live sprawl/hygiene) + governed commit-only mutations.

## Not Doing (and why)
- One global (non-federated) graph — hub-fan-in bottleneck at scale.
- Ratchet-from-debt enforcement — founder chose fail-closed; the debt gets CLEANED, not tolerated (bigger upfront cost, accepted).
- A separate GitOps reconciler — git artifacts are nodes in the ONE graph; one lifecycle mechanism, not two.
- LLM/NLI as a merge gate — advisory-only (evidence-admissibility bar).
- Committing `.omc/.omx/tmp` — the fix is harvest-into-corpus + reap, not commit-the-scratch.
- A parallel write path from the console — governed mutations are commit/PR only.

## Open Questions
- Fail-closed **sequencing**: flip the gate AFTER the one-time reap (clean baseline), or ship a frozen grandfather-baseline that must burn to zero?
- Node home for the new kinds (corpus/core vs doc-parser — the D3 open Q) + where ephemeral-work-nodes live.
- Federation shard key + how one-logical-graph queries span shards.
- Is this an **amendment to the ADR-0516..0535 fabric cluster** (the dev-pipeline product) or its own ADR? (Recommend: amends the fabric cluster; the corpus is its substrate.)
- Console tech (greenfield Leptos/multi-platform per console doctrine) + staying a pure projection.
