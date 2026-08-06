---
id: ADR-0618
title: "Contract-slice conformance gate scope boundary: single-document internal-shape validation, cross-reference/registry integrity is a separate owned gate"
status: Accepted
planning_impact: false
deciders: founder
date: 2026-07-10
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0515, ADR-0523, ADR-0528]
amends: []
related: [ADR-0111, ADR-0363, ADR-0541]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0618: Contract-slice conformance gate scope boundary

## Status

**Proposed — 2026-07-10** (authored under the founder's autonomous-drive delegation as the
root-cause fix for the contract-slice conversion whack-a-mole; door: two-way — the boundary can be
revised or a follow-up gate can absorb more of Group C as the owned cross-reference substrate lands).
Lifecycle status stays **Proposed** until formal Accept rides cross-artifact propagation.

## Context

The `ci/facade/contract-slice-conformance` gate (ADR-0515 WS-D pure gate; the
`source_migration_slice` Python→Rust retirement pattern) replaces the fleet of
`scripts/tests/*_check.py` contract-slice validators with one owned-Rust, declarative,
data-driven gate. It evaluates a policy of slices, each pointing at one committed JSON
document, against a full-fidelity primitive set (required/enum/array/object-array checks,
exact ordered/set/sequence equality, required-true/false, required and forbidden markers with
separator normalization and sub-tree scoping/exclusion, conditional per-member assertions,
field regex patterns, and array cardinality).

While consolidating the two disjoint DSL branches (#1294 `scalar_str`; #1296 the six primitives)
and porting the four retired Python validators (compliance, residency, talos, release), a set of
checks surfaced that the retired scripts performed but that are **categorically different** from
"validate one committed document's internal shape":

- **C1 — cross-document reference joins.** A field in document A must reference a value that
  exists in document B (residency has ~40% join-shaped checks).
- **C2 — cross-fixture negative joins.** A value present in one fixture must be *absent* from
  another.
- **C3 — filesystem path existence.** A declared `spec_path`/artifact path must exist on disk
  (talos substrate checks).
- **C4 — non-JSON corpora.** Raw-text/YAML documents scanned with regex (release runtime-safety,
  ~70% YAML + cross-document).
- **C5 — full JSON-Schema-instance validation.** Validating an instance against a complete
  embedded JSON Schema (residency `offlineChannelProtocol`).

Forcing these into the contract-slice gate would break its defining invariant (a pure evaluator
over a map of *already-parsed single JSON documents*, one slice = one document's internal shape),
require it to read ambient filesystem state or parse foreign formats, and re-introduce exactly the
per-check bespoke logic the gate exists to retire.

## Decision

**The contract-slice conformance gate validates the internal shape of one committed JSON document
per slice (extendable to N documents evaluated in isolation via an optional `additional_specs`,
with no joins between them). Cross-document joins, cross-fixture negative joins, filesystem
path-existence, non-JSON/YAML corpora, and full JSON-Schema-instance validation (Group C, items
C1–C5) are out of scope and belong to a SEPARATE owned-Rust cross-reference / registry-integrity
(and, for C4, format-aware) gate — recorded here, not silently dropped.**

The primitive set landed by the contract-slice DSL enrichment (Groups A + B) makes the
compliance, residency (single-document parts), talos, and finops slices byte-faithful as
data-only policy entries. Release/#1294 is the honest casualty: its ~70% YAML + cross-document
surface is **not** forced into this gate. Its JSON parts route through the optional
`additional_specs` follow-up; its YAML/cross-document parts route to the separate gate this ADR
scopes. No check is dropped — each is either covered here or explicitly assigned to the
cross-reference/registry-integrity gate.

The boundary test: *if a check needs a second document's contents, the filesystem, or a non-JSON
parser to decide pass/fail, it is not a contract-slice check.*

**Obfuscation boundary (forbidden-marker content).** The deterministic forbidden-marker check
covers case, separator, zero-width, and bidirectional-**reorder** obfuscation: matching
canonicalizes to an `[a-z0-9]`-only form, and the presence of a bidi-reorder control
(U+202A–202E, U+2066–2069) in scanned content is itself a fail-closed violation
(`contract_slice_bidi_control_in_content`). Visually-**confusable homoglyph** substitution
(non-ASCII lookalikes, e.g. Greek/Cyrillic characters that render like Latin) is **explicitly out
of the deterministic gate's scope**: it is unbounded (the full Unicode confusables space),
legitimate internationalized content (e.g. the Korea localization pack) legitimately uses
non-ASCII, and rejecting non-ASCII wholesale would break that content. Homoglyph/confusable
detection is caught by the **advisory LLM/NLI + human review** layer under ADR-0617's
deterministic-invariants-plus-advisory doctrine — the same evidence-admissibility boundary that
separates mechanical gates from advisory review. This is a recorded scope line, not a silent gap.

## Consequences

- **Positive.** The contract-slice gate keeps its pure, single-document invariant and stays a
  data-only paved road: the four conversion PRs become policy entries that touch no Rust. The
  honest scope is written down, so "release is only ~70% covered here" is a recorded routing
  decision, not a silent gap.
- **Negative / successor work.** A separate owned-Rust cross-reference/registry-integrity gate
  (covering C1–C3, C5) and a format-aware extension (C4) are now owed as follow-up IPs. Until
  they land, the Group C checks the retired Python performed are unenforced by any gate and must
  be tracked as open coverage, not assumed covered.
- **Operational.** The gate README records this boundary under "Known scope (and what is out)".
  A converting author who hits a Group C need must route it to the cross-reference gate backlog
  rather than distorting their spec to fit (the failure mode #1290/#1297 hit: flattening a matrix
  array to an object, downgrading an exact set to a superset, dropping order/`==false`/hex shapes).

## Alternatives considered

- **Absorb Group C into this gate.** Rejected: breaks the pure single-document invariant, forces
  ambient filesystem/foreign-format reads, and re-grows bespoke per-check logic.
- **Force release fully into this gate by distorting its specs.** Rejected: that is the exact
  whack-a-mole (spec distortion + silent drops) this enrichment exists to end.
- **Leave the boundary implicit.** Rejected: an unwritten boundary is indistinguishable from a
  silent coverage gap; the founder-visible record is the point.
