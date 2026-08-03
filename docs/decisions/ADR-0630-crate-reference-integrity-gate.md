---
id: ADR-0630
title: "Crate-reference integrity: a governed artifact naming a crate that no longer exists is a dangling reference, not documentation"
status: Proposed
planning_impact: false
deciders: council-architecture
date: 2026-07-29
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0562]
amends: []
related: [ADR-0515, ADR-0538, ADR-0628, ADR-0629]
affected_surfaces:
  crates: [ci-crate-reference-integrity]
---

# ADR-0630 — Crate-reference integrity

## Status

Proposed (2026-07-29). Two-way door: the gate is additive, carries a frozen shrink-only
baseline of today's dangling references, and can be removed without unwinding any other
decision.

## Context

When a crate is renamed or relocated, the compiler and the build graph both move with it.
The governed artifacts that NAME the crate do not. Nothing reads them, so nothing reports
them, and the reference simply dies in place.

An adversarial review of eight real ADR-0562 move batches found **89** such missed
references. They are not prose mentions; every one of them sits at a STRUCTURAL site that
some other tool keys on:

- `specs/http-stack-policy.json` `justified_crates.<framework>` is a JSON object **keyed by
  package name**. A moved crate strands its justification, and the http-stack gate silently
  stops covering it.
- `registry/graph/architecture-map.json` labels crate nodes by package name; the review
  found node ids AND labels still naming crates that had moved.
- `registry/vendor-lockin-phaseout/index.json` stores seam adapter implementations as paths
  whose last segment is the package name.
- `tasks/pooling-openai-apikey-pool-plan.md:65` carries a **runnable** `cargo check -p
  <old-name>` command, left broken. A reader copies it and it fails.
- `docs/oya-ci/gate-catalog.md` names five `//cloud/cloud-ci/gates/...` build-graph targets
  whose directories no longer hold a build file.
- 25 decision records declare `affected_surfaces.crates: [...]`; three of them
  (ADR-0364, ADR-0365, ADR-0366) name `oya-dev-cli`, which has zero live manifests and zero
  lockfile entries.

`cargo check` is green. `buck2 build` is green. Every one of these references is dead.

## Decision

Add `cloud-ci-crate-reference-integrity`, a blocking gate in the `oya-ci-required` matrix.

**D1 — the known-name census is workspace ∪ lockfile.** Workspace membership resolves at
the canonical `oya-workspace-members-kernel` (ADR-0538), never by textually parsing
`[workspace].members`, which after globbing holds only `*` literals. The lockfile union is
load-bearing: without it, a documented `cargo test -p serde` reads as a dangling reference.
Measured on `dev`: 896 workspace packages, 1479 lockfile names.

**D2 — every structural site class is DATA.** Each rule in
`crate-reference-integrity-policy.json` names a file glob, a structural locator, and a
subject normalization. `kind` is a closed enum of collector shapes; adding a site TYPE is a
policy edit, and only a genuinely new SHAPE needs Rust.

**D3 — a stale path PREFIX around a live package name is out of scope.** The
architecture-map node ids carry dead `microservices/...` prefixes while the package names
themselves are live. That is path liveness, a different defect, and conflating the two would
make this gate's findings unactionable.

**D4 — the dated-corpus discrimination is the point of the gate.** A dated audit snapshot is
a record of what the tree looked like on a date. Rewriting its crate names to match today's
would FALSIFY THE RECORD, not fix a reference. Exclusions are therefore DATA with a
MANDATORY reason, and they are shrink-only under the same staleness machinery as the
baseline: an exclusion that matches no tracked file, suppresses no would-be finding, or
carries no reason is itself RED.

**D5 — no rule may carry a zero floor.** An earlier draft of this decision declared the
decision-record rule DORMANT with `min_sites: 0`, on a measurement that matched only at
column 0 and therefore missed all 25 files whose `crates:` key is nested under
`affected_surfaces` and indented. A declared-but-unmeasured rule is precisely the false
green this gate exists to stop, so the dormancy escape hatch does not exist: `min_sites: 0`
is a violation (`rule_without_measured_floor`), and a site class with no sites today must
not be declared at all. Every floor in the shipped policy was set by running its own locator
against the real corpus, not by estimate.

**D6 — exclusions cannot buy vacuity.** They are applied BEFORE the per-rule floors and the
census floor, so a broad exclusion that empties a corpus trips `rule_yielded_no_sites`
instead of producing a green.

**D7 — no `--fix`.** A dangling reference has three legitimate resolutions and only the
author can choose: repoint it to the crate's new name, delete it because the crate is gone
for good, or recognize the site as a dated historical record that must NOT be rewritten. An
autofixer gets the third case wrong by construction, which is exactly the discrimination
this gate exists to make. Recorded in `gate-self-conformance` `no_autofix_reason`.

## Consequences

- Every ADR-0562 capability move now has to carry its governed-artifact references with it,
  or say in the policy why a corpus is a record rather than a reference.
- The frozen baseline is shrink-only: removing an entry is burn-down and always allowed;
  adding one is not. An entry whose defect is gone is STALE and must be removed in the same
  change.
- `scan-root-liveness` registration is deliberately NOT required. The per-rule glob field is
  named `file_globs`, not a coverage-bearing key, so the fleet gate does not collect it — and
  does not need to, because `rule_glob_matches_nothing` is strictly stronger than an
  existence check: it also fails when a glob resolves but its locator yields nothing.

## Governed paths

This decision governs, and justifies the existence of:
`ci/facade/crate-reference-integrity/Cargo.toml`,
`ci/facade/crate-reference-integrity/BUCK`,
`ci/facade/crate-reference-integrity/OWNERS`,
`ci/facade/crate-reference-integrity/crate-reference-integrity-policy.json`,
`ci/facade/crate-reference-integrity/crate-reference-integrity-baseline.json`,
`ci/facade/crate-reference-integrity/src/lib.rs`,
`ci/facade/crate-reference-integrity/tests/crate_reference_integrity.rs`,
`ci/facade/crate-reference-integrity/fixtures/red/dangling.md`,
`ci/facade/crate-reference-integrity/fixtures/red/dangling.json`, and
`registry/catalog/ci-crate-reference-integrity.yaml`.

## Alternatives considered

- **Grep for old crate names during each move.** Rejected: it is a manual step performed by
  the author least able to see what they missed — which is how 89 references accumulated.
- **Flag every kebab-shaped token that is not a live crate.** Rejected: it drowns the signal
  in prose. Only structural sites, where some tool actually keys on the name, are findings.
- **Rewrite dangling references automatically.** Rejected under D7: the third resolution
  class is "do not rewrite this record", and an autofixer cannot tell it apart from the
  other two.
- **Fold the check into `scan-root-liveness`.** Rejected: that gate answers "does this
  declared path resolve"; this one answers "does this named package still exist". Different
  subject, different remedy, different corpus.
