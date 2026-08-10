# app/sites reorg drain notes (`integ/sites`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/sites/**` (this rail).
- **Source (read-only):** `oya/sites/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/sites/**` on this tip.
- **OVERRULE 3d:** migrated off shared `integ/app` (wrong multi-product absorb shape).

## Completed (this rail)

- Slice 1: product metadata absorb — `manifest.json`, `README.md`, `REORG-DRAIN.md` (replayed from `integ/app@f7133b24b`).

## Next gaps (ordered)

1. **Contracts + policy** — `contracts/`, `policy/`, `cedar/`, `catalog/` from `oya/sites`.
2. **Capabilities + crates** — bounded-context manifests and `oya-sites-*` crate rehome.
3. **IaC + dashboards** — `iac/`, `dashboards/`, `runbooks/`, `scorecards/`, `slos/`, `IPs/`, `decisions/`, `dpia/`.
4. **Shrink-only burn** — after verify, delete absorbed paths on `integ/oya` (not this rail).

## Out of envelope (do not touch from `integ/sites`)

- `oya/sites/**` deletes — `integ/oya` shrink-only rail only.
- Other products under `oya/*` or `app/*`.
