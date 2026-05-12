# PR template

> Per [`docs/STANDARDS-AND-TEMPLATES.md`](../STANDARDS-AND-TEMPLATES.md) §2, every PR uses this template. The 5 H2 sections are CI-enforced by `traceability-validator`.

## Issue
Closes #<n> (or Refs #<n> if not closing). One line.

## Summary
- 1-3 bullet points on what changed.
- Include the *why*; the diff already shows the *what*.

## Verification
- ☐ `cargo nextest run --workspace --all-features` (paste the pass/fail line)
- ☐ `cargo clippy -D warnings` (pass)
- ☐ `oya dev check` (pass)
- ☐ Per-change-class fitness lane(s): `<list>`
- ☐ Per-change-class reviewer agent run (paste verdict)

## Traceability
- Catalog records touched: `<list>`
- Cross-axis contracts touched: `<list>` (per [DESIGN §10](../DESIGN.md))
- ADRs cited: `<list>`

## Evidence
- Audit-chain emission: `<event-id>`
- Foundation-bypass referenced (if any): `<bypass-id>`
- Per-pack regulator-watch impact (if any): `<list>`
