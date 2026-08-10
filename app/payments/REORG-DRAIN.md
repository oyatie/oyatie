# app/payments reorg drain notes (`integ/payments`)

## Ownership (rule 3e)

- **Forever home:** `app/payments/**` (this rail).
- **Source (read-only):** `oya/payments/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/payments/**` on this tip.

## Completed (this rail)

- Slice 1: product metadata absorb — `manifest.json`, `README.md`, `PRD.md`, `PHASE-01-PAYMENTS-MVP.md`, `slos/**`.
- Slice 2: contracts + policy + capabilities + catalog + IPs + iac + dashboards + runbooks + scorecards + security + dpia + decisions (96 files total).
- Path cites rewritten `oya/payments` → `app/payments` inside forever home.
- `AUDIT-FINDINGS-2026-05-20.json` excluded per judgment (delete_permanently).

## Remaining for shrink phase (`integ/oya`)

- Delete absorbed `oya/payments/**` paths after verify (shrink-only rail).
- Hub retargets (`specs/capability-registry.json` app_products) on tip-free `integ/specs`.

## Out of envelope (do not touch from `integ/payments`)

- `oya/payments/**` deletes — `integ/oya` shrink-only rail only.
- Other products under `oya/*` or `app/*`.
