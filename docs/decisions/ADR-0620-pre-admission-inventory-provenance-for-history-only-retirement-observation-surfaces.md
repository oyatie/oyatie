---
id: ADR-0620
title: "Pre-admission inventory provenance for history-only retirement observation surfaces"
status: Accepted
planning_impact: false
deciders: pending-qualified-authority
date: 2026-07-24
door: two-way
owner: cloud-ci-platform
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0515, ADR-0552, ADR-0555, ADR-0613]
amends: []
related: [ADR-0116, ADR-0363, ADR-0610, ADR-0618]
related_specs:
  - /registry/history-only-retirement/control-plane.json
  - /specs/history-only-retirement-control-plane.schema.json
  - /specs/history-only-retirement-facts.schema.json
milestone: W0
---

> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Pre-admission inventory provenance

# ADR-0620: Pre-admission inventory provenance for history-only retirement observation surfaces

## Status

**Proposed — 2026-07-24.** This two-way record is awaiting qualified-authority review. It is a
pre-admission inventory-provenance registration for candidate-branch paths that already exist; it
does not claim to precede their local creation.

This record is not acceptance, qualified-human authorization, closure evidence, a Stage-1 PASS,
planning approval, implementation or roadmap dispatch, or a lift of `HOLD(Planning)`. Protected
admission of its implementation would prove only the admitted code and gate mechanics. It would
not change this record's lifecycle status or supply any missing qualified authority.

## Context

The total-accounting resolver creates a mechanical path-to-ADR inventory reference when an ADR
contains an exact tracked repo-relative path token. It does not interpret ADR lifecycle status and
must not be used as a planning-authority oracle. The history-only retirement observation candidate
adds seven paths whose reachability and implementation ownership are recorded, but whose exact
tokens do not otherwise occur in the ADR corpus.

Laundering those paths through Accepted ADR-0613 would silently amend an unrelated decision and
would falsely imply accepted authority. Leaving them unreferenced would fail born accounting.
This Proposed record therefore supplies only the missing inventory-provenance references while
preserving the authority boundary.

## Decision

Record non-authoritative path-to-ADR inventory references for exactly these candidate surfaces:

- `ci/facade/scm-facts-snapshot/src/lib.rs`
- `ci/facade/scm-facts-snapshot/src/retirement.rs`
- `ci/facade/scm-facts-snapshot/tests/snapshot_integration.rs`
- `registry/history-only-retirement/OWNERS`
- `registry/history-only-retirement/control-plane.json`
- `specs/history-only-retirement-control-plane.schema.json`
- `specs/history-only-retirement-facts.schema.json`

The references have one effect: the total-accounting resolver may report `ADR-0620` as the
inventory justification for these exact paths. They do not make the paths binding authority,
closure receipts, accepted architecture, a product-completion claim, or a dispatch surface.
The observation face remains ignored, controller-produced, non-authoritative evidence under
`HOLD(Planning)` with dispatch disabled.

A later Accepted authority must replace or supersede this record before any substantive decision
about these surfaces can become binding. If qualified authority rejects this proposal and no
other admitted provenance record replaces it, the seven candidate surfaces and dependent wiring
must be removed. This record creates no exception to that requirement.

## Consequences

- Born accounting can distinguish an intentionally reviewed candidate surface from an
  unexplained file without confusing that inventory fact with lifecycle acceptance.
- ADR-0613 remains byte-faithful to its accepted scope; this proposal neither amends nor
  supersedes it.
- `HOLD(Planning)`, all Stage-1 authority gates, and the prohibition on implementation-roadmap
  dispatch remain unchanged.
- The record is reversible. Rejection removes the candidate surfaces rather than preserving them
  through an invented authority claim.

## Alternatives considered

- **Add the paths to ADR-0613.** Rejected because it would silently expand an Accepted decision
  and convert implementation provenance into apparent authority.
- **Treat reachability or ownership as sufficient justification.** Rejected because those are
  separate accounting dimensions and do not create the required path-to-ADR reference.
- **Rollback the seven surfaces immediately.** Valid if this proposal is rejected; not selected
  while protected review of the reversible candidate is still available.
- **Redesign the justification resolver before admitting this slice.** Deferred. A future typed
  provenance model may distinguish lifecycle and justification classes directly, but that larger
  change is not required to state this candidate's narrow non-authoritative provenance honestly.
