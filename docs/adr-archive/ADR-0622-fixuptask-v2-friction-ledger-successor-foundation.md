---
id: ADR-0622
title: "Define a nonbinding FixupTask v2 successor foundation"
status: Superseded
planning_impact: false
deciders: []
date: 2026-07-24
door: two-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
amends: []
amended_by: []
depends_on: [ADR-0363, ADR-0515, ADR-0619]
related: [ADR-0544, ADR-0558]
related_specs:
  - /registry/fixuptasks.jsonl
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** FixupTask v2 foundation — nonbinding successor design Accepted as design intent

# ADR-0622: Define a nonbinding FixupTask v2 successor foundation

## Frontmatter

| Field | Value |
|---|---|
| **id** | ADR-0622 |
| **title** | Define a nonbinding FixupTask v2 successor foundation |
| **status** | Proposed |
| **date** | 2026-07-24 |
| **supersedes** | - |
| **superseded_by** | - |
| **owner** | `council-architecture` |
| **related** | ADR-0544, ADR-0558 |
| **bominal_source** | no Bominal equivalent |

## Status

**Proposed — 2026-07-24.** This is an automatable foundation proposal, not a
qualified-human decision. `planning_impact: false` is binding: **HOLD(Planning)**
continues, no implementation-roadmap dispatch is authorized, and no completion
claim may be promoted from this work.

## Context

The append-only `registry/fixuptasks.jsonl` has historical rows but lacks a
machine-checkable lifecycle for new or modified work. ADR-0619 requires retired
predecessor context to remain in Git history rather than a readable in-tree
archive. A successor can therefore carry only identity-level mapping facts; it
must not copy predecessor prose, evidence, status history, or human disposition.

## Decision

If separately accepted under qualified authority, the existing cloud-CI Rust lane
could enforce a durable FixupTask v2 contract. This proposal does not amend or
supersede ADR-0363, ADR-0515, ADR-0544, or ADR-0558 and does not create a binding
lifecycle edge.

The bounded design is:

1. A pure evaluator compares a protected merge-base snapshot with the candidate;
   only byte-identical legacy rows are grandfathered.
2. New or modified rows require the closed lifecycle enum and accountability
   fields. `resolved`, `accepted-risk`, and `blocked` require their mechanical
   evidence fields, but a decision reference never proves qualified authority.
3. A separately named legacy adapter owns any predecessor source, identity-only
   mapping, or qualified-human population work. The durable target has none of
   those source dependencies.
4. An executable prototype was evaluated in intermediate PR commits and is
   intentionally absent from the final tree. The protected Buck2 admission
   universes can select any landed target, so no executable, schema, gate-catalog,
   or workflow surface may be presented as nonbinding while this ADR remains
   Proposed. The prototype is history-only evidence, not admission authority.

## Consequences

### Concrete file and crate changes

| Path / Crate | Change type | BNF v4.1 name | Layer |
|---|---|---|---|
| `docs/decisions/ADR-0622-fixuptask-v2-friction-ledger-successor-foundation.md` | create Proposed record | — | — |
| `docs/ADR-INDEX.md` | producer-generated projection update | — | — |
| `docs/CHANGELOG.md` | record the Tier-1 canonical-document lifecycle event | — | — |
| `docs/machine-readable/decisions.json` | producer-generated projection update | — | — |

No crate, schema, executable, gate catalog, workflow, masterplan, or root-hub
surface changes under this proposal.

### Integration via Workflow + Ontology

Not applicable. This proposal does not emit or consume Workflow events and does
not read or write Ontology objects. Any future accepted implementation must name
its product integration point before activation.

### Positive

- Records the bounded successor design without making it protected admission
  authority.
- Keeps the executable prototype outside the active tree and ordinary authority
  discovery.
- Makes the qualified-human boundary and truthful blocked state explicit.

### Negative

- The proposal cannot validate, migrate, or classify any FixupTask row.
- Historical prototype evidence remains deliberately unavailable to ordinary
  tree-based discovery and cannot serve as a current implementation dependency.
- A later accepted decision and fresh implementation review remain necessary.

### Operational

- This proposal adds no CI lane and changes no existing fitness-lane behavior.
- After qualified acceptance, a fresh implementation PR must restore RED and
  GREEN coverage for protected-input absence and staleness, new and modified
  lifecycle validation, merge-base-only grandfathering, and the explicit Buck
  source boundary.
- ADR projections are emitted only through the sanctioned Buck2 producer.
- The history-only prototype may inform later work but cannot be resurrected or
  treated as current authority without a fresh review, protected admission, and
  the accepted decision receipt.
- The first non-automatable join is the qualified-human selection and review of
  any protected predecessor source and identity-only mapping. Until that evidence
  exists, the truthful terminal state is `BLOCKED_QUALIFIED_HUMAN_INPUT`, not a
  synthetic approval.

This documentation-only proposal has no runtime SLO. Its delivery objective is
fail-closed and exact: every validation must preserve projection parity and the
nonbinding authority ceiling.

| Delivery criterion | Success | Blocking failure | Failure injection or proof |
|---|---|---|---|
| Authority ceiling | ADR remains `Proposed`, `planning_impact: false`, `HOLD(Planning)` remains active, and no acceptance or dispatch surface is added | authority laundering, synthetic acceptance, or roadmap/implementation dispatch | inspect the exact four-document delta and run the live prose/frontmatter agreement gate |
| Projection integrity | producer output has exactly one matching ADR row and byte-exact index/JSON parity | projection drift, duplicate ID, or a hand-edited generated face | `an_adr_file_added_without_regenerating_the_projections_is_flagged` and `a_hand_edited_markdown_projection_fails_closed` |
| Active-tree boundary | executable prototype and protected-admission bindings are absent from the candidate tree | prototype resurrection, new CI veto surface, or executable dependency on historical content | assert no executable, Buck, Cargo, schema, registry, workflow, masterplan, or root-hub delta |
| Future implementation boundary | any implementation starts from fresh accepted authority, protected predecessor evidence, RED regressions, and protected admission | stale protected evidence, unqualified identity mapping, or reuse of the historical prototype as current authority | require exact protected-base/candidate binding and negative tests for missing, stale, or unauthorized predecessor input before activation |

## Clean Architecture Impact

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Not affected | none; no crate or dependency is added |
| `cross-product-refusal` (LEAN-A2) | Not affected | none; no product boundary changes |
| `port-location` | Not affected | none; no port trait is introduced or moved |
| `layer-correctness` | Not affected | none; no layer assignment changes |
| `composition-root-only` | Not affected | none; no app-layer binary changes |
| `sdk-kernel-only` | Not affected | none; no SDK crate changes |

No port trait is introduced by this proposal.

## Alternatives Considered

**Alternative 1 — Keep the executable prototype in the active tree**

- Description: retain the prototype as a supposedly nonbinding implementation.
- Pros: preserves immediately runnable design experiments.
- Cons: Buck2 protected admission can select any landed target, so executable
  presence can create a binding veto surface despite nonbinding labels.
- Reason rejected: a Proposed ADR cannot bootstrap protected admission authority.

**Alternative 2 — Preserve the proposal only in issue text**

- Description: remove the ADR and rely on issue discussion plus Git history.
- Pros: produces the smallest active-tree change.
- Cons: leaves no canonical decision record for the bounded design, authority
  ceiling, or qualified-human boundary.
- Reason rejected: the proposal is useful current context when it remains
  explicitly nonbinding and contains no executable surface.

**Alternative 3 — Accept and activate the successor immediately**

- Description: promote the lifecycle and implementation contract in the same
  change.
- Pros: would make the successor mechanically enforceable.
- Cons: the required qualified-human input, protected source selection, and
  identity-only mapping review do not exist.
- Reason rejected: authorship, tests, and CI cannot manufacture that authority.

## References

- ADR-0363: retired bespoke VCS coordination and protected-PR admission boundary.
- ADR-0515: single current CI admission authority.
- ADR-0544: friction-ledger accounting contract.
- ADR-0558: friction-ledger structural merge-driver context; it does not own
  protected admission authority.
- ADR-0619: history-only predecessor context and retirement boundary.
- Issue: Refs #1346.
