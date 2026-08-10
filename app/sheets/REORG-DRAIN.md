# app/sheets reorg drain notes (`integ/sheets`)

## Ownership (rule 3e)

- **Forever home:** `app/sheets/**` (this rail).
- **Source (read-only):** `oya/sheets/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/sheets/**` on this tip.

## Completed (this rail)

- Slice 1: product metadata absorb — `manifest.json`, `README.md`, `slos/**`.

## Next gaps (ordered)

1. **Contracts + policy** — `contracts/`, `policy/`, `cedar/`, `catalog/` from `oya/sheets`.
2. **Capabilities + crates** — bounded-context manifests and `oya-sheets-*` crate rehome.
3. **IaC + dashboards** — `iac/`, `dashboards/`, `runbooks/`, `scorecards/`, `decisions/`, `dpia/`, `IPs/`.
4. **Shrink-only burn** — after verify, delete absorbed paths on `integ/oya` (not this rail).

## Out of envelope (do not touch from `integ/sheets`)

- `oya/sheets/**` deletes — `integ/oya` shrink-only rail only.
- Other products under `oya/*` or `app/*`.
