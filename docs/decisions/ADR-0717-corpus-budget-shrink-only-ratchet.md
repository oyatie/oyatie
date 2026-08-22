---
doc_status: published
id: ADR-0717
title: "Corpus-budget sprawl ratchet: shrink-only ceilings over evidence, planning, docs, and live ADRs"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-14
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0600]
amended_by: []
depends_on: []
related: [ADR-0711]
milestone: W0
deliverables:
  - id: ADR-0717-D1
    description: "Extend the repo-root-hygiene gate with a policy-driven corpus-budget dimension: class match rules (prefixes + optional suffixes) and frozen ceilings live in DATA; growth past a ceiling is born-blocking with a one-in-one-out remediation; an absent or malformed block fails closed."
    exit_criteria: "Unit tests cover per-class growth, absent-block fail-closed, and malformed counts; a live merge-base test requires any corpus reduction to lower the frozen ceiling in the same PR; the docs/markdown classes seed the gate as a synthetic dependency in the affected-set policy so docs-only PRs execute it."
    verified_by: "presubmit"
---

# ADR-0717: Corpus-budget sprawl ratchet

## Status

**Accepted** (founder directive 2026-08-14, anti-friction wave 3). Amends ADR-0600 (the
repo-root-hygiene gate gains this dimension) and lands on the ADR-0716 cargo merge path.

## Context

Wave 2 deleted the accumulated doc/evidence/planning corpora, but deletion alone does not
prevent re-accumulation. The doctrine already says evidence appends to one ledger and planning
files are deleted on completion; nothing enforced either rule, and nothing capped docs or live
ADRs. Enforcement must be an engine + policy-as-data, not another doc.

## Decision

1. The repo-root-hygiene gate evaluates a **corpus-budget** dimension over the tracked-path
   inventory. Class match rules (prefixes + optional suffixes) and frozen ceilings are policy
   DATA, so any repo adopts the engine with its own corpus roots.
2. Four classes are frozen at the post-wave-2 corpus: evidence files, planning artifacts
   (tasks/ + plan/ + ci/evidence/), docs markdown, and live apex ADRs.
3. Growth past a frozen ceiling is **born-blocking** with a one-in-one-out remediation in the
   finding itself. A deliberate budget change is a reviewed DATA edit.
4. **Absence fails closed:** a policy without `corpus_budget` (or with malformed classes/counts)
   emits `corpus_budget_malformed` — the ratchets can never be silently disabled.
5. **Reductions preserve:** a live merge-base test fails any PR whose corpus shrank without
   lowering the frozen ceiling in the same PR, so cleanup never leaves growth headroom.
6. The docs/markdown affected-set classes seed the gate as a synthetic dependency so a docs-only
   PR still executes the ratchet.

## Consequences

- Positive: sprawl re-accumulation is mechanically blocked; wave-2 reductions are permanent.
- Negative: legit growth now costs a reviewed policy edit (intended); the ratchet requires the
  materialized scm-facts face, which the merge path already produces.
- Negative: a numeric ceiling/pin is a poor identity for the set being ratcheted. Concurrent
  isolated-green PRs can each write the same count against their own base and still collide.

## Amendment (2026-08-17): count is a poor ratchet key

Recorded from the #2100–#2103 merge train and the #2104 restore of `dev` (pin **770** vs
corpus **772**). This is a doctrine note, not a new ADR and not a silent ceiling raise.

Three PRs each added one catalog row and each set the exact pin to `770` against their own
base. Merged in sequence the corpus reached **772** while the pin stayed **770**. A *count*
only says the total moved, so concurrent PRs silently collide. A frozen *set* rebases cleanly
and names the row that appeared.

Shrink-only ceilings remain law. Replacement of the numeric pin with a frozen set / named
members is a same-wave engine change under this ADR, not a newly numbered decision or any new path.

Cite: #2100, #2101, #2102, #2103 → #2104.

## Rules carry why

- **achieves:** permanent shrink-only corpus; enforcement by engine, not prose.
- **origin:** wave-2 cleanup had no backstop; one-ledger and plan-lifecycle rules were advisory.
  The #2100–#2103 → #2104 train showed a count pin (770 vs corpus 772) cannot identify which
  row appeared under concurrent isolated-green PRs.
- **rule:** corpus classes and ceilings are policy DATA; growth is born-blocking; absent block
  fails closed; reductions must lower the ceiling same-PR. A count is a poor ratchet key;
  prefer a frozen set that names members.
- **ensure:** repo-root-hygiene unit + live merge-base tests under `cargo test --workspace`.
- **overturn_when:** a recorded challenge shows the ratchet blocks legitimate delivery AND a
  replacement with five fields lands same-wave.
