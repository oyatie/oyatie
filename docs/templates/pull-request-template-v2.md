---
doc_class: Template
template_id: TPL-PR
status: Accepted
date: 2026-05-12
purpose: |
  Compatibility projection of the canonical PR body. The live authority is
  `docs/templates/pull-request-template.md`: four H2 sections ending in independent
  reviewer evidence under `## Code Review`.
canonical_authority: docs/templates/pull-request-template.md
superseded_by: docs/templates/pull-request-template.md
enforcing_fitness_lane: retired by ADR-0716
merge_status_requirement: presubmit
review_requirement: independent reviewer approval; F-PR5-06 tracks cloud enforcement
owner_team: platform-governance + council-architecture
related:
  - docs/AGENTS.md  # §PR shape + §Done-Definition
  - docs/STANDARDS-AND-TEMPLATES.md  # §2
  - templates/checklists/done-definition-checklist.md
  - templates/checklists/pr-review-checklist.md
adrs_cited:
  - ADR-0716
rfc_2119_active: true
doc_status: published
---

<!-- Compatibility copy. Prefer docs/templates/pull-request-template.md for new edits.
Reviewer evidence is not live cloud admission enforcement until F-PR5-06 closes through a
trusted server-side/pipeline producer. -->

## Issue

`Closes #<n>` (or `Refs #<n>` if not closing). Change class **MUST** be named on the same line: `feature | bugfix | refactor | migration | docs | chore | capability | plugin | runbook | ADR | pack-update`.

## Summary

- 1-3 bullets on **what + why**. The diff already shows the *what*; this section adds the *why*.
- Cite the canonical authority read first per `docs/AGENTS.md §Pre-flight checklist` item 2.

## Verification

- Local verification evidence — `<PASS|FAIL>` — `<commands and excerpt>`
- `presubmit` PR context — `<PASS|PENDING>` — `<check URL>`

## Code Review

- Reviewer: `<agent>` — verdict `<APPROVE|REQUEST CHANGES|PENDING>`
- Resolved items: `<list>`
- Deferred items: `<list with owners + follow-up issue refs>`
