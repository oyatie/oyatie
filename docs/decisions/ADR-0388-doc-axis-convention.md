---
id: ADR-0388
title: Doc-axis convention to prevent doc sprawl
status: Amended
date: 2026-05-28
amended_date: 2026-07-22
authority: founder
owner: founder
planning_impact: true
supersedes: []
superseded_by: []
related: [ADR-0364, ADR-0377]
---

# ADR-0388: Doc-axis convention to prevent doc sprawl

## Status

Amended — 2026-07-22 (original Accepted — 2026-05-28). This in-place
amendment preserves the original decision's authority, owner, planning impact,
and relations, while replacing the former readable-archive lifecycle clauses
with the current authority below.

## Amendment — history-only preparation authority (2026-07-22)

The readable idea archive is no longer terminal compliance. The direct user
instruction to **assume qualified authorization** is recorded as qualified
founder, product, architecture, and repository-disposition authority for this
preparation amendment. It is not a claim of legal or JCR facts, affected-party
consent, operations, custody, veto, pilot results, independent review,
Stage-1 closure, planning PASS, roadmap approval, dispatch, rollout, or
completion. Its scope is preparation authority only: planning remains HOLD.

Only the following three E4 baseline bodies are open transition inventory,
byte-frozen and non-authoritative until E10. They are explicitly noncompliant
with the ordinary transient-ideas lifecycle and do not make the readable
archive a compliant or terminal archive surface:

| Baseline path | Blob OID | SHA-256 | Bytes |
|---|---|---|---:|
| `cloud-intelligence-bedrock-on-talos-2026-05-28.md` | `ffc3aafd802f57d7d6f69a248d90360deecbf9cd` | `2fad4ac166f3a410a0c7aeaef8632c0fe580f034da48f6be4e2bca642e304eca` | 7180 |
| `cloud-intelligence-v1-pipeline-2026-05-28.md` | `4d05288a0b3c8585a478f843b824288ff35faf02` | `740ae04afc93c41240128c22e3bbf2e1ea84dc63f168ebd974a4f0972a72b2a8` | 16014 |
| `n-lane-parallel-safety-and-unified-devops-console-2026-05-28.md` | `820fb46ef556bedaaef22f10e5669791b3143b0d` | `0a1d134ceb7267e1f8e3e7cc6d16a273da5f36266b53febbc4935b937719589f` | 8553 |

The admitted provenance is baseline
`IDEA-ARCHIVE-TRANSITION-2026-07-22-V1` at
`ci/facade/cross-artifact-agreement/src/idea-archive-transition-baseline.json`,
manifest SHA-256
`df46f4ae9eea0c6a59831eb5a47126b6f24475ad499e48a993bda53561ff3d4c`,
captured from commit `1fa09da22be819b062881eb59252f4dd4c6b550a` and tree
`d7b15539396db21b219d68779362850cce9afa8f`. No other archive body, path,
or byte identity is admitted: expansion is forbidden. The transition remains
open and non-authoritative until the required successor epochs E6, E7, E9, and
E10 have supplied their own qualifying evidence; this amendment itself does
not supply or claim that evidence.

The preparation-only evaluator surface is limited to
`ci/facade/cross-artifact-agreement/src/idea-archive-transition-baseline.json`,
`ci/facade/cross-artifact-agreement/src/idea_archive_transition.rs`, and
`ci/facade/cross-artifact-agreement/tests/idea_archive_transition/mod.rs`.
Its declared Buck package markers are `docs/ideas/BUCK` and `specs/BUCK`;
`docs/ideas/OWNERS` is the narrow ownership marker for the ideas package. These
paths supply no closure evidence and do not expand the admitted body set.

The E6 receipt evaluator additionally lives at
`ci/facade/cross-artifact-agreement/src/retirement_receipt.rs`. Its mixed
carried/new-scope regression is part of the same enforcement-only surface: it
accepts neither a new baseline body nor a completion, cutover, Stage-1 PASS,
roadmap-planning, or implementation-dispatch claim.

### Current authoritative lifecycle

This amendment is the current authority for the former transient-ideas archive
rule. The readable archive is not a canonical or compliant archive location,
and the former rule authorizing archive placement is removed. The three rows
listed above are the only temporary, noncompliant, byte-identity exceptions;
they remain transition inventory only until E10 and cannot be expanded.

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
| `DECISIONS` | `docs/decisions/ADR-NNNN-*.md` | no | Authoritative. Status field MUST be one of `Accepted`, `Amended`, `Proposed`, `Superseded`, `Deprecated`, or `Rejected` (exact case). |
| `PLANS` | `docs/machine-readable/masterplan.generated.json` | yes (`oya gen masterplan`) | Derived from ADRs with `planning_impact: true`. Never hand-edit. |
| `INDEX` | `docs/ADR-INDEX.md` | yes (`oya doc adr-index`) | Derived. Never hand-edit. |
| `SPECS-MS` | `microservices/<ms>/manifest.json` | no | Per-µservice. One file per service. |
| `SPECS-CRATE` | `registry/catalog/<crate>.yaml` | no | Per-crate. One file per crate. |
| `RUNBOOKS` | `microservices/<ms>/runbooks/<topic>.md` | no | Per-µservice operational procedure. |
| `IPS` | `microservices/<ms>/IP-NNN-<title>.md` | no | Per-µservice implementation plan. |

### Transient axis (ideas)

| Axis | Canonical home | Auto-gen | Lifecycle rule |
|---|---|---|---|
| `IDEAS` (transient) | `docs/ideas/<topic>-<YYYY-MM-DD>.md` | no | MUST be promoted to an ADR within 14 days of its date-stamp, then removed from the current tree. Readable archive placement is never compliance; only the exact three temporary noncompliant transition-inventory rows in this amendment remain until E10. |

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
`Accepted`, `Amended`, `Proposed`, `Superseded`, `Deprecated`, `Rejected`.

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
- The `docs/` tree is now closed: new subdirectory types require an ADR amendment.
- Existing ADR casing violations are warnings until the follow-up normalisation
  sweep ships.

## Historical Notes

The following is historical provenance only and is not current authority:
completed in chore/doc-consolidation-2026-05-28 (PR coming): 3 idea-pagers promoted to ADR-0389/0390/0391; originals archived to docs/ideas/archive/; 11 Superseded/Deprecated ADRs archived to docs/decisions/archive/.
(and any sibling idea-pagers) to formal ADRs (using the next available
ADR id minted at promotion time) before the 14-day timer expires on
2026-06-11.
