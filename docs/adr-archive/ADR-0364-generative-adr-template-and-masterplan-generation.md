---
id: ADR-0364
status: Superseded
amended_by: [ADR-0613]
deciders: council-architecture, founder
date: 2026-05-26
owner: council-architecture
supersedes: []
superseded_by: [ADR-0709]
related: [ADR-0357, ADR-0363, ADR-0217, ADR-0247]
# --- generative front-matter (this ADR models the template it defines) ---
planning_impact: true
milestone: M-PLANNING-SSOT
depends_on: [ADR-0363]
affected_surfaces:
  crates: [oya-dev-cli]
  microservices: []
  specs: [/specs/masterplan.json, /registry/glossary-vocabulary/warning-baseline.tsv]
deliverables:
  - id: ADR-0364-D1
    description: "This ADR — the generative ADR template + front-matter schema (the contract)."
    exit_criteria: "ADR-0364 present, passes oya lint adr-shape + aspirational-enforcement."
    verified_by: "oya lint adr-shape"
  - id: ADR-0364-D2
    description: "ADR completeness gate — fail any planning_impact:true ADR missing deliverables/exit_criteria/milestone."
    exit_criteria: "gate validate adr-planning-completeness green; rejects a fixture ADR missing deliverables."
    verified_by: "oya gate validate adr-planning-completeness"
  - id: ADR-0364-D3
    description: "`oya gen masterplan` — generate the masterplan projection from accepted planning_impact ADRs."
    exit_criteria: "generator emits masterplan with milestone-grouped deliverables + CI-derived status."
    verified_by: "oya gen masterplan --check"
  - id: ADR-0364-D4
    description: "Masterplan drift gate — committed masterplan == regenerated."
    exit_criteria: "gate validate masterplan-drift green; fails on a hand-edited masterplan."
    verified_by: "oya gate validate masterplan-drift"
  - id: ADR-0364-D5
    description: "Contract-traceability + compatibility gate (registry entry -> ratifying ADR; computed diff == declared change_type)."
    exit_criteria: "gate validate contract-traceability green."
    verified_by: "oya gate validate contract-traceability"
  - id: ADR-0364-D6
    description: "Diátaxis reorg of docs/ (tutorials/how-to/reference[generated]/explanation/decisions)."
    exit_criteria: "docs/ reorganized; doc-catalog green under the new tree."
    verified_by: "oya gate validate doc-catalog"
  - id: ADR-0364-D7
    description: "Re-foundation: distill ~300 ADRs -> clean ADR-0000+ series (archive old, consolidates: provenance, rewrite refs)."
    exit_criteria: "clean series builds the masterplan; old series archived; no dangling ADR refs."
    verified_by: "oya gate validate aspirational-enforcement"
purpose: Make the masterplan a GENERATED projection of the ADR decision log. Define a generative ADR template (rich structured front-matter, lean prose) so an actionable roadmap + a shared-contract registry are derived from accepted ADRs — eliminating the parallel hand-maintained planning sources that drift.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0364: Generative ADR template; masterplan generated from the ADR log

## Status
Accepted — 2026-05-26.

## Context
The planning surface sprawled into parallel, drifting sources (`tasks/*.md`, `.omx/plans/`,
`specs/masterplan.json` + `master-plan-sequencing.json` + `planning-closure-contract.json`, scattered
`docs/`), and the 300-ADR log is 41% never-ratified `Proposed` decision-debt. Best-practice research
(2026-05-26: Google SWE-book canonical-doc rule, Diátaxis, ADR/MADR immutability, GitLab "create once
/ reference", AWS "docs as a build artifact", and the **Kubernetes KEP** precedent — the one validated
instance of generating a roadmap from decision-record metadata) converges on one fix: **designate one
canonical source per concern and GENERATE the rest.** The decision log (ADRs) is the durable, immutable
source; a hand-maintained masterplan that restates it is a parallel source that drifts.

## Decision

### 1. The masterplan is a GENERATED projection of the ADR log
`oya gen masterplan` reads accepted `planning_impact: true` ADRs, topo-sorts by `depends_on`/`supersedes`,
groups by `milestone`, and emits each `deliverable` as a roadmap line. `specs/masterplan.json` becomes
build output, never hand-authored. A **drift gate** (committed == regenerated) is the inspection
mechanism (Amazon "mechanisms, not intentions").

### 2. Generative ADR template — rich structure, lean prose (KEP-anchored)
ADR front-matter carries the machine-extractable fields (this ADR models them above):
`planning_impact`, `milestone`, `depends_on`, `affected_surfaces`, `deliverables[]`, `contracts[]`,
plus the standard `status`/`supersedes`/`superseded_by`/`related`. Narrative stays MADR-minimal
(Context/Decision/Consequences). Resist heavyweight prose ("spec-saturated" antipattern).

### 3. Deliverable status is DERIVED, never authored (KEP graduation model)
`deliverables[]` hold only `{id, description, exit_criteria, verified_by}` — **no `status` field**.
planned/in_progress/done is computed at generation time from `verified_by` (gate green ⇒ done). This
keeps the record immutable (only `status`/`supersedes` ever change — MADR/Log4brains/Azure-WAF rule),
makes roadmap drift structurally impossible, and prevents spec-saturation (a deliverable cannot be
`done` without a passing gate). Cap status to KEP's ~2 axes; no `blocked`/`at-risk` sprawl; no
assignees/due-dates (≠ a task tracker).

### 4. Shared contracts are ratified by ADRs, by reference (not copied)
A contract change (schema/API/event/proto) IS a decision. The ADR's `contracts[]` cites the canonical
artifact `{id/path, pinned_version, change_type, compatibility_mode, consumers, migration}` — it does
**not** embed the schema (the registry artifact is the single normative source; embedding = dual-source
drift, a gate failure). Ratification is risk-tiered (Google AIP anti-bottleneck): additive auto-ratifies
via CI diff+lint; breaking/novel needs an AIP-style quorum + migration + consumer accounting. A
**contract-traceability + compatibility gate** enforces: every registry entry → a ratifying ADR;
computed diff (`buf breaking`/OpenAPI-diff/etc.) == declared `change_type`; hard proto wire-invariants;
breaking ⇒ new version + deprecation + `can-i-deploy`-style consumer disposition.

### 5. `docs/` reorganizes into Diátaxis quadrants
`tutorials/` (onboarding), `how-to/` (runbooks), `reference/` (generated — masterplan, crate/µservice
catalog, contract registry; never hand-edited), `explanation/` (architecture, ideas), and `decisions/`
(the immutable ADR log). CODEOWNERS + a per-doc freshness field become a gate.

### 6. Re-foundation: distill the log to a clean ADR-0000+ series
The ~300 ADRs are distilled (LIVE / superseded / obsolete / duplicate) to ~44 clean ADRs. Survivors are
re-authored into a fresh **ADR-0000+** series in this template, each with `consolidates: [old-ADR-…]`
provenance; the old series is archived frozen (history preserved — NOT in-place renumber, which would
destroy the immutable audit trail). All ADR references repo-wide are rewritten old→new. This is a
dedicated, high-blast-radius migration (see `.omx/adr-distillation/MASTER-DISTILLATION.md`).

## Rejected alternatives
- **Hand-maintain the masterplan** — the drift antipattern this ADR exists to kill.
- **Status inside the ADR `deliverables`** — pollutes the immutable record; rejected for CI-derived status.
- **Embed schemas in ADR prose** — dual-source drift; rejected for cite-as references.
- **In-place renumber of 300 ADRs** — destroys the audit trail; rejected for archive + re-author.

## Consequences
- Positive: one canonical planning source; roadmap can't disagree with decisions; contracts gate-enforced;
  decision-debt distilled away. The masterplan + contract registry become trustworthy build artifacts.
- Negative/cost: build the generator + 3 gates; the re-foundation (D7) is a large one-time migration.
- Neutral: package names / dependency graph unchanged; this is doctrine + tooling, not a code rewrite.

## Verification
Per-deliverable `verified_by` above. Net: `oya lint adr-shape`, `oya gate validate
adr-planning-completeness | masterplan-drift | contract-traceability | doc-catalog | aspirational-enforcement`
all green; `oya gen masterplan --check` reproduces the committed masterplan.

## References
- Best-practice research 2026-05-26 (banked): Google SWE-book ch10; diataxis.fr; adr.github.io / MADR;
  Kubernetes KEP (kep.yaml stage/milestone + PRR graduation); AWS Well-Architected; Google AIP; Confluent
  Schema Registry + Buf; GitLab handbook. Distillation: `.omx/adr-distillation/MASTER-DISTILLATION.md`.
- ADR-0363 (substrate / oya = gate engine), ADR-0357 (vertical-slice nesting), ADR-0247 (self-hosting),
  ADR-0217 (vertical-slice rollout order).
