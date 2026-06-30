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

## Code Review
- Reviewer agent: `<reviewer-agent>`
- Verdict: APPROVE
- Resolved items: `<items-or-none>`
- Deferred items: `<items-or-none>`
