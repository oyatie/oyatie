# app/itsm reorg drain notes (`integ/itsm`)

## Ownership (rule 3e)

- **Forever home:** `app/itsm/**` (this rail).
- **Source (read-only):** `oya/itsm/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/itsm/**` on this tip.

## Completed (this rail)

- Slice 1: product metadata absorb — `manifest.json`, `README.md`, `slos/**`.

## Next gaps (ordered)

1. **Contracts + policy** — `contracts/`, `policy/`, `policies/`, `cedar/`, `catalog/` from `oya/itsm`.
2. **Capabilities + crates** — bounded-context manifests and `oya-itsm-*` crate rehome.
3. **IaC + dashboards** — `iac/`, `dashboards/`, `runbooks/`, `scorecards/`, `IPs/`, `decisions/`, `dpia/`.
4. **Shrink-only burn** — after verify, delete absorbed paths on `integ/oya` (not this rail).

## Out of envelope (do not touch from `integ/itsm`)

- `oya/itsm/**` deletes — `integ/oya` shrink-only rail only.
- Other products under `oya/*` or `app/*`.
- `Cargo.lock` / `AUDIT-FINDINGS-*` / `supported-oses.json` — deferred or judgment-gated.
