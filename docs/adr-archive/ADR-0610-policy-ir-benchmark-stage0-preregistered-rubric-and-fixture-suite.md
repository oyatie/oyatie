---
id: ADR-0610
title: "Policy-IR benchmark stage-0: pre-registered frozen rubric + fixture suite as governed data"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-07-03
door: two-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0709]
amended_by: []
depends_on: [ADR-0211, ADR-0555, ADR-0562]
amends: []
related: [ADR-0609]
related_specs:
  - /specs/policy-ir-benchmark-rubric.json
  - /specs/policy-ir-benchmark-fixture-suite.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Policy-IR benchmark stage-0

# ADR-0610: Policy-IR benchmark stage-0 — pre-registered frozen rubric + fixture suite as governed data

## Status

**Proposed - 2026-07-03 (authored for founder sign-off; anchors the stage-0 artifacts shipped by PR #1189).**

## Context

The owned Policy IR direction (see `docs/ideas/policy-pack-substrate.md`; Cedar is a benchmark
dialect, not the north star) requires an engine-selection benchmark whose evidence is admissible:
the grading rubric and the fixture corpus must be **frozen before any engine measurement runs**,
so results cannot be tuned toward PASS after the fact (receipt-based evidence admissibility;
amendment RA-001 is logged inside the rubric per its own amendment protocol).

## Decision

Pre-register the stage-0 benchmark artifacts as frozen, machine-readable governed data:

- `specs/policy-ir-benchmark-rubric.json` (`/specs/policy-ir-benchmark-rubric.json`) — the
  pre-registered grading rubric (`POL-IR-BENCH-RUBRIC`, `_meta.status: Frozen`), sole grade
  authority for the benchmark harvest matrix; amendments only via its embedded amendment log.
- `specs/policy-ir-benchmark-fixture-suite.json` (`/specs/policy-ir-benchmark-fixture-suite.json`)
  — the engine-neutral verdict schema plus Core-6 adapter contracts and Fixture-1 corpus the
  rubric grades against.

Both files are data-plane specs (ADR-0562 accounting class: governed spec data; ADR-0555
reachability is registered in `specs/reachability-registry.json` anchored on this ADR). The
eventual owned Policy IR ADR ships the rubric's amendment log as a mandatory section and
supersedes nothing here — this record only fixes the pre-registration point in time.

## Consequences

- Benchmark verdicts citing a rubric or fixture version other than the frozen, committed content
  of these paths are inadmissible as selection evidence.
- Stage-1+ artifacts (harvest matrices, engine adapters, verdict receipts) anchor on this ADR for
  born-accounting until the owned Policy IR ADR lands.
