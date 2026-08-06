---
id: ADR-0527
title: "oya-ci conformance FLOOR as a config-driven portable engine: engine-vs-policy seam + closed-schema oya-ci.toml loader + gate INPUT-BINDING abstraction (producer-face | raw-corpus-collector | frozen-empty-meta)"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-0700]
depends_on: [ADR-0515]
amends: [ADR-0515]
related: [ADR-0515, ADR-0516, ADR-0525, ADR-0528, ADR-0530, ADR-0533]
related_specs:
  - /specs/phase0-ci-enforcement-baseline.json
  - /specs/masterplan.json
milestone: W1
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0527: oya-ci conformance FLOOR as a config-driven portable engine

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

**Amends ADR-0515** (firewall substrate byte-unchanged). The floor that ADR-0533 generalizes into the
config-driven public boundary.

## Context

ADR-0515's firewall engine is the RED/GREEN-proven crown jewel, but its POLICY is hardcoded in compiled
`const`s and producer literals, so another repo cannot adopt oya-ci without forking. The founder
directive: "CI is a product itself … it can't be hermetic for us once and not work on other projects or
be of little value to others." The floor blocker is that gates do not declare HOW their input is built,
so a portable engine cannot dispatch correctly.

## Decision

Extract ALL oyatie POLICY out of compiled-in `const`s + producer literals into a CLOSED-schema
repo-rooted config (`oya-ci.toml` at repo root, OR `.oya-ci/config.json` — format is OQ-1, carried to
the founder; one canonical format, identical schema either way), loaded by a NEW pure I/O-free
`oya-ci-config` (kernel-role) crate that parses + validates into typed structs and REJECTS unknown
keys. The externalized policy is exactly the three surveyed hiding-places: the naming-kernel consts,
the forbidden-vocab consts, and the producer literals.

Introduce the gate **INPUT-BINDING abstraction**: each enabled gate DECLARES one of THREE verified
input KINDS —

- `producer-face` (gates whose keys come from `evaluate_keyed` over a producer-built face),
- `raw-corpus-collector` (brand-residue; keys arrive pre-grouped as a `BTreeMap<String,BTreeSet<String>>`, NOT a face),
- `frozen-empty-meta` (codes like `registry_drift` stamped empty by the disposition join, NOT
  collected) —

and the two engine loops DISPATCH on that KIND, preserving the BTreeMap/BTreeSet ordering and the
disposition-completeness join byte-for-byte. Config flows through the PRODUCER (not into gate
evaluators); the gate `evaluate_keyed(&Value) -> BTreeSet<Finding>` surface is left UNTOUCHED in this
floor (its hoist into a shared gate-contract crate is ADR-0528/0534). Introduce a gate-PACK abstraction
(`core` language-agnostic; `rust-cargo` for the cargo-specific gates). Zero-config materializes a sane
GREEN empty-but-present default. Land INCREMENTALLY, proving a byte-for-byte backward-compat
green-invariant gate-by-gate and KIND-by-KIND.

## Drivers

- D1 live-green preservation (`oya-ci-required` is the single required check on `dev`; any refactor
  that flips it RED mid-flight is unacceptable — dominates sequencing).
- D2 portability WITH shared value (config lets another repo adopt oya-ci, but a CLOSED schema + gate
  catalog + defaults are the shared spine).
- D3 minimal churn to the proven engine (route config through input-building, not by rewriting
  evaluators).

## Alternatives considered

- **Option A (big-bang full config-loader + ALL policy in one campaign)** — rejected: largest
  simultaneous diff over the single live required check, non-bisectable byte-parity break.
- **Option C (extract a separate publishable `oya-ci` repo NOW)** — DEFERRED to the productization
  cluster (right eventual shape, premature on an unproven seam).
- **Option B (chosen)** — reaches the same FLOOR end-state with lowest blast radius per step, proves
  the supreme invariant incrementally + per-KIND.

## Consequences

The firewall ENGINE (ratchet compare-mode, registry-drift byte-parity, baseline-block-on-new,
frozen_empty, advisory-until-infra, the signoff one-way door, canonical-JSON + provenance digest) is
byte-for-byte UNCHANGED; the one engine touch-point is the disposition-join KIND-dispatch (semantics
preserved). Config becomes DATA validated by a closed schema. Distribution surface for adopters = a
COMPOSITE GitHub Action (NOT a `workflow_call` reusable workflow — a called workflow renames published
check-runs and breaks the `oya-ci-required` required-context name) + a copy-in matrix template.
Supreme acceptance test = oyatie's own config reproduces today's faces byte-for-byte across all three
KINDS, and a non-oyatie fixture repo produces a valid baseline naming ZERO oyatie paths. Broader
product work (third-party gate SDK, dev-env, dep-bot) is OUT of this door and into the productization
cluster (ADR-0532–0535), none blocking the floor. **Amends ADR-0515.** door:one-way.

W1 Task #26 applies this engine-vs-policy seam one legacy check at a time. The first SLO coverage
slice is justified by the same input-binding decision: `oya-ci.toml` declares the catalog input
globs, the producer emits the face, and the pure gate app reuses the existing kernel at
`ci/facade/slo-coverage/Cargo.toml`,
`ci/facade/slo-coverage/BUCK`,
`ci/facade/slo-coverage/OWNERS`,
`ci/facade/slo-coverage/src/lib.rs`, and
`ci/facade/slo-coverage/tests/slo_coverage.rs`.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source: OYA-CI-CONFORMANCE-FLOOR-PLAN.md
(RATIFY-TO-ADR). Amends ADR-0515 (firewall byte-unchanged). Generalized by ADR-0533.*
