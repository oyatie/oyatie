# app/payments reorg drain notes (`integ/payments`)

## Ownership (rule 3e)

- **Forever home:** `app/payments/**` (this rail).
- **Source (read-only):** `oya/payments/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/payments/**` on this tip.

## Completed (this rail)

- Slice 1: product metadata absorb — `manifest.json`, `README.md`, `PRD.md`, `PHASE-01-PAYMENTS-MVP.md`, `slos/**`.

## Next gaps (ordered)

1. **Contracts + policy** — `contracts/`, `policy/`, `cedar/`, `catalog/` from `oya/payments`.
2. **Capabilities + crates** — bounded-context manifests and `oya-payments-*` crate rehome.
3. **IaC + dashboards** — `iac/`, `dashboards/`, `runbooks/`, `scorecards/`.
4. **Shrink-only burn** — after verify, delete absorbed paths on `integ/oya` (not this rail).

## Out of envelope (do not touch from `integ/payments`)

- `oya/payments/**` deletes — `integ/oya` shrink-only rail only.
- Other products under `oya/*` or `app/*`.
