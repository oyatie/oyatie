---
doc_class: Template
template_id: TPL-ADR
status: Accepted
date: 2026-05-12
purpose: |
  Canonical ADR shape with autogen-friendly frontmatter (id, owners, supersedes, superseded_by) suitable for ADR-INDEX rendering and supersession-graph validation. Aligns to hyperscaler convergent ADR practice (AWS / Google / Microsoft / Oracle) and oyatie's existing pack convention (`docs/decisions/ADR-####-<slug>.md`).
supersedes: docs/templates/adr-template.md
header_note: "Supersedes prior docs/templates/adr-template.md once reviewed."
enforcing_fitness_lane: governance-adr-shape
owner_team: crew-adr-promotion
related:
  - docs/ADR-INDEX.md
  - docs/ADR-CONSOLIDATION-PLAN.md
  - docs/decisions/
adrs_cited:
  - ADR-0052  # inventory ledger (traceability)
  - ADR-0053  # sanctioned primitives
  - ADR-0054  # scaffold-claim pattern
doc_status: published
---

<!-- Supersedes prior docs/templates/adr-template.md once reviewed. -->

```yaml
# Required frontmatter on every ADR (for ADR-INDEX autogen + supersession graph)
---
id: ADR-####
title: "<Decision title in imperative form>"
status: Proposed | Accepted | Deprecated | Superseded
date: YYYY-MM-DD
owner_team: <team-id from docs/teams/>
co_owners: [<team-id>, <team-id>]
supersedes: [ADR-####, ...]
superseded_by: [ADR-####, ...]
related: [ADR-####, ...]
tags: [architecture, security, privacy, capability, tooling, ...]
purpose: |
  One paragraph stating what this ADR decides and why future engineers should read it. Used by ADR-INDEX renderer.
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class > working drafts.
---
```

# ADR-####: <Decision Title in imperative form>

> **Status:** Proposed | Accepted | Deprecated | Superseded
> **Date:** YYYY-MM-DD
> **Owner:** `<team-id>` — see [`teams/`](../teams/)
> **Supersedes:** ADR-#### (or `-`) **Superseded-by:** ADR-#### (or `-`)

---

## Context

What is the problem? What forces drove the decision? Quote relevant constraints (regulatory, technical, organizational). Cite source ADRs that bear on this decision. Maximum two paragraphs unless the context requires deeper exposition.

## Decision

The decision in declarative form. **Active voice. Present tense. Specific.** If the decision is structural (e.g., "we will adopt X library") include exact target (library + version range, or contract surface), boundary (where the decision applies; where it does not), and migration path from current state (if any). RFC-2119 normative keywords (**MUST**, **MUST NOT**, **SHOULD**) **MAY** be used when they appear in all-caps per `docs/AGENTS.md §RFC-2119`.

## Decision drivers

Top 3 drivers, each one line. Example shape: "Cohesion over portfolio per CONSTITUTION §Decision principles Do.1." "Eliminate orchestration-glue rot." "Provider-agnostic posture by construction."

## Alternatives considered

For each alternative considered, include: **Name**, **Pros**, **Cons**, **Reason rejected**. **MUST** list at least 2 viable alternatives (including "status quo" if applicable). If only 1 viable option survives, document explicit invalidation rationale for the rest. This matches the consensus-mode RALPLAN-DR contract.

## Why chosen

Map the decision back to: (a) spec acceptance criteria it satisfies; (b) Master Plan principles it honors (per `.omc/plans/MASTERPLAN.md §2 compound principles`); (c) prior ADRs it builds on; (d) the alternatives it beats and why.

## Consequences

### Positive
- Bullet list.

### Negative
- Bullet list. **MUST** be honest; do not soft-pedal.

### Operational
- What this means for on-call, CI, runbooks, audit chain.

## Compounding principles incorporated by reference

Optional but **RECOMMENDED**. List the Master Plan principles (1-12) this ADR explicitly inherits, with one-line justification each. Used by `governance-adr-shape` to verify principle coverage on cross-cutting ADRs.

## Follow-ups

Numbered list of items that need a future decision or implementation. Each gets a target ADR ID (if known), an owner team, and a tracking issue/IP reference.

## References

- Industry references (papers, RFCs, vendor docs, hyperscaler-best-practices doc).
- Internal ADRs cited: ADR-0052 (inventory), ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim).
- Codex / Claude / Gemini / external review feedback (if applicable).
- Related issues (`Refs #N`, `Closes #N`).
- Source spec / plan if lifted from a `/specs/*` draft.
