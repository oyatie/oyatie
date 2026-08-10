# app/healthcare reorg drain notes (`integ/healthcare`)

## Ownership (rule 3e)

- **Forever home:** `app/healthcare/**` (this rail) — ADR-0615 §6 ONE product with contexts.
- **Source (read-only / historical):** `oya/{emr,pharmacy,patient-monitoring,healthcare-integration}/**` at pre-delete SHA `4aa692919b12` (#1611 merge parent).
- **Writes:** only under `app/healthcare/**` on this tip.

## Completed (this rail)

- Reclaim wave-1: product scaffolds from #1611 delete → context nests:
  - `app/healthcare/emr/`
  - `app/healthcare/pharmacy/`
  - `app/healthcare/patient-monitoring/`
  - `app/healthcare/healthcare-integration/`
- `AUDIT-FINDINGS-*.json` excluded (point-in-time receipts).

## Remaining

- Durable-shape path-cite rewrite (`oya/*` → `app/healthcare/<context>/`) as follow-up slices.
- Hub retargets on tip-free `integ/specs` when absorb verified.

## Out of envelope

- Do not resurrect under `oya/`.
- Do not merge this PR to `dev` until programme admits the rail (PARKED).
