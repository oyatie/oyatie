---
id: ADR-0388
title: Doc-axis convention to prevent doc sprawl
status: Accepted
date: 2026-05-28
authority: founder
owner: founder
planning_impact: true
supersedes: []
superseded_by: []
related: [ADR-0364, ADR-0377]
---

# ADR-0388: Doc-axis convention to prevent doc sprawl

## Status

Accepted — 2026-05-28. The transient-idea disposition below is constrained by later Accepted
ADR-0515 and ADR-0619: retired content leaves candidate HEAD; Git object history is the sole content
store and no readable archive directory or tombstone copy remains.

## Context

Uncontrolled document creation produces shadow zones: one-off markdown files at
arbitrary paths, idea notes that never become decisions, duplicate catalog data
outside the registry, and implementation plans scattered outside their owning
microservice. Every new doc type that lands outside a canonical axis becomes a
precedent for the next, and the accumulation blocks grep-based discovery, breaks
cross-reference gates, and makes agent navigation unreliable.

The repo already enforces several structural conventions (ADR-0131 per-µservice
flat layout, ADR-0364 generative ADR template, ADR-0377 catalog crate schema)
but lacks a single stated taxonomy that names all canonical axes and
explicitly forbids everything else.

## Decision

### Seven canonical doc axes

| Axis | Canonical home | Auto-gen | Lifecycle rule |
|---|---|---|---|
| `DECISIONS` | `docs/decisions/ADR-NNNN-*.md` | no | Authoritative. Status field MUST be one of `Accepted`, `Proposed`, `Superseded`, `Deprecated`, or `Rejected` (exact case). |
| `PLANS` | `docs/machine-readable/masterplan.generated.json` | yes (`oya gen masterplan`) | Derived from ADRs with `planning_impact: true`. Never hand-edit. |
| `INDEX` | `docs/ADR-INDEX.md` | yes (`oya doc adr-index`) | Derived. Never hand-edit. |
| `SPECS-MS` | `microservices/<ms>/manifest.json` | no | Per-µservice. One file per service. |
| `SPECS-CRATE` | `registry/catalog/<crate>.yaml` | no | Per-crate. One file per crate. |
| `RUNBOOKS` | `microservices/<ms>/runbooks/<topic>.md` | no | Per-µservice operational procedure. |
| `IPS` | `microservices/<ms>/IP-NNN-<title>.md` | no | Per-µservice implementation plan. |

### Transient axis (ideas)

| Axis | Canonical home | Auto-gen | Lifecycle rule |
|---|---|---|---|
| `IDEAS` (transient) | `docs/ideas/<topic>-<YYYY-MM-DD>.md` | no | MUST be promoted into an Accepted successor or declined within 14 days, then deleted from candidate HEAD through a reviewed protected PR. A neutral receipt records predecessor Git blob OID, SHA-256, and successor or decline disposition. A `superseded_by` field does not authorize a readable retired copy. |

### Allowed `docs/` subdirectories

Only the following subdirectories are canonical under `docs/`:

- `docs/decisions/`
- `docs/ideas/`
- `docs/conventions/`
- `docs/machine-readable/`
- `docs/products/`
- `docs/site/`

Any markdown file placed directly under `docs/` or under an unlisted subdirectory
is a gate violation (`no-shadow-docs` rule).

### How to add a new doc — decision tree

1. **Recording a decision?** Create an ADR. Assign the next sequential number.
   Set `planning_impact: true` if the decision changes the masterplan.
2. **Crate-level metadata?** Add or update the catalog YAML under `registry/catalog/<crate>.yaml`.
3. **Microservice metadata?** Update `microservices/<ms>/manifest.json`.
4. **Implementation plan?** Create `microservices/<ms>/IP-NNN-<title>.md`.
5. **Operational procedure?** Create `microservices/<ms>/runbooks/<topic>.md`.
6. **Early-stage ideation?** Create `docs/ideas/<topic>-<YYYY-MM-DD>.md` and
   start the 14-day promotion clock immediately.

### ADR status casing

The gate enforces case-sensitive status values. Allowed values are exactly:
`Accepted`, `Proposed`, `Superseded`, `Deprecated`, `Rejected`.

For the current corpus of existing ADRs the status check emits **warnings**
(not errors) unless `--strict` is passed. A follow-up sweep ADR + script will
normalise all existing ADR statuses and promote this check to error-level.

### Catalog/manifest crate-claim consistency

Every `bounded_contexts[].crates[]` list in a microservice `manifest.json`
MUST have a corresponding entry in `registry/catalog/`. Drift between the two
is a gate violation.

## Consequences

- The `oya-check-doc-axis` gate (registered as `cloud-ci/Rust gate packet doc-axis`)
  enforces all four rules on every PR.
- Idea-pagers that age past 14 days without promotion automatically block the
  gate, creating intentional self-pressure toward decision closure.
- An expired or promoted idea that remains in candidate HEAD blocks even when it carries a
  `superseded_by` marker. Current-tree archive directories are independently blocking.
- The `docs/` tree is now closed: new subdirectory types require an ADR amendment.
- Existing ADR casing violations are warnings until the follow-up normalisation
  sweep ships.

## Notes

Historical 2026-05-28 note: three idea-pagers were promoted to ADR-0389/0390/0391 but their source
copies were moved into a readable directory. ADR-0515 and ADR-0619 later invalidated that disposition;
the copies must be deleted from candidate HEAD with neutral Git-object receipts.
(and any sibling idea-pagers) to formal ADRs (using the next available
ADR id minted at promotion time) before the 14-day timer expires on
2026-06-11.
