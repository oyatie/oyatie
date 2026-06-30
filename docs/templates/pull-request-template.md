---
doc_status: published
---

# PR template

> Per [`docs/STANDARDS-AND-TEMPLATES.md`](../STANDARDS-AND-TEMPLATES.md) §2, every PR uses this template. The traceability H2 sections plus automated reviewer-agent verdict are validated by the `oya-ci-required` PR metadata preflight; F-PR5-06 still owns trusted server-side/cloud-ci live review-producer closure, so this is not live cloud admission enforcement.

## Issue
Closes #<n> (or Refs #<n> if not closing). One line.

## Summary
- 1-3 bullet points on what changed.
- Include the *why*; the diff already shows the *what*.

## Verification
- ☐ Targeted Buck2 tests: `buck2 test <target(s)>` (paste pass/fail excerpt)
- ☐ Targeted Buck2 builds, if applicable: `buck2 build <target(s)>` (paste pass/fail excerpt)
- ☐ Required cloud-ci context: `oya-ci-required` green on the PR head
- ☐ Review/fix evidence packet completed in `## Evidence` for `oya-ci-required`, review threads, reviewer approval, and local-CLI non-authority
- ☐ Worker completion gate: protected PR URL against `dev` exists; local diff or pushed branch alone is not completion evidence
- ☐ Per-change-class Buck2/cloud-ci lane(s): `<list>`
- ☐ Per-change-class reviewer agent run (paste verdict)

## Traceability
- Catalog records touched: `<list>`
- Cross-axis contracts touched: `<list>` (per [DESIGN §10](../DESIGN.md))
- ADRs cited: `<list>`

## Evidence
- Audit-chain emission: `<event-id>`
- Foundation-bypass referenced (if any): `<bypass-id>`
- Per-pack regulator-watch impact (if any): `<list>`
- Review/fix evidence packet:
  - `oya-ci-required` status: `<PASS|FAIL|PENDING>` — `<check/status URL>` — observed at `<timestamp>` on head `<sha>`
  - Exact failing checks before fix: `<check names + failure URLs/log excerpts | none>`
  - Exact fixed checks after fix: `<check names + fix commit(s) | none>`
  - Review threads: `<resolved/unresolved counts + thread IDs or links>`; unresolved threads MUST be `0` before merge
  - Reviewer approval state: `<APPROVE|REQUEST_CHANGES|PENDING>` by `<reviewer>` on approved head `<sha>` (MUST match PR head) — `<review URL>`
  - Local CLI merge authority: `none`; local commands/hooks are advisory shift-left evidence only and are not protected-branch authority
  - Generated faces: `<none touched | producer-materialized only>`; no hand edits to `*.generated.json`
  - SEC-001 threat-model addendum: `<artifact/link | N/A with scope rationale>`

## Code Review
- Reviewer agent: `<reviewer-agent>`
- Verdict: APPROVE
- Resolved items: `<items-or-none>`
- Deferred items: `<items-or-none>`
