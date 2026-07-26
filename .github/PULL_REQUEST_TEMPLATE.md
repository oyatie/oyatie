<!--
Canonical authority: templates/pull-request-template.md (TPL-PR) + docs/AGENTS.md §PR shape.
Fill the 5 author-owned sections before review. CI (pr-traceability-admission) fails the
gate if any required section or field is missing. `## Code Review` is reviewer evidence:
leave the PENDING placeholder; the reviewer/evidence producer replaces it before merge
(guard-pr-merge-review.mjs refuses worker-authored verdicts).
-->

## Issue

`Closes #<n>` (or `Refs #<n>` if not closing) — change class: `feature | bugfix | refactor | migration | docs | chore | capability | plugin | runbook | ADR | pack-update`

## Summary

- <!-- 1–3 bullets on what + why; the diff already shows the what -->
- <!-- cite the canonical authority read first per docs/AGENTS.md §Pre-flight checklist item 2 -->

## Verification

<!-- Each applicable line MUST carry a PASS/FAIL/N/A token and actual evidence excerpt or link. -->

- `buck2 test <targeted test targets>` — `<PASS|FAIL|N/A>` — `<excerpt>`
- `buck2 build <targeted build targets>` — `<PASS|FAIL|N/A>` — `<excerpt>`
- `oya-ci-required` PR context — `<PASS|FAIL|PENDING>` — `<excerpt or check URL>`
- Review/fix evidence packet — `<COMPLETE|INCOMPLETE>` — `<evidence section anchor or note>`
- Worker completion gate — `<COMPLETE|INCOMPLETE>` — `<protected PR URL against dev; local diff/branch alone is insufficient>`
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
- Review/fix evidence packet:
  - `oya-ci-required` status: `<PASS|FAIL|PENDING>` — `<check/status URL>` — observed at `<timestamp>` on head `<sha>`
  - Exact failing checks before fix: `<check names + failure URLs/log excerpts | none>`
  - Exact fixed checks after fix: `<check names + fix commit(s) | none>`
  - Review threads: `<resolved/unresolved counts + thread IDs or links>`; unresolved threads MUST be `0` before merge
  - Reviewer approval state: `<APPROVE|REQUEST CHANGES|PENDING>` by `<reviewer>` on approved head `<sha>` (MUST match PR head) — `<review URL>`
  - Local CLI merge authority: `none`; local commands/hooks are advisory shift-left evidence only and are not protected-branch authority
  - Generated faces: `<none touched | producer-materialized only>`; no hand edits to `*.generated.json`
  - SEC-001 threat-model addendum: `<artifact/link | N/A with scope rationale>`
- Post-merge product-completion packet (after squash merge):
  - promoted SHA + `oya-ci-required` status URL
  - rollout verification + rollback note
  - observability check + browser UX/user-story evidence
  - release-governance/release-note impact (Release Please only when repo config proves it)

## Code Review

- Target reviewer evidence producer: `oya-pr-review` (not live cloud admission enforcement until `F-PR5-06` closes; requires a trusted server-side/cloud-ci producer)
- Reviewer agent: `<rust-reviewer | typescript-reviewer | python-reviewer | database-reviewer | security-reviewer | privacy-reviewer | tdd-guide | silent-failure-hunter | doc-updater | doc-style-reviewer | capability-reviewer | perf-reviewer>`
- Verdict: pending  <!-- reviewer replaces with: approved (or requests-changes) before merge -->
- Resolved items: `<list>`
- Deferred items: `<list with owners + follow-up issue refs>`
