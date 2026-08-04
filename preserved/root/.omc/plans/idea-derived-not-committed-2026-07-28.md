# Idea — Derived, not committed

## Problem Statement

**HMW make backlog items independently workable in parallel — by storing authored
content once and DERIVING everything else, instead of hand-maintaining both?**

## The evidence that reframed it

Two of my own hypotheses were falsified by experiment today. Recording both,
because the plan should inherit corrected premises:

1. **"Stamped-template sprawl."** FALSE. Tested 3 classes at 40 files each,
   normalizing away service slugs, hyphenated identifiers and all numbers:
   **40/40 distinct normalized groups, largest group = 1.** The 179 user-journeys
   are 818–927 lines each with genuinely distinct regulatory anchors
   (HIPAA §164.312 vs 18 USC §2258A), personas and locales. **The ~5,300 markdown
   files are authored work, not waste.** Collapsing them would have repeated the
   "delete all 12 crates" error.
2. **"245 ADRs fail on missing `date`, blocking derivation."** WRONG on count and
   cause. Real distribution over 438 files: 238 parse clean; 200 fail —
   **166 missing `owner`**, 66 `date`, 26 no frontmatter, 13 `id`.
   And **200/200 are mechanically derivable**: `id` ← filename,
   `date` ← `git log --follow --diff-filter=A`, `owner` ← OWNERS lookup.
   Zero genuine gaps.

So the defect is NOT volume and NOT the authored corpus. It is **derived state
stored as source**:

| Confirmed derived-state failure | Measured |
|---|---|
| ADR index projections | 9 hand-edits per ADR, across 2 files, + a GLOBAL counter that serializes all ADR work |
| architecture-map.json | 19 of 453 ids point at `microservices/`, a root that no longer exists; no runnable producer |
| registry/catalog | 784 files keyed by FILENAME — invisible to content search; strands on every crate move |
| Helm charts | 42 name crates that never existed in ANY commit |
| Phantom verifier citation | 521 files cite `./bin/oya verify`; `bin/` absent |
| Placeholders | 506 files / 84,697 lines `rust_code_status: not-authored-in-this-wave` |
| Unimplemented lanes | 776 files reference 5 ADR-0349 lanes; ZERO implemented |

Every one is a function of other files, kept in sync by hand. **None is authored content.**

## Recommended Direction

**Authored stays text. Derived becomes a rebuilt index, never committed.**

The engine already exists and is founder-approved in direction: `governance/corpus/`
(ADR-0580, status Proposed, two-way door) — ~6,100 lines across 5 crates,
content-addressed blake3 facts with a split signature/body anchor behind a stable
`AstSource` seam. Its own doc says the **node/query model was deferred** — that
deferred layer IS the database.

This is not new design. It is finishing a started substrate and pointing it at
documents as well as Rust functions.

Why not the alternatives:
- **SQLite committed to git** — fatal: a binary blob is unreviewable and
  unmergeable, and PR review IS this repo's merge-authority mechanism.
- **External Postgres** — breaks hermeticity; CI would need a live service to
  establish correctness.
- **Structured source + rendered markdown** — keeps the file count, moves prose
  into JSON. No win.
- **Automate-only (wire producers, commit output)** — leaves drift possible;
  today's failures are exactly committed-derived-output rotting.

## Key Assumptions to Validate

- [x] **The authored corpus is mechanically repairable.** VALIDATED: 200/200
      derivable from filename + git history + OWNERS.
- [x] **The authored corpus is real, not stamped.** VALIDATED: 40/40 distinct
      after aggressive normalization.
- [ ] **Rebuild is fast enough to run per gate lane.** Test: time a full corpus
      extract over 438 ADRs + 872 registry files.
- [ ] **Nothing human-facing depends on a COMMITTED projection.** Test: who reads
      `docs/ADR-INDEX.md`? If only gates, de-commit is free.
- [ ] **De-committing removes the serialization point.** Test: two ADR-bearing
      branches that today conflict on 9 counters should merge clean.

## MVP Scope (the N=1 experiment)

ONE class, end to end: **the ADR index**.

IN: derive `id`/`date`/`owner` for the 200 incomplete ADRs; render ADR-INDEX.md +
decisions.json from the authored set; assert byte-parity with committed; then
de-commit both and let the gate regenerate.

OUT: everything else until this proves out.

Success = two ADR-adding branches merge without touching a shared counter.
That single result decides whether the pattern generalizes to catalog,
architecture-map, scorecards, and the 27 `*generated*` artifacts.

## Not Doing (and Why)

- **Touching the 179 journeys or any authored corpus** — measured distinct and
  real; this is the error the experiment exists to avoid.
- **Committing any binary/database file** — destroys PR-review merge authority.
- **External database service** — destroys hermeticity.
- **A big-bang sweep of all 14,000 artifacts** — founder chose experiment-first;
  N=1 on the class with the most measured pain.
- **Deleting the 506 placeholders by hand** — a staleness GATE beats a sweep
  (same shape as scan-root-liveness, shipped #1440).

## Open Questions

- Does the corpus fact model extend to documents, or does it need a second node
  kind? (ADR-0580 models `Function` facts only.)
- ADR-0580 is Proposed — does this work ratify it, or supersede it?
- 5,904 YAML files are mostly k8s/Helm — declarative infra, a third class again.
  In scope or explicitly out?
