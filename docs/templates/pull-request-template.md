---
doc_class: Template
template_id: TPL-PR
status: Accepted
purpose: |
  Canonical PR body for every change. Slim by design (ADR-0716): issue, summary,
  verification, reviewer verdict. CI logs and the review thread are the evidence;
  no other sections are required or checked.
canonical_authority: docs/templates/pull-request-template.md
owner_team: platform-governance + council-architecture
related:
  - docs/AGENTS.md  # PR shape + Done-Definition
rfc_2119_active: true
---

<!-- Author-owned: fill the 4 sections below before review. Reviewer evidence is captured in `## Code Review`.
Reviewer evidence is not live cloud admission enforcement until F-PR5-06 closes through a trusted server-side/cloud-ci producer. -->

## Issue

`Closes #<n>` (or `Refs #<n>` if not closing). Change class on the same line: `feature | bugfix | refactor | migration | docs | chore | capability | plugin | runbook | ADR | pack-update`.

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
