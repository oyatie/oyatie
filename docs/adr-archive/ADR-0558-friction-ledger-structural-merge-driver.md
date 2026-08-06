---
id: ADR-0558
title: "Friction-ledger structural merge driver: id-aware union + second-author conversion"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-12
door: two-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
depends_on: [ADR-0544, ADR-0546, ADR-0548]
amends: []
related: [ADR-0111, ADR-0363, ADR-0515, ADR-0539, ADR-0552, ADR-0555]
related_specs: []
milestone: W0
---

> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Friction-ledger structural merge driver

# ADR-0558: Friction-ledger structural merge driver — id-aware union + second-author conversion

## Status

**Proposed - 2026-06-12 (registration record for shipped local tooling; door: two-way —
deleting the `.gitattributes` entry and the crate restores plain text merges).**

## Context

`.omc/ultragoal/friction-ledger.jsonl` is an append-only, event-sourced JSONL surface: PRIMARY
rows (`friction` + `status`) anchor a friction id, UPDATE rows (`status_update`) append
disposition transitions, and the ADR-0544 friction-accounting gate folds the physical rows per
id. With parallel agent lanes each appending rows, git's text merge re-conflicts the
trailing-line region on every advance of `dev`. Three leader incidents in one session
(FRIC-1781370000) form the RED corpus:

1. two lanes both authored PRIMARY rows for one FRIC id; the union kept both and the gate failed
   closed on `friction_duplicate_primary_row` — correct gate, wrong union;
2. a hand-rolled union crashed mid-resolution and committed raw conflict markers;
3. an exact-line dedup pass mangled byte-divergent-but-logically-identical rows.

The Cargo.lock structural driver (`tools/oya-cargo-lock-merge-driver-app`, FRIC-1781069288)
established the in-repo shape: a Rust binary invoked by git's merge machinery via a
`.gitattributes` `merge=` driver, fail-closed to a normal conflict on anything unmodeled. That
crate predates ADR-0555 structural accounting; this decision record exists primarily because new
artifacts must be ownership-registered and justified at creation (ADR-0555 D2) — the driver
itself follows the cargo-lock precedent and needs no new doctrine.

## Decision

Ship `tools/oya-friction-ledger-merge-driver-app`, a structural three-way merge driver for the
friction ledger, registered as `/.omc/ultragoal/friction-ledger.jsonl merge=friction-ledger` in
`.gitattributes`, with these pinned semantics:

- **Id-aware union.** Base rows preserved in base order (append-only doctrine: a side-deleted
  base row is preserved; legitimate redaction is a linearised commit on `dev`); additions append
  ours-then-theirs; logical identity is parsed-JSON equality realized as canonical-byte equality
  (number lexemes identity-significant), never raw-byte equality.
- **Second-author conversion.** An id keeps exactly one PRIMARY: the base primary wins, else the
  earliest author by content (`(seen_at, canonical bytes)` minimum — never the merge side);
  losing primaries auto-convert to event-sourced update rows
  `{id, seen_at, status_update: <its status>, evidence: <its evidence + enforcement_fix>,
  story/goal carried}`.
- **Canonical serialization, single-owner.** Row bytes are produced by the ADR-0546
  canonical-json kernel (`sort_keys`, literal UTF-8, LF, verbatim number lexemes) plus one
  documented sound projection to single-line JSONL; the canonical form is a fixed point, ending
  the byte-divergence class at the source.
- **Fail-closed + D7 self-validation.** Any unparseable or unmodeled side — including the whole
  zone where the driver's primary/update classification could diverge from the ADR-0544 fold —
  refuses the merge with `%A` byte-untouched (atomic rename write); the driver's own output must
  reparse and re-satisfy single-primary-per-id, conservation, and no-new-orphans before it is
  written (ADR-0548 D7).
- **Enforcement layering.** The driver is the local automation layer and only helps actors who
  configured `merge.friction-ledger` in git config; merge authority remains the cloud-ci gates
  behind `oya-ci-required` (ADR-0515), with the ADR-0544 gate as the canonical backstop. Its
  Talos-era successor is server-side merge intelligence in the ADR-0515 Tide admission path,
  at which point this driver and its `.gitattributes` entry are deleted (two-way door).

### Registered artifacts (ADR-0555 D2)

The files owned by this decision, all under `OWNERS` owner `cloud-ci-platform`:
`tools/oya-friction-ledger-merge-driver-app/BUCK`,
`tools/oya-friction-ledger-merge-driver-app/Cargo.toml`,
`tools/oya-friction-ledger-merge-driver-app/OWNERS`,
`tools/oya-friction-ledger-merge-driver-app/README.md`,
`tools/oya-friction-ledger-merge-driver-app/src/lib.rs`,
`tools/oya-friction-ledger-merge-driver-app/src/main.rs`,
`tools/oya-friction-ledger-merge-driver-app/tests/cli_fixtures.rs`,
`tools/oya-friction-ledger-merge-driver-app/tests/merge_fixtures.rs`.

### Integration via Workflow + Ontology

Not applicable. This ADR registers local merge tooling only; it does not emit Workflow events,
consume Workflow events, or write Ontology objects.

## Consequences

### Positive

- The three incident classes are mechanically closed for configured actors: duplicate primaries
  auto-convert, garbage can never be written (a refused merge is a normal git conflict), and
  byte-divergent logical twins collapse to one canonical row.
- The ledger's serialization converges on one canonical dialect (ADR-0546 single-owner), so
  byte-level merge noise shrinks over time instead of compounding.
- The incident corpus is pinned as RED/GREEN fixtures cross-validated against the live ADR-0544
  fold and policy, so the union semantics cannot silently drift from the gate's fold.

### Negative

- Per-clone git config is not versioned: unconfigured actors still resolve ledger conflicts by
  hand, and the gate fleet remains the only universal enforcement (accepted per
  enforcement-layering; the Tide successor removes this gap).
- Concurrent divergent `status_update` rows for one id have no total order across merge
  orientations; the gate's latest-update fold can differ by which side merged first — inherent
  to physical-order folding of parallel appends, and no worse than the text union it replaces.
- **Committed-history logical-duplicate collapse (disclosed):** dedup is logical-set union over
  the whole document, committed history included — canonically-identical update rows for one id
  collapse to one on any merge even when non-adjacent, so an `A,B,A` update interleave collapses
  to `A,B` and silently rewrites the ADR-0544 latest-update fold (effective status `A` becomes
  `B`; e.g. a re-logged `fix-in-flight` after a `RESOLVED` flips the fold to `RESOLVED`). The
  live corpus carries zero such duplicates and the red direction fails closed; this
  green-to-green rewrite is accepted together with the **authoring rule**: a re-logged
  transition (a reopen after a terminal status, or any repeat of an earlier status for the same
  id) must differ in content — a fresh `seen_at` or `evidence` — so it is a distinct logical
  event rather than a dedup target.
- The first driver-mediated merge rewrites the whole file once (canonical normalization churn);
  subsequent merges are byte-stable.
