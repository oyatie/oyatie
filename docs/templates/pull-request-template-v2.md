---
doc_class: Template
template_id: TPL-PR
status: Accepted
date: 2026-05-12
purpose: |
  Compatibility projection of the canonical PR body. The live authority is `docs/templates/pull-request-template.md`: 5 author-owned traceability H2 sections plus reviewer evidence in `## Code Review`.
canonical_authority: docs/templates/pull-request-template.md
superseded_by: docs/templates/pull-request-template.md
enforcing_fitness_lane: oya-governance-pr-shape (delegates to `traceability-validator`)
owner_team: platform-governance + council-architecture
related:
  - docs/AGENTS.md  # §PR shape + §Done-Definition
  - docs/STANDARDS-AND-TEMPLATES.md  # §2
  - docs/checklists/done-definition-checklist.md
  - docs/checklists/pr-review-checklist.md
adrs_cited:
  - ADR-0052  # inventory ledger (traceability row)
rfc_2119_active: true
doc_status: published
---

<!-- Compatibility copy. Prefer docs/templates/pull-request-template.md for new edits. Reviewer evidence is target contract until F-PR5-06 closes. -->

## Issue

`Closes #<n>` (or `Refs #<n>` if not closing). Change class **MUST** be named on the same line: `feature | bugfix | refactor | migration | docs | chore | capability | plugin | runbook | ADR | pack-update`.

## Summary

- 1-3 bullets on **what + why**. The diff already shows the *what*; this section adds the *why*.
- Cite the canonical authority read first per `docs/AGENTS.md §Pre-flight checklist` item 2.

## Verification

Each applicable line **MUST** be present with a pass/fail token (`PASS` / `FAIL` / `N/A`) and actual evidence excerpt or link.

- `buck2 test <targeted test targets>` — `<PASS|FAIL|N/A>` — `<excerpt>`
- `buck2 build <targeted build targets>` — `<PASS|FAIL|N/A>` — `<excerpt>`
- `oya-ci-required` PR context — `<PASS|FAIL|PENDING>` — `<excerpt or check URL>`
- Per-change-class Buck2/cloud-ci lanes — `<list lanes + PASS|FAIL each>`
- Reviewer evidence — `<agent-name>` — verdict `<APPROVE|REQUEST CHANGES|PENDING>`

## Traceability

- Catalog records touched: `<list under registry/catalog/>`
- Cross-axis contracts touched: `<list under contracts/>` (per `docs/DESIGN.md §10`)
- ADRs cited: `<ADR-#### list>`
- `MISTAKES-LEDGER` row referenced (if regression-class): `MFL-NNNN`
- Cross-axis review label applied (if cross-axis contract change): `<label>` (see `docs/checklists/cross-axis-contract-change-checklist.md`)
- Implementation Plan ID (if executing an IP): `IP-NNN-<slug>`
- Inventory ledger row (if migration-class): `INV-NNNN` (per ADR-0052)

## Evidence

- Audit-chain emission ID: `EVT-<topic>-<ulid>` (per ADR-0003)
- Foundation-bypass referenced (if any): `<bypass-id>` + renewal date
- Per-pack regulator-watch impact (if any): `<oya-pack-XX.regulator list>`
- Distroless image build (if shipping a binary): `<image:tag>` + Cosign attestation digest
- SBOM artifact: `<path|registry-ref>` (Syft/CycloneDX)
- SLSA provenance level achieved: `L1 | L2 | L3`

## Code Review

- Target reviewer evidence producer: `oya-pr-review` (not live cloud admission enforcement until `F-PR5-06` closes; requires a trusted server-side/cloud-ci producer)
- Reviewer agent: `<rust-reviewer | typescript-reviewer | python-reviewer | database-reviewer | security-reviewer | privacy-reviewer | tdd-guide | silent-failure-hunter | doc-updater | doc-style-reviewer | capability-reviewer | perf-reviewer>`
- Verdict: `<APPROVE | REQUEST CHANGES | PENDING>`
- Resolved items: `<list>`
- Deferred items: `<list with owners + follow-up issue refs>`
