---
id: ADR-0629
title: "Crate-catalog coverage: every live crate carries a catalog row, closing the crate→row direction"
status: Rejected
planning_impact: false
deciders: council-architecture
date: 2026-07-28
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0562]
amends: []
related: [ADR-0515, ADR-0527, ADR-0562, ADR-0568, ADR-0628]
---

# ADR-0629 — Crate-catalog coverage

## Status

Proposed (2026-07-28). Two-way door: additive, carries a frozen shrink-only baseline
of today's uncatalogued crates, removable without unwinding any other decision.

## Context

`registry/catalog/` holds one YAML file per crate, **keyed by FILENAME**. That single
property makes a missing row invisible to the search anyone would actually run: the
crate name lives in the PATH, not the contents, so `grep -r <crate-name>` never finds
it.

Consequence, observed on PR #1437: a crate was moved out of the legacy `oya/` root,
every code reference was repointed, the crate and its consumer both built clean
locally — and CI failed anyway, because a YAML file 300 directories away was still
named after the old package. Three born-blocking gates failed (`slo-coverage`,
`catalog-liveness`, the ADR census receipt) and **none named the missing row as the
cause**. Diagnosis took roughly an hour.

The existing checks run the row→crate direction: a catalog row whose crate is gone.
Nothing ran crate→row: a crate with no row.

With ~250 crates still to move out of the legacy roots under ADR-0562, this recurs on
every move that renames a package.

## Decision

Every live first-party crate MUST carry `registry/catalog/<package-name>.yaml`.

Born-blocking against a FROZEN, shrink-only baseline of the crates lacking a row at
adoption (197 of 926). Pre-existing gaps are tolerated debt. A NEW crate must ship its
row. A MOVED crate has a new package name, is therefore absent from the baseline, and
fails unless its row moves in the same change — which is exactly the coupled edit no
codemod authors.

`catalog_coverage_stale_baseline_entry` fires when a baselined crate gains a row or
disappears, so baseline slack cannot outlive the debt it was granted for.

Together with the existing row→crate checks, the crate set and the catalog set are now
mutually total.

## Consequences

- A crate move that strands its catalog row fails with the crate named and the exact
  `git mv registry/catalog/<old>.yaml registry/catalog/<new>.yaml` printed, rather
  than surfacing as three unrelated gate failures elsewhere.
- The 197 uncatalogued crates are frozen, not fixed here; each is its own reviewed
  change.
- No autofix. A catalog row's `context`, `role`, `capability`, `plane`, `slo` tier and
  data/operational classes are governance DECLARATIONS, not facts derivable from the
  crate; a gate authoring them would fabricate ownership and data-classification
  metadata that downstream gates then consume as authoritative. Recorded in
  `ci/facade/gate-self-conformance/gate-self-conformance-policy.json`.
- The gate flagged ITSELF on first run — its own crate had no row. The row was
  authored rather than baselined, because baselining one's own new crate is precisely
  the self-waiver the gate exists to prevent.

## Governed paths

This decision governs, and justifies the existence of:
`ci/facade/crate-catalog-coverage/Cargo.toml`,
`ci/facade/crate-catalog-coverage/BUCK`,
`ci/facade/crate-catalog-coverage/OWNERS`,
`ci/facade/crate-catalog-coverage/crate-catalog-coverage-policy.json`,
`ci/facade/crate-catalog-coverage/src/lib.rs`,
`ci/facade/crate-catalog-coverage/tests/crate_catalog_coverage.rs`, and
`registry/catalog/ci-crate-catalog-coverage.yaml`.

## Alternatives considered

- **Extend the existing row→crate gates.** Rejected: they consume a producer face
  enumerating catalog ROWS, so a crate with no row is invisible to them by
  construction — the corpus does not contain it.
- **Have the move tooling author the row.** Preferred long-term, and ADR-0568's
  `register_crate` orchestrator already implements it — but that orchestrator has no
  invocation surface today, so nothing runs it. A gate that fails loudly is the
  available enforcement until an emitter exists.
- **Baseline the gate's own crate.** Rejected as self-waiving.
